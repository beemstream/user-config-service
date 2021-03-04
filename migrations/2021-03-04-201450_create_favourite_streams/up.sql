-- Your SQL goes here
CREATE TABLE favourite_streams (
  id SERIAL PRIMARY KEY,
  associated_user VARCHAR NOT NULL,
  identifier VARCHAR NOT NULL,
  source VARCHAR NOT NULL
);
