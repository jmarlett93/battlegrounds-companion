# bgc-catalog

Fetches and caches HearthstoneJSON card data. Keeps current-pool minions
(`isBattlegroundsPoolMinion`, not Duos-exclusive), then filters by tavern tier
and optional lobby races.

Network failures fall back to on-disk cache; empty cache uses a tiny embedded fixture so the overlay works offline.
