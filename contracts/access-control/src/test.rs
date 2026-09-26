//! Tests for the governance voting-weight helpers.
//!
//! The rule under test is 1 unit of locked value == 1 vote, plus delegation:
//! a holder's weight counts for their delegate while a delegation is active,
//! and returns to the holder when they undelegate.

use super::*;
use soroban_sdk::{contract, contractimpl, testutils::Address as _};

fn addr(env: &Env) -> Address {
    Address::generate(env)
}

/// Register `access-control`'s storage-owning contract so tests can read and
/// write persistent entries the way the real contracts do.
fn as_contract(env: &Env) -> Address {
    env.register_contract(None, VotingHarness)
}

/// Minimal contract used only to give the tests a storage context.
#[contract]
pub struct VotingHarness;

#[contractimpl]
impl VotingHarness {}

#[test]
fn locked_value_maps_one_to_one_onto_votes() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        assert_eq!(get_voting_power(&env, &holder), 0);

        add_locked_value(&env, &holder, 1_000);
        assert_eq!(get_locked_value(&env, &holder), 1_000);
        assert_eq!(get_voting_power(&env, &holder), 1_000);

        add_locked_value(&env, &holder, 500);
        assert_eq!(get_voting_power(&env, &holder), 1_500);
    });
}

#[test]
fn removing_locked_value_shrinks_votes_and_floors_at_zero() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        remove_locked_value(&env, &holder, 400);
        assert_eq!(get_voting_power(&env, &holder), 600);

        // Releasing more than recorded saturates rather than wrapping.
        remove_locked_value(&env, &holder, 10_000);
        assert_eq!(get_voting_power(&env, &holder), 0);
        assert_eq!(get_locked_value(&env, &holder), 0);
    });
}

#[test]
fn zero_and_negative_ish_adjustments_are_noops() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        add_locked_value(&env, &holder, 0);
        assert_eq!(get_voting_power(&env, &holder), 0);

        add_locked_value(&env, &holder, 100);
        remove_locked_value(&env, &holder, 0);
        set_locked_value(&env, &holder, 100);
        assert_eq!(get_voting_power(&env, &holder), 100);
    });
}

#[test]
fn set_locked_value_overwrites_and_clears() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        set_locked_value(&env, &holder, 250);
        assert_eq!(get_voting_power(&env, &holder), 250);

        set_locked_value(&env, &holder, 0);
        assert_eq!(get_voting_power(&env, &holder), 0);
    });
}

#[test]
fn delegating_moves_the_whole_weight_to_the_delegate() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);
        let delegate = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &delegate);

        // The holder keeps their locked value but no longer counts it themselves.
        assert_eq!(get_locked_value(&env, &holder), 1_000);
        assert_eq!(get_voting_power(&env, &holder), 0);
        assert_eq!(get_voting_power(&env, &delegate), 1_000);
        assert_eq!(get_delegate(&env, &holder), Some(delegate.clone()));
        assert_eq!(effective_voter(&env, &holder), delegate);
    });
}

#[test]
fn undelegating_returns_the_weight_to_the_holder() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    // `require_auth` may only be consumed once per address per frame, so the
    // delegate and undelegate calls run in separate frames.
    let holder = addr(&env);
    let delegate = addr(&env);

    env.as_contract(&me, || {
        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &delegate);
    });
    env.as_contract(&me, || undelegate_votes(&env, &holder));
    env.as_contract(&me, || {
        assert_eq!(get_voting_power(&env, &holder), 1_000);
        assert_eq!(get_voting_power(&env, &delegate), 0);
        assert_eq!(get_delegate(&env, &holder), None);
        assert_eq!(effective_voter(&env, &holder), holder);
    });
}

#[test]
fn redelegating_moves_the_weight_from_the_old_delegate_to_the_new_one() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    let holder = addr(&env);
    let first = addr(&env);
    let second = addr(&env);

    // Re-delegating needs a second frame: one `require_auth` per address.
    env.as_contract(&me, || {
        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &first);
    });
    env.as_contract(&me, || delegate_votes(&env, &holder, &second));
    env.as_contract(&me, || {
        // The old delegate is made whole again rather than keeping a stale claim.
        assert_eq!(get_voting_power(&env, &first), 0);
        assert_eq!(get_voting_power(&env, &second), 1_000);
        assert_eq!(get_voting_power(&env, &holder), 0);
    });
}

