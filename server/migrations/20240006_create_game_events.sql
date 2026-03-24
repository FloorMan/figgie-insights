CREATE TABLE game_events (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id    UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload    JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX game_events_game_id_idx ON game_events(game_id);
