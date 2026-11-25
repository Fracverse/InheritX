//! Lending-pool API (issue #1270).
//!
//! `frontend/app/lib/api/lending.ts` targets pool-state, user-share, deposit
//! and withdrawal routes. Reads are served straight from the deployed lending
//! contract through a Soroban simulation; writes relay an already-signed
//! wallet transaction after checking that it actually invokes `deposit` /
//! `withdraw` on our configured contract for the caller's own wallet.

use crate::api::AppState;
use crate::auth::UserContext;
use crate::stellar_submit::{scval_bool_field, scval_u64_field, StellarSubmitClient};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use stellar_xdr::{
    ContractId, Hash, HostFunction, InvokeContractArgs, OperationBody, ScAddress, ScSymbol, ScVal,
    TransactionEnvelope,
};
use thiserror::Error;
use uuid::Uuid;

/// Base units in the lending contract are `u64`, so amounts in requests and
/// responses are decimal strings of those units.
#[derive(Debug, Clone, Error)]
pub enum LendingError {
    #[error("lending contract is not configured (set SOROBAN_RPC_URL, STELLAR_SIGNER_SECRET and LENDING_CONTRACT_ID)")]
    NotConfigured,
    #[error("lending asset is not configured (set LENDING_ASSET_ADDRESS)")]
    AssetNotConfigured,
    #[error("lending contract call failed: {0}")]
    Contract(String),
    #[error("lending state is unavailable: {0}")]
    Unavailable(String),
}

