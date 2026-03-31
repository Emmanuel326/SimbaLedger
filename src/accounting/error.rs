
use std::fmt;

/// Errors that can occur during accounting operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountingError {
    /// Account doesn't exist
    AccountNotFound(u128),
    
    /// Transfer already processed (idempotency)
    TransferAlreadyProcessed(u128),
    
    /// Insufficient funds
    InsufficientFunds {
        account_id: u128,
        available: i128,
        required: u64,
    },
    
    /// Invalid transfer (validation failed)
    InvalidTransfer(String),
    
    /// Linked event failed (batch atomicity)
    LinkedEventFailed {
        transfer_id: u128,
        cause: Box<AccountingError>,
    },
    
    /// Pending transfer not found (for post/void)
    PendingTransferNotFound(u128),
    
    /// Cannot post/void a transfer that's not pending
    InvalidPendingOperation(String),
    
    /// Storage error (wrapped)
    StorageError(String),
}

impl fmt::Display for AccountingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountingError::AccountNotFound(id) => {
                write!(f, "Account {} not found", id)
            }
            AccountingError::TransferAlreadyProcessed(id) => {
                write!(f, "Transfer {} already processed", id)
            }
            AccountingError::InsufficientFunds { account_id, available, required } => {
                write!(f, "Account {} has {} available but needs {}", account_id, available, required)
            }
            AccountingError::InvalidTransfer(msg) => {
                write!(f, "Invalid transfer: {}", msg)
            }
            AccountingError::LinkedEventFailed { transfer_id, cause } => {
                write!(f, "Linked event {} failed: {}", transfer_id, cause)
            }
            AccountingError::PendingTransferNotFound(id) => {
                write!(f, "Pending transfer {} not found", id)
            }
            AccountingError::InvalidPendingOperation(msg) => {
                write!(f, "Invalid pending operation: {}", msg)
            }
            AccountingError::StorageError(msg) => {
                write!(f, "Storage error: {}", msg)
            }
        }
    }
}

impl std::error::Error for AccountingError {}
