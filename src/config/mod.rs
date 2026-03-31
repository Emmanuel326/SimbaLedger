/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub demo_accounts: bool,
    pub demo_balance: u64,
}

impl Config {
    pub fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            demo_accounts: true,
            demo_balance: 1_000_000,
        }
    }
    
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
