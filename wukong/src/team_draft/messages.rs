use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TeamDraftTimerReason {
    WaitingForPoolAndCompetition,
    YapperStartingDraft,
    DraftPickShowcase,
    WaitingForDraftPick,
    TransitionToAwarding,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetPoolMessage {
    pub pool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetCompetitionMessage {
    pub competition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StartDraft {
    pub starting_drafter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DraftPick {
    pub drafter_id: String,
    pub pick: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NextDrafter {
    pub drafter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AwardingPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AwardPoint {
    pub player_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NextRound {
    pub round: u8,
    pub team_size: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompleteGame {
    pub player_points: HashMap<String, u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "msg_type")]
pub enum TeamDraftMessage {
    SetPool(SetPoolMessage),
    SetCompetition(SetCompetitionMessage),
    StartDraft(StartDraft),
    DraftPick(DraftPick),
    NextDrafter(NextDrafter),
    AwardingPhase(AwardingPhase),
    AwardPoint(AwardPoint), // check max_rounds here, otherwise back
    NextRound(NextRound), // clear all state, fresh lobby
    CompleteGame(CompleteGame), // clear all state, fresh lobby
}