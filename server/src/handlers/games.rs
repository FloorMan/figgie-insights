use crate::error::{AppError, AppResult};
use crate::game::engine::GameEngine;
use crate::models::game::{Game, GamePlayer, GameWithPlayers};
use crate::models::trade::Trade;
use crate::state::{AppState, GameRoom};
use crate::ws::messages::ServerMsg;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

pub async fn create_game(State(state): State<AppState>) -> AppResult<Json<Game>> {
    let game = sqlx::query_as::<_, Game>(
        "INSERT INTO games DEFAULT VALUES \
         RETURNING id, status, common_suit, target_suit, pot, created_at, finished_at",
    )
    .fetch_one(&state.db)
    .await?;

    let engine = GameEngine::new(game.id);
    let room = GameRoom::new(engine);
    state.rooms.write().await.insert(game.id, room);

    Ok(Json(game))
}

pub async fn list_games(State(state): State<AppState>) -> AppResult<Json<Vec<Game>>> {
    let games = sqlx::query_as::<_, Game>(
        "SELECT id, status, common_suit, target_suit, pot, created_at, finished_at \
         FROM games WHERE status = 'lobby' ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(games))
}

pub async fn get_game(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
) -> AppResult<Json<GameWithPlayers>> {
    let game = sqlx::query_as::<_, Game>(
        "SELECT id, status, common_suit, target_suit, pot, created_at, finished_at \
         FROM games WHERE id = $1",
    )
    .bind(game_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Game {} not found", game_id)))?;

    let players = sqlx::query_as::<_, GamePlayer>(
        "SELECT game_id, player_id, seat, is_bot, chips_start, chips_end \
         FROM game_players WHERE game_id = $1 ORDER BY seat",
    )
    .bind(game_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(GameWithPlayers { game, players }))
}

#[derive(Deserialize)]
pub struct JoinGameBody {
    pub player_id: Uuid,
}

pub async fn join_game(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Json(body): Json<JoinGameBody>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify player exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM players WHERE id = $1")
        .bind(body.player_id)
        .fetch_one(&state.db)
        .await?;

    if count.0 == 0 {
        return Err(AppError::NotFound(format!("Player {} not found", body.player_id)));
    }

    // Add to in-memory engine
    let seat = {
        let rooms = state.rooms.read().await;
        let room = rooms
            .get(&game_id)
            .ok_or_else(|| AppError::NotFound(format!("Game {} not found", game_id)))?;
        let mut engine = room.engine.lock().await;
        engine
            .add_player(body.player_id, false)
            .map_err(AppError::BadRequest)?
    };

    // Persist to DB
    sqlx::query(
        "INSERT INTO game_players (game_id, player_id, seat, is_bot, chips_start) \
         VALUES ($1, $2, $3, false, 50) ON CONFLICT DO NOTHING",
    )
    .bind(game_id)
    .bind(body.player_id)
    .bind(seat as i32)
    .execute(&state.db)
    .await?;

    // Broadcast join event
    {
        let rooms = state.rooms.read().await;
        if let Some(room) = rooms.get(&game_id) {
            let _ = room.broadcast.send(ServerMsg::PlayerJoined {
                player_id: body.player_id,
                seat,
            });

            let should_start = {
                let engine = room.engine.lock().await;
                engine.players.len() == 4
            };

            if should_start {
                start_game_internal(game_id, &state, room).await?;
            }
        }
    }

    Ok(Json(serde_json::json!({ "seat": seat, "game_id": game_id })))
}

async fn start_game_internal(
    game_id: Uuid,
    state: &AppState,
    room: &GameRoom,
) -> AppResult<()> {
    let (view, first_hand) = {
        let mut engine = room.engine.lock().await;
        engine.start().map_err(AppError::BadRequest)?;

        let first_player = engine.players[0];
        let hand = engine.hand_for(first_player).cloned()
            .unwrap_or_default();
        (engine.view(), hand)
    };

    sqlx::query("UPDATE games SET status = 'active' WHERE id = $1")
        .bind(game_id)
        .execute(&state.db)
        .await?;

    let _ = room.broadcast.send(ServerMsg::GameStarted {
        your_hand: first_hand,
        game: view,
    });

    Ok(())
}

pub async fn get_game_history(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
) -> AppResult<Json<Vec<Trade>>> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games WHERE id = $1")
        .bind(game_id)
        .fetch_one(&state.db)
        .await?;

    if count.0 == 0 {
        return Err(AppError::NotFound(format!("Game {} not found", game_id)));
    }

    let trades = sqlx::query_as::<_, Trade>(
        "SELECT id, game_id, buyer_id, seller_id, suit, price, created_at \
         FROM trades WHERE game_id = $1 ORDER BY created_at",
    )
    .bind(game_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(trades))
}
