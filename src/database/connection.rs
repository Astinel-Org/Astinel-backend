use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub enable_ssl: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "astinel".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_seconds: 10,
            idle_timeout_seconds: 300,
            enable_ssl: false,
        }
    }
}

impl DbConfig {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn from_env() -> Self {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            Self::from_url(&url)
        } else {
            Self {
                host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: std::env::var("DB_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5432),
                database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "astinel".to_string()),
                username: std::env::var("DB_USERNAME").unwrap_or_else(|_| "postgres".to_string()),
                password: std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                min_connections: std::env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2),
                connect_timeout_seconds: std::env::var("DB_CONNECT_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                idle_timeout_seconds: std::env::var("DB_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
                enable_ssl: std::env::var("DB_ENABLE_SSL")
                    .ok()
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            }
        }
    }

    fn from_url(url: &str) -> Self {
        let trimmed = url.trim_start_matches("postgres://");
        let (userinfo, rest) = trimmed.split_once('@').unwrap_or(("", trimmed));
        let (username, password) = userinfo.split_once(':').unwrap_or((userinfo, ""));
        let (hostport, database) = rest.split_once('/').unwrap_or((rest, "astinel"));
        let (host, port_str) = hostport.split_once(':').unwrap_or((hostport, "5432"));
        let port: u16 = port_str.parse().unwrap_or(5432);

        Self {
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_seconds: 10,
            idle_timeout_seconds: 300,
            enable_ssl: false,
        }
    }
}
