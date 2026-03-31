use std::collections::HashMap;
use crate::core::{Account, Transfer};
use super::error::AccountingError;

/// Storage backend trait — implemented by pluggable storage
pub trait StorageBackend {
    fn create_account(&mut self, account: Account) -> Result<(), String>;
    fn get_account(&self, id: u128) -> Result<Option<Account>, String>;
    fn create_transfer(&mut self, transfer: Transfer) -> Result<(), String>;
    fn get_transfer(&self, id: u128) -> Result<Option<Transfer>, String>;
}

/// Result of processing a transfer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferResult {
    Applied,
    AlreadyProcessed,
    Failed(String),
}

/// The main accounting engine that enforces double-entry rules
pub struct DoubleEntryEngine<S> {
    storage: S,
    processed_transfers: HashMap<u128, TransferResult>,
}

impl<S> DoubleEntryEngine<S>
where
    S: StorageBackend,
{
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            processed_transfers: HashMap::new(),
        }
    }

    pub fn process_transfer(&mut self, transfer: &Transfer) -> Result<TransferResult, AccountingError> {
        if self.processed_transfers.contains_key(&transfer.id) {
            return Ok(TransferResult::AlreadyProcessed);
        }

        transfer.validate().map_err(|e| AccountingError::InvalidTransfer(e))?;

        let result = if transfer.is_pending() {
            self.process_pending_transfer(transfer)
        } else if transfer.is_post_pending() {
            self.process_post_pending(transfer)
        } else if transfer.is_void_pending() {
            self.process_void_pending(transfer)
        } else {
            self.process_normal_transfer(transfer)
        };

        match &result {
            Ok(TransferResult::Applied) => {
                self.processed_transfers.insert(transfer.id, TransferResult::Applied);
            }
            Ok(TransferResult::AlreadyProcessed) => {}
            Ok(TransferResult::Failed(msg)) => {
                self.processed_transfers.insert(transfer.id, TransferResult::Failed(msg.clone()));
            }
            Err(e) => {
                self.processed_transfers.insert(transfer.id, TransferResult::Failed(e.to_string()));
            }
        }

        result
    }

    fn process_normal_transfer(&mut self, transfer: &Transfer) -> Result<TransferResult, AccountingError> {
        let mut debit_account = self.storage.get_account(transfer.debit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(transfer.debit_account_id))?;

        let mut credit_account = self.storage.get_account(transfer.credit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(transfer.credit_account_id))?;

        if !debit_account.can_debit(transfer.amount) {
            return Err(AccountingError::InsufficientFunds {
                account_id: debit_account.id,
                available: debit_account.available_balance(),
                required: transfer.amount,
            });
        }

        debit_account.debit_posted(transfer.amount)
            .map_err(|e| AccountingError::InvalidTransfer(e))?;
        credit_account.credit_posted(transfer.amount);

        self.storage.create_account(debit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_account(credit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_transfer(transfer.clone())
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;

        Ok(TransferResult::Applied)
    }

    fn process_pending_transfer(&mut self, transfer: &Transfer) -> Result<TransferResult, AccountingError> {
        let mut debit_account = self.storage.get_account(transfer.debit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(transfer.debit_account_id))?;

        let mut credit_account = self.storage.get_account(transfer.credit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(transfer.credit_account_id))?;

        if debit_account.total_balance() < transfer.amount as i128 {
            return Err(AccountingError::InsufficientFunds {
                account_id: debit_account.id,
                available: debit_account.total_balance(),
                required: transfer.amount,
            });
        }

        debit_account.debit_pending(transfer.amount);
        credit_account.credit_pending(transfer.amount);

        self.storage.create_account(debit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_account(credit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_transfer(transfer.clone())
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;

        Ok(TransferResult::Applied)
    }

    fn process_post_pending(&mut self, transfer: &Transfer) -> Result<TransferResult, AccountingError> {
        let pending_id = transfer.batch_id.ok_or_else(|| {
            AccountingError::InvalidPendingOperation("No batch_id for post_pending".to_string())
        })?;

        let pending = self.storage.get_transfer(pending_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::PendingTransferNotFound(pending_id))?;

        if !pending.is_pending() {
            return Err(AccountingError::InvalidPendingOperation(
                "Transfer is not pending".to_string()
            ));
        }

        let mut debit_account = self.storage.get_account(pending.debit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(pending.debit_account_id))?;

        let mut credit_account = self.storage.get_account(pending.credit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(pending.credit_account_id))?;

        debit_account.post_pending_debit(pending.amount)
            .map_err(|e| AccountingError::InvalidTransfer(e.to_string()))?;
        credit_account.post_pending_credit(pending.amount)
            .map_err(|e| AccountingError::InvalidTransfer(e.to_string()))?;

        self.storage.create_account(debit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_account(credit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_transfer(transfer.clone())
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;

        Ok(TransferResult::Applied)
    }

    fn process_void_pending(&mut self, transfer: &Transfer) -> Result<TransferResult, AccountingError> {
        let pending_id = transfer.batch_id.ok_or_else(|| {
            AccountingError::InvalidPendingOperation("No batch_id for void_pending".to_string())
        })?;

        let pending = self.storage.get_transfer(pending_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::PendingTransferNotFound(pending_id))?;

        if !pending.is_pending() {
            return Err(AccountingError::InvalidPendingOperation(
                "Transfer is not pending".to_string()
            ));
        }

        let mut debit_account = self.storage.get_account(pending.debit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(pending.debit_account_id))?;

        let mut credit_account = self.storage.get_account(pending.credit_account_id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?
            .ok_or(AccountingError::AccountNotFound(pending.credit_account_id))?;

        debit_account.void_pending_debit(pending.amount)
            .map_err(|e| AccountingError::InvalidTransfer(e.to_string()))?;
        credit_account.void_pending_credit(pending.amount)
            .map_err(|e| AccountingError::InvalidTransfer(e.to_string()))?;

        self.storage.create_account(debit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_account(credit_account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;
        self.storage.create_transfer(transfer.clone())
            .map_err(|e| AccountingError::StorageError(e.to_string()))?;

        Ok(TransferResult::Applied)
    }

    pub fn get_account(&self, id: u128) -> Result<Option<Account>, AccountingError> {
        self.storage.get_account(id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))
    }

    pub fn get_transfer(&self, id: u128) -> Result<Option<Transfer>, AccountingError> {
        self.storage.get_transfer(id)
            .map_err(|e| AccountingError::StorageError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Account, Transfer};
    use std::collections::HashMap;

    // Test-only in-memory storage
    struct TestStorage {
        accounts: HashMap<u128, Account>,
        transfers: HashMap<u128, Transfer>,
    }

    impl TestStorage {
        fn new() -> Self {
            Self {
                accounts: HashMap::new(),
                transfers: HashMap::new(),
            }
        }
    }

    impl StorageBackend for TestStorage {
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

    #[test]
    fn test_normal_transfer() {
        let storage = TestStorage::new();
        let mut engine = DoubleEntryEngine::new(storage);

        let mut acc1 = Account::new(1);
        acc1.credit_posted(1000);
        let acc2 = Account::new(2);

        engine.storage.create_account(acc1).unwrap();
        engine.storage.create_account(acc2).unwrap();

        let transfer = Transfer::simple(100, 1, 2, 500);
        let result = engine.process_transfer(&transfer).unwrap();
        assert_eq!(result, TransferResult::Applied);

        let acc1 = engine.get_account(1).unwrap().unwrap();
        let acc2 = engine.get_account(2).unwrap().unwrap();
        assert_eq!(acc1.available_balance(), 500);
        assert_eq!(acc2.available_balance(), 500);
    }

    #[test]
    fn test_insufficient_funds() {
        let storage = TestStorage::new();
        let mut engine = DoubleEntryEngine::new(storage);

        let mut acc1 = Account::new(1);
        acc1.credit_posted(100);
        let acc2 = Account::new(2);

        engine.storage.create_account(acc1).unwrap();
        engine.storage.create_account(acc2).unwrap();

        let transfer = Transfer::simple(100, 1, 2, 500);
        let result = engine.process_transfer(&transfer);

        assert!(result.is_err());
        match result {
            Err(AccountingError::InsufficientFunds { account_id, available, required }) => {
                assert_eq!(account_id, 1);
                assert_eq!(available, 100);
                assert_eq!(required, 500);
            }
            _ => panic!("Expected InsufficientFunds error"),
        }
    }

    #[test]
    fn test_idempotency() {
        let storage = TestStorage::new();
        let mut engine = DoubleEntryEngine::new(storage);

        let mut acc1 = Account::new(1);
        acc1.credit_posted(1000);
        let acc2 = Account::new(2);

        engine.storage.create_account(acc1).unwrap();
        engine.storage.create_account(acc2).unwrap();

        let transfer = Transfer::simple(100, 1, 2, 500);

        let result1 = engine.process_transfer(&transfer).unwrap();
        assert_eq!(result1, TransferResult::Applied);

        let result2 = engine.process_transfer(&transfer).unwrap();
        assert_eq!(result2, TransferResult::AlreadyProcessed);

        let acc1 = engine.get_account(1).unwrap().unwrap();
        assert_eq!(acc1.available_balance(), 500);
    }

    #[test]
    fn test_pending_and_post() {
        let storage = TestStorage::new();
        let mut engine = DoubleEntryEngine::new(storage);

        let mut acc1 = Account::new(1);
        acc1.credit_posted(1000);
        let acc2 = Account::new(2);

        engine.storage.create_account(acc1).unwrap();
        engine.storage.create_account(acc2).unwrap();

        let pending = Transfer::pending(100, 1, 2, 500);
        let result = engine.process_transfer(&pending).unwrap();
        assert_eq!(result, TransferResult::Applied);

        let acc1 = engine.get_account(1).unwrap().unwrap();
        assert_eq!(acc1.available_balance(), 1000);
        assert_eq!(acc1.total_balance(), 500);

        let post = Transfer::post_pending(101, 100);
        let result = engine.process_transfer(&post).unwrap();
        assert_eq!(result, TransferResult::Applied);

        let acc1 = engine.get_account(1).unwrap().unwrap();
        let acc2 = engine.get_account(2).unwrap().unwrap();
        assert_eq!(acc1.available_balance(), 500);
        assert_eq!(acc2.available_balance(), 500);
    }

    #[test]
    fn test_pending_and_void() {
        let storage = TestStorage::new();
        let mut engine = DoubleEntryEngine::new(storage);

        let mut acc1 = Account::new(1);
        acc1.credit_posted(1000);
        let acc2 = Account::new(2);

        engine.storage.create_account(acc1).unwrap();
        engine.storage.create_account(acc2).unwrap();

        let pending = Transfer::pending(100, 1, 2, 500);
        let result = engine.process_transfer(&pending).unwrap();
        assert_eq!(result, TransferResult::Applied);

        let void = Transfer::void_pending(101, 100);
        let result = engine.process_transfer(&void).unwrap();
        assert_eq!(result, TransferResult::Applied);

        let acc1 = engine.get_account(1).unwrap().unwrap();
        let acc2 = engine.get_account(2).unwrap().unwrap();
        assert_eq!(acc1.available_balance(), 1000);
        assert_eq!(acc2.available_balance(), 0);
        assert_eq!(acc1.total_balance(), 1000);
        assert_eq!(acc2.total_balance(), 0);
    }
}

impl<S: StorageBackend> DoubleEntryEngine<S> {
    /// Create a new account (public API)
    pub fn create_account(&mut self, account: Account) -> Result<(), AccountingError> {
        self.storage.create_account(account)
            .map_err(|e| AccountingError::StorageError(e.to_string()))
    }
    
    /// Get an account (already exists)
    // pub fn get_account(&self, id: u128) -> Result<Option<Account>, AccountingError>
    
    /// Create a transfer (public API)
    pub fn create_transfer(&mut self, transfer: Transfer) -> Result<TransferResult, AccountingError> {
        self.process_transfer(&transfer)
    }
}
