use std::collections::HashMap;
use uuid::Uuid;
use crate::error::{ErrorResponse, ErrorCode};

pub type ActionTimerResult<T> = Result<T, ErrorResponse>;

#[derive(Default, Debug)]
pub struct ActionTimerManager {
    active_action_keys: HashMap<String, String>, // action_key -> context (e.g., game_id)
    expired_action_keys: HashMap<String, String>, // action_key -> context
}

impl Clone for ActionTimerManager {
    fn clone(&self) -> Self {
        // Create a new ActionTimerManager with empty state
        // We don't clone the actual keys since they're runtime state
        Self::new()
    }
}

impl ActionTimerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_action_key(&mut self, context: &str) -> String {
        let action_key = Uuid::new_v4().to_string();
        self.active_action_keys.insert(action_key.clone(), context.to_string());
        action_key
    }

    pub fn validate_action_key(&self, action_key: &str) -> bool {
        self.active_action_keys.contains_key(action_key) && !self.expired_action_keys.contains_key(action_key)
    }

    pub fn consume_action_key(&mut self, action_key: &str) -> ActionTimerResult<String> {
        if self.expired_action_keys.contains_key(action_key) {
            return Err(ErrorResponse {
                error: ErrorCode::InvalidInput("Action key has expired".to_string()),
                message: "This action is no longer valid".to_string(),
            });
        }
        
        self.active_action_keys.remove(action_key).ok_or(ErrorResponse {
            error: ErrorCode::InvalidInput("Invalid action key".to_string()),
            message: "Action key not found or already used".to_string(),
        })
    }

    pub fn expire_action_key(&mut self, action_key: &str) {
        if let Some(context) = self.active_action_keys.remove(action_key) {
            self.expired_action_keys.insert(action_key.to_string(), context);
        }
    }
} 