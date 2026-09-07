-- Suggested Hyprland / Omarchy window rules for Battlegrounds Companion.
-- Merge into your hyprland config or Omarchy lua window rules as appropriate.
--
-- Keep the overlay above the game; do not steal focus permanently.
-- Game window class is often steam_app_battlenet under umu/Proton.

-- Layer-shell clients usually appear as layers, not normal windows.
-- If the GTK app is treated as a normal window, pin it:
--
-- windowrulev2 = float, class:^(dev.battlegrounds.companion)$
-- windowrulev2 = pin, class:^(dev.battlegrounds.companion)$
-- windowrulev2 = noinitialfocus, class:^(dev.battlegrounds.companion)$

return {
  note = "Prefer HS windowed/borderless; exclusive fullscreen fights overlays",
  app_id = "dev.battlegrounds.companion",
  game_class = "steam_app_battlenet",
}
