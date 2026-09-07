# Battlegrounds Companion

Native Linux Battlegrounds overlay for Omarchy/Hyprland. Reads Hearthstone `Power.log` from a Proton Battlenet prefix, shows a GTK4 layer-shell tier browser (current tavern pool, filtered by lobby tribes when known), and stores combat telemetry in SQLite.

## Run on Omarchy (desktop test)

No Flatpak/AppImage packaging — this is a normal GTK4 binary. Build once, put it on your `PATH`, launch it next to Hearthstone.

### 1. System packages

```bash
sudo pacman -S --needed rust gtk4 gtk4-layer-shell base-devel
source "$HOME/.cargo/env"
```

### 2. Clone, build, install

```bash
git clone git@github.com:jmarlett93/battlegrounds-companion.git
cd battlegrounds-companion
git checkout fix/current-pool-and-omarchy-install   # or master once merged

cargo build --release -p bgc-app
install -Dm755 target/release/bgc ~/.local/bin/bgc
```

Ensure `~/.local/bin` is on your `PATH` (Omarchy/user shells usually already do).

Optional app-menu launcher:

```bash
install -Dm644 contrib/bgc.desktop ~/.local/share/applications/bgc.desktop
update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

### 3. Hearthstone setup

- Battlenet/Hearthstone under Proton at `~/Games/battlenet` (default Omarchy path).
- Enable logging: in the Hearthstone install dir (inside the prefix), ensure `log.config` turns on `Power` logging (same as usual BG tracker setups).
- Run the game **windowed or borderless**, not exclusive fullscreen.

### 4. Launch with the game

```bash
# Live overlay: discovers Power.log, fetches current pool from HearthstoneJSON, tails events
bgc

# Offline smoke test (fixture cards + fake combat telemetry, no game required)
bgc --demo
```

Data: `~/.local/share/battlegrounds-companion/` (`bgc.sqlite`, `cards_cache.json`).

If `Power.log` is missing, the overlay still opens with status `waiting for logs`.

### 5. Hyprland

See [`contrib/hyprland.lua`](contrib/hyprland.lua) for suggested layer rules. Prefer not stealing focus from the game.

### Dev rebuild loop

```bash
cd /path/to/battlegrounds-companion
cargo run -p bgc-app            # live
cargo run -p bgc-app -- --demo  # fixture
cargo test --workspace
```

## Workspace crates

| Crate | Role |
|---|---|
| `bgc-storage` | XDG paths + SQLite migrate |
| `bgc-catalog` | HearthstoneJSON current pool + tier/race filter |
| `bgc-logs` | Power.log discovery + tail + parse stubs |
| `bgc-state` | Pure phase/races/opponent-board state |
| `bgc-telemetry` | `combat_v1` JSON → SQLite |
| `bgc-overlay` | GTK4 layer-shell UI |
| `bgc-app` | Binary wiring (`bgc`) |

## Not in MVP

Bob's Buddy / rules engine, Postgres, Duos UI, art CDN, Flatpak packaging.
