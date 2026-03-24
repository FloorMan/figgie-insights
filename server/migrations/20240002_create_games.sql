CREATE TABLE games (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status      TEXT NOT NULL DEFAULT 'lobby',
    common_suit TEXT,
    target_suit TEXT,
    pot         INT NOT NULL DEFAULT 200,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
