CREATE TABLE IF NOT EXISTS memberships (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    owner INTEGER NOT NULL,
    community INTEGER NOT NULL,
    role INTEGER NOT NULL
)
