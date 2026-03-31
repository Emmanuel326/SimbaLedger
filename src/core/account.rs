use serde::{Serialize, Deserialize};
use std::fmt;

/// An account in the double-entry ledger.
/// 
/// This is the fundamental atomic unit. Every transfer debits one account
/// and credits another. Balance invariants must always hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// 128-bit identifier (like TigerBeetle)
    pub id: u128,
    
    /// Pending debits (two-phase transfers)
    pub debits_pending: u64,
    
    /// Posted debits (settled transfers)
    pub debits_posted: u64,
    
    /// Pending credits (two-phase transfers)
    pub credits_pending: u64,
    
    /// Posted credits (settled transfers)
    pub credits_posted: u64,
}

impl Account {
    /// Create a new account with zero balances
    pub fn new(id: u128) -> Self {
        Self {
            id,
            debits_pending: 0,
            debits_posted: 0,
            credits_pending: 0,
            credits_posted: 0,
        }
    }
    
    /// Available balance = posted credits - posted debits
    /// This is what can be spent immediately
    pub fn available_balance(&self) -> i128 {
        (self.credits_posted as i128) - (self.debits_posted as i128)
    }
    
    /// Total balance = (posted + pending) credits - (posted + pending) debits
    /// This includes pending transfers that haven't settled
    pub fn total_balance(&self) -> i128 {
        (self.credits_posted as i128 + self.credits_pending as i128) -
        (self.debits_posted as i128 + self.debits_pending as i128)
    }
    
    /// Check if a debit is possible
    pub fn can_debit(&self, amount: u64) -> bool {
        self.available_balance() >= amount as i128
    }
    
    /// Apply a posted debit
    pub fn debit_posted(&mut self, amount: u64) -> Result<(), String> {
        if self.can_debit(amount) {
            self.debits_posted += amount;
            Ok(())
        } else {
            Err(format!(
                "Insufficient funds: available {}, required {}",
                self.available_balance(),
                amount
            ))
        }
    }
    
    /// Apply a posted credit
    pub fn credit_posted(&mut self, amount: u64) {
        self.credits_posted += amount;
    }
    
    /// Apply a pending debit
    pub fn debit_pending(&mut self, amount: u64) {
        self.debits_pending += amount;
    }
    
    /// Apply a pending credit
    pub fn credit_pending(&mut self, amount: u64) {
        self.credits_pending += amount;
    }
    
    /// Post a pending debit (move from pending to posted)
    pub fn post_pending_debit(&mut self, amount: u64) -> Result<(), String> {
        if self.debits_pending >= amount {
            self.debits_pending -= amount;
            self.debits_posted += amount;
            Ok(())
        } else {
            Err("Not enough pending debit".to_string())
        }
    }
    
    /// Post a pending credit (move from pending to posted)
    pub fn post_pending_credit(&mut self, amount: u64) -> Result<(), String> {
        if self.credits_pending >= amount {
            self.credits_pending -= amount;
            self.credits_posted += amount;
            Ok(())
        } else {
            Err("Not enough pending credit".to_string())
        }
    }
    
    /// Void a pending debit (cancel it entirely)
    pub fn void_pending_debit(&mut self, amount: u64) -> Result<(), String> {
        if self.debits_pending >= amount {
            self.debits_pending -= amount;
            Ok(())
        } else {
            Err("Not enough pending debit to void".to_string())
        }
    }
    
    /// Void a pending credit (cancel it entirely)
    pub fn void_pending_credit(&mut self, amount: u64) -> Result<(), String> {
        if self.credits_pending >= amount {
            self.credits_pending -= amount;
            Ok(())
        } else {
            Err("Not enough pending credit to void".to_string())
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Account(id={}, available={}, pending_dr={}, pending_cr={})",
            self.id,
            self.available_balance(),
            self.debits_pending,
            self.credits_pending
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_account() {
        let acc = Account::new(1);
        assert_eq!(acc.id, 1);
        assert_eq!(acc.available_balance(), 0);
        assert_eq!(acc.total_balance(), 0);
    }
    
    #[test]
    fn test_credit_and_debit() {
        let mut acc = Account::new(1);
        acc.credit_posted(1000);
        assert_eq!(acc.available_balance(), 1000);
        
        acc.debit_posted(300).unwrap();
        assert_eq!(acc.available_balance(), 700);
    }
    
    #[test]
    fn test_insufficient_funds() {
        let mut acc = Account::new(1);
        acc.credit_posted(500);
        
        let result = acc.debit_posted(1000);
        assert!(result.is_err());
        assert_eq!(acc.available_balance(), 500);
    }
    
    #[test]
    fn test_pending_transfers() {
        let mut acc = Account::new(1);
        acc.credit_posted(1000);
        acc.debit_pending(200);  // Pending debit doesn't affect available balance
        
        assert_eq!(acc.available_balance(), 1000);  // Still 1000 available
        assert_eq!(acc.total_balance(), 800);       // But total is locked
        
        acc.post_pending_debit(200).unwrap();
        assert_eq!(acc.available_balance(), 800);
        assert_eq!(acc.total_balance(), 800);
    }
    
    #[test]
    fn test_two_phase_commit() {
        let mut sender = Account::new(1);
        let mut receiver = Account::new(2);
        
        sender.credit_posted(1000);
        
        // Phase 1: Pending
        sender.debit_pending(500);
        receiver.credit_pending(500);
        
        assert_eq!(sender.available_balance(), 1000);  // Can't spend locked funds
        assert_eq!(sender.total_balance(), 500);
        
        // Phase 2: Post
        sender.post_pending_debit(500).unwrap();
        receiver.post_pending_credit(500).unwrap();
        
        assert_eq!(sender.available_balance(), 500);
        assert_eq!(receiver.available_balance(), 500);
    }
}
