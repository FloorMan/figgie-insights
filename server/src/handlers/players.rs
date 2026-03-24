use crate::error::{AppError, AppResult};
use crate::models::player::{CreatePlayer, Player, PlayerStats};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

pub async fn create_player(
    State(state): State<AppState>,
    Json(body): Json<CreatePlayer>,
) -> AppResult<Json<Player>> {
    let username = body.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".into()));
    }

    let player = sqlx::query_as::<_, Player>(
        "INSERT INTO players (username) VALUES ($1) RETURNING id, username, created_at",
    )
    .bind(&username)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
            AppError::Conflict(format!("Username '{}' already taken", username))
        }
        other => AppError::Database(other),
    })?;

    Ok(Json(player))
}

pub async fn get_player(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> AppResult<Json<Player>> {
    let player = sqlx::query_as::<_, Player>(
        "SELECT id, username, created_at FROM players WHERE id = $1",
    )
    .bind(player_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Player {} not found", player_id)))?;

    Ok(Json(player))
}

pub async fn get_player_stats(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> AppResult<Json<PlayerStats>> {
    let player = sqlx::query_as::<_, Player>(
        "SELECT id, username, created_at FROM players WHERE id = $1",
    )
    .bind(player_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Player {} not found", player_id)))?;

    let row: (Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*),
            COALESCE(SUM(GREATEST(gp.chips_end - gp.chips_start, 0)), 0),
            COALESCE(SUM(GREATEST(gp.chips_start - gp.chips_end, 0)), 0)
        FROM game_players gp
        WHERE gp.player_id = $1
          AND gp.chips_end IS NOT NULL
        "#,
    )
    .bind(player_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(PlayerStats {
        player_id: player.id,
        username: player.username,
        games_played: row.0.unwrap_or(0),
        total_chips_won: row.1.unwrap_or(0),
        total_chips_lost: row.2.unwrap_or(0),
    }))
}
