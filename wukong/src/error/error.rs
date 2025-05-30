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