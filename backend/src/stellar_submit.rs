use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::{Signer, SigningKey};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use stellar_xdr::{
    ContractEvent, ContractEventBody, ContractId, DecoratedSignature, FeeBumpTransaction,
    FeeBumpTransactionEnvelope, FeeBumpTransactionExt, FeeBumpTransactionInnerTx, Hash,
    HostFunction, InnerTransactionResultResult, InvokeContractArgs, InvokeHostFunctionOp, Limits,
    Memo, MuxedAccount, Operation, OperationBody, Preconditions, ReadXdr, ScAddress, ScSymbol,
    ScVal, SequenceNumber, Signature, SignatureHint, SorobanAuthorizationEntry,
    SorobanTransactionData, TimeBounds, TimePoint, Transaction, TransactionEnvelope,
    TransactionExt, TransactionMeta, TransactionResult, TransactionResultResult,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    TransactionV1Envelope, Uint256, VecM, WriteXdr,
};
use thiserror::Error;
use tracing::{debug, warn};

/// Per-operation base fee, in stroops. The Soroban resource fee reported by
/// `simulateTransaction` is added on top of this.
const BASE_FEE_STROOPS: i64 = 100;

/// Maximum extra fee, in stroops, that a single fee-bump retry adds on top of
/// the fee the transaction was originally assembled with. The bump is bounded
/// so an unexpected fee spike cannot make the signer overpay without limit.
const FEE_BUMP_MAX_STROOPS: i64 = 1000;

/// How long a watchdog-built transaction stays valid once assembled. Bounding
/// this means a submission that never lands cannot be replayed later by a node
/// that was partitioned when we gave up on it.
const TX_VALID_FOR_SECS: u64 = 300;

const DEFAULT_POLL_INTERVAL_MS: u64 = 1_000;
const DEFAULT_POLL_TIMEOUT_SECS: u64 = 60;

/// Passphrase of the network the backend talks to by default.
pub const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

#[derive(Debug, Error)]
pub enum StellarSubmitError {
    #[error("failed to reach the Stellar network: {0}")]
    Network(String),
    #[error("Stellar network rejected the transaction")]
    Rejected(Value),
    #[error("Soroban contract invocation is not configured")]
    NotConfigured,
    #[error("invalid Soroban configuration: {0}")]
    Config(String),
    #[error("failed to encode/decode XDR: {0}")]
    Xdr(String),
    #[error("Soroban RPC returned an error: {0}")]
    Rpc(String),
    #[error("transaction simulation failed: {0}")]
    Simulation(String),
    #[error("transaction {hash} failed on-chain: {detail}")]
    TransactionFailed { hash: String, detail: String },
    #[error("timed out waiting for transaction {hash} to be included in a ledger")]
    Timeout { hash: String },
}

/// Everything needed to build, sign and submit a Soroban contract invocation.
///
/// All of it comes from the environment so that deployments without an
/// on-chain signer (local development, CI) simply run without it.
#[derive(Clone, Debug)]
pub struct SorobanConfig {
    /// Soroban JSON-RPC endpoint (distinct from Horizon).
    pub rpc_url: String,
    /// `C…` strkey of the inheritance contract.
    pub contract_id: String,
    /// Network passphrase, hashed into the transaction signature payload.
    pub network_passphrase: String,
    /// `S…` strkey of the account that submits watchdog transactions.
    pub signer_secret: String,
    /// Delay between `getTransaction` polls.
    pub poll_interval: Duration,
    /// How long to keep polling before giving up on a submitted transaction.
    pub poll_timeout: Duration,
}

impl SorobanConfig {
    /// Reads the Soroban settings from the environment. Returns `None` when
    /// the deployment has not opted in — the RPC URL, the contract id and the
    /// signer secret are all required for on-chain execution to be possible.
    pub fn from_env() -> Option<Self> {
        let rpc_url = non_empty_env("SOROBAN_RPC_URL")?;
        let contract_id = non_empty_env("INHERITANCE_CONTRACT_ID")?;
        let signer_secret = non_empty_env("STELLAR_SIGNER_SECRET")?;

        Some(Self {
            rpc_url,
            contract_id,
            network_passphrase: non_empty_env("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|| TESTNET_PASSPHRASE.to_string()),
            signer_secret,
            poll_interval: Duration::from_millis(
                parse_env_u64("SOROBAN_POLL_INTERVAL_MS", DEFAULT_POLL_INTERVAL_MS).max(50),
            ),
            poll_timeout: Duration::from_secs(
                parse_env_u64("SOROBAN_POLL_TIMEOUT_SECS", DEFAULT_POLL_TIMEOUT_SECS).max(1),
            ),
        })
    }
}

/// Parsed, ready-to-use form of [`SorobanConfig`].
struct SorobanContext {
    config: SorobanConfig,
    signing_key: SigningKey,
    public_key: [u8; 32],
    source_account: String,
    contract: ContractId,
    network_id: Hash,
}

/// Result of a successful contract invocation that made it into a ledger.
#[derive(Debug, Clone)]
pub struct InvocationOutcome {
    pub tx_hash: String,
    pub events: Vec<ContractEvent>,
    pub return_value: Option<ScVal>,
}

/// Thin client for submitting already-validated, signed transaction XDR to
/// a Stellar Horizon instance, and — when a [`SorobanConfig`] is supplied —
/// for building, signing and submitting Soroban contract invocations.
#[derive(Clone)]
pub struct StellarSubmitClient {
    client: Client,
    horizon_url: String,
    soroban: Option<Arc<SorobanContext>>,
}

impl StellarSubmitClient {
    pub fn new(horizon_url: String) -> Self {
        Self {
            client: Client::new(),
            horizon_url,
            soroban: None,
        }
    }

    /// Enables Soroban contract invocation on this client. Fails when the
    /// signer secret or the contract id are not valid strkeys, so that a
    /// misconfigured deployment is caught at startup rather than on the first
    /// expired plan.
    pub fn with_soroban(mut self, config: SorobanConfig) -> Result<Self, StellarSubmitError> {
        let secret = stellar_strkey::ed25519::PrivateKey::from_string(&config.signer_secret)
            .map_err(|e| {
                StellarSubmitError::Config(format!(
                    "STELLAR_SIGNER_SECRET is not a valid S… key: {e}"
                ))
            })?;
        let contract = stellar_strkey::Contract::from_string(&config.contract_id).map_err(|e| {
            StellarSubmitError::Config(format!(
                "INHERITANCE_CONTRACT_ID is not a valid C… key: {e}"
            ))
        })?;

        let signing_key = SigningKey::from_bytes(&secret.0);
        let public_key = signing_key.verifying_key().to_bytes();
        let source_account = stellar_strkey::ed25519::PublicKey(public_key).to_string();
        let network_id = network_id(&config.network_passphrase);

        self.soroban = Some(Arc::new(SorobanContext {
            config,
            signing_key,
            public_key,
            source_account,
            contract: ContractId(Hash(contract.0)),
            network_id,
        }));

        Ok(self)
    }

