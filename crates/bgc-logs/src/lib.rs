//! Power.log discovery and simple line parse hooks.

use bgc_state::{BoardMinion, Event, OpponentBoard};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Candidate relative paths under the Proton prefix.
const LOG_REL_CANDIDATES: &[&str] = &[
    "drive_c/Program Files (x86)/Hearthstone/Logs/Power.log",
    "drive_c/Program Files/Hearthstone/Logs/Power.log",
    "drive_c/program files (x86)/hearthstone/logs/Power.log",
    "drive_c/program files (x86)/Hearthstone/Logs/power.log",
];

pub fn default_prefix() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Games/battlenet")
}

/// Resolve first existing Power.log under prefix (case path variants).
pub fn find_power_log(prefix: &Path) -> Option<PathBuf> {
    for rel in LOG_REL_CANDIDATES {
        let p = prefix.join(rel);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Map a Power.log line into an optional state event (stub markers).
pub fn parse_line(line: &str) -> Option<Event> {
    let l = line.trim();
    if l.contains("BGC_DEMO_COMBAT_END")
        || (l.contains("TAG_CHANGE") && l.contains("BOARD_VISUAL_STATE"))
    {
        // ponytail: crude marker only; real combat-end from zone/tag parse later
        return Some(Event::EndCombat {
            opponent: OpponentBoard {
                player_id: "log_opp".into(),
                hero: "Unknown".into(),
                minions: vec![BoardMinion {
                    card_id: "LOG".into(),
                    name: "Parsed Minion".into(),
                    atk: 1,
                    health: 1,
                    position: 0,
                }],
            },
        });
    }
    if l.contains("GameType=GT_BATTLEGROUNDS")
        || (l.contains("CREATE_GAME") && l.contains("Battlegrounds"))
    {
        return Some(Event::EnterMatch {
            game_id: "log_game".into(),
        });
    }
    if l.contains("BGC_RACES=") {
        if let Some(rest) = l.split("BGC_RACES=").nth(1) {
            let races: Vec<String> = rest
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Some(Event::SetRaces(races));
        }
    }
    None
}

/// Polling tailer: read new bytes every interval and yield events.
pub struct LogTailer {
    path: PathBuf,
    pos: u64,
}

impl LogTailer {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
        Ok(Self {
            path,
            pos: meta.len(),
        })
    }

    /// Read newly appended text; return parsed events.
    pub fn poll(&mut self) -> Result<Vec<Event>, String> {
        let mut file = File::open(&self.path).map_err(|e| e.to_string())?;
        let len = file.metadata().map_err(|e| e.to_string())?.len();
        if len < self.pos {
            // log rotated / truncated
            self.pos = 0;
        }
        if len == self.pos {
            return Ok(Vec::new());
        }
        file.seek(SeekFrom::Start(self.pos))
            .map_err(|e| e.to_string())?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
        self.pos = len;
        let events = buf.lines().filter_map(parse_line).collect();
        Ok(events)
    }
}

pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_races_marker() {
        let e = parse_line("BGC_RACES=BEAST,MECH").unwrap();
        match e {
            Event::SetRaces(r) => assert_eq!(r, vec!["BEAST", "MECH"]),
            _ => panic!("expected races"),
        }
    }
}
