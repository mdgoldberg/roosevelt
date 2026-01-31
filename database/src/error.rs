use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Query execution error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Player not found: {0}")]
    PlayerNotFound(String),

    #[error("Game not found: {0}")]
    GameNotFound(i64),

    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),

    #[error("UUID parsing error: {0}")]
    UuidParsing(#[from] uuid::Error),
}
