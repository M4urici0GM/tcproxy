-- Your SQL goes here

CREATE TABLE users (
  id BINARY(16) PRIMARY KEY NOT NULL,
  name VARCHAR(50) NOT NULL,
  email VARCHAR(50) NOT NULL,
  password_hash VARCHAR(255) NOT NULL
)
