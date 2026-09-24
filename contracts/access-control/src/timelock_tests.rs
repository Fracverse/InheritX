//! Tests for nonce revocation (Issue #1175) and the parameter time-lock
//! (Issue #1159).

#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

use crate::{
    cancel_parameter_change, execute_parameter_change, get_min_nonce, get_parameter_proposal,
    get_parameter_value, get_parameter_value_or, is_nonce_valid, is_parameter_change_ready,
    propose_parameter_change, revoke_user_nonces, TimelockFailure, PARAMETER_GRACE_SECONDS,
    PARAMETER_TIMELOCK_SECONDS,
};

/// `access-control` is a library, so its storage-touching functions need a
/// contract context to run in. This host exists only to provide one.
#[contract]
struct TestHost;

#[contractimpl]
impl TestHost {
    pub fn noop(_env: Env) {}
}

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestHost, ());
    (env, contract_id)
}

// ─── Nonce revocation (Issue #1175) ──────────────────────────────────

#[test]
fn nonces_are_all_valid_before_any_revocation() {
    let (env, contract_id) = setup();
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        assert_eq!(get_min_nonce(&env, &user), 0);
        assert!(is_nonce_valid(&env, &user, 0));
        assert!(is_nonce_valid(&env, &user, u64::MAX));
    });
}

#[test]
fn revoking_invalidates_everything_below_the_floor() {
    let (env, contract_id) = setup();
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        revoke_user_nonces(&env, &user, 100);

        assert!(!is_nonce_valid(&env, &user, 0));
        assert!(!is_nonce_valid(&env, &user, 99));
        // The floor itself is still honoured — it is the lowest *accepted*
        // nonce, not the highest rejected one.
        assert!(is_nonce_valid(&env, &user, 100));
        assert!(is_nonce_valid(&env, &user, 101));
    });
}

#[test]
fn the_floor_never_moves_down() {
    let (env, contract_id) = setup();
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        revoke_user_nonces(&env, &user, 500);
        // An attacker holding the compromised key calls this to undo the
        // revocation. It must not work, or the mechanism is worthless.
        revoke_user_nonces(&env, &user, 1);

        assert_eq!(get_min_nonce(&env, &user), 500);
        assert!(!is_nonce_valid(&env, &user, 400));
    });
}

#[test]
fn revoking_twice_at_the_same_floor_is_harmless() {
    let (env, contract_id) = setup();
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        revoke_user_nonces(&env, &user, 42);
        revoke_user_nonces(&env, &user, 42);
        assert_eq!(get_min_nonce(&env, &user), 42);
    });
}

#[test]
fn one_accounts_revocation_does_not_touch_another() {
    let (env, contract_id) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    env.as_contract(&contract_id, || {
        revoke_user_nonces(&env, &alice, 1_000);

        assert!(!is_nonce_valid(&env, &alice, 10));
        assert!(is_nonce_valid(&env, &bob, 10));
        assert_eq!(get_min_nonce(&env, &bob), 0);
    });
}

// ─── Parameter time-lock (Issue #1159) ───────────────────────────────

#[test]
fn a_proposal_matures_exactly_48_hours_later() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "fee_bp");
        let proposal = propose_parameter_change(&env, &admin, param.clone(), 250);

        assert_eq!(
            proposal.executable_at - proposal.proposed_at,
            PARAMETER_TIMELOCK_SECONDS
        );
        assert_eq!(PARAMETER_TIMELOCK_SECONDS, 172_800);
        assert!(!is_parameter_change_ready(&env, param));
    });
}

#[test]
fn executing_before_the_delay_elapses_is_refused() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "fee_bp");
        propose_parameter_change(&env, &admin, param.clone(), 250);

        assert_eq!(
            execute_parameter_change(&env, &admin, param.clone()),
            Err(TimelockFailure::TooEarly)
        );
        // One second short still counts as early.
        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS - 1);
        assert_eq!(
            execute_parameter_change(&env, &admin, param),
            Err(TimelockFailure::TooEarly)
        );
    });
}

