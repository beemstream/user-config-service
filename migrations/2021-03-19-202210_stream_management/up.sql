-- Your SQL goes here
CREATE TABLE stream_title (
    id SERIAL PRIMARY KEY,
    associated_user VARCHAR NOT NULL,
    title VARCHAR NOT NULL
);

CREATE TABLE stream_tag (
    id SERIAL PRIMARY KEY,
    associated_title INT NOT NULL references stream_title(id),
    source_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL
);
