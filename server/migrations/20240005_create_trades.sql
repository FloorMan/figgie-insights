CREATE TABLE trades (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id    UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    buyer_id   UUID NOT NULL REFERENCES players(id),
    seller_id  UUID NOT NULL REFERENCES players(id),
    suit       TEXT NOT NULL,
    price      INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX trades_game_id_idx ON trades(game_id);
