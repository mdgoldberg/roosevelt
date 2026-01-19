pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: usize,
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

        Self { url, pool_size: 20 }
    }

    pub async fn create_pool(&self) -> Result<sqlx::SqlitePool, sqlx::Error> {
        sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&self.url)
                .create_if_missing(true),
        )
        .await
    }
}
