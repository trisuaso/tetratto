CREATE TABLE IF NOT EXISTS reports (
    ip TEXT NOT NULL,
    created INTEGER NOT NULL PRIMARY KEY,
    owner TEXT NOT NULL,
    content TEXT NOT NULL,
    asset INTEGER NOT NULL,
    asset_type TEXT NOT NULL
)
