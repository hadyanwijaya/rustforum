-- Your SQL goes here

CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  token VARCHAR NULL,
  created_at TIMESTAMP NOT NULL,
  logout_at TIMESTAMP NULL
)