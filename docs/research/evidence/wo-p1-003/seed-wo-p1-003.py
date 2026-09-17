#!/usr/bin/env python3
"""Seed the Flauz lab state DB with a multi-folder project (WO-P1-003).

Usage: seed-wo-p1-003.py <state.sqlite3> <primary> <related>...
Inserts the primary into recent_workspaces (INSERT OR REPLACE, preserving
name/pinned if the row exists) and replaces workspace_folders with the
related list in order. Unix paths are stored as raw UTF-8 BLOBs.
"""
import sqlite3, sys, time

db, primary, related = sys.argv[1], sys.argv[2], sys.argv[3:]
conn = sqlite3.connect(db)
now = int(time.time())
cur = conn.execute("SELECT name, pinned FROM recent_workspaces WHERE path = ?", (primary.encode(),))
row = cur.fetchone()
name, pinned = row if row else ("Alpha", 0)
conn.execute(
    "INSERT OR REPLACE INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?, ?, ?, ?)",
    (primary.encode(), now, name, pinned),
)
conn.execute("DELETE FROM workspace_folders WHERE workspace_path = ?", (primary.encode(),))
for pos, folder in enumerate(related):
    conn.execute(
        "INSERT INTO workspace_folders(workspace_path, folder_path, position) VALUES (?, ?, ?)",
        (primary.encode(), folder.encode(), pos),
    )
conn.commit()
print(f"seeded: primary={primary} related={related} name={name} pinned={pinned}")
