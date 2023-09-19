-- Your SQL goes here
CREATE TABLE indexers
(
    id         uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    status     VARCHAR NOT NULL,
    type       VARCHAR NOT NULL,
    process_id INT
)
