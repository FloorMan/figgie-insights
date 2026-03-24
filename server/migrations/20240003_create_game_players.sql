CREATE TABLE game_players (
    game_id     UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id   UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    seat        INT NOT NULL,
    is_bot      BOOLEAN NOT NULL DEFAULT false,
    chips_start INT NOT NULL DEFAULT 50,
    chips_end   INT,
    PRIMARY KEY (game_id, player_id)
);
