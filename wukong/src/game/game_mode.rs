use crate::cache::redis_client::RedisClient;
use crate::game::types::Player;

pub const SERVER_ONLY_AUTHORIZED: &str = "00000000-0000-0000-0000-000000000000";


pub struct GameModeService {
    redis_client: RedisClient,
}


trait GameModeManager {
    async fn init_state_cached(&self, game_id: String, host_player: Player) -> Result<(), ErrorResponse>;
}