    /// Whether this client can invoke Soroban contracts.
    pub fn soroban_enabled(&self) -> bool {
        self.soroban.is_some()
    }

    /// `C…` strkey of the configured inheritance contract, if any.
    pub fn contract_id(&self) -> Option<&str> {
        self.soroban.as_ref().map(|s| s.config.contract_id.as_str())
    }

    /// XDR contract id of the configured inheritance contract, if any. Used to
    /// attribute emitted events to the contract we actually called.
    pub fn contract(&self) -> Option<&ContractId> {
        self.soroban.as_ref().map(|s| &s.contract)
    }

    /// Address the watchdog signs and submits transactions with.
    pub fn source_account(&self) -> Option<&str> {
        self.soroban.as_ref().map(|s| s.source_account.as_str())
    }

    pub async fn submit(&self, xdr_base64: &str) -> Result<Value, StellarSubmitError> {
        let url = format!("{}/transactions", self.horizon_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .form(&[("tx", xdr_base64)])
            .send()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        let success = response.status().is_success();
        let body: Value = response
            .json()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        if success {
            Ok(body)
        } else {
            Err(StellarSubmitError::Rejected(body))
        }
    }

    /// Health-check: probes the Stellar Horizon root endpoint to verify
    /// network reachability of the RPC node.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/", self.horizon_url.trim_end_matches('/'));
        self.client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Calls `trigger_inheritance(caller, plan_id)` on the configured
    /// inheritance contract and waits for the transaction to be included in a
    /// ledger. The caller is the configured signer account, which the contract
    /// authorises as admin, plan owner or trusted emergency contact.
    pub async fn trigger_inheritance(
        &self,
        plan_id: u64,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        let ctx = self.soroban()?;
        self.invoke_contract(
            "trigger_inheritance",
            vec![signer_account_scval(ctx), ScVal::U64(plan_id)],
        )
        .await
    }

    /// Calls `freeze_loans(admin, plan_id)` to halt new borrowing against the
    /// plan's vault collateral after inheritance has been triggered.
    pub async fn freeze_loans(
        &self,
        plan_id: u64,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        let ctx = self.soroban()?;
        self.invoke_contract(
            "freeze_loans",
            vec![signer_account_scval(ctx), ScVal::U64(plan_id)],
        )
        .await
    }

    /// Calls `recall_loan(admin, plan_id, recall_amount)` to pull loaned
    /// capital (and any already-harvested yield sitting in `total_loaned`)
    /// back into the vault.
    pub async fn recall_loan(
        &self,
        plan_id: u64,
        recall_amount: u64,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        let ctx = self.soroban()?;
        self.invoke_contract(
            "recall_loan",
            vec![
                signer_account_scval(ctx),
                ScVal::U64(plan_id),
                ScVal::U64(recall_amount),
            ],
        )
        .await
    }

    /// Calls `liquidation_fallback(admin, plan_id)` to write off unrecoverable
    /// loaned amounts so settlement can complete.
    pub async fn liquidation_fallback(
        &self,
        plan_id: u64,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        let ctx = self.soroban()?;
        self.invoke_contract(
            "liquidation_fallback",
            vec![signer_account_scval(ctx), ScVal::U64(plan_id)],
        )
        .await
    }

    /// Simulates `get_inheritance_trigger(plan_id)` and, when the plan has
    /// been triggered, returns the outstanding loaned amount still sitting
    /// against the vault.
    pub async fn outstanding_loaned(
        &self,
        plan_id: u64,
    ) -> Result<Option<u64>, StellarSubmitError> {
        let return_value = self
            .simulate_contract("get_inheritance_trigger", vec![ScVal::U64(plan_id)])
            .await?;
        Ok(parse_outstanding_loaned(&return_value))
    }

    /// Builds, simulates, signs and submits a contract invocation, then polls
    /// until the transaction lands (or fails) and returns the events it
    /// emitted.
    pub async fn invoke_contract(
        &self,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        let ctx = self.soroban()?;

        let function_name = ScSymbol(
            function_name
                .try_into()
                .map_err(|_| StellarSubmitError::Config(format!("{function_name} is too long")))?,
        );
        let args: VecM<ScVal> = args
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("invocation arguments: {e:?}")))?;

        let sequence = self.next_sequence(ctx).await?;
        let valid_until = unix_now() + TX_VALID_FOR_SECS;

        let invocation = InvokeContractArgs {
            contract_address: ScAddress::Contract(ctx.contract.clone()),
            function_name,
            args,
        };

        // 1. Simulate an unsigned, unauthorised transaction to learn the
        //    footprint, the resource fee, and the authorisation entries the
        //    host expects.
        let unsigned = build_transaction(
            ctx,
            sequence,
            valid_until,
            BASE_FEE_STROOPS,
            invocation.clone(),
            VecM::default(),
            TransactionExt::V0,
        )?;
        let simulation = self.simulate(ctx, &unsigned).await?;

        // 2. Re-assemble the transaction with the simulated resources.
        let assembled = build_transaction(
            ctx,
            sequence,
            valid_until,
            BASE_FEE_STROOPS.saturating_add(simulation.min_resource_fee),
            invocation,
            simulation.auth,
            TransactionExt::V1(simulation.transaction_data),
        )?;

        // 3. Sign and submit. When the network rejects the transaction because
        //    the inclusion fee spiked, `submit_with_fee_bump` retries once with
        //    a bounded fee bump instead of surfacing a hard failure.
        let inner_fee = i64::from(assembled.fee);
        let (inner, tx_hash) = sign(ctx, assembled)?;
        let envelope_xdr = TransactionEnvelope::Tx(inner.clone())
            .to_xdr_base64(Limits::none())
            .map_err(|e| StellarSubmitError::Xdr(e.to_string()))?;

        submit_with_fee_bump(self, ctx, inner, &envelope_xdr, &tx_hash, inner_fee).await
    }

    /// Simulates a contract invocation without submitting it. Used for view
    /// functions such as `get_inheritance_trigger`.
    pub async fn simulate_contract(
        &self,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<ScVal, StellarSubmitError> {
        let ctx = self.soroban()?;

        let function_name = ScSymbol(
            function_name
                .try_into()
                .map_err(|_| StellarSubmitError::Config(format!("{function_name} is too long")))?,
        );
        let args: VecM<ScVal> = args
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("invocation arguments: {e:?}")))?;

        let sequence = self.next_sequence(ctx).await?;
        let valid_until = unix_now() + TX_VALID_FOR_SECS;

