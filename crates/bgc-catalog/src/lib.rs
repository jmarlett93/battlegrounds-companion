//! Card catalog from HearthstoneJSON with local cache and offline fixture.

use serde::{Deserialize, Serialize};
use std::path::Path;

const HSJSON_URL: &str = "https://api.hearthstonejson.com/v1/latest/enUS/cards.json";

/// One Battlegrounds pool minion (or fixture stand-in).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub tier: u8,
    #[serde(default)]
    pub race: Option<String>,
}

/// In-memory catalog of BG minions.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub cards: Vec<Card>,
    pub source: String,
}

impl Catalog {
    /// Tier minions; if `races` is non-empty, keep neutrals/`ALL` plus matching tribes.
    pub fn names_for_tier(&self, tier: u8, races: &[String]) -> Vec<String> {
        self.cards_for_tier(tier, races)
            .into_iter()
            .map(|c| c.name.clone())
            .collect()
    }

    pub fn cards_for_tier(&self, tier: u8, races: &[String]) -> Vec<&Card> {
        self.cards
            .iter()
            .filter(|c| c.tier == tier)
            .filter(|c| race_allowed(&c.race, races))
            .collect()
    }
}

fn race_allowed(race: &Option<String>, available: &[String]) -> bool {
    if available.is_empty() {
        return true;
    }
    match race.as_deref() {
        None | Some("ALL") => true,
        Some(r) => available.iter().any(|a| a.eq_ignore_ascii_case(r)),
    }
}

/// Tiny offline fixture so UI works with no network/cache.
pub fn fixture_cards() -> Vec<Card> {
    vec![
        Card {
            id: "BG_FIXTURE_1".into(),
            name: "Fixture Murloc Pup".into(),
            tier: 1,
            race: Some("MURLOC".into()),
        },
        Card {
            id: "BG_FIXTURE_2".into(),
            name: "Fixture Alleycat".into(),
            tier: 1,
            race: Some("BEAST".into()),
        },
        Card {
            id: "BG_FIXTURE_3".into(),
            name: "Fixture Harvest Golem".into(),
            tier: 2,
            race: Some("MECH".into()),
        },
        Card {
            id: "BG_FIXTURE_4".into(),
            name: "Fixture Sellemental".into(),
            tier: 2,
            race: Some("ELEMENTAL".into()),
        },
        Card {
            id: "BG_FIXTURE_5".into(),
            name: "Fixture Imp Gang Boss".into(),
            tier: 3,
            race: Some("DEMON".into()),
        },
        Card {
            id: "BG_FIXTURE_6".into(),
            name: "Fixture Bronze Warden".into(),
            tier: 3,
            race: Some("DRAGON".into()),
        },
        Card {
            id: "BG_FIXTURE_7".into(),
            name: "Fixture Cave Hydra".into(),
            tier: 4,
            race: Some("BEAST".into()),
        },
        Card {
            id: "BG_FIXTURE_8".into(),
            name: "Fixture Lightfang Enforcer".into(),
            tier: 5,
            race: None,
        },
        Card {
            id: "BG_FIXTURE_9".into(),
            name: "Fixture Kalecgos".into(),
            tier: 6,
            race: Some("DRAGON".into()),
        },
        Card {
            id: "BG_FIXTURE_10".into(),
            name: "Fixture Amalgadon".into(),
            tier: 6,
            race: Some("ALL".into()),
        },
    ]
}

#[derive(Deserialize)]
struct RawCard {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "techLevel")]
    tech_level: Option<u8>,
    #[serde(rename = "isBattlegroundsPoolMinion")]
    is_bg_pool: Option<bool>,
    #[serde(rename = "isBattlegroundsDuosExclusive")]
    is_duos: Option<bool>,
    race: Option<String>,
}

fn parse_bg_minions(json: &str) -> Result<Vec<Card>, String> {
    let raw: Vec<RawCard> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for c in raw {
        if c.is_bg_pool != Some(true) || c.is_duos == Some(true) {
            continue;
        }
        let tier = match c.tech_level {
            Some(t) if (1..=6).contains(&t) => t,
            _ => continue,
        };
        let (Some(id), Some(name)) = (c.id, c.name) else {
            continue;
        };
        out.push(Card {
            id,
            name,
            tier,
            race: c.race,
        });
    }
    out.sort_by(|a, b| a.tier.cmp(&b.tier).then(a.name.cmp(&b.name)));
    out.dedup_by(|a, b| a.id == b.id);
    Ok(out)
}

fn load_cache(path: &Path) -> Option<Vec<Card>> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_cache(path: &Path, cards: &[Card]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string(cards).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

fn fetch_remote() -> Result<Vec<Card>, String> {
    let body = ureq::get(HSJSON_URL)
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    parse_bg_minions(&body)
}

/// Refresh from network when possible; else cache; else fixture.
pub fn load_or_refresh(cache_path: &Path) -> Catalog {
    match fetch_remote() {
        Ok(cards) if !cards.is_empty() => {
            let _ = save_cache(cache_path, &cards);
            return Catalog {
                cards,
                source: "hearthstonejson".into(),
            };
        }
        Ok(_) => {}
        Err(_) => {}
    }
    if let Some(cards) = load_cache(cache_path) {
        if !cards.is_empty() {
            return Catalog {
                cards,
                source: "cache".into(),
            };
        }
    }
    Catalog {
        cards: fixture_cards(),
        source: "fixture".into(),
    }
}

/// Build catalog only from fixture (tests / forced offline).
pub fn load_fixture() -> Catalog {
    Catalog {
        cards: fixture_cards(),
        source: "fixture".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_fixture_by_tier_and_race() {
        let cat = load_fixture();
        let t1 = cat.names_for_tier(1, &[]);
        assert!(t1.iter().any(|n| n.contains("Murloc")));
        assert_eq!(cat.cards_for_tier(1, &[]).len(), 2);
        assert_eq!(cat.cards_for_tier(6, &[]).len(), 2);
        assert!(cat.names_for_tier(4, &[]).iter().any(|n| n.contains("Hydra")));
        let murloc_only = cat.names_for_tier(1, &["MURLOC".into()]);
        assert_eq!(murloc_only, vec!["Fixture Murloc Pup".to_string()]);
    }

    #[test]
    fn parse_keeps_current_pool_drops_duos_and_rotated() {
        let json = r#"[
          {"id":"BG_POOL","name":"Pool Pup","techLevel":1,"isBattlegroundsPoolMinion":true,"race":"MURLOC"},
          {"id":"BG_DUO","name":"Duo Only","techLevel":2,"isBattlegroundsPoolMinion":true,"isBattlegroundsDuosExclusive":true,"race":"MECH"},
          {"id":"BG_OLD","name":"Rotated","techLevel":3,"type":"MINION","race":"BEAST"},
          {"id":"BG_T7","name":"Bad Tier","techLevel":7,"isBattlegroundsPoolMinion":true}
        ]"#;
        let cards = parse_bg_minions(json).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "BG_POOL");
    }
}
