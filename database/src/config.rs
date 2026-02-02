pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: usize,
    pub writer_type: DatabaseWriterType,
}

/// Type of database writer to use for game recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatabaseWriterType {
    /// Bulk writer collects all events in memory and saves atomically at game end
    #[default]
    Bulk,
    /// Streaming writer persists events immediately as they occur
    Streaming,
}

impl std::str::FromStr for DatabaseWriterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bulk" => Ok(Self::Bulk),
            "streaming" => Ok(Self::Streaming),
            _ => Err(format!("Unknown writer type: {}", s)),
        }
    }
}

impl std::fmt::Display for DatabaseWriterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bulk => write!(f, "bulk"),
            Self::Streaming => write!(f, "streaming"),
        }
    }
}

impl DatabaseConfig {
    pub fn from_cli_or_env_or_yaml(cli_arg: Option<String>, yaml_config: Option<String>) -> Self {
        let url = if let Some(arg) = cli_arg {
            arg
        } else if let Ok(env) = std::env::var("DATABASE_URL") {
            env
        } else if let Some(yaml) = yaml_config {
            yaml
        } else {
            "sqlite::memory:".to_string()
        };

        Self {
            url,
            pool_size: 20,
            writer_type: DatabaseWriterType::default(),
        }
    }

    pub async fn create_pool(&self) -> Result<sqlx::SqlitePool, sqlx::Error> {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&self.url)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal),
        )
        .await?;

        // Increase cache size for better performance with bulk writes
        sqlx::query("PRAGMA cache_size = -128000")
            .execute(&pool)
            .await?;

        Ok(pool)
    }
}
