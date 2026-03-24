CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE players (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username   TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
