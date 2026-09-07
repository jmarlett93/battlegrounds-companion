//! SQLite storage under XDG share for battlegrounds-companion.

use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS combat_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    game_id TEXT NOT NULL,
    body TEXT NOT NULL
);
"#;

/// Opened SQLite DB plus resolved data directory.
pub struct Storage {
    pub data_dir: PathBuf,
    conn: Connection,
}

impl Storage {
    /// Create XDG dirs, open DB, run migrate.
    pub fn open() -> Result<Self, String> {
        let dirs = ProjectDirs::from("dev", "battlegrounds", "battlegrounds-companion")
            .ok_or_else(|| "could not resolve XDG dirs".to_string())?;
        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let db_path = data_dir.join("bgc.sqlite");
        Self::open_at(&data_dir, &db_path)
    }

    /// Open at explicit paths (tests).
    pub fn open_at(data_dir: &Path, db_path: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        let s = Self {
            data_dir: data_dir.to_path_buf(),
            conn,
        };
        s.migrate()?;
        Ok(s)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(SCHEMA)
            .map_err(|e| e.to_string())
    }

    pub fn catalog_cache_path(&self) -> PathBuf {
        self.data_dir.join("cards_cache.json")
    }

    /// Insert a combat_v1 JSON document.
    pub fn insert_combat_doc(
        &self,
        schema: &str,
        captured_at: &str,
        game_id: &str,
        body: &str,
    ) -> Result<i64, String> {
        self.conn
            .execute(
                "INSERT INTO combat_docs (schema, captured_at, game_id, body) VALUES (?1, ?2, ?3, ?4)",
                params![schema, captured_at, game_id, body],
            )
            .map_err(|e| e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn count_combat_docs(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM combat_docs", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn migrate_smoke() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bgc-storage-test-{stamp}"));
        let db = dir.join("test.sqlite");
        let s = Storage::open_at(&dir, &db).expect("open");
        assert_eq!(s.count_combat_docs().unwrap(), 0);
        let _ = s
            .insert_combat_doc("combat_v1", "now", "g1", "{}")
            .unwrap();
        assert_eq!(s.count_combat_docs().unwrap(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
