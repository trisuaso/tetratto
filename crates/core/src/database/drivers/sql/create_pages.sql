CREATE TABLE IF NOT EXISTS pages (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    title TEXT NOT NULL,
    prompt TEXT NOT NULL,
    owner TEXT NOT NULL,
    read_access TEXT NOT NULL,
    write_access TEXT NOT NULL
)
