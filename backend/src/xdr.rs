use stellar_xdr::{Limits, ReadXdr, TransactionEnvelope};
use thiserror::Error;

/// Stellar's own protocol limit on the base64-encoded size of a transaction
/// envelope that Horizon/core will accept. Rejecting anything larger up
/// front avoids doing XDR decode work on obviously-oversized payloads.
const MAX_XDR_BASE64_LEN: usize = 100 * 1024;

/// Recursion depth allowed while decoding. Stellar transaction envelopes are
/// shallow structures; this is generous headroom while still bounding
/// worst-case stack usage on malicious input.
const MAX_XDR_DEPTH: u32 = 64;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum XdrValidationError {
    #[error("xdr field must not be empty")]
    Empty,
    #[error("xdr payload exceeds the maximum allowed size of {MAX_XDR_BASE64_LEN} bytes")]
    TooLarge,
    #[error("xdr is not valid base64/XDR for a Stellar transaction envelope: {0}")]
    Malformed(String),
    #[error("transaction envelope has no operations")]
    NoOperations,
    #[error("transaction envelope is unsigned")]
    Unsigned,
    #[error("transaction fee must be greater than zero")]
    InvalidFee,
}

/// Parses and validates a raw base64 Stellar `TransactionEnvelope` XDR
/// string, without submitting it anywhere. This is a pure format/shape
/// check: it confirms the payload decodes to a well-formed envelope with a
/// sane fee, at least one operation, and at least one signature. It does
/// **not** check the signature(s) are valid for the source account, that
/// the sequence number is current, or anything else that requires network
/// state — that verification happens on the Stellar network itself when the
/// envelope is submitted.
pub fn validate_transaction_xdr(xdr: &str) -> Result<TransactionEnvelope, XdrValidationError> {
    let trimmed = xdr.trim();
    if trimmed.is_empty() {
        return Err(XdrValidationError::Empty);
    }
    if trimmed.len() > MAX_XDR_BASE64_LEN {
        return Err(XdrValidationError::TooLarge);
    }

    let limits = Limits {
        depth: MAX_XDR_DEPTH,
        len: MAX_XDR_BASE64_LEN,
    };
    let envelope = TransactionEnvelope::from_xdr_base64(trimmed, limits)
        .map_err(|e| XdrValidationError::Malformed(e.to_string()))?;

    let (fee_is_positive, op_count, sig_count) = match &envelope {
        TransactionEnvelope::TxV0(v0) => {
            (v0.tx.fee > 0, v0.tx.operations.len(), v0.signatures.len())
        }
        TransactionEnvelope::Tx(v1) => (v1.tx.fee > 0, v1.tx.operations.len(), v1.signatures.len()),
        TransactionEnvelope::TxFeeBump(fee_bump) => {
            let stellar_xdr::FeeBumpTransactionInnerTx::Tx(inner) = &fee_bump.tx.inner_tx;
            (
                fee_bump.tx.fee > 0 && inner.tx.fee > 0,
                inner.tx.operations.len(),
                fee_bump.signatures.len(),
            )
        }
    };

    if !fee_is_positive {
        return Err(XdrValidationError::InvalidFee);
    }
    if op_count == 0 {
        return Err(XdrValidationError::NoOperations);
    }
    if sig_count == 0 {
        return Err(XdrValidationError::Unsigned);
    }

    Ok(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{
        DecoratedSignature, Memo, MuxedAccount, Operation, OperationBody, Preconditions,
        SequenceNumber, Signature, SignatureHint, Transaction, TransactionExt,
        TransactionV1Envelope, Uint256, WriteXdr,
    };

    fn dummy_account() -> MuxedAccount {
        MuxedAccount::Ed25519(Uint256([7u8; 32]))
    }

    fn build_envelope(
        operations: Vec<Operation>,
        signatures: Vec<DecoratedSignature>,
        fee: u32,
    ) -> TransactionEnvelope {
        let tx = Transaction {
            source_account: dummy_account(),
            fee,
            seq_num: SequenceNumber(1),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: operations.try_into().unwrap(),
            ext: TransactionExt::V0,
        };
        TransactionEnvelope::Tx(TransactionV1Envelope {
            tx,
            signatures: signatures.try_into().unwrap(),
        })
    }

    fn one_op() -> Operation {
        Operation {
            source_account: None,
            body: OperationBody::BumpSequence(stellar_xdr::BumpSequenceOp {
                bump_to: SequenceNumber(1),
            }),
        }
    }

    fn one_sig() -> DecoratedSignature {
        DecoratedSignature {
            hint: SignatureHint([0u8; 4]),
            signature: Signature(vec![0u8; 64].try_into().unwrap()),
        }
    }

    fn to_base64(envelope: &TransactionEnvelope) -> String {
        envelope.to_xdr_base64(Limits::none()).unwrap()
    }

    #[test]
    fn accepts_a_well_formed_signed_envelope() {
        let envelope = build_envelope(vec![one_op()], vec![one_sig()], 100);
        let xdr = to_base64(&envelope);
        assert_eq!(validate_transaction_xdr(&xdr).unwrap(), envelope);
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(validate_transaction_xdr(""), Err(XdrValidationError::Empty));
        assert_eq!(
            validate_transaction_xdr("   "),
            Err(XdrValidationError::Empty)
        );
    }

    #[test]
    fn rejects_garbage_base64() {
        assert!(matches!(
            validate_transaction_xdr("not-valid-xdr-at-all"),
            Err(XdrValidationError::Malformed(_))
        ));
    }

    #[test]
    fn rejects_oversized_input() {
        let huge = "A".repeat(MAX_XDR_BASE64_LEN + 1);
        assert_eq!(
            validate_transaction_xdr(&huge),
            Err(XdrValidationError::TooLarge)
        );
    }

    #[test]
    fn rejects_envelope_with_no_operations() {
        let envelope = build_envelope(vec![], vec![one_sig()], 100);
        let xdr = to_base64(&envelope);
        assert_eq!(
            validate_transaction_xdr(&xdr),
            Err(XdrValidationError::NoOperations)
        );
    }

    #[test]
    fn rejects_unsigned_envelope() {
        let envelope = build_envelope(vec![one_op()], vec![], 100);
        let xdr = to_base64(&envelope);
        assert_eq!(
            validate_transaction_xdr(&xdr),
            Err(XdrValidationError::Unsigned)
        );
    }

    #[test]
    fn rejects_zero_fee() {
        let envelope = build_envelope(vec![one_op()], vec![one_sig()], 0);
        let xdr = to_base64(&envelope);
        assert_eq!(
            validate_transaction_xdr(&xdr),
            Err(XdrValidationError::InvalidFee)
        );
    }

    #[test]
    fn allows_trailing_whitespace() {
        let envelope = build_envelope(vec![one_op()], vec![one_sig()], 100);
        let xdr = format!("  {}\n", to_base64(&envelope));
        assert_eq!(validate_transaction_xdr(&xdr).unwrap(), envelope);
    }
}
