use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use chrono;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    // stream?
}

#[derive(Debug, Clone, Serialize)]
pub struct Lobby {
    pub id: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub max_players: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub enum LobbyError {
    LobbyNotFound,
    LobbyFull,
    PlayerAlreadyInLobby,
    InvalidInput(String),
}

impl std::fmt::Display for LobbyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyError::LobbyNotFound => write!(f, "Lobby not found"),
            LobbyError::LobbyFull => write!(f, "Lobby is full"),
            LobbyError::PlayerAlreadyInLobby => write!(f, "Player already in a lobby"),
            LobbyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for LobbyError {}
pub type LobbyResult<T> = Result<T, LobbyError>;

#[derive(Default)]
pub struct LobbyManager {
    lobbies: RwLock<HashMap<String, Lobby>>,
    player_to_lobby: RwLock<HashMap<String, String>>,
    lobby_broadcasters: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_lobby(&self, host_username: String) -> LobbyResult<Lobby> {
        if host_username.trim().is_empty() {
            return Err(LobbyError::InvalidInput("Username cannot be empty".to_string()));
        }

        let lobby_id = Uuid::new_v4().to_string();
        let host_id = Uuid::new_v4().to_string();

        let host = Player {
            id: host_id.clone(),
            username: host_username,
        };

        let lobby = Lobby {
            id: lobby_id.clone(),
            host_id: host_id.clone(),
            players: vec![host],
            max_players: 8,
            created_at: chrono::Utc::now(),
        };

        let mut lobbies = self.lobbies.write().await;
        let mut player_to_lobby = self.player_to_lobby.write().await;
        let mut lobby_broadcasters = self.lobby_broadcasters.write().await;

        if player_to_lobby.contains_key(&host_id) {
            return Err(LobbyError::PlayerAlreadyInLobby);
        }

        let (tx, _) = broadcast::channel(100);
        lobby_broadcasters.insert(lobby_id.clone(), tx);

        lobbies.insert(lobby_id.clone(), lobby.clone());
        player_to_lobby.insert(host_id, lobby_id);

        Ok(lobby)
    }

    pub async fn join_lobby(&self, player_username: String, lobby_id: String) -> LobbyResult<Lobby> {
        if player_username.trim().is_empty() {
            return Err(LobbyError::InvalidInput("Username cannot be empty".to_string()));
        }

        let player_id = Uuid::new_v4().to_string();
        let player = Player {
            id: player_id.clone(),
            username: player_username,
        };

        let mut lobbies = self.lobbies.write().await;
        let mut player_to_lobby = self.player_to_lobby.write().await;

        if player_to_lobby.contains_key(&player_id) {
            return Err(LobbyError::PlayerAlreadyInLobby);
        }

        let lobby = lobbies.get_mut(&lobby_id).ok_or(LobbyError::LobbyNotFound)?;
        if lobby.players.len() >= lobby.max_players as usize {
            return Err(LobbyError::LobbyFull);
        }

        lobby.players.push(player);
        player_to_lobby.insert(player_id, lobby_id);
        
        Ok(lobby.clone())
    }

    pub async fn get_broadcaster(&self, lobby_id: &str) -> LobbyResult<broadcast::Sender<String>> {
        let broadcasters = self.lobby_broadcasters.read().await;
        broadcasters
            .get(lobby_id)
            .cloned()
            .ok_or(LobbyError::LobbyNotFound)
    }

    pub async fn broadcast_to_lobby(&self, lobby_id: &str, message: String) -> LobbyResult<()> {
        let broadcaster = self.get_broadcaster(lobby_id).await?;
        let _ = broadcaster.send(message);
        Ok(())
    }
}