        let invocation = InvokeContractArgs {
            contract_address: ScAddress::Contract(ctx.contract.clone()),
            function_name,
            args,
        };
        let unsigned = build_transaction(
            ctx,
            sequence,
            valid_until,
            BASE_FEE_STROOPS,
            invocation,
            VecM::default(),
            TransactionExt::V0,
        )?;
        let simulation = self.simulate(ctx, &unsigned).await?;
        simulation
            .return_value
            .ok_or_else(|| StellarSubmitError::Simulation("no return value".into()))
    }

    fn soroban(&self) -> Result<&SorobanContext, StellarSubmitError> {
        self.soroban
            .as_deref()
            .ok_or(StellarSubmitError::NotConfigured)
    }

    /// Reads the signer's current sequence number from Horizon and returns the
    /// next one to use.
    async fn next_sequence(&self, ctx: &SorobanContext) -> Result<i64, StellarSubmitError> {
        #[derive(Deserialize)]
        struct AccountResponse {
            sequence: String,
        }

        let url = format!(
            "{}/accounts/{}",
            self.horizon_url.trim_end_matches('/'),
            ctx.source_account
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(StellarSubmitError::Network(format!(
                "Horizon returned {status} for account {}",
                ctx.source_account
            )));
        }

        let account: AccountResponse = response
            .json()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        let current: i64 = account.sequence.parse().map_err(|_| {
            StellarSubmitError::Network("Horizon returned a malformed sequence number".to_string())
        })?;

        Ok(current.saturating_add(1))
    }

    async fn rpc<T: serde::de::DeserializeOwned>(
        &self,
        ctx: &SorobanContext,
        method: &str,
        params: Value,
    ) -> Result<T, StellarSubmitError> {
        #[derive(Deserialize)]
        struct RpcEnvelope<T> {
            result: Option<T>,
            error: Option<RpcError>,
        }

        #[derive(Deserialize)]
        struct RpcError {
            message: String,
        }

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = self
            .client
            .post(ctx.config.rpc_url.trim_end_matches('/'))
            .json(&body)
            .send()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        let envelope: RpcEnvelope<T> = response
            .json()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        if let Some(error) = envelope.error {
            return Err(StellarSubmitError::Rpc(format!(
                "{method}: {}",
                error.message
            )));
        }

        envelope
            .result
            .ok_or_else(|| StellarSubmitError::Rpc(format!("{method}: empty result")))
    }

    async fn simulate(
        &self,
        ctx: &SorobanContext,
        transaction: &Transaction,
    ) -> Result<Simulation, StellarSubmitError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimulateResponse {
            error: Option<String>,
            transaction_data: Option<String>,
            min_resource_fee: Option<String>,
            results: Option<Vec<SimulateResult>>,
        }

        #[derive(Deserialize)]
        struct SimulateResult {
            auth: Option<Vec<String>>,
            xdr: Option<String>,
        }

        let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: transaction.clone(),
            signatures: VecM::default(),
        });
        let xdr = envelope
            .to_xdr_base64(Limits::none())
            .map_err(|e| StellarSubmitError::Xdr(e.to_string()))?;

        let response: SimulateResponse = self
            .rpc(ctx, "simulateTransaction", json!({ "transaction": xdr }))
            .await?;

        if let Some(error) = response.error {
            return Err(StellarSubmitError::Simulation(error));
        }

        let transaction_data = response
            .transaction_data
            .ok_or_else(|| StellarSubmitError::Simulation("no transactionData returned".into()))?;
        let transaction_data =
            SorobanTransactionData::from_xdr_base64(transaction_data.trim(), Limits::none())
                .map_err(|e| StellarSubmitError::Xdr(format!("transactionData: {e}")))?;

        let min_resource_fee = response
            .min_resource_fee
            .as_deref()
            .unwrap_or("0")
            .parse::<i64>()
            .map_err(|_| StellarSubmitError::Simulation("malformed minResourceFee".into()))?;

        let first_result = response.results.unwrap_or_default().into_iter().next();

        let return_value = first_result
            .as_ref()
            .and_then(|result| result.xdr.as_deref())
            .map(|xdr| {
                ScVal::from_xdr_base64(xdr.trim(), Limits::none())
                    .map_err(|e| StellarSubmitError::Xdr(format!("simulate return value: {e}")))
            })
            .transpose()?;

        let auth = first_result
            .and_then(|result| result.auth)
            .unwrap_or_default()
            .iter()
            .map(|entry| {
                SorobanAuthorizationEntry::from_xdr_base64(entry.trim(), Limits::none())
                    .map_err(|e| StellarSubmitError::Xdr(format!("auth entry: {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let auth: VecM<SorobanAuthorizationEntry> = auth
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("auth entries: {e:?}")))?;

        Ok(Simulation {
            transaction_data,
            min_resource_fee,
            auth,
            return_value,
        })
    }

    async fn send_transaction(
        &self,
        ctx: &SorobanContext,
        envelope_xdr: &str,
        expected_hash: &str,
    ) -> Result<(), StellarSubmitError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SendResponse {
            status: String,
            hash: Option<String>,
            error_result_xdr: Option<String>,
        }

        let response: SendResponse = self
            .rpc(
                ctx,
                "sendTransaction",
                json!({ "transaction": envelope_xdr }),
            )
            .await?;

        if let Some(hash) = response.hash.as_deref() {
            if !hash.eq_ignore_ascii_case(expected_hash) {
                warn!(
                    rpc_hash = %hash,
                    local_hash = %expected_hash,
                    "Soroban RPC reported a different transaction hash than the one we computed"
                );
            }
        }

        match response.status.as_str() {
            // DUPLICATE means this exact transaction is already in flight —
            // polling for it is the right move, not resubmitting.
            "PENDING" | "DUPLICATE" => Ok(()),
            "TRY_AGAIN_LATER" => Err(StellarSubmitError::Rpc(
                "sendTransaction: TRY_AGAIN_LATER".to_string(),
            )),
            other => Err(StellarSubmitError::TransactionFailed {
                hash: expected_hash.to_string(),
                detail: format!(
                    "sendTransaction returned {other}{}",
                    response
                        .error_result_xdr
                        .map(|xdr| format!(" ({xdr})"))
                        .unwrap_or_default()
                ),
            }),
        }
    }

    async fn await_transaction(
        &self,
        ctx: &SorobanContext,
        tx_hash: &str,
    ) -> Result<InvocationOutcome, StellarSubmitError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct GetTransactionResponse {
            status: String,
            result_meta_xdr: Option<String>,
            result_xdr: Option<String>,
        }

        let deadline = tokio::time::Instant::now() + ctx.config.poll_timeout;

        loop {
            let response: GetTransactionResponse = self
                .rpc(ctx, "getTransaction", json!({ "hash": tx_hash }))
                .await?;

            match response.status.as_str() {
                "SUCCESS" => {
                    let (events, return_value) = match response.result_meta_xdr.as_deref() {
                        Some(meta) => {
                            let meta =
                                TransactionMeta::from_xdr_base64(meta.trim(), Limits::none())
                                    .map_err(|e| {
                                        StellarSubmitError::Xdr(format!("resultMetaXdr: {e}"))
                                    })?;
                            (contract_events(&meta), return_value(&meta))
                        }
                        None => {
                            debug!(tx_hash, "transaction succeeded without result meta");
                            (Vec::new(), None)
                        }
                    };

                    return Ok(InvocationOutcome {
                        tx_hash: tx_hash.to_string(),
                        events,
                        return_value,
                    });
                }
                "FAILED" => {
                    return Err(StellarSubmitError::TransactionFailed {
                        hash: tx_hash.to_string(),
                        detail: response.result_xdr.unwrap_or_else(|| "unknown".to_string()),
                    })
                }
                // NOT_FOUND simply means the transaction has not been included
                // in a ledger yet.
                _ => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(StellarSubmitError::Timeout {
                            hash: tx_hash.to_string(),
                        });
                    }
                    tokio::time::sleep(ctx.config.poll_interval).await;
                }
            }
        }
    }
}

