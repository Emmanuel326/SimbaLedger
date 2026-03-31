use crate::core::Transfer;

/// Parse a transfer from JSON
pub fn parse_transfer(data: &str) -> Result<Transfer, String> {
    // Simple parser for demo
    // Format: {"id":123,"debit_account_id":1,"credit_account_id":2,"amount":100}
    
    let id = extract_field(data, "id")?
        .parse()
        .map_err(|_| "Invalid id")?;
    
    let debit = extract_field(data, "debit_account_id")?
        .parse()
        .map_err(|_| "Invalid debit_account_id")?;
    
    let credit = extract_field(data, "credit_account_id")?
        .parse()
        .map_err(|_| "Invalid credit_account_id")?;
    
    let amount = extract_field(data, "amount")?
        .parse()
        .map_err(|_| "Invalid amount")?;
    
    Ok(Transfer::simple(id, debit, credit, amount))
}

fn extract_field(data: &str, field: &str) -> Result<String, String> {
    let pattern = format!("\"{}\":", field);
    let start = data.find(&pattern)
        .ok_or_else(|| format!("Field '{}' not found", field))?;
    let start = start + pattern.len();
    
    let rest = &data[start..];
    let end = rest.find(',').unwrap_or_else(|| {
        rest.find('}').unwrap_or(rest.len())
    });
    
    Ok(rest[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_transfer() {
        let json = r#"{"id":1,"debit_account_id":2,"credit_account_id":3,"amount":500}"#;
        let transfer = parse_transfer(json).unwrap();
        
        assert_eq!(transfer.id, 1);
        assert_eq!(transfer.debit_account_id, 2);
        assert_eq!(transfer.credit_account_id, 3);
        assert_eq!(transfer.amount, 500);
    }
    
    #[test]
    fn test_parse_invalid() {
        let json = r#"{"id":"bad","debit_account_id":2,"credit_account_id":3,"amount":500}"#;
        assert!(parse_transfer(json).is_err());
    }
}
