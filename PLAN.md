# Battlegrounds Companion — Plan

Companion document to [`ROADMAP.md`](./ROADMAP.md). This is the build plan: architecture, feature specs, data schemas, stack choices, and phase work breakdown for a **new repo**.

---

## 1. Problem

Hearthstone Deck Tracker (HDT) Battlegrounds overlay is a WPF `Topmost` window that does not integrate cleanly with Omarchy (Hyprland Wayland + Proton/umu). Most desired BG UX does **not** require HDT or `BobsBuddy.dll`. Combat win odds do require a maintained rules engine; that can be grown later from public logs and an owned ruleset.

### In scope (product)

1. Clickable **tier browser** (pool cards by tier / race)
2. **Available races** + **races in current match**
3. **Past opponent boards** on hover
4. **Fight telemetry** corpus (enriched JSON)
5. Later: **experiment-driven rules engine** + Monte Carlo odds panel

### Out of scope

- Decompiling / wrapping `BobsBuddy.dll`
- HSReplay OAuth, Tier7, inspiration cloud features
- Constructed / Arena / Mercenaries
- Process memory reading (HearthMirror-style) in v1 — logs + card DB only

---

## 2. Runtime context (Omarchy)

| Item | Expected path / behavior |
|---|---|
| Prefix | `~/Games/battlenet` |
| Launch | `omarchy launch battlenet` → umu + `GE-Proton` |
| Window class (game) | `steam_app_battlenet` |
| Logs | Under prefix `drive_c/Program Files (x86)/Hearthstone/Logs/` (confirm on machine) |
| Config fix | `client.config` → `FileSizeLimit.Int=-1` |
| Display | Prefer HS **windowed / borderless**; exclusive fullscreen may still fight overlays |

Overlay must be a **native Wayland client** (layer-shell preferred), not a Wine window.

---

## 3. Recommended stack

Prefer one process, low RAM, Hyprland-friendly.

| Layer | Suggestion | Notes |
|---|---|---|
| Language | **Python 3.12+** or **TypeScript (Node)** | Python matches log-tooling / corpus scripts; TS fine if preferred |
| Overlay UI | **Qt 6 (PySide6) + layer-shell** or **GTK4 layer-shell** | Real compositor overlay; avoid Electron |
| Optional desktop prefs UI | Small Qt window or later Angular webview | Keep overlay native |
| Card data | HearthstoneJSON | Etag/HEAD refresh |
| Config / data | XDG: `~/.config/…`, `~/.local/share/…` | |
| Hyprland | Documented `o.window` / layer rules in repo `contrib/hyprland.lua` | |

**Decision to lock in Phase 0:** Python+PySide6 vs TS+GTK. Default recommendation: **Python + PySide6** for faster log/corpus iteration.

---

## 4. Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Companion process (native)                             │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ LogTailer    │→ │ GameState    │→ │ Overlay UI    │ │
│  │ Power.log    │  │ BG match     │  │ layer-shell   │ │
│  └──────────────┘  │ boards/races │  └───────────────┘ │
│                    └──────┬───────┘                     │
│                           │                             │
│                    ┌──────▼───────┐  ┌────────────────┐ │
│                    │ Telemetry    │  │ CardCatalog    │ │
│                    │ writer       │  │ HearthstoneJSON│ │
│                    └──────┬───────┘  └────────────────┘ │
│                           ▼                             │
│                 ~/.local/share/.../corpus/              │
│                           │                             │
│                    ┌──────▼───────┐                     │
│                    │ Rules lab    │  (Phase 3 tools)    │
│                    │ miner+sim+MC │                     │
│                    └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
         ▲
         │ read-only
 ~/Games/battlenet/.../Hearthstone/Logs/Power.log
