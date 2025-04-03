CREATE TABLE IF NOT EXISTS userfollows (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    initiator BIGINT NOT NULL,
    receiver BIGINT NOT NULL
)
