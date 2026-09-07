# Battlegrounds Companion — Roadmap

Native Linux (Omarchy / Hyprland) Battlegrounds companion. Reads Hearthstone logs from the Proton/umu prefix; draws a real Wayland overlay. No Wine tracker process. No `BobsBuddy.dll`.

**Target environment:** Omarchy + Hyprland; Hearthstone via `omarchy launch battlenet` (`~/Games/battlenet`, GE-Proton / umu).

**Explicit non-goals (v1–v3):** HSReplay / Tier7 cloud features, constructed/arena tracking, decompiling or wrapping proprietary combat sims.

---

## North star

| Capability | Status target |
|---|---|
| Tier browser (click tiers → pool cards) | Ship early |
| Available / lobby races | Ship early |
| Past-opponent boards on hover | Ship early |
| Fight telemetry corpus | Collect continuously |
| Experiment-driven combat rules + odds | Grow later, honestly |

---

## Phase 0 — Bootstrap (days)

- [ ] `git init` this repo; pick stack (see `PLAN.md`)
- [ ] Detect Omarchy Battle.net prefix (`~/Games/battlenet`) and Hearthstone `Logs/` + `client.config`
- [ ] Ensure `FileSizeLimit.Int=-1` (Wine log truncation fix)
- [ ] Hyprland window/layer rules sketch for overlay (`app_id` / title stable for `o.window` / layer-shell)
- [ ] Card DB client: HearthstoneJSON download + etag/HEAD refresh on launch + manual refresh

**Exit:** App starts, finds logs, refreshes cards, empty overlay shell on Hyprland.

---

## Phase 1 — Low-hanging oneshot (1–3 weeks)

The playable companion without combat odds.

1. **Power.log tailer** — reconnect-safe, BG match detect  
2. **Lobby races + anomaly** — from log/game tags (no process memory)  
3. **Tier browser** — filter pool by tier / race / anomaly; clickable in-overlay UI  
4. **Opponent history** — snapshot boards after each fight; hover roster → last seen board  
5. **Wayland overlay** — layer-shell (or pinned float with documented Hyprland Lua); windowed/borderless HS assumed  

**Exit:** You can play BG with browser + races + past boards; no Wine HDT required for those features.

---

## Phase 2 — Fight telemetry (parallel / immediately after Phase 1 core)

Instrument every combat so Phase 3 has fuel.

- [ ] Structured combat session JSON (boards in, event stream, outcome)
- [ ] Enriched entities (card id, dbfId, keywords, stats, enchantments at snapshot time)
- [ ] Local corpus under XDG (`~/.local/share/battlegrounds-companion/corpus/`)
- [ ] Opt-in only; no network upload by default
- [ ] Corpus browser / export for offline analysis
- [ ] Schema version field so parsers can evolve

**Exit:** Every match you play grows a local dataset of enriched fights.

---

## Phase 3 — Experiment-driven rules engine (ongoing)

Build *your* odds engine from public data + authored rules — not a reverse‑engineered DLL.

1. **Constraint miner** — infer ordering hypotheses from corpus (e.g. SoC ≺ first attack)  
2. **Core combat loop** — attack order, taunt targeting, DS / venomous / reborn basics  
3. **Handler registry** — per-mechanic / per-card behavior with tests tied to corpus cases  
4. **Monte Carlo runner** — only after the loop is trustworthy on simple boards  
5. **Odds panel** — win / tie / loss (+ lethal later); show confidence / “unsupported mechanic” warnings  
6. **Patch workflow** — card DB auto-update; failing corpus tests → fix handlers; miner suggests new constraints  

**Exit:** Useful odds on common boards; honest degradation when rules are unknown; continuous improvement loop after each patch.

---

## Priority order (execution)

```
Phase 0 → Phase 1.1–1.3 (logs, races, browser)
       → Phase 1.4–1.5 (history + overlay polish)
       → Phase 2 (telemetry)  ← start as soon as combat parsing exists
       → Phase 3 (rules + MC) ← only with a growing corpus
```

Phase 2 should **overlap** late Phase 1: once you can parse a fight, start recording it even before the hover UI is perfect.

---

## Success metrics

| Phase | Done when |
|---|---|
| 1 | Full BG lobby→game session usable without HDT for browser/races/history |
| 2 | ≥ N enriched combats stored with stable schema (pick N, e.g. 200) |
| 3 | MC odds within tolerance of empirical corpus rates on a held-out simple-board set; patch checklist takes &lt; 1 evening for typical minion pool updates |

---

## Risks (roadmap level)

- Log-only tribe detection flaky in early lobby → tolerate “unknown until confirmed”
- Hyprland fullscreen eating overlays → document windowed/borderless; prefer layer-shell
- Phase 3 scope creep → gate MC behind corpus size + passing core tests
- Patch breaks handlers → telemetry + tests are the safety net, not patch notes alone

---

## Related doc

Detailed design, schemas, stack, and work breakdown: [`PLAN.md`](./PLAN.md)
