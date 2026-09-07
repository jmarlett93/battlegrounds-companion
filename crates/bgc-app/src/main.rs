//! Battlegrounds companion entrypoint.

use bgc_logs::{default_prefix, find_power_log, LogTailer, POLL_INTERVAL};
use bgc_overlay::{run as run_overlay, OverlaySnapshot, UiMsg};
use bgc_state::{apply, BoardMinion, Event, GameState, OpponentBoard};
use bgc_storage::Storage;
use bgc_telemetry::write_combat;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

fn usage() {
    eprintln!("Usage: bgc [--demo]");
}

fn push_snap(
    tx: &mpsc::Sender<OverlaySnapshot>,
    status: &str,
    state: &GameState,
    tier: u8,
) {
    let _ = tx.send(OverlaySnapshot {
        status: status.to_string(),
        state: state.clone(),
        selected_tier: tier,
    });
}

fn handle_event(
    state: GameState,
    event: Event,
    storage: &Storage,
    status: &mut String,
) -> GameState {
    // Capture opponent before apply for telemetry on EndCombat.
    let opponent_for_telem = match &event {
        Event::EndCombat { opponent } => Some(opponent.clone()),
        _ => None,
    };
    let next = apply(state, event);
    if let Some(opp) = opponent_for_telem {
        match write_combat(storage, &next, &opp) {
            Ok(id) => *status = format!("telemetry combat id={id}"),
            Err(e) => *status = format!("telemetry error: {e}"),
        }
    }
    next
}

fn main() {
    let demo = std::env::args().any(|a| a == "--demo");
    if std::env::args().any(|a| a == "-h" || a == "--help") {
        usage();
        return;
    }

    let storage = Storage::open().unwrap_or_else(|e| {
        eprintln!("storage: {e}");
        std::process::exit(1);
    });
    let cache = storage.catalog_cache_path();
    let catalog = if demo {
        // Demo prefers fixture so it always has known tier cards offline.
        bgc_catalog::load_fixture()
    } else {
        bgc_catalog::load_or_refresh(&cache)
    };
    let catalog = Arc::new(catalog);

    let prefix = default_prefix();
    let log_path = find_power_log(&prefix);
    let mut status = match &log_path {
        Some(p) => format!("logs: {}", p.display()),
        None => "waiting for logs".to_string(),
    };
    if demo {
        status = "demo mode".to_string();
    }

    let (snap_tx, snap_rx) = mpsc::channel::<OverlaySnapshot>();
    let (ui_tx, ui_rx) = mpsc::channel::<UiMsg>();

    let mut state = GameState::default();
    let selected_tier: u8 = 1;

    if demo {
        state = apply(state, Event::EnterLobby);
        state = apply(
            state,
            Event::EnterMatch {
                game_id: "demo-game".into(),
            },
        );
        state = apply(
            state,
            Event::SetRaces(vec![
                "BEAST".into(),
                "MECH".into(),
                "MURLOC".into(),
            ]),
        );
        let opp = OpponentBoard {
            player_id: "demo_opp".into(),
            hero: "Demo Hero".into(),
            minions: vec![
                BoardMinion {
                    card_id: "BG_FIXTURE_1".into(),
                    name: "Fixture Murloc Pup".into(),
                    atk: 2,
                    health: 2,
                    position: 0,
                },
                BoardMinion {
                    card_id: "BG_FIXTURE_3".into(),
                    name: "Fixture Harvest Golem".into(),
                    atk: 2,
                    health: 3,
                    position: 1,
                },
            ],
        };
        state = apply(state, Event::EnterCombat);
        state = handle_event(state, Event::EndCombat { opponent: opp }, &storage, &mut status);
    }

    let initial = OverlaySnapshot {
        status: status.clone(),
        state: state.clone(),
        selected_tier,
    };

    thread::spawn(move || {
        let mut state = state;
        let mut status = status;
        let mut selected_tier = selected_tier;
        let storage = storage;
        let mut tailer = log_path.and_then(|p| LogTailer::open(p).ok());
        loop {
            while let Ok(msg) = ui_rx.try_recv() {
                match msg {
                    UiMsg::SelectTier(t) => {
                        selected_tier = t;
                        push_snap(&snap_tx, &status, &state, selected_tier);
                    }
                    UiMsg::Quit => return,
                }
            }

            if let Some(t) = tailer.as_mut() {
                if let Ok(events) = t.poll() {
                    for ev in events {
                        state = handle_event(state, ev, &storage, &mut status);
                        push_snap(&snap_tx, &status, &state, selected_tier);
                    }
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });

    run_overlay(catalog, ui_tx, snap_rx, initial);
}
