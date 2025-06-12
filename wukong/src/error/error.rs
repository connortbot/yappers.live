use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum ErrorCode {
    // Game
    GameNotFound,
    GameFull,
    PlayerNotFound,
    PlayerAlreadyExists,
    InvalidGameCode,
    PlayerAlreadyInGame,
    UsernameTaken,
    
    
    InvalidInput(String),
    InternalServerError,
}


#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorCode,
    pub message: String,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErrorResponse {{ error: {:?}, message: {} }}", self.error, self.message)
    }
}

impl From<String> for ErrorResponse {
    fn from(error_message: String) -> Self {
        ErrorResponse {
            error: ErrorCode::InternalServerError,
            message: format!("Key building error (programming bug): {}", error_message),
        }
    }
}

impl From<redis::RedisError> for ErrorResponse {
    fn from(error: redis::RedisError) -> Self {
        ErrorResponse {
            error: ErrorCode::InternalServerError,
            message: format!("Redis error: {}", error),
        }
    }
}

impl From<serde_json::Error> for ErrorResponse {
    fn from(error: serde_json::Error) -> Self {
        ErrorResponse {
            error: ErrorCode::InternalServerError,
            message: format!("JSON serialization error: {}", error),
        }
    }
}
