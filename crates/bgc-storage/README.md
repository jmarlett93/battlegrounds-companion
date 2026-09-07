# bgc-storage

Owns XDG data paths under `~/.local/share/battlegrounds-companion` and SQLite open/migrate.

Side effects live here: directory creation, DB file I/O. Callers get a `Storage` handle and run SQL through thin helpers.
