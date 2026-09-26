//! Contract error codes are stable wire values. The SDK limits each exported
//! error specification to 50 cases, so the specification is split into two
//! non-overlapping groups while Rust callers use one macro-derived error type.

macro_rules! inheritance_errors {
    (
        existing { $( $old:ident = $old_code:tt, )* }
        additional { $( $new:ident = $new_code:tt, )* }
    ) => {
        #[soroban_sdk::contracterror(export = false)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[repr(u32)]
        pub enum InheritanceError {
            $( $old = $old_code, )*
            $( $new = $new_code, )*
        }

        // Preserve the original specification's name and codes for existing clients.
        // The runtime enum above owns conversions; these types only export metadata.
        #[allow(dead_code)]
        mod specification {
            #[soroban_sdk::contracterror]
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum InheritanceError { $( $old = $old_code, )* }

            #[soroban_sdk::contracterror]
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum InheritanceErrorExtension { $( $new = $new_code, )* }
        }
    };
}

inheritance_errors! {
    existing {
        InvalidAssetType = 1,
        InvalidTotalAmount = 2,
        MissingRequiredField = 3,
        TooManyBeneficiaries = 4,
        InvalidClaimCode = 5,
        AllocationPercentageMismatch = 6,
        DescriptionTooLong = 7,
        InvalidBeneficiaryData = 8,
        Unauthorized = 9,
        PlanNotFound = 10,
        InvalidBeneficiaryIndex = 11,
        ZkProofRequired = 12,
        InvalidAllocation = 13,
        InvalidClaimCodeRange = 14,
        ClaimNotAllowedYet = 15,
        AlreadyClaimed = 16,
        BeneficiaryNotFound = 17,
        PlanAlreadyDeactivated = 18,
        PlanNotActive = 19,
        AdminNotSet = 20,
        AdminAlreadyInitialized = 21,
        NotAdmin = 22,
        KycNotSubmitted = 23,
        PlanNotClaimed = 27,
        KycAlreadyRejected = 28,
        InsufficientBalance = 29,
        FeeTransferFailed = 30,
        InsufficientLiquidity = 31,
        InheritanceAlreadyTriggered = 32,
        EmergencyCooldownActive = 33,
        VestingScheduleActive = 34,
        NothingToClaim = 35,
        EmergencyAccessAlreadyActive = 36,
        InvalidGuardianThreshold = 37,
        EmergencyContactAlreadyExists = 38,
        TooManyEmergencyContacts = 39,
        EmergencyContactNotFound = 40,
        GuardianNotFound = 41,
        AlreadyApproved = 42,
        InheritanceNotTriggered = 43,
        NoOutstandingLoans = 44,
        LoanRecallFailed = 45,
        WillHashAlreadyStored = 46,
        VaultNotFound = 47,
        WillAlreadyLinked = 48,
        WillAlreadyFinalized = 49,
        WillVersionNotFound = 50,
        ReentrantCall = 51,
        Blk = 52,
        NotWhitelisted = 53,
    }
    additional {
        PlanNotExpired = 24,
        DisputeActive = 25,
        ContractPaused = 26,
        IncompatibleVersion = 54,
    }
}

pub type Error = InheritanceError;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::xdr::{Limits, ReadXdr, ScErrorCode, ScErrorType, ScSpecEntry};

    #[test]
    fn error_specs_cover_every_wire_code_without_collisions() {
        let mut seen = 0u64;
        for bytes in [
            specification::InheritanceError::spec_xdr().as_slice(),
            specification::InheritanceErrorExtension::spec_xdr().as_slice(),
        ] {
            let entry = ScSpecEntry::from_xdr(bytes, Limits::none()).unwrap();
            let ScSpecEntry::UdtErrorEnumV0(definition) = entry else {
                panic!("expected error spec")
            };
            assert!(definition.cases.len() <= 50);
            for case in definition.cases.iter() {
                let bit = 1u64 << case.value;
                assert_eq!(seen & bit, 0, "duplicate contract error code");
                seen |= bit;
                let wire = soroban_sdk::Error::from_contract_error(case.value);
                let error = InheritanceError::try_from(wire).unwrap();
                assert_eq!(error as u32, case.value);
                assert_eq!(soroban_sdk::Error::from(error), wire);
            }
        }
        // All legacy values 1..=53, the gaps 24..=26, and new code 54.
        assert_eq!(seen, ((1u64 << 55) - 1) & !1);
    }

    #[test]
    fn error_conversion_rejects_unknown_codes_and_host_errors() {
        let unknown = soroban_sdk::Error::from_contract_error(999);
        assert_eq!(InheritanceError::try_from(unknown), Err(unknown));
        let host =
            soroban_sdk::Error::from_type_and_code(ScErrorType::Value, ScErrorCode::InvalidInput);
        assert_eq!(InheritanceError::try_from(host), Err(host));
    }
}