struct Simulation {
    transaction_data: SorobanTransactionData,
    min_resource_fee: i64,
    auth: VecM<SorobanAuthorizationEntry>,
    return_value: Option<ScVal>,
}

#[allow(clippy::too_many_arguments)]
fn build_transaction(
    ctx: &SorobanContext,
    sequence: i64,
    valid_until: u64,
    fee: i64,
    invocation: InvokeContractArgs,
    auth: VecM<SorobanAuthorizationEntry>,
    ext: TransactionExt,
) -> Result<Transaction, StellarSubmitError> {
    let operation = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(invocation),
            auth,
        }),
    };

    Ok(Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(ctx.public_key)),
        // Fees are capped at u32 on the wire; a resource fee large enough to
        // overflow means the invocation is not affordable anyway.
        fee: u32::try_from(fee)
            .map_err(|_| StellarSubmitError::Simulation("resource fee exceeds u32".into()))?,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::Time(TimeBounds {
            min_time: TimePoint(0),
            max_time: TimePoint(valid_until),
        }),
        memo: Memo::None,
        operations: vec![operation]
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("operations: {e:?}")))?,
        ext,
    })
}

/// Signs `transaction` for the configured network, returning the signed v1
/// envelope and the transaction hash (hex) that identifies it on-chain.
fn sign(
    ctx: &SorobanContext,
    transaction: Transaction,
) -> Result<(TransactionV1Envelope, String), StellarSubmitError> {
    let payload = TransactionSignaturePayload {
        network_id: ctx.network_id.clone(),
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(transaction.clone()),
    };
    let encoded = payload
        .to_xdr(Limits::none())
        .map_err(|e| StellarSubmitError::Xdr(e.to_string()))?;

    let tx_hash: [u8; 32] = Sha256::digest(&encoded).into();
    let signature = ctx.signing_key.sign(&tx_hash);

    let decorated = DecoratedSignature {
        hint: signature_hint(&ctx.public_key),
        signature: Signature(
            signature
                .to_bytes()
                .to_vec()
                .try_into()
                .map_err(|e| StellarSubmitError::Xdr(format!("signature: {e:?}")))?,
        ),
    };

    let envelope = TransactionV1Envelope {
        tx: transaction,
        signatures: vec![decorated]
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("signatures: {e:?}")))?,
    };

    Ok((envelope, hex::encode(tx_hash)))
}

/// Internal seam over the network round-trip of a single submit attempt, so
/// the fee-bump retry can be exercised in unit tests without a live Stellar
/// RPC. Production uses the [`StellarSubmitClient`] implementation below.
type TransportFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait SubmitTransport {
    /// Sends `envelope_xdr` and waits for it to land, returning the emitted
    /// outcome or the submission failure.
    fn submit_once<'a>(
        &'a self,
        ctx: &'a SorobanContext,
        envelope_xdr: &'a str,
        expected_hash: &'a str,
    ) -> TransportFuture<'a, Result<InvocationOutcome, StellarSubmitError>>;
}

impl SubmitTransport for StellarSubmitClient {
    fn submit_once<'a>(
        &'a self,
        ctx: &'a SorobanContext,
        envelope_xdr: &'a str,
        expected_hash: &'a str,
    ) -> TransportFuture<'a, Result<InvocationOutcome, StellarSubmitError>> {
        Box::pin(async move {
            self.send_transaction(ctx, envelope_xdr, expected_hash)
                .await?;
            self.await_transaction(ctx, expected_hash).await
        })
    }
}

/// Submits `inner` and, if — and only if — the network rejects it because the
/// inclusion fee was too small, retries exactly once with a v1 fee-bump
/// transaction that raises `inner_fee` by at most [`FEE_BUMP_MAX_STROOPS`].
///
/// Invariant: a fee bump is attempted for `txINSUFFICIENT_FEE` failures only.
/// Every other failure is returned unchanged, and if the fee-bump attempt does
/// not succeed the *original* error is surfaced so callers keep their existing
/// diagnostics. The retry never loops, so a persistent fee spike cannot pin the
/// submitter in a retry cycle.
async fn submit_with_fee_bump<T: SubmitTransport>(
    transport: &T,
    ctx: &SorobanContext,
    inner: TransactionV1Envelope,
    inner_xdr: &str,
    inner_hash: &str,
    inner_fee: i64,
) -> Result<InvocationOutcome, StellarSubmitError> {
    match transport.submit_once(ctx, inner_xdr, inner_hash).await {
        Ok(outcome) => Ok(outcome),
        Err(original) => {
            if !is_insufficient_fee_error(&original) {
                return Err(original);
            }

            warn!(
                tx_hash = %inner_hash,
                "Soroban submission rejected for insufficient fee; retrying once with a fee bump"
            );

            let (bump_envelope, bump_hash) = match sign_fee_bump(ctx, inner, inner_fee) {
                Ok(bumped) => bumped,
                Err(error) => {
                    debug!(%error, "could not build fee-bump transaction; returning original error");
                    return Err(original);
                }
            };
            let bump_xdr = match bump_envelope.to_xdr_base64(Limits::none()) {
                Ok(xdr) => xdr,
                Err(error) => {
                    debug!(%error, "could not encode fee-bump transaction; returning original error");
                    return Err(original);
                }
            };

            match transport.submit_once(ctx, &bump_xdr, &bump_hash).await {
                Ok(outcome) => Ok(outcome),
                Err(retry_error) => {
                    debug!(
                        %retry_error,
                        "fee-bump retry failed; returning the original submission error"
                    );
                    Err(original)
                }
            }
        }
    }
}

