#!/usr/bin/env bash
# Install bgc on Omarchy/Arch from the latest GitHub Release.
# Agent-friendly: pacman deps + curl binary. No Flatpak.
set -euo pipefail

REPO="jmarlett93/battlegrounds-companion"
BIN_URL="https://github.com/${REPO}/releases/latest/download/bgc"
PREFIX="${PREFIX:-$HOME/.local}"

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "missing: $1" >&2; exit 1; }; }
need_cmd curl
need_cmd install

echo "Installing runtime deps (gtk4, gtk4-layer-shell)…"
sudo pacman -S --needed --noconfirm gtk4 gtk4-layer-shell

mkdir -p "$PREFIX/bin" "$PREFIX/share/applications"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

echo "Downloading $BIN_URL …"
curl -fsSL "$BIN_URL" -o "$tmp"
chmod +x "$tmp"
install -Dm755 "$tmp" "$PREFIX/bin/bgc"

cat >"$tmp" <<EOF
[Desktop Entry]
Type=Application
Name=Battlegrounds Companion
Comment=Hearthstone Battlegrounds overlay for Omarchy
Exec=$PREFIX/bin/bgc
Icon=applications-games
Terminal=false
Categories=Game;
StartupNotify=false
EOF
install -Dm644 "$tmp" "$PREFIX/share/applications/bgc.desktop"
update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo "Installed: $PREFIX/bin/bgc"
echo "Try: bgc --demo   (or bgc with Hearthstone running)"
if ! command -v bgc >/dev/null 2>&1; then
  echo "Note: add $PREFIX/bin to PATH if needed."
fi
