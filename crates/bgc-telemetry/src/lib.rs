//! Combat telemetry writer (`combat_v1` JSON → SQLite).

use bgc_state::{BoardMinion, GameState, OpponentBoard};
use bgc_storage::Storage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CombatDoc {
    pub schema: String,
    pub captured_at: String,
    pub game_id: String,
    pub combat_index: u32,
    pub available_races: Vec<String>,
    pub board_opponent: Vec<BoardMinion>,
    pub opponent_hero: String,
    pub opponent_player_id: String,
}

fn now_iso() -> String {
    // ponytail: plain UTC-ish timestamp string; chrono crate if RFC3339 needed
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

pub fn build_combat_doc(state: &GameState, opponent: &OpponentBoard) -> CombatDoc {
    CombatDoc {
        schema: "combat_v1".into(),
        captured_at: now_iso(),
        game_id: state.game_id.clone().unwrap_or_else(|| "unknown".into()),
        combat_index: state.combat_index,
        available_races: state.available_races.clone(),
        board_opponent: opponent.minions.clone(),
        opponent_hero: opponent.hero.clone(),
        opponent_player_id: opponent.player_id.clone(),
    }
}

/// Serialize and insert one combat document.
pub fn write_combat(storage: &Storage, state: &GameState, opponent: &OpponentBoard) -> Result<i64, String> {
    let doc = build_combat_doc(state, opponent);
    let body = serde_json::to_string(&doc).map_err(|e| e.to_string())?;
    storage.insert_combat_doc(&doc.schema, &doc.captured_at, &doc.game_id, &body)
}
