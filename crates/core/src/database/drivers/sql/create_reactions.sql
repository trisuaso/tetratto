CREATE TABLE IF NOT EXISTS reactions (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    owner INTEGER NOT NULL,
    asset INTEGER NOT NULL,
    asset_type TEXT NOT NULL,
    is_like INTEGER NOT NULL
)
