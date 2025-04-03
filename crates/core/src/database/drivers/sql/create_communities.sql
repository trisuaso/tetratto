CREATE TABLE IF NOT EXISTS communities (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    title TEXT NOT NULL,
    context TEXT NOT NULL,
    owner BIGINT NOT NULL,
    read_access TEXT NOT NULL,
    write_access TEXT NOT NULL,
    join_access TEXT NOT NULL,
    -- likes
    likes BIGINT NOT NULL,
    dislikes BIGINT NOT NULL,
    -- counts
    member_count BIGINT NOT NULL
)