```

Side-effect isolation: log I/O, card download, and corpus writes behind narrow modules; UI only binds to immutable view-models / snapshots.

---

## 5. Feature specs

### 5.1 Card catalog + tier browser

**Behavior**
- On launch (throttled ≤ every 12h) and on manual refresh: check HearthstoneJSON; download if changed
- Build BG minion pool for current season/patch from DB + filters
- Overlay: tier tabs/buttons 1–6 (or anomaly-adjusted tier set); optional race filters
- Click tier → scrollable/grid list of legal pool cards (name, tier, race, stats, key text)
- When lobby races known, default filter to those races (+ neutral / all-tribe as appropriate)

**Anomaly-aware tiers** (same idea as HDT `GetAvailableTiers`): map known anomaly card ids to allowed tavern tiers; default `{1..6}`.

**Patch impact:** data refresh only for pool changes; code change only if filter tags or anomaly ids need new handlers.

### 5.2 Available races / match races

**Behavior**
- Detect BG game start from Power.log
- Resolve available tribes from game entity tags / logged race availability (log-only; no memory)
- Show persistent “races in this lobby” chip/row on overlay
- Until known: show `Unknown` (do not invent)

**Risk:** early lobby may lack tags briefly — update when first definitive event arrives; cache per `game_id`.

### 5.3 Past opponent boards

**Behavior**
- After each combat, snapshot opponent board (minions: card id, attack, health, divine shield, poisonous/venomous, reborn, golden, position)
- Key by stable player id / hero entity for the match
- On hover (or click) of opponent in roster / last-opponent affordance: show last board faced
- Clear on match end

**UI:** compact minion row (stats + art thumbnail if cached); no need for full HDT chrome.

### 5.4 Fight telemetry (Phase 2)

**Goal:** every combat becomes a schema-versioned JSON document for mining and tests.

**Suggested schema (`combat_v1`)**

```json
{
  "schema": "combat_v1",
  "captured_at": "ISO-8601",
  "game_id": "string",
  "combat_index": 0,
  "build": { "hearthstone_version": "optional", "companion_version": "0.1.0" },
  "you": { "player_id": "", "hero_card_id": "", "tavern_tier": 1, "armor": 0, "health": 0 },
  "opponent": { "player_id": "", "hero_card_id": "", "tavern_tier": 1, "armor": 0, "health": 0 },
  "available_races": ["BEAST", "MECH"],
  "anomaly_card_id": null,
  "board_you": [ { "entity_id": 1, "card_id": "", "dbf_id": 0, "atk": 0, "health": 0,
                   "position": 0, "golden": false, "keywords": ["DIVINE_SHIELD"],
                   "enchantments": [] } ],
  "board_opponent": [ ],
  "events": [
    { "t": 0, "op": "START_OF_COMBAT", "source": {}, "targets": [], "raw": {} },
    { "t": 1, "op": "ATTACK", "source": {}, "targets": [], "raw": {} }
  ],
  "outcome": { "result": "WIN|TIE|LOSS", "damage_dealt": 0, "damage_taken": 0 }
}
```

**Storage:** `~/.local/share/battlegrounds-companion/corpus/YYYY/MM/<game_id>/<combat_index>.json`

**Privacy:** local by default; strip battletags from exports; no upload unless a future explicit opt-in is designed.

**UI:** Settings toggle “Record fight telemetry” (default on for power users / default off for strangers — pick one and document; recommend **default on** for this personal tool).

### 5.5 Rules lab + odds (Phase 3)

**Principles**
- Author rules; use corpus as experiments
- Never require proprietary binaries
- Degrade gracefully: if an unsupported mechanic is on the board, show odds with warning or refuse to sim

**Pipeline**

1. **Normalize** combat JSON → ordered event tokens  
2. **Mine constraints** — e.g. count support for `START_OF_COMBAT ≺ first ATTACK`; flag conflicts  
3. **Core engine** — implement high-confidence global rules + basic keywords  
4. **Handlers** — registry keyed by mechanic/card; unit tests load corpus fixtures  
5. **MC** — N simulations (start 1k–10k), report win/tie/loss (+ optional lethal later)  
6. **Overlay panel** — odds + “rules coverage” indicator  
7. **Patch checklist** — refresh cards → run corpus suite → fix failing handlers → re-mine new cards’ first appearances  

**Inference standard (example):** a ordering rule is *candidate* at ≥20 supporting combats and 0 conflicts; *accepted* at higher threshold (e.g. ≥100) or after manual review. Conditional rules require stratified support.

---

## 6. Phase work breakdown

### Phase 0 — Bootstrap

- [ ] Repo skeleton, README pointing at ROADMAP + PLAN
- [ ] Prefix/log discovery for Omarchy battlenet install
- [ ] `client.config` FileSizeLimit helper
- [ ] CardCatalog module + cache dir
- [ ] Blank overlay window with Hyprland contrib rules
- [ ] Logging (app log under XDG)

### Phase 1 — Oneshot companion

- [ ] Power.log parser sufficient for BG: match start/end, heroes, races, combat boundaries, boards
- [ ] Race display
- [ ] Tier browser wired to CardCatalog + race filter
- [ ] Opponent board history + hover UI
- [ ] Overlay layout: top races, side browser, hover popover
- [ ] Manual card refresh action

### Phase 2 — Telemetry

- [ ] Combat recorder emitting `combat_v1`
- [ ] Enrichment join to CardCatalog at snapshot time
- [ ] Corpus index (sqlite or JSONL manifest)
- [ ] Simple CLI: `companion corpus stats` / `export`
- [ ] Ensure recorder runs even if overlay UI hidden

### Phase 3 — Rules lab

- [ ] Event vocabulary + constraint miner CLI
- [ ] Core combat simulator (no fancy cards first)
- [ ] Corpus regression harness
- [ ] MC + odds view-model
- [ ] Overlay odds panel with coverage warnings
- [ ] Patch runbook in `docs/PATCHING.md`

---

## 7. Testing strategy

| Layer | Approach |
|---|---|
| Parser | Checked-in anonymized Power.log snippets → expected state |
| Browser | Snapshot tests of filtered pools for fixed card DB fixture |
| Telemetry | Golden JSON for one scripted combat |
| Rules | Corpus fixtures; miner conflict reports in CI |
| Overlay | Manual Hyprland checklist (windowed HS, layer above game, click browser) |

Do not require live Hearthstone for unit tests.

---

## 8. Hyprland / Omarchy notes

Ship `contrib/hyprland.lua` examples:

- Match overlay `app_id` / title
- Float or layer; full opacity override (avoid Omarchy default opacity tag)
- `decorate = false`; avoid stealing focus except when interacting with browser
- Document: play **windowed/borderless**

Prefer **wlr-layer-shell** `overlay` layer so stacking does not depend on Wine `Topmost`.

---

## 9. Milestone checklist (copy into issues)

**M1 — See cards**  
Logs found · card DB cached · tier browser usable in a window (overlay optional)

**M2 — Play a lobby**  
Races shown · browser race-filtered · overlay on Hyprland over HS

**M3 — Remember opponents**  
Hover shows last board · history correct across multiple fights

**M4 — Corpus alive**  
Telemetry on · ≥ 50 combats stored · export works

**M5 — First odds**  
Core sim + MC on boards with only basic keywords · panel visible · warnings for unknown mechanics

**M6 — Lab loop**  
Miner + regression harness · patch runbook used once successfully after a real BG patch

---

## 10. Open decisions (resolve in Phase 0)

1. Python+PySide6 vs TypeScript+GTK  
2. Telemetry default on vs off  
3. Art: download card tiles from HearthstoneJSON CDN or text-only v1  
4. Duos: support in parser v1 or defer  
5. Odds UI: always visible vs combat-only  

---

## 11. Repo layout (suggested)

```
battlegrounds-companion/
  ROADMAP.md
  PLAN.md
  README.md
  contrib/hyprland.lua
  docs/PATCHING.md          # add in Phase 3
  src/
    catalog/
    logs/
    state/
    overlay/
    telemetry/
    rules/                  # Phase 3
  tests/
  fixtures/logs/
  fixtures/corpus/
```

---

## 12. One-line summary

Ship a native BG overlay for tier browser, races, and opponent history; record every fight as enriched JSON; grow an owned, test-driven combat rules engine and MC odds from that corpus — without depending on HDT or `BobsBuddy.dll`.
