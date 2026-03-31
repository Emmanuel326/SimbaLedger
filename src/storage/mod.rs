use std::collections::HashMap;
use crate::accounting::StorageBackend;
use crate::core::{Account, Transfer};

/// In-memory storage implementation (for development and testing)
pub struct InMemoryStorage {
    accounts: HashMap<u128, Account>,
    transfers: HashMap<u128, Transfer>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transfers: HashMap::new(),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryStorage {
    fn create_account(&mut self, account: Account) -> Result<(), String> {
        self.accounts.insert(account.id, account);
        Ok(())
    }

    fn get_account(&self, id: u128) -> Result<Option<Account>, String> {
        Ok(self.accounts.get(&id).copied())
    }

    fn create_transfer(&mut self, transfer: Transfer) -> Result<(), String> {
        self.transfers.insert(transfer.id, transfer);
        Ok(())
    }

    fn get_transfer(&self, id: u128) -> Result<Option<Transfer>, String> {
        Ok(self.transfers.get(&id).copied())
    }
}
