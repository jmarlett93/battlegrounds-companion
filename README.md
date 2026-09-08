# Battlegrounds Companion

Native Linux Battlegrounds overlay for Omarchy/Hyprland. Reads Hearthstone `Power.log` from a Proton Battlenet prefix, shows a GTK4 layer-shell tier browser (current tavern pool, filtered by lobby tribes when known), and stores combat telemetry in SQLite.

## Install on Omarchy (agent / other machine)

Runtime needs system GTK (normal for Arch). Pick **one** path.

### A — Prebuilt (fastest)

```bash
curl -fsSL https://raw.githubusercontent.com/jmarlett93/battlegrounds-companion/fix/current-pool-and-omarchy-install/contrib/install-omarchy.sh | bash
```

Or manually:

```bash
sudo pacman -S --needed gtk4 gtk4-layer-shell
curl -fsSL -o ~/.local/bin/bgc \
  https://github.com/jmarlett93/battlegrounds-companion/releases/latest/download/bgc
chmod +x ~/.local/bin/bgc
```

Ensure `~/.local/bin` is on `PATH`, then:

```bash
bgc --demo    # smoke test without Hearthstone
bgc           # live overlay next to the game
```

### B — AUR-style PKGBUILD (local makepkg)

Uses the same GitHub Release binary; declares pacman deps.

```bash
git clone https://github.com/jmarlett93/battlegrounds-companion.git
cd battlegrounds-companion
git checkout fix/current-pool-and-omarchy-install   # or master once merged
cd contrib/aur
makepkg -si
bgc --demo
```

### C — Build from source

```bash
sudo pacman -S --needed rust gtk4 gtk4-layer-shell base-devel
source "$HOME/.cargo/env"
git clone https://github.com/jmarlett93/battlegrounds-companion.git
cd battlegrounds-companion
git checkout fix/current-pool-and-omarchy-install
cargo build --release -p bgc-app
install -Dm755 target/release/bgc ~/.local/bin/bgc
```

## Hearthstone setup

- Battlenet/Hearthstone under Proton at `~/Games/battlenet` (default Omarchy path).
- Enable `Power` logging in the prefix `log.config` (same as usual BG trackers).
- Run the game **windowed or borderless**, not exclusive fullscreen.
- Optional Hyprland notes: [`contrib/hyprland.lua`](contrib/hyprland.lua).

Data: `~/.local/share/battlegrounds-companion/` (`bgc.sqlite`, `cards_cache.json`).

If `Power.log` is missing, the overlay still opens with status `waiting for logs`.

## Dev rebuild

```bash
cargo run -p bgc-app            # live
cargo run -p bgc-app -- --demo  # fixture
cargo test --workspace
```

## Publish a new prebuilt (maintainer)

On Omarchy (so the binary matches Arch libs):

```bash
cargo build --release -p bgc-app
sha256sum target/release/bgc   # update contrib/aur/PKGBUILD sha256sums[0]
gh release create v0.1.1 target/release/bgc --title "v0.1.1" --generate-notes
```

Bump `pkgver` / `sha256sums` in `contrib/aur/PKGBUILD` to match.

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

Bob's Buddy / rules engine, Postgres, Duos UI, art CDN, Flatpak, official AUR submit.
