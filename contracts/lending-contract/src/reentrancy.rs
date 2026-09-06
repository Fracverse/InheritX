use soroban_sdk::Env;
use crate::{DataKey, LendingError};

pub struct ReentrancyGuard<'a> {
    env: &'a Env,
}

impl<'a> ReentrancyGuard<'a> {
    pub fn new(env: &'a Env) -> Result<Self, LendingError> {
        if env.storage().temporary().has(&DataKey::ReentrancyGuard) {
            return Err(LendingError::ReentrantCall);
        }
        env.storage().temporary().set(&DataKey::ReentrancyGuard, &true);
        Ok(Self { env })
    }
}

impl<'a> Drop for ReentrancyGuard<'a> {
    fn drop(&mut self) {
        self.env.storage().temporary().remove(&DataKey::ReentrancyGuard);
    }
}
