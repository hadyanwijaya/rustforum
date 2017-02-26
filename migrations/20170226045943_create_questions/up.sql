-- Your SQL goes here

CREATE TABLE questions (
  id SERIAL PRIMARY KEY,
  question_text TEXT NOT NULL,
  tags VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL,
  user_id VARCHAR NOT NULL
)