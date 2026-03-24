CREATE TABLE orders (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id    UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id  UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    suit       TEXT NOT NULL,
    side       TEXT NOT NULL,
    price      INT NOT NULL,
    active     BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX orders_game_id_idx ON orders(game_id);
CREATE INDEX orders_active_idx ON orders(game_id, suit, side) WHERE active = true;