/// Signs `inner` with a fee-bump transaction whose total fee is
/// [`fee_bump_fee`] stroops, reusing the configured signer as the fee source.
/// Returns the fee-bump envelope and its on-chain transaction hash.
fn sign_fee_bump(
    ctx: &SorobanContext,
    inner: TransactionV1Envelope,
    inner_fee: i64,
) -> Result<(TransactionEnvelope, String), StellarSubmitError> {
    let fee_bump = FeeBumpTransaction {
        fee_source: MuxedAccount::Ed25519(Uint256(ctx.public_key)),
        fee: fee_bump_fee(inner_fee),
        inner_tx: FeeBumpTransactionInnerTx::Tx(inner),
        ext: FeeBumpTransactionExt::V0,
    };

    let payload = TransactionSignaturePayload {
        network_id: ctx.network_id.clone(),
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::TxFeeBump(
            fee_bump.clone(),
        ),
    };
    let encoded = payload
        .to_xdr(Limits::none())
        .map_err(|e| StellarSubmitError::Xdr(e.to_string()))?;

    let tx_hash: [u8; 32] = Sha256::digest(&encoded).into();
    let signature = ctx.signing_key.sign(&tx_hash);

    let decorated = DecoratedSignature {
        hint: signature_hint(&ctx.public_key),
        signature: Signature(
            signature
                .to_bytes()
                .to_vec()
                .try_into()
                .map_err(|e| StellarSubmitError::Xdr(format!("signature: {e:?}")))?,
        ),
    };

    let envelope = TransactionEnvelope::TxFeeBump(FeeBumpTransactionEnvelope {
        tx: fee_bump,
        signatures: vec![decorated]
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("signatures: {e:?}")))?,
    });

    Ok((envelope, hex::encode(tx_hash)))
}

/// Total fee, in stroops, charged by a fee-bump retry: the original fee plus at
/// most [`FEE_BUMP_MAX_STROOPS`].
fn fee_bump_fee(original_fee: i64) -> i64 {
    original_fee.saturating_add(FEE_BUMP_MAX_STROOPS)
}

/// Whether `error` is the protocol's `txINSUFFICIENT_FEE` rejection — the one
/// submission failure a fee bump is able to recover from.
fn is_insufficient_fee_error(error: &StellarSubmitError) -> bool {
    match error {
        StellarSubmitError::TransactionFailed { detail, .. } => {
            transaction_result_is_insufficient_fee(detail)
        }
        StellarSubmitError::Rejected(body) => value_mentions_insufficient_fee(body),
        _ => false,
    }
}

/// Detects an insufficient-fee failure from either a human-readable detail or
/// the base64 `TransactionResult` XDR Stellar RPC returns in the failure body.
fn transaction_result_is_insufficient_fee(detail: &str) -> bool {
    if text_mentions_insufficient_fee(detail) {
        return true;
    }

    let trimmed = detail.trim();
    if result_xdr_is_insufficient_fee(trimmed) {
        return true;
    }

    extract_parenthesised(trimmed)
        .map(result_xdr_is_insufficient_fee)
        .unwrap_or(false)
}

fn result_xdr_is_insufficient_fee(xdr: &str) -> bool {
    TransactionResult::from_xdr_base64(xdr, Limits::none())
        .is_ok_and(|result| result_code_is_insufficient_fee(&result.result))
}

/// The `txINSUFFICIENT_FEE` code, whether it failed directly or inside a
/// previously-attempted fee bump.
fn result_code_is_insufficient_fee(result: &TransactionResultResult) -> bool {
    match result {
        TransactionResultResult::TxInsufficientFee => true,
        TransactionResultResult::TxFeeBumpInnerFailed(pair) => {
            matches!(
                pair.result.result,
                InnerTransactionResultResult::TxInsufficientFee
            )
        }
        _ => false,
    }
}

fn extract_parenthesised(value: &str) -> Option<&str> {
    let start = value.rfind('(')? + 1;
    let rest = value.get(start..)?;
    let end = rest.find(')')?;
    rest.get(..end)
}

fn text_mentions_insufficient_fee(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("insufficient_fee")
        || lower.contains("insufficientfee")
        || lower.contains("insufficient fee")
}

fn value_mentions_insufficient_fee(value: &Value) -> bool {
    match value {
        Value::String(text) => text_mentions_insufficient_fee(text),
        Value::Array(items) => items.iter().any(value_mentions_insufficient_fee),
        Value::Object(map) => map.values().any(value_mentions_insufficient_fee),
        _ => false,
    }
}

/// SHA-256 of the network passphrase — the network id mixed into every
/// transaction signature.
pub fn network_id(passphrase: &str) -> Hash {
    Hash(Sha256::digest(passphrase.as_bytes()).into())
}

/// The last four bytes of the signer's public key, used by validators to pick
/// the right signature out of an envelope.
pub fn signature_hint(public_key: &[u8; 32]) -> SignatureHint {
    let mut hint = [0u8; 4];
    hint.copy_from_slice(&public_key[28..32]);
    SignatureHint(hint)
}

