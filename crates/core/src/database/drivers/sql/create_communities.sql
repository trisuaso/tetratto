CREATE TABLE IF NOT EXISTS communities (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    title TEXT NOT NULL,
    context TEXT NOT NULL,
    owner INTEGER NOT NULL,
    read_access TEXT NOT NULL,
    write_access TEXT NOT NULL,
    join_access TEXT NOT NULL,
    -- likes
    likes INTEGER NOT NULL,
    dislikes INTEGER NOT NULL,
    -- counts
    member_count INTEGER NOT NULL
)
