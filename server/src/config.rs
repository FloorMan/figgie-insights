use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://figgie:figgie@localhost:5432/figgie".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
        })
    }
}