#[test]
fn executing_after_the_delay_commits_the_value() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "fee_bp");
        propose_parameter_change(&env, &admin, param.clone(), 250);

        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS);

        assert!(is_parameter_change_ready(&env, param.clone()));
        assert_eq!(execute_parameter_change(&env, &admin, param.clone()), Ok(250));
        assert_eq!(get_parameter_value(&env, param.clone()), Some(250));
        // The proposal is consumed, so it cannot be replayed.
        assert!(get_parameter_proposal(&env, param).is_none());
    });
}

#[test]
fn a_matured_proposal_goes_stale_after_the_grace_period() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "treasury");
        propose_parameter_change(&env, &admin, param.clone(), 7);

        // Long past maturity: the notice window this proposal provided expired
        // ages ago, so executing on it now would defeat the mechanism.
        env.ledger()
            .set_timestamp(PARAMETER_TIMELOCK_SECONDS + PARAMETER_GRACE_SECONDS);

        assert!(!is_parameter_change_ready(&env, param.clone()));
        assert_eq!(
            execute_parameter_change(&env, &admin, param.clone()),
            Err(TimelockFailure::Expired)
        );
        // And it is cleared, so the parameter is not blocked by a dead entry.
        assert!(get_parameter_proposal(&env, param).is_none());
    });
}

#[test]
fn executing_a_parameter_with_no_proposal_is_refused() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        assert_eq!(
            execute_parameter_change(&env, &admin, Symbol::new(&env, "nothing")),
            Err(TimelockFailure::NotFound)
        );
    });
}

#[test]
fn a_second_proposal_replaces_the_first_and_restarts_the_clock() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "fee_bp");
        propose_parameter_change(&env, &admin, param.clone(), 250);

        env.ledger().set_timestamp(100_000);
        propose_parameter_change(&env, &admin, param.clone(), 900);

        // The clock restarts — otherwise an admin could propose something
        // innocuous, wait 48 hours, then swap in a hostile value and execute
        // immediately with no notice at all.
        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS);
        assert_eq!(
            execute_parameter_change(&env, &admin, param.clone()),
            Err(TimelockFailure::TooEarly)
        );

        env.ledger()
            .set_timestamp(100_000 + PARAMETER_TIMELOCK_SECONDS);
        assert_eq!(execute_parameter_change(&env, &admin, param), Ok(900));
    });
}

#[test]
fn cancelling_removes_a_pending_proposal() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "fee_bp");
        propose_parameter_change(&env, &admin, param.clone(), 250);

        assert!(cancel_parameter_change(&env, &admin, param.clone()));
        assert!(get_parameter_proposal(&env, param.clone()).is_none());

        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS);
        assert_eq!(
            execute_parameter_change(&env, &admin, param),
            Err(TimelockFailure::NotFound)
        );
    });
}

#[test]
fn cancelling_nothing_reports_that_it_did_nothing() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        assert!(!cancel_parameter_change(
            &env,
            &admin,
            Symbol::new(&env, "nothing")
        ));
    });
}

#[test]
fn parameters_are_independent_of_each_other() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let fee = Symbol::new(&env, "fee_bp");
        let treasury = Symbol::new(&env, "treasury");

        propose_parameter_change(&env, &admin, fee.clone(), 250);
        propose_parameter_change(&env, &admin, treasury.clone(), 7);

        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS);

        assert_eq!(execute_parameter_change(&env, &admin, fee.clone()), Ok(250));
        // Executing one must not consume the other.
        assert!(get_parameter_proposal(&env, treasury.clone()).is_some());
        assert_eq!(execute_parameter_change(&env, &admin, treasury), Ok(7));
    });
}

#[test]
fn an_unset_parameter_falls_back_to_the_compiled_in_default() {
    let (env, contract_id) = setup();

    env.as_contract(&contract_id, || {
        // This is what lets a contract adopt the time-lock without a
        // migration: the default applies until the first executed change.
        assert_eq!(
            get_parameter_value_or(&env, Symbol::new(&env, "fee_bp"), 200),
            200
        );
    });
}

#[test]
fn a_negative_value_survives_the_round_trip() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let param = Symbol::new(&env, "adjust");
        propose_parameter_change(&env, &admin, param.clone(), -1_500);

        env.ledger().set_timestamp(PARAMETER_TIMELOCK_SECONDS);
        assert_eq!(execute_parameter_change(&env, &admin, param.clone()), Ok(-1_500));
        assert_eq!(get_parameter_value(&env, param), Some(-1_500));
    });
}