/// Flattens every contract event a transaction emitted, across the meta
/// versions Soroban RPC may return.
pub fn contract_events(meta: &TransactionMeta) -> Vec<ContractEvent> {
    match meta {
        TransactionMeta::V3(v3) => v3
            .soroban_meta
            .as_ref()
            .map(|soroban| soroban.events.to_vec())
            .unwrap_or_default(),
        // Protocol 23 moved contract events onto the operation meta, and added
        // transaction-level (fee) events alongside them.
        TransactionMeta::V4(v4) => v4
            .operations
            .iter()
            .flat_map(|operation| operation.events.iter().cloned())
            .chain(v4.events.iter().map(|event| event.event.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

/// The invoked function's return value, when the meta carries one.
pub fn return_value(meta: &TransactionMeta) -> Option<ScVal> {
    match meta {
        TransactionMeta::V3(v3) => v3
            .soroban_meta
            .as_ref()
            .map(|soroban| soroban.return_value.clone()),
        TransactionMeta::V4(v4) => v4
            .soroban_meta
            .as_ref()
            .and_then(|soroban| soroban.return_value.clone()),
        _ => None,
    }
}

/// Finds the first event emitted by `contract` whose leading topics match
/// `topics`, e.g. `["INHERIT", "TRIGGER"]`.
pub fn find_event<'a>(
    events: &'a [ContractEvent],
    contract: &ContractId,
    topics: &[&str],
) -> Option<&'a ContractEvent> {
    events.iter().find(|event| {
        if event.contract_id.as_ref() != Some(contract) {
            return false;
        }

        let ContractEventBody::V0(body) = &event.body;
        if body.topics.len() < topics.len() {
            return false;
        }

        body.topics
            .iter()
            .zip(topics)
            .all(|(topic, expected)| matches_symbol(topic, expected))
    })
}

/// Reads a `u64` field out of a Soroban struct event payload, which is encoded
/// as a map keyed by the field names.
pub fn event_u64_field(event: &ContractEvent, field: &str) -> Option<u64> {
    let ContractEventBody::V0(body) = &event.body;
    let ScVal::Map(Some(map)) = &body.data else {
        return None;
    };

    map.0.iter().find_map(|entry| {
        if !matches_symbol(&entry.key, field) {
            return None;
        }
        match entry.val {
            ScVal::U64(value) => Some(value),
            ScVal::U32(value) => Some(u64::from(value)),
            _ => None,
        }
    })
}

fn matches_symbol(value: &ScVal, expected: &str) -> bool {
    matches!(value, ScVal::Symbol(symbol) if symbol.0.as_vec().as_slice() == expected.as_bytes())
}

fn signer_account_scval(ctx: &SorobanContext) -> ScVal {
    ScVal::Address(ScAddress::Account(stellar_xdr::AccountId(
        stellar_xdr::PublicKey::PublicKeyTypeEd25519(Uint256(ctx.public_key)),
    )))
}

/// Outstanding loaned amount still encumbering the vault, derived from a
/// `get_inheritance_trigger` return value. `None` means the plan has not
/// been triggered (the contract returns a Soroban `Option::None`).
pub fn parse_outstanding_loaned(value: &ScVal) -> Option<u64> {
    let inner = unwrap_option_scval(value)?;
    let original = scval_u64_field(inner, "original_loaned")?;
    let recalled = scval_u64_field(inner, "recalled_amount").unwrap_or(0);
    let settled = scval_u64_field(inner, "settled_amount").unwrap_or(0);
    let liquidation = scval_bool_field(inner, "liquidation_triggered").unwrap_or(false);
    if liquidation {
        return Some(0);
    }
    Some(original.saturating_sub(recalled).saturating_sub(settled))
}

fn unwrap_option_scval(value: &ScVal) -> Option<&ScVal> {
    match value {
        ScVal::Void => None,
        ScVal::Vec(Some(vec)) => {
            let items = vec.as_slice();
            if items.is_empty() {
                return None;
            }
            if matches_symbol(&items[0], "None") {
                return None;
            }
            if matches_symbol(&items[0], "Some") {
                return items.get(1);
            }
            Some(value)
        }
        ScVal::Map(Some(_)) => Some(value),
        _ => Some(value),
    }
}

fn scval_map(value: &ScVal) -> Option<&stellar_xdr::ScMap> {
    match value {
        ScVal::Map(Some(map)) => Some(map),
        _ => None,
    }
}

pub fn scval_u64_field(value: &ScVal, field: &str) -> Option<u64> {
    let map = scval_map(value)?;
    map.0.iter().find_map(|entry| {
        if !matches_symbol(&entry.key, field) {
            return None;
        }
        match entry.val {
            ScVal::U64(v) => Some(v),
            ScVal::U32(v) => Some(u64::from(v)),
            _ => None,
        }
    })
}

pub fn scval_bool_field(value: &ScVal, field: &str) -> Option<bool> {
    let map = scval_map(value)?;
    map.0.iter().find_map(|entry| {
        if !matches_symbol(&entry.key, field) {
            return None;
        }
        match entry.val {
            ScVal::Bool(v) => Some(v),
            _ => None,
        }
    })
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use stellar_xdr::{
        ExtensionPoint, ScMap, ScMapEntry, SorobanTransactionMeta, SorobanTransactionMetaExt,
        TransactionMetaV3, TransactionResultExt,
    };

    fn symbol(value: &str) -> ScVal {
        ScVal::Symbol(ScSymbol(value.try_into().unwrap()))
    }

    fn contract(byte: u8) -> ContractId {
        ContractId(Hash([byte; 32]))
    }

    fn triggered_event(contract_id: ContractId, plan_id: u64) -> ContractEvent {
        ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: Some(contract_id),
            type_: stellar_xdr::ContractEventType::Contract,
            body: ContractEventBody::V0(stellar_xdr::ContractEventV0 {
                topics: vec![symbol("INHERIT"), symbol("TRIGGER")]
                    .try_into()
                    .unwrap(),
                data: ScVal::Map(Some(ScMap(
                    vec![
                        ScMapEntry {
                            key: symbol("outstanding_loans"),
                            val: ScVal::I128(stellar_xdr::Int128Parts { hi: 0, lo: 0 }),
                        },
                        ScMapEntry {
                            key: symbol("plan_id"),
                            val: ScVal::U64(plan_id),
                        },
                        ScMapEntry {
                            key: symbol("triggered_at"),
                            val: ScVal::U64(1_700_000_000),
                        },
                    ]
                    .try_into()
                    .unwrap(),
                ))),
            }),
        }
    }

    fn meta_v3(events: Vec<ContractEvent>) -> TransactionMeta {
        TransactionMeta::V3(TransactionMetaV3 {
            ext: ExtensionPoint::V0,
            tx_changes_before: vec![].try_into().unwrap(),
            operations: vec![].try_into().unwrap(),
            tx_changes_after: vec![].try_into().unwrap(),
            soroban_meta: Some(SorobanTransactionMeta {
                ext: SorobanTransactionMetaExt::V0,
                events: events.try_into().unwrap(),
                return_value: ScVal::Void,
                diagnostic_events: vec![].try_into().unwrap(),
            }),
        })
    }

    #[test]
    fn network_id_matches_the_published_testnet_hash() {
        // Well-known SHA-256 of the testnet passphrase.
        assert_eq!(
            hex::encode(network_id(TESTNET_PASSPHRASE).0),
            "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd472"
        );
    }

    #[test]
    fn signature_hint_is_the_public_key_suffix() {
        let mut public_key = [0u8; 32];
        public_key[28..32].copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(signature_hint(&public_key), SignatureHint([1, 2, 3, 4]));
    }

    #[test]
    fn extracts_contract_events_from_v3_meta() {
        let meta = meta_v3(vec![triggered_event(contract(1), 42)]);
        assert_eq!(contract_events(&meta).len(), 1);
    }

    #[test]
    fn finds_the_trigger_event_and_reads_the_plan_id() {
        let events = vec![triggered_event(contract(1), 42)];
        let event = find_event(&events, &contract(1), &["INHERIT", "TRIGGER"]).unwrap();
        assert_eq!(event_u64_field(event, "plan_id"), Some(42));
    }

    #[test]
    fn ignores_events_from_a_different_contract() {
        let events = vec![triggered_event(contract(9), 42)];
        assert!(find_event(&events, &contract(1), &["INHERIT", "TRIGGER"]).is_none());
    }

    #[test]
    fn ignores_events_with_different_topics() {
        let events = vec![triggered_event(contract(1), 42)];
        assert!(find_event(&events, &contract(1), &["LOAN", "FREEZE"]).is_none());
    }

    #[test]
    fn missing_event_fields_read_as_none() {
        let events = vec![triggered_event(contract(1), 42)];
        let event = find_event(&events, &contract(1), &["INHERIT", "TRIGGER"]).unwrap();
        assert_eq!(event_u64_field(event, "nope"), None);
        // `outstanding_loans` is an i128, not a u64 — it must not be coerced.
        assert_eq!(event_u64_field(event, "outstanding_loans"), None);
    }

    #[test]
    fn a_client_without_soroban_config_refuses_to_invoke() {
        let client = StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string());
        assert!(!client.soroban_enabled());
        assert!(client.contract_id().is_none());
    }

    fn test_config() -> SorobanConfig {
        SorobanConfig {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            contract_id: stellar_strkey::Contract([2u8; 32]).to_string(),
            network_passphrase: TESTNET_PASSPHRASE.to_string(),
            signer_secret: stellar_strkey::ed25519::PrivateKey([3u8; 32]).to_string(),
            poll_interval: Duration::from_millis(10),
            poll_timeout: Duration::from_secs(1),
        }
    }

    #[test]
    fn with_soroban_rejects_an_invalid_secret() {
        let client = StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string());
        let error = client
            .with_soroban(SorobanConfig {
                signer_secret: "not-a-secret".to_string(),
                ..test_config()
            })
            .err()
            .expect("invalid secret must be rejected");
        assert!(matches!(error, StellarSubmitError::Config(_)));
    }

    #[test]
    fn with_soroban_rejects_an_invalid_contract_id() {
        let client = StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string());
        let error = client
            .with_soroban(SorobanConfig {
                // An account strkey where a contract strkey is required.
                contract_id: stellar_strkey::ed25519::PublicKey([2u8; 32]).to_string(),
                ..test_config()
            })
            .err()
            .expect("invalid contract id must be rejected");
        assert!(matches!(error, StellarSubmitError::Config(_)));
    }

    #[test]
    fn valid_config_derives_the_source_account_from_the_secret() {
        let config = test_config();
        let contract_id = config.contract_id.clone();
        let client = StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string())
            .with_soroban(config)
            .expect("valid config");

        assert!(client.soroban_enabled());
        assert_eq!(client.contract_id(), Some(contract_id.as_str()));

        let source = client.source_account().expect("source account");
        let expected = stellar_strkey::ed25519::PublicKey(
            SigningKey::from_bytes(&[3u8; 32])
                .verifying_key()
                .to_bytes(),
        )
        .to_string();
        assert_eq!(source, expected);
    }

    fn trigger_info_map(
        original_loaned: u64,
        recalled_amount: u64,
        settled_amount: u64,
        liquidation_triggered: bool,
    ) -> ScVal {
        ScVal::Map(Some(ScMap(
            vec![
                ScMapEntry {
                    key: symbol("original_loaned"),
                    val: ScVal::U64(original_loaned),
                },
                ScMapEntry {
                    key: symbol("recalled_amount"),
                    val: ScVal::U64(recalled_amount),
                },
                ScMapEntry {
                    key: symbol("settled_amount"),
                    val: ScVal::U64(settled_amount),
                },
                ScMapEntry {
                    key: symbol("liquidation_triggered"),
                    val: ScVal::Bool(liquidation_triggered),
                },
            ]
            .try_into()
            .unwrap(),
        )))
    }

    fn loan_event(
        contract_id: ContractId,
        topics: [&str; 2],
        fields: Vec<(&str, u64)>,
    ) -> ContractEvent {
        ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: Some(contract_id),
            type_: stellar_xdr::ContractEventType::Contract,
            body: ContractEventBody::V0(stellar_xdr::ContractEventV0 {
                topics: vec![symbol(topics[0]), symbol(topics[1])]
                    .try_into()
                    .unwrap(),
                data: ScVal::Map(Some(ScMap(
                    fields
                        .into_iter()
                        .map(|(key, val)| ScMapEntry {
                            key: symbol(key),
                            val: ScVal::U64(val),
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ))),
            }),
        }
    }

    #[test]
    fn parse_outstanding_loaned_treats_void_as_not_triggered() {
        assert_eq!(parse_outstanding_loaned(&ScVal::Void), None);
    }

    #[test]
    fn parse_outstanding_loaned_subtracts_recalled_and_settled() {
        let value = trigger_info_map(50_000, 30_000, 0, false);
        assert_eq!(parse_outstanding_loaned(&value), Some(20_000));
    }

    #[test]
    fn parse_outstanding_loaned_is_zero_after_liquidation() {
        let value = trigger_info_map(40_000, 10_000, 30_000, true);
        assert_eq!(parse_outstanding_loaned(&value), Some(0));
    }

    #[test]
    fn finds_loan_freeze_recall_and_liquidate_events() {
        let events = vec![
            loan_event(
                contract(1),
                ["LOAN", "FREEZE"],
                vec![("plan_id", 7), ("frozen_at", 1)],
            ),
            loan_event(
                contract(1),
                ["LOAN", "RECALL"],
                vec![
                    ("plan_id", 7),
                    ("recalled_amount", 1_000),
                    ("remaining_loaned", 500),
                ],
            ),
            loan_event(
                contract(1),
                ["LOAN", "LIQUIDAT"],
                vec![
                    ("plan_id", 7),
                    ("settled_amount", 500),
                    ("claimable_amount", 9_500),
                ],
            ),
        ];

        let freeze = find_event(&events, &contract(1), &["LOAN", "FREEZE"]).unwrap();
        assert_eq!(event_u64_field(freeze, "plan_id"), Some(7));

        let recall = find_event(&events, &contract(1), &["LOAN", "RECALL"]).unwrap();
        assert_eq!(event_u64_field(recall, "recalled_amount"), Some(1_000));
        assert_eq!(event_u64_field(recall, "remaining_loaned"), Some(500));

        let liquidate = find_event(&events, &contract(1), &["LOAN", "LIQUIDAT"]).unwrap();
        assert_eq!(event_u64_field(liquidate, "settled_amount"), Some(500));
    }

    // ── Fee-bump retry ──────────────────────────────────────────────────────

    fn test_client() -> StellarSubmitClient {
        StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string())
            .with_soroban(test_config())
            .expect("valid config")
    }

    fn sample_inner(client: &StellarSubmitClient, fee: i64) -> TransactionV1Envelope {
        let ctx = client.soroban().expect("soroban configured");
        let invocation = InvokeContractArgs {
            contract_address: ScAddress::Contract(ctx.contract.clone()),
            function_name: ScSymbol("noop".try_into().unwrap()),
            args: VecM::default(),
        };
        let transaction = build_transaction(
            ctx,
            1,
            1_700_000_000,
            fee,
            invocation,
            VecM::default(),
            TransactionExt::V0,
        )
        .expect("build transaction");
        sign(ctx, transaction).expect("sign transaction").0
    }

    fn encode_inner(inner: &TransactionV1Envelope) -> String {
        TransactionEnvelope::Tx(inner.clone())
            .to_xdr_base64(Limits::none())
            .expect("encode inner envelope")
    }

    fn outcome(hash: &str) -> InvocationOutcome {
        InvocationOutcome {
            tx_hash: hash.to_string(),
            events: Vec::new(),
            return_value: None,
        }
    }

    fn insufficient_fee_error() -> StellarSubmitError {
        StellarSubmitError::TransactionFailed {
            hash: "original".to_string(),
            detail: "txINSUFFICIENT_FEE".to_string(),
        }
    }

    struct MockTransport {
        replies: Mutex<VecDeque<Result<InvocationOutcome, StellarSubmitError>>>,
        submitted: Mutex<Vec<(String, String)>>,
    }

    impl MockTransport {
        fn new(replies: Vec<Result<InvocationOutcome, StellarSubmitError>>) -> Self {
            Self {
                replies: Mutex::new(replies.into()),
                submitted: Mutex::new(Vec::new()),
            }
        }

        fn submitted(&self) -> Vec<(String, String)> {
            self.submitted.lock().unwrap().clone()
        }
    }

    impl SubmitTransport for MockTransport {
        fn submit_once<'a>(
            &'a self,
            _ctx: &'a SorobanContext,
            envelope_xdr: &'a str,
            expected_hash: &'a str,
        ) -> TransportFuture<'a, Result<InvocationOutcome, StellarSubmitError>> {
            self.submitted
                .lock()
                .unwrap()
                .push((envelope_xdr.to_string(), expected_hash.to_string()));
            let reply = self.replies.lock().unwrap().pop_front().unwrap_or_else(|| {
                Err(StellarSubmitError::Rpc(
                    "mock transport exhausted".to_string(),
                ))
            });
            Box::pin(async move { reply })
        }
    }

    #[test]
    fn detects_insufficient_fee_from_text_and_result_xdr() {
        assert!(is_insufficient_fee_error(&insufficient_fee_error()));

        assert!(!is_insufficient_fee_error(
            &StellarSubmitError::TransactionFailed {
                hash: "h".to_string(),
                detail: "txBAD_SEQ".to_string(),
            }
        ));

        // A real TransactionResult XDR encoding `txINSUFFICIENT_FEE`, as
        // getTransaction returns in its `resultXdr` field.
        let result = TransactionResult {
            fee_charged: 0,
            result: TransactionResultResult::TxInsufficientFee,
            ext: TransactionResultExt::V0,
        };
        let xdr = result.to_xdr_base64(Limits::none()).unwrap();
        assert!(is_insufficient_fee_error(
            &StellarSubmitError::TransactionFailed {
                hash: "h".to_string(),
                detail: xdr,
            }
        ));
    }

    #[tokio::test]
    async fn successful_submission_is_returned_unchanged() {
        let client = test_client();
        let ctx = client.soroban().unwrap();
        let inner = sample_inner(&client, 500);
        let inner_xdr = encode_inner(&inner);
        let transport = MockTransport::new(vec![Ok(outcome("ok"))]);

        let result = submit_with_fee_bump(&transport, ctx, inner, &inner_xdr, "hash", 500)
            .await
            .expect("submission succeeds");

        assert_eq!(result.tx_hash, "ok");
        let submitted = transport.submitted();
        assert_eq!(submitted.len(), 1);
        assert_eq!(submitted[0].1, "hash");
    }

    #[tokio::test]
    async fn insufficient_fee_is_retried_with_a_bounded_fee_bump() {
        let client = test_client();
        let ctx = client.soroban().unwrap();
        let inner = sample_inner(&client, 500);
        let inner_xdr = encode_inner(&inner);
        let transport =
            MockTransport::new(vec![Err(insufficient_fee_error()), Ok(outcome("bumped"))]);

        let result = submit_with_fee_bump(&transport, ctx, inner, &inner_xdr, "inner-hash", 500)
            .await
            .expect("fee-bump retry succeeds");
        assert_eq!(result.tx_hash, "bumped");

        let submitted = transport.submitted();
        assert_eq!(
            submitted.len(),
            2,
            "there must be one original attempt and one fee-bump retry"
        );

        let bump = TransactionEnvelope::from_xdr_base64(submitted[1].0.as_str(), Limits::none())
            .expect("fee-bump envelope decodes");
        let TransactionEnvelope::TxFeeBump(envelope) = bump else {
            panic!("the retry must submit a fee-bump transaction");
        };
        assert_eq!(envelope.tx.fee, 500 + FEE_BUMP_MAX_STROOPS);
        assert!(
            envelope.tx.fee <= 500 + FEE_BUMP_MAX_STROOPS,
            "the bump may add at most {FEE_BUMP_MAX_STROOPS} stroops"
        );

        let inner_envelope = match envelope.tx.inner_tx {
            FeeBumpTransactionInnerTx::Tx(inner) => inner,
        };
        assert_eq!(
            inner_envelope.tx.fee, 500,
            "the fee bump must wrap the original transaction unchanged"
        );
    }

    #[tokio::test]
    async fn non_fee_failure_is_returned_without_a_fee_bump() {
        let client = test_client();
        let ctx = client.soroban().unwrap();
        let inner = sample_inner(&client, 500);
        let inner_xdr = encode_inner(&inner);
        let transport = MockTransport::new(vec![Err(StellarSubmitError::TransactionFailed {
            hash: "h".to_string(),
            detail: "txBAD_SEQ".to_string(),
        })]);

        let error = submit_with_fee_bump(&transport, ctx, inner, &inner_xdr, "h", 500)
            .await
            .expect_err("a non-fee failure propagates");

        match error {
            StellarSubmitError::TransactionFailed { detail, .. } => assert_eq!(detail, "txBAD_SEQ"),
            other => panic!("unexpected error: {other}"),
        }
        assert_eq!(
            transport.submitted().len(),
            1,
            "no fee bump may be attempted for a non-fee failure"
        );
    }

    #[tokio::test]
    async fn failed_fee_bump_surfaces_the_original_error() {
        let client = test_client();
        let ctx = client.soroban().unwrap();
        let inner = sample_inner(&client, 500);
        let inner_xdr = encode_inner(&inner);
        let transport = MockTransport::new(vec![
            Err(insufficient_fee_error()),
            Err(StellarSubmitError::Network(
                "retry network down".to_string(),
            )),
        ]);

        let error = submit_with_fee_bump(&transport, ctx, inner, &inner_xdr, "h", 500)
            .await
            .expect_err("a failed retry surfaces the original error");

        match error {
            StellarSubmitError::TransactionFailed { detail, .. } => {
                assert_eq!(detail, "txINSUFFICIENT_FEE");
            }
            other => panic!("expected the original error, got {other}"),
        }
        assert_eq!(
            transport.submitted().len(),
            2,
            "the fee bump must have been attempted"
        );
    }
}