impl LendingError {
    fn response(&self) -> Response {
        let (status, message) = match self {
            LendingError::NotConfigured | LendingError::AssetNotConfigured => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            LendingError::Contract(detail) if detail.contains("AssetNotSupported") => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            LendingError::Contract(detail) if detail.contains("PoolPaused") => {
                (StatusCode::CONFLICT, self.to_string())
            }
            LendingError::Unavailable(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            LendingError::Contract(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Authoritative pool values decoded from `get_pool_state(asset)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSnapshot {
    pub total_deposits: u64,
    pub total_shares: u64,
    pub total_borrowed: u64,
    pub base_rate_bps: u32,
    pub multiplier_bps: u32,
    pub reserve_factor_bps: u32,
    pub is_paused: bool,
}

impl PoolSnapshot {
    /// Mirrors `shares_for_deposit` in the lending contract.
    pub fn shares_for_deposit(&self, amount: u64) -> u64 {
        if self.total_shares == 0 || self.total_deposits == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(self.total_shares as u128)
                .and_then(|value| value.checked_div(self.total_deposits as u128))
                .unwrap_or(0) as u64
        }
    }

    /// Mirrors `assets_for_shares` in the lending contract.
    pub fn assets_for_shares(&self, shares: u64) -> u64 {
        if self.total_shares == 0 {
            0
        } else {
            (shares as u128)
                .checked_mul(self.total_deposits as u128)
                .and_then(|value| value.checked_div(self.total_shares as u128))
                .unwrap_or(0) as u64
        }
    }

    /// Mirrors `get_utilization_bps`: borrows / (cash + borrows).
    pub fn utilization_bps(&self) -> u32 {
        let cash = self.total_deposits.saturating_sub(self.total_borrowed);
        let denominator = (cash as u128).saturating_add(self.total_borrowed as u128);
        if denominator == 0 {
            return 0;
        }
        ((self.total_borrowed as u128)
            .saturating_mul(10_000)
            .checked_div(denominator)
            .unwrap_or(0)
            .min(10_000)) as u32
    }

    /// Borrow rate for the legacy linear rate model, matching the contract's
    /// fallback when no `RateModel` is configured.
    pub fn borrow_rate_bps(&self) -> u32 {
        let variable =
            (self.utilization_bps() as u64).saturating_mul(self.multiplier_bps as u64) / 10_000;
        self.base_rate_bps.saturating_add(variable as u32)
    }

    /// `supply_rate = borrow_rate × utilization × (1 − reserve_factor)`.
    pub fn supply_rate_bps(&self) -> u32 {
        let supply = (self.borrow_rate_bps() as u128)
            .saturating_mul(self.utilization_bps() as u128)
            .saturating_mul((10_000u32.saturating_sub(self.reserve_factor_bps)) as u128)
            / (10_000u128 * 10_000u128);
        supply.min(u32::MAX as u128) as u32
    }
}

/// Decode the `PoolState` struct returned by the contract. Structs are encoded
/// as symbol-keyed maps; a missing field means the returned value was not a
/// pool state at all.
pub fn parse_pool_state(value: &ScVal) -> Option<PoolSnapshot> {
    Some(PoolSnapshot {
        total_deposits: scval_u64_field(value, "total_deposits")?,
        total_shares: scval_u64_field(value, "total_shares")?,
        total_borrowed: scval_u64_field(value, "total_borrowed")?,
        base_rate_bps: scval_u64_field(value, "base_rate_bps")? as u32,
        multiplier_bps: scval_u64_field(value, "multiplier_bps")? as u32,
        reserve_factor_bps: scval_u64_field(value, "reserve_factor_bps")? as u32,
        is_paused: scval_bool_field(value, "is_paused").unwrap_or(false),
    })
}

/// `C…`/`G…` strkey → Soroban address `ScVal`.
pub fn address_scval(address: &str) -> Option<ScVal> {
    if let Ok(contract) = stellar_strkey::Contract::from_string(address) {
        return Some(ScVal::Address(ScAddress::Contract(ContractId(Hash(
            contract.0,
        )))));
    }
    let public_key = stellar_strkey::ed25519::PublicKey::from_string(address).ok()?;
    Some(ScVal::Address(ScAddress::Account(
        stellar_xdr::AccountId(stellar_xdr::PublicKey::PublicKeyTypeEd25519(
            stellar_xdr::Uint256(public_key.0),
        )),
    )))
}

fn strkey_of_scval(value: &ScVal) -> Option<String> {
    match value {
        ScVal::Address(ScAddress::Contract(ContractId(Hash(bytes)))) => {
            Some(stellar_strkey::Contract(*bytes).to_string())
        }
        ScVal::Address(ScAddress::Account(account)) => match &account.0 {
            stellar_xdr::PublicKey::PublicKeyTypeEd25519(key) => {
                Some(stellar_strkey::ed25519::PublicKey(key.0).to_string())
            }
        },
        _ => None,
    }
}

fn amount_of_scval(value: &ScVal) -> Option<u64> {
    match value {
        ScVal::U64(value) => Some(*value),
        ScVal::U32(value) => Some(u64::from(*value)),
        ScVal::I128(stellar_xdr::Int128Parts { hi, lo }) => {
            if *hi != 0 {
                return None;
            }
            Some(*lo)
        }
        _ => None,
    }
}

/// A validated view of the lending call carried by a client-signed envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LendingInvocation {
    pub depositor: String,
    pub asset: String,
    pub amount: u64,
}

/// Checks that `xdr` contains exactly one Soroban invocation of `action`
/// (`deposit` / `withdraw`) on `contract_id`, and returns its arguments.
pub fn inspect_lending_invocation(
    xdr: &str,
    contract_id: &str,
    action: &str,
) -> Result<LendingInvocation, String> {
    let envelope = crate::xdr::validate_transaction_xdr(xdr).map_err(|e| e.to_string())?;
    let operations = match &envelope {
        TransactionEnvelope::TxV0(v0) => v0.tx.operations.as_slice(),
        TransactionEnvelope::Tx(v1) => v1.tx.operations.as_slice(),
        TransactionEnvelope::TxFeeBump(fee_bump) => {
            let stellar_xdr::FeeBumpTransactionInnerTx::Tx(inner) = &fee_bump.tx.inner_tx;
            inner.tx.operations.as_slice()
        }
    };

    if operations.len() != 1 {
        return Err("signed transaction must contain exactly one operation".to_string());
    }

    let OperationBody::InvokeHostFunction(invoke) = &operations[0].body else {
        return Err("signed transaction must invoke the lending contract".to_string());
    };
    let HostFunction::InvokeContract(InvokeContractArgs {
        contract_address,
        function_name,
        args,
    }) = &invoke.host_function
    else {
        return Err("signed transaction must be a Soroban contract invocation".to_string());
    };

    let ScAddress::Contract(ContractId(Hash(bytes))) = contract_address else {
        return Err("signed transaction targets an unexpected contract".to_string());
    };
    if stellar_strkey::Contract(*bytes).to_string() != contract_id {
        return Err("signed transaction targets an unexpected contract".to_string());
    }
    let expected = ScSymbol(
        action
            .try_into()
            .map_err(|_| "invalid lending action".to_string())?,
    );
    if function_name != &expected {
        return Err(format!("signed transaction must call `{action}`"));
    }

    let args = args.as_slice();
    if args.len() != 3 {
        return Err("signed transaction has unexpected arguments".to_string());
    }

    Ok(LendingInvocation {
        depositor: strkey_of_scval(&args[0])
            .ok_or_else(|| "depositor argument is not a Stellar address".to_string())?,
        asset: strkey_of_scval(&args[1])
            .ok_or_else(|| "asset argument is not a Stellar address".to_string())?,
        amount: amount_of_scval(&args[2])
            .ok_or_else(|| "amount argument is not an unsigned integer".to_string())?,
    })
}

/// Shared read/write client for one lending contract and one asset pool.
#[derive(Clone)]
pub struct LendingClient {
    submit: StellarSubmitClient,
    contract_id: String,
    asset: String,
    asset_scval: ScVal,
}

impl LendingClient {
    pub fn new(submit: StellarSubmitClient, asset: &str) -> Result<Self, LendingError> {
        let contract_id = submit
            .contract_id()
            .ok_or(LendingError::NotConfigured)?
            .to_string();
        let asset = asset.trim().to_string();
        let asset_scval = address_scval(&asset).ok_or(LendingError::AssetNotConfigured)?;
        Ok(Self {
            submit,
            contract_id,
            asset,
            asset_scval,
        })
    }

    pub fn asset(&self) -> &str {
        &self.asset
    }

    pub async fn pool_state(&self) -> Result<PoolSnapshot, LendingError> {
        let value = self
            .submit
            .simulate_contract("get_pool_state", vec![self.asset_scval.clone()])
            .await
            .map_err(|e| match e {
                crate::stellar_submit::StellarSubmitError::NotConfigured => {
                    LendingError::NotConfigured
                }
                other => LendingError::Contract(other.to_string()),
            })?;

        parse_pool_state(&value).ok_or_else(|| {
            LendingError::Unavailable("get_pool_state returned an unknown shape".to_string())
        })
    }

    pub async fn shares_of(&self, wallet: &str) -> Result<u64, LendingError> {
        let wallet_scval = address_scval(wallet).ok_or_else(|| {
            LendingError::Contract(format!("{wallet} is not a Stellar address"))
        })?;
        let value = self
            .submit
            .simulate_contract(
                "get_shares_of",
                vec![self.asset_scval.clone(), wallet_scval],
            )
            .await
            .map_err(|e| LendingError::Contract(e.to_string()))?;
        amount_of_scval(&value)
            .ok_or_else(|| LendingError::Unavailable("get_shares_of returned a non-u64".into()))
    }

    pub async fn submit_signed(&self, xdr: &str) -> Result<serde_json::Value, LendingError> {
        self.submit
            .submit(xdr)
            .await
            .map_err(|e| LendingError::Contract(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PoolStateResponse {
    pub total_deposits: String,
    pub total_borrowed: String,
    /// Utilization as a percentage (0–100).
    pub utilization_rate: f64,
    /// Current deposit (supply) APY as a percentage.
    pub current_apy: f64,
    /// Reserve factor as a percentage.
    pub reserve_factor: f64,
}

impl From<&PoolSnapshot> for PoolStateResponse {
    fn from(snapshot: &PoolSnapshot) -> Self {
        Self {
            total_deposits: snapshot.total_deposits.to_string(),
            total_borrowed: snapshot.total_borrowed.to_string(),
            utilization_rate: snapshot.utilization_bps() as f64 / 100.0,
            current_apy: snapshot.supply_rate_bps() as f64 / 100.0,
            reserve_factor: snapshot.reserve_factor_bps as f64 / 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LendingTransactionResponse {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: String,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserLendingDataResponse {
    pub shares: String,
    pub underlying_balance: String,
    pub total_earnings: String,
    pub deposit_history: Vec<LendingTransactionResponse>,
}

fn lending_client(state: &Arc<AppState>) -> Result<&LendingClient, LendingError> {
    state.lending.as_deref().ok_or(LendingError::NotConfigured)
}

fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

fn db_error(action: &str, error: sqlx::Error) -> Response {
    tracing::error!(error = %error, "Failed to {action}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("Failed to {action}") })),
    )
        .into_response()
}

/// GET /api/lending/pool-state
pub async fn get_pool_state(State(state): State<Arc<AppState>>) -> Response {
    let client = match lending_client(&state) {
        Ok(client) => client,
        Err(error) => return error.response(),
    };
    match client.pool_state().await {
        Ok(snapshot) => Json(PoolStateResponse::from(&snapshot)).into_response(),
        Err(error) => error.response(),
    }
}

/// GET /api/lending/shares/{address}
pub async fn get_user_shares(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Response {
    if address_scval(&address).is_none() {
        return bad_request("address must be a Stellar G… or C… strkey");
    }

    let client = match lending_client(&state) {
        Ok(client) => client,
        Err(error) => return error.response(),
    };

    let (snapshot, shares) = match client.pool_state().await {
        Ok(snapshot) => match client.shares_of(&address).await {
            Ok(shares) => (snapshot, shares),
            Err(error) => return error.response(),
        },
        Err(error) => return error.response(),
    };

    let history = match sqlx::query_as::<_, LendingTransactionResponse>(
        r#"
        SELECT id,
               tx_type,
               amount::text AS amount,
               created_at   AS timestamp,
               tx_hash,
               status
        FROM lending_transactions
        WHERE wallet_address = $1 AND asset = $2
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&address)
    .bind(client.asset())
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(history) => history,
        Err(error) => return db_error("load lending history", error),
    };

    // Net contributed capital, recorded only for transactions relayed through
    // this API. Earnings are the authoritative on-chain value of the shares
    // minus that cost basis.
    let net_contributed: i64 = match sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(
            CASE WHEN tx_type = 'DEPOSIT' THEN amount ELSE -amount END
        ), 0)::BIGINT
        FROM lending_transactions
        WHERE wallet_address = $1 AND asset = $2 AND status = 'COMPLETED'
        "#,
    )
    .bind(&address)
    .bind(client.asset())
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(value) => value,
        Err(error) => return db_error("load lending cost basis", error),
    };

    let underlying = snapshot.assets_for_shares(shares);
    let earnings: i128 = if net_contributed <= 0 && history.is_empty() {
        0
    } else {
        underlying as i128 - net_contributed as i128
    };

    Json(UserLendingDataResponse {
        shares: shares.to_string(),
        underlying_balance: underlying.to_string(),
        total_earnings: earnings.to_string(),
        deposit_history: history,
    })
    .into_response()
}

/// Resolve the authenticated caller to a Stellar wallet address. Signature
/// auth identifies users by hex public key; JWTs carry the address in `sub`.
fn caller_wallet(ctx: &UserContext) -> Option<String> {
    if stellar_strkey::ed25519::PublicKey::from_string(&ctx.user_id).is_ok() {
        return Some(ctx.user_id.clone());
    }
    let bytes = hex::decode(ctx.user_id.trim_start_matches("0x")).ok()?;
    let key = <[u8; 32]>::try_from(bytes.as_slice()).ok()?;
    Some(stellar_strkey::ed25519::PublicKey(key).to_string())
}

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub amount: String,
    pub signed_transaction: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub shares: String,
    pub signed_transaction: String,
}

fn parse_u64(value: &str, label: &str) -> Result<u64, Response> {
    let parsed = value
        .trim()
        .parse::<u64>()
        .map_err(|_| bad_request(format!("{label} must be a positive integer string")))?;
    if parsed == 0 {
        return Err(bad_request(format!("{label} must be greater than zero")));
    }
    Ok(parsed)
}

/// POST /api/lending/deposit
pub async fn deposit(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(request): Json<DepositRequest>,
) -> Response {
    let amount = match parse_u64(&request.amount, "amount") {
        Ok(amount) => amount,
        Err(response) => return response,
    };
    if request.signed_transaction.trim().is_empty() {
        return bad_request("signed_transaction is required to authorise a deposit");
    }
    let Some(wallet) = caller_wallet(&ctx) else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "an authenticated wallet is required for deposits" })),
        )
            .into_response();
    };
    let client = match lending_client(&state) {
        Ok(client) => client,
        Err(error) => return error.response(),
    };

    let invocation =
        match inspect_lending_invocation(
            &request.signed_transaction,
            &client.contract_id,
            "deposit",
        ) {
            Ok(invocation) => invocation,
            Err(message) => return bad_request(message),
        };
    if invocation.depositor != wallet {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "signed transaction depositor does not match the caller" })),
        )
            .into_response();
    }
    if invocation.asset != client.asset() {
        return bad_request("signed transaction targets an unsupported asset");
    }
    if invocation.amount != amount {
        return bad_request("amount does not match the signed transaction");
    }

    // Read the pool before relaying so asset support is verified and the
    // minted-share estimate is based on the state the deposit will see.
    let snapshot = match client.pool_state().await {
        Ok(snapshot) => snapshot,
        Err(error) => return error.response(),
    };
    if snapshot.is_paused {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "lending pool for this asset is paused" })),
        )
            .into_response();
    }

    let result = match client.submit_signed(&request.signed_transaction).await {
        Ok(result) => result,
        Err(error) => return error.response(),
    };
    let Some(tx_hash) = result.get("hash").and_then(|value| value.as_str()) else {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "Stellar network did not return a transaction hash" })),
        )
            .into_response();
    };

    let shares = snapshot.shares_for_deposit(amount);
    if let Err(error) = record_transaction(
        &state,
        &wallet,
        client.asset(),
        "DEPOSIT",
        amount,
        shares,
        tx_hash,
    )
    .await
    {
        // The transaction is already on-chain; failing the request would tell
        // the client nothing new, so log and still report success.
        tracing::error!(error = %error, tx_hash, "Failed to record lending deposit");
    }

    Json(json!({ "tx_hash": tx_hash, "shares": shares.to_string(), "status": "COMPLETED" }))
        .into_response()
}

/// POST /api/lending/withdraw
pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(request): Json<WithdrawRequest>,
) -> Response {
    let shares = match parse_u64(&request.shares, "shares") {
        Ok(shares) => shares,
        Err(response) => return response,
    };
    if request.signed_transaction.trim().is_empty() {
        return bad_request("signed_transaction is required to authorise a withdrawal");
    }
    let Some(wallet) = caller_wallet(&ctx) else {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "an authenticated wallet is required for withdrawals" })),
        )
            .into_response();
    };
    let client = match lending_client(&state) {
        Ok(client) => client,
        Err(error) => return error.response(),
    };

    let invocation = match inspect_lending_invocation(
        &request.signed_transaction,
        &client.contract_id,
        "withdraw",
    ) {
        Ok(invocation) => invocation,
        Err(message) => return bad_request(message),
    };
    if invocation.depositor != wallet {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "signed transaction depositor does not match the caller" })),
        )
            .into_response();
    }
    if invocation.asset != client.asset() {
        return bad_request("signed transaction targets an unsupported asset");
    }
    if invocation.amount != shares {
        return bad_request("shares do not match the signed transaction");
    }

    let snapshot = match client.pool_state().await {
        Ok(snapshot) => snapshot,
        Err(error) => return error.response(),
    };
    if snapshot.is_paused {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "lending pool for this asset is paused" })),
        )
            .into_response();
    }

    let onchain_shares = match client.shares_of(&wallet).await {
        Ok(shares) => shares,
        Err(error) => return error.response(),
    };
    if shares > onchain_shares {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "insufficient shares for withdrawal" })),
        )
            .into_response();
    }
    let amount = snapshot.assets_for_shares(shares);
    let available = snapshot.total_deposits.saturating_sub(snapshot.total_borrowed);
    if amount > available {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "insufficient pool liquidity for this withdrawal" })),
        )
            .into_response();
    }

    let result = match client.submit_signed(&request.signed_transaction).await {
        Ok(result) => result,
        Err(error) => return error.response(),
    };
    let Some(tx_hash) = result.get("hash").and_then(|value| value.as_str()) else {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "Stellar network did not return a transaction hash" })),
        )
            .into_response();
    };

    if let Err(error) = record_transaction(
        &state,
        &wallet,
        client.asset(),
        "WITHDRAW",
        amount,
        shares,
        tx_hash,
    )
    .await
    {
        tracing::error!(error = %error, tx_hash, "Failed to record lending withdrawal");
    }

    Json(json!({ "tx_hash": tx_hash, "amount": amount.to_string(), "status": "COMPLETED" }))
        .into_response()
}

async fn record_transaction(
    state: &Arc<AppState>,
    wallet: &str,
    asset: &str,
    tx_type: &str,
    amount: u64,
    shares: u64,
    tx_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO lending_transactions
            (wallet_address, asset, tx_type, amount, shares, tx_hash, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'COMPLETED')
        ON CONFLICT (tx_hash) DO NOTHING
        "#,
    )
    .bind(wallet)
    .bind(asset)
    .bind(tx_type)
    .bind(amount as i64)
    .bind(shares as i64)
    .bind(tx_hash)
    .execute(&state.db_pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{ScMap, ScMapEntry};

    fn symbol(value: &str) -> ScVal {
        ScVal::Symbol(ScSymbol(value.try_into().unwrap()))
    }

    #[test]
    fn parses_a_contract_pool_state_map() {
        let value = ScVal::Map(Some(ScMap(
            vec![
                ScMapEntry {
                    key: symbol("total_deposits"),
                    val: ScVal::U64(10_000),
                },
                ScMapEntry {
                    key: symbol("total_shares"),
                    val: ScVal::U64(9_000),
                },
                ScMapEntry {
                    key: symbol("total_borrowed"),
                    val: ScVal::U64(5_000),
                },
                ScMapEntry {
                    key: symbol("base_rate_bps"),
                    val: ScVal::U32(100),
                },
                ScMapEntry {
                    key: symbol("multiplier_bps"),
                    val: ScVal::U32(2_000),
                },
                ScMapEntry {
                    key: symbol("reserve_factor_bps"),
                    val: ScVal::U32(1_000),
                },
                ScMapEntry {
                    key: symbol("is_paused"),
                    val: ScVal::Bool(false),
                },
            ]
            .try_into()
            .unwrap(),
        )));

        let snapshot = parse_pool_state(&value).expect("valid pool state");
        assert_eq!(snapshot.total_deposits, 10_000);
        assert_eq!(snapshot.total_shares, 9_000);
        assert_eq!(snapshot.utilization_bps(), 5_000);
        // borrow = 100 + 5000*2000/10000 = 1100 bps
        assert_eq!(snapshot.borrow_rate_bps(), 1_100);
        // supply = 1100 * 5000 * 9000 / 10000^2 = 495 bps
        assert_eq!(snapshot.supply_rate_bps(), 495);
        assert!(!snapshot.is_paused);
    }

    #[test]
    fn rejects_values_that_are_not_a_pool_state() {
        assert!(parse_pool_state(&ScVal::U64(1)).is_none());
    }

    #[test]
    fn share_conversions_match_the_contract_rounding() {
        let snapshot = PoolSnapshot {
            total_deposits: 1_000,
            total_shares: 500,
            total_borrowed: 0,
            base_rate_bps: 0,
            multiplier_bps: 0,
            reserve_factor_bps: 0,
            is_paused: false,
        };
        assert_eq!(snapshot.shares_for_deposit(100), 50);
        assert_eq!(snapshot.assets_for_shares(50), 100);
        assert_eq!(snapshot.assets_for_shares(0), 0);
    }

    #[test]
    fn empty_pool_converts_one_to_one() {
        let snapshot = PoolSnapshot {
            total_deposits: 0,
            total_shares: 0,
            total_borrowed: 0,
            base_rate_bps: 0,
            multiplier_bps: 0,
            reserve_factor_bps: 0,
            is_paused: false,
        };
        assert_eq!(snapshot.shares_for_deposit(250), 250);
        assert_eq!(snapshot.assets_for_shares(250), 0);
    }

    #[test]
    fn parses_stellar_addresses() {
        let account = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        assert!(matches!(
            address_scval(account),
            Some(ScVal::Address(ScAddress::Account(_)))
        ));
        assert!(matches!(
            address_scval(contract),
            Some(ScVal::Address(ScAddress::Contract(_)))
        ));
        assert!(address_scval("USDC").is_none());
    }

    #[test]
    fn rejects_xdr_that_is_not_a_lending_call() {
        let error = inspect_lending_invocation("not-xdr", "CAAAA", "deposit").unwrap_err();
        assert!(!error.is_empty());
    }
}
