#![no_std]

use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contracterror, contractimpl, contracttype, symbol_short, token, vec, Address, Env,
    IntoVal, InvokeError, Val, Vec,
};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanVaultError {
    Unauth = 1,
    BadAmt = 2,
    TxFail = 3,
}

#[contracttype]
#[derive(Clone)]
pub enum PlanVaultKey {
    Init,
    Ctl,
    Owner,
    Pid,
}

#[contract]
pub struct PlanVaultContract;

#[contractimpl]
impl PlanVaultContract {
    pub fn initialize(
        env: Env,
        controller: Address,
        owner: Address,
        plan_id: u64,
    ) -> Result<(), PlanVaultError> {
        if env
            .storage()
            .instance()
            .get::<PlanVaultKey, bool>(&PlanVaultKey::Init)
            .unwrap_or(false)
        {
            return Err(PlanVaultError::Unauth);
        }

        env.storage()
            .instance()
            .set(&PlanVaultKey::Ctl, &controller);
        env.storage().instance().set(&PlanVaultKey::Owner, &owner);
        env.storage().instance().set(&PlanVaultKey::Pid, &plan_id);
        env.storage().instance().set(&PlanVaultKey::Init, &true);
        Ok(())
    }

    pub fn release(
        env: Env,
        token: Address,
        recipient: Address,
        amount: u64,
    ) -> Result<(), PlanVaultError> {
        if amount == 0 {
            return Err(PlanVaultError::BadAmt);
        }

        let controller: Address = env
            .storage()
            .instance()
            .get(&PlanVaultKey::Ctl)
            .ok_or(PlanVaultError::Unauth)?;
        controller.require_auth();

        let vault_address = env.current_contract_address();
        let args: Vec<Val> = vec![
            &env,
            vault_address.into_val(&env),
            recipient.into_val(&env),
            (amount as i128).into_val(&env),
        ];
        let fn_name = symbol_short!("transfer");
        env.authorize_as_current_contract(vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token.clone(),
                    fn_name: fn_name.clone(),
                    args: args.clone(),
                },
                sub_invocations: vec![&env],
            }),
        ]);

        let res = env.try_invoke_contract::<(), InvokeError>(&token, &fn_name, args);
        if res.is_err() {
            return Err(PlanVaultError::TxFail);
        }
        Ok(())
    }

    pub fn balance(env: Env, token: Address) -> i128 {
        token::Client::new(&env, &token).balance(&env.current_contract_address())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mock_token::{MockToken, MockTokenClient};
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn release_transfers_from_vault_when_controller_authorizes() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();

        let vault_id = env.register_contract(None, PlanVaultContract);
        let token_id = env.register_contract(None, MockToken);
        let controller = Address::generate(&env);
        let owner = Address::generate(&env);
        let recipient = Address::generate(&env);

        let vault = PlanVaultContractClient::new(&env, &vault_id);
        vault.initialize(&controller, &owner, &1);
        MockTokenClient::new(&env, &token_id).mint(&vault_id, &1_000);

        vault.release(&token_id, &recipient, &250);

        let token = token::Client::new(&env, &token_id);
        assert_eq!(token.balance(&vault_id), 750);
        assert_eq!(token.balance(&recipient), 250);
    }

    #[test]
    fn initialize_only_runs_once() {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register_contract(None, PlanVaultContract);
        let controller = Address::generate(&env);
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        let vault = PlanVaultContractClient::new(&env, &vault_id);

        vault.initialize(&controller, &owner, &1);
        let result = vault.try_initialize(&other, &owner, &1);

        assert_eq!(result, Err(Ok(PlanVaultError::Unauth)));
    }
}