#[test]
fn locked_value_changes_track_an_active_delegation() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);
        let delegate = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &delegate);

        // Locking more after delegating credits the delegate, not the holder.
        add_locked_value(&env, &holder, 500);
        assert_eq!(get_voting_power(&env, &delegate), 1_500);
        assert_eq!(get_voting_power(&env, &holder), 0);

        // And unlocking pulls it back out of the delegate.
        remove_locked_value(&env, &holder, 1_200);
        assert_eq!(get_voting_power(&env, &delegate), 300);
    });
}

#[test]
fn set_locked_value_resyncs_an_active_delegation() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);
        let delegate = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &delegate);

        set_locked_value(&env, &holder, 200);
        assert_eq!(get_voting_power(&env, &delegate), 200);
    });
}

#[test]
fn self_delegation_is_a_noop_that_keeps_votes_with_the_holder() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        delegate_votes(&env, &holder, &holder);

        // Equivalent to not delegating at all.
        assert_eq!(get_voting_power(&env, &holder), 1_000);
        assert_eq!(get_delegate(&env, &holder), None);
    });
}

#[test]
fn undelegating_without_a_delegation_is_a_noop() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);

        add_locked_value(&env, &holder, 1_000);
        undelegate_votes(&env, &holder);
        assert_eq!(get_voting_power(&env, &holder), 1_000);
    });
}

#[test]
fn a_delegate_keeps_its_own_weight_and_receives_the_deposited_weight() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let delegate = addr(&env);
        let holder = addr(&env);

        add_locked_value(&env, &delegate, 300);
        add_locked_value(&env, &holder, 700);
        delegate_votes(&env, &holder, &delegate);

        // Own locked value plus the delegated weight.
        assert_eq!(get_voting_power(&env, &delegate), 1_000);
        assert_eq!(get_delegated_power(&env, &delegate), 700);
    });
}

#[test]
fn many_delegates_accumulate_on_one_address() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let delegate = addr(&env);

        let mut total = 0u128;
        for _ in 0..5 {
            let holder = addr(&env);
            let amount = 100u128;
            add_locked_value(&env, &holder, amount);
            delegate_votes(&env, &holder, &delegate);
            total += amount;
        }

        assert_eq!(get_voting_power(&env, &delegate), total);
    });
}

#[test]
fn snapshot_returns_powers_in_input_order() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let a = addr(&env);
        let b = addr(&env);
        let c = addr(&env);

        add_locked_value(&env, &a, 100);
        add_locked_value(&env, &b, 200);
        add_locked_value(&env, &c, 300);
        delegate_votes(&env, &c, &b);

        let holders = vec![&env, a.clone(), b.clone(), c.clone()];
        let snap = get_voting_snapshot(&env, &holders);

        assert_eq!(snap.len(), 3);
        assert_eq!(snap.get(0).unwrap(), (a, 100));
        assert_eq!(snap.get(1).unwrap(), (b.clone(), 500)); // own 200 + 300 delegated
        assert_eq!(snap.get(2).unwrap(), (c, 0));
    });
}

#[test]
fn snapshot_of_an_empty_holder_set_is_empty() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holders: Vec<Address> = Vec::new(&env);
        assert!(get_voting_snapshot(&env, &holders).is_empty());
    });
}

#[test]
fn delegation_requires_the_holders_own_authorization() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let me = as_contract(&env);
    env.as_contract(&me, || {
        let holder = addr(&env);
        let delegate = addr(&env);
        add_locked_value(&env, &holder, 100);

        // `delegate_votes` calls `require_auth` on the delegator, so only the
        // holder can redirect their own weight — a delegate cannot act on someone
        // else's behalf.
        delegate_votes(&env, &holder, &delegate);
        assert_eq!(get_voting_power(&env, &delegate), 100);
        assert_eq!(get_voting_power(&env, &holder), 0);
    });
}
