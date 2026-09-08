//! Pure BG game state: phases, races, opponent boards.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Lobby,
    Match,
    Combat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoardMinion {
    pub card_id: String,
    pub name: String,
    pub atk: i32,
    pub health: i32,
    pub position: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct OpponentBoard {
    pub player_id: String,
    pub hero: String,
    pub minions: Vec<BoardMinion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameState {
    pub phase: Phase,
    pub game_id: Option<String>,
    pub available_races: Vec<String>,
    pub combat_index: u32,
    /// Last known board per opponent player id.
    pub opponent_boards: HashMap<String, OpponentBoard>,
    pub last_opponent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EnterLobby,
    EnterMatch { game_id: String },
    SetRaces(Vec<String>),
    EnterCombat,
    EndCombat { opponent: OpponentBoard },
    MatchEnd,
}

/// Apply one event; returns updated state (functional style).
pub fn apply(mut state: GameState, event: Event) -> GameState {
    match event {
        Event::EnterLobby => {
            state.phase = Phase::Lobby;
            state.game_id = None;
            state.available_races.clear();
            state.opponent_boards.clear();
            state.last_opponent_id = None;
            state.combat_index = 0;
        }
        Event::EnterMatch { game_id } => {
            state.phase = Phase::Match;
            state.game_id = Some(game_id);
            state.opponent_boards.clear();
            state.last_opponent_id = None;
            state.combat_index = 0;
        }
        Event::SetRaces(races) => {
            state.available_races = races;
        }
        Event::EnterCombat => {
            state.phase = Phase::Combat;
        }
        Event::EndCombat { opponent } => {
            state.phase = Phase::Match;
            state.combat_index = state.combat_index.saturating_add(1);
            state.last_opponent_id = Some(opponent.player_id.clone());
            state
                .opponent_boards
                .insert(opponent.player_id.clone(), opponent);
        }
        Event::MatchEnd => {
            state.phase = Phase::Idle;
            state.game_id = None;
            state.available_races.clear();
            state.opponent_boards.clear();
            state.last_opponent_id = None;
            state.combat_index = 0;
        }
    }
    state
}

impl GameState {
    pub fn races_label(&self) -> String {
        if self.available_races.is_empty() {
            "Races: Unknown".into()
        } else {
            format!("Races: {}", self.available_races.join(", "))
        }
    }

    pub fn last_opponent_board(&self) -> Option<&OpponentBoard> {
        self.last_opponent_id
            .as_ref()
            .and_then(|id| self.opponent_boards.get(id))
    }

    pub fn phase_label(&self) -> &'static str {
        match self.phase {
            Phase::Idle => "Idle",
            Phase::Lobby => "Lobby",
            Phase::Match => "Match",
            Phase::Combat => "Combat",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_match_and_combat() {
        let s = GameState::default();
        let s = apply(s, Event::EnterLobby);
        assert_eq!(s.phase, Phase::Lobby);
        let s = apply(
            s,
            Event::EnterMatch {
                game_id: "g1".into(),
            },
        );
        assert_eq!(s.phase, Phase::Match);
        let s = apply(s, Event::SetRaces(vec!["BEAST".into(), "MECH".into()]));
        assert_eq!(s.available_races.len(), 2);
        let s = apply(s, Event::EnterCombat);
        assert_eq!(s.phase, Phase::Combat);
        let s = apply(
            s,
            Event::EndCombat {
                opponent: OpponentBoard {
                    player_id: "p2".into(),
                    hero: "Hero".into(),
                    minions: vec![BoardMinion {
                        card_id: "X".into(),
                        name: "Pup".into(),
                        atk: 2,
                        health: 2,
                        position: 0,
                    }],
                },
            },
        );
        assert_eq!(s.phase, Phase::Match);
        assert_eq!(s.combat_index, 1);
        assert!(s.last_opponent_board().unwrap().minions[0].name == "Pup");
    }
}
