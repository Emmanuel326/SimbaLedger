use serde::{Serialize, Deserialize};
use std::fmt;

/// Flags that control transfer behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferFlags {
    pub linked: bool,      // Part of a batch that must succeed/fail together
    pub pending: bool,     // Two-phase commit: pending transfer
    pub post_pending: bool,// Post a previously pending transfer
    pub void_pending: bool,// Void a previously pending transfer
}

impl TransferFlags {
    pub fn none() -> Self {
        Self {
            linked: false,
            pending: false,
            post_pending: false,
            void_pending: false,
        }
    }
    
    pub fn is_two_phase(&self) -> bool {
        self.pending || self.post_pending || self.void_pending
    }
}

/// A transfer moves value between two accounts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transfer {
    /// Unique identifier for idempotency
    pub id: u128,
    
    /// Account to debit (money comes from here)
    pub debit_account_id: u128,
    
    /// Account to credit (money goes here)
    pub credit_account_id: u128,
    
    /// Amount in smallest unit (e.g., cents)
    pub amount: u64,
    
    /// Ledger to group transfers (e.g., 1 = customer funds, 2 = fees)
    pub ledger: u32,
    
    /// Code for categorization (e.g., 1 = transfer, 2 = payment)
    pub code: u16,
    
    /// Flags controlling behavior
    pub flags: TransferFlags,
    
    /// Timestamp in nanoseconds since epoch
    pub timestamp: u64,
    
    /// For linked events: the ID of the batch this belongs to
    pub batch_id: Option<u128>,
}

impl Transfer {
    /// Create a simple transfer (no flags, no batch)
    pub fn simple(
        id: u128,
        debit_account_id: u128,
        credit_account_id: u128,
        amount: u64,
    ) -> Self {
        Self {
            id,
            debit_account_id,
            credit_account_id,
            amount,
            ledger: 1,
            code: 1,
            flags: TransferFlags::none(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            batch_id: None,
        }
    }
    
    /// Create a pending transfer (two-phase commit)
    pub fn pending(
        id: u128,
        debit_account_id: u128,
        credit_account_id: u128,
        amount: u64,
    ) -> Self {
        let mut flags = TransferFlags::none();
        flags.pending = true;
        
        Self {
            id,
            debit_account_id,
            credit_account_id,
            amount,
            ledger: 1,
            code: 1,
            flags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            batch_id: None,
        }
    }
    
    /// Create a transfer that posts a pending transfer
    pub fn post_pending(
        id: u128,
        pending_transfer_id: u128,
    ) -> Self {
        let mut flags = TransferFlags::none();
        flags.post_pending = true;
        
        Self {
            id,
            debit_account_id: 0,  // Will be looked up from pending transfer
            credit_account_id: 0,
            amount: 0,
            ledger: 1,
            code: 1,
            flags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            batch_id: Some(pending_transfer_id),
        }
    }
    
    /// Create a transfer that voids a pending transfer
    pub fn void_pending(
        id: u128,
        pending_transfer_id: u128,
    ) -> Self {
        let mut flags = TransferFlags::none();
        flags.void_pending = true;
        
        Self {
            id,
            debit_account_id: 0,
            credit_account_id: 0,
            amount: 0,
            ledger: 1,
            code: 1,
            flags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            batch_id: Some(pending_transfer_id),
        }
    }
    

/// Validate the transfer (basic checks)
pub fn validate(&self) -> Result<(), String> {
    // ID must not be zero (0 is often reserved for system)
    if self.id == 0 {
        return Err("Transfer ID cannot be zero".to_string());
    }
    
    // For post/void pending transfers, accounts will be resolved from the pending transfer
    // So we skip account validation for these types
    let is_post_or_void = self.flags.post_pending || self.flags.void_pending;
    
    if !is_post_or_void {
        // Regular transfers need both accounts
        if self.debit_account_id == 0 || self.credit_account_id == 0 {
            return Err("Debit and credit accounts must be specified".to_string());
        }
        
        // Debit and credit accounts must be different
        if self.debit_account_id == self.credit_account_id {
            return Err("Debit and credit accounts must be different".to_string());
        }
        
        // Amount must be positive
        if self.amount == 0 {
            return Err("Transfer amount must be greater than zero".to_string());
        }
    } else {
        // For post/void pending transfers, we need batch_id to reference the pending transfer
        if self.batch_id.is_none() {
            return Err("Post/void pending transfers must specify batch_id".to_string());
        }
    }
    
    Ok(())
} 
    /// Check if this is a linked transfer (part of a batch)
    pub fn is_linked(&self) -> bool {
        self.flags.linked
    }
    
    /// Check if this is a pending transfer
    pub fn is_pending(&self) -> bool {
        self.flags.pending
    }
    
    /// Check if this posts a pending transfer
    pub fn is_post_pending(&self) -> bool {
        self.flags.post_pending
    }
    
    /// Check if this voids a pending transfer
    pub fn is_void_pending(&self) -> bool {
        self.flags.void_pending
    }
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transfer(id={}, {} → {}, amount={}, flags=[{}])",
            self.id,
            self.debit_account_id,
            self.credit_account_id,
            self.amount,
            if self.flags.pending { "pending" } else if self.flags.post_pending { "post" } else if self.flags.void_pending { "void" } else { "normal" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_transfer() {
        let transfer = Transfer::simple(100, 1, 2, 500);
        
        assert_eq!(transfer.id, 100);
        assert_eq!(transfer.debit_account_id, 1);
        assert_eq!(transfer.credit_account_id, 2);
        assert_eq!(transfer.amount, 500);
        assert!(!transfer.is_pending());
        assert!(!transfer.is_linked());
        assert!(transfer.validate().is_ok());
    }
    
    #[test]
    fn test_pending_transfer() {
        let transfer = Transfer::pending(101, 1, 2, 500);
        
        assert!(transfer.is_pending());
        assert!(!transfer.is_linked());
        assert!(transfer.validate().is_ok());
    }
    
    #[test]
    fn test_post_pending_transfer() {
        let transfer = Transfer::post_pending(102, 101);
        
        assert!(transfer.is_post_pending());
        assert_eq!(transfer.batch_id, Some(101));
        // Validation passes even with zero accounts because it's a post operation
        assert!(transfer.validate().is_ok());
    }


#[test]
fn test_validate_fails_same_account() {
    let transfer = Transfer::simple(103, 1, 1, 500);
    
    let result = transfer.validate();
    assert!(result.is_err());
    // Fix: check the error message using .unwrap_err()
    assert_eq!(result.unwrap_err(), "Debit and credit accounts must be different");
}

#[test]
fn test_validate_fails_zero_amount() {
    let transfer = Transfer::simple(104, 1, 2, 0);
    
    let result = transfer.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Transfer amount must be greater than zero");
}

#[test]
fn test_validate_fails_zero_id() {
    let transfer = Transfer::simple(0, 1, 2, 500);
    
    let result = transfer.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Transfer ID cannot be zero");
}
    
}
