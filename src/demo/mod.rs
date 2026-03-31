use crate::accounting::{DoubleEntryEngine, StorageBackend};
use crate::core::Account;

/// Setup demo accounts for testing
pub fn setup_demo_accounts<S: StorageBackend>(
    engine: &mut DoubleEntryEngine<S>,
    initial_balance: u64,
) -> Result<(), String> {
    // Account 1: Rich account
    let mut acc1 = Account::new(1);
    acc1.credit_posted(initial_balance);
    engine.create_account(acc1).map_err(|e| e.to_string())?;
    
    // Account 2: Empty account
    let acc2 = Account::new(2);
    engine.create_account(acc2).map_err(|e| e.to_string())?;
    
    // Account 3: Empty account
    let acc3 = Account::new(3);
    engine.create_account(acc3).map_err(|e| e.to_string())?;
    
    println!("   Demo accounts created:");
    println!("     Account 1: {} (balance)", initial_balance);
    println!("     Account 2: 0");
    println!("     Account 3: 0");
    
    Ok(())
}
