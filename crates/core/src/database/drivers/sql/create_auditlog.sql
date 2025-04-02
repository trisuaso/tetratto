CREATE TABLE IF NOT EXISTS auditlog (
    ip TEXT NOT NULL,
    created INTEGER NOT NULL PRIMARY KEY,
    moderator TEXT NOT NULL,
    content TEXT NOT NULL
)
