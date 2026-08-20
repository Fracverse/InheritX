use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::{Signer, SigningKey};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use stellar_xdr::{
    ContractEvent, ContractEventBody, ContractId, DecoratedSignature, Hash, HostFunction,
    InvokeContractArgs, InvokeHostFunctionOp, Limits, Memo, MuxedAccount, Operation, OperationBody,
    Preconditions, ReadXdr, ScAddress, ScSymbol, ScVal, SequenceNumber, Signature, SignatureHint,
    SorobanAuthorizationEntry, SorobanTransactionData, TimeBounds, TimePoint, Transaction,
    TransactionEnvelope, TransactionExt, TransactionMeta, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, TransactionV1Envelope, Uint256, VecM, WriteXdr,
};
use thiserror::Error;
use tracing::{debug, warn};

/// Per-operation base fee, in stroops. The Soroban resource fee reported by
/// `simulateTransaction` is added on top of this.
const BASE_FEE_STROOPS: i64 = 100;

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
        let caller = ScVal::Address(ScAddress::Account(stellar_xdr::AccountId(
            stellar_xdr::PublicKey::PublicKeyTypeEd25519(Uint256(ctx.public_key)),
        )));

        self.invoke_contract("trigger_inheritance", vec![caller, ScVal::U64(plan_id)])
            .await
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

        // 3. Sign and submit.
        let (envelope, tx_hash) = sign(ctx, assembled)?;
        let envelope_xdr = envelope
            .to_xdr_base64(Limits::none())
            .map_err(|e| StellarSubmitError::Xdr(e.to_string()))?;

        self.send_transaction(ctx, &envelope_xdr, &tx_hash).await?;
        self.await_transaction(ctx, &tx_hash).await
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

        let auth = response
            .results
            .unwrap_or_default()
            .into_iter()
            .next()
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

/// Signs `transaction` for the configured network, returning the envelope and
/// the transaction hash (hex) that identifies it on-chain.
fn sign(
    ctx: &SorobanContext,
    transaction: Transaction,
) -> Result<(TransactionEnvelope, String), StellarSubmitError> {
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

    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: transaction,
        signatures: vec![decorated]
            .try_into()
            .map_err(|e| StellarSubmitError::Xdr(format!("signatures: {e:?}")))?,
    });

    Ok((envelope, hex::encode(tx_hash)))
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
    use stellar_xdr::{
        ExtensionPoint, ScMap, ScMapEntry, SorobanTransactionMeta, SorobanTransactionMetaExt,
        TransactionMetaV3,
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
}
