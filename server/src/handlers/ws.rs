use crate::state::AppState;
use crate::ws::session::handle_session;
use axum::{
    extract::{
        ws::WebSocketUpgrade,
        Path, Query, State,
    },
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WsQuery {
    pub player_id: Uuid,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_session(socket, game_id, query.player_id, state))
}
