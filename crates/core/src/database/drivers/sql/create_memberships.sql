CREATE TABLE IF NOT EXISTS memberships (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    owner BIGINT NOT NULL,
    community BIGINT NOT NULL,
    role INT NOT NULL
)
