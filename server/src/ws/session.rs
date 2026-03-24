use crate::state::AppState;
use crate::ws::messages::{ClientMsg, ServerMsg};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;
use uuid::Uuid;

/// Handle a single WebSocket connection for a player in a game.
pub async fn handle_session(
    socket: WebSocket,
    game_id: Uuid,
    player_id: Uuid,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to the game's broadcast channel
    let mut rx = {
        let rooms = state.rooms.read().await;
        match rooms.get(&game_id) {
            Some(room) => room.broadcast.subscribe(),
            None => {
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&ServerMsg::Error {
                            message: "Game not found".into(),
                        })
                        .unwrap(),
                    ))
                    .await;
                return;
            }
        }
    };

    // Send the current game state immediately on connect
    {
        let rooms = state.rooms.read().await;
        if let Some(room) = rooms.get(&game_id) {
            let engine = room.engine.lock().await;
            let view = engine.view();
            if let Some(hand) = engine.hand_for(player_id) {
                let msg = ServerMsg::GameState { game: view };
                let _ = sender
                    .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                    .await;
                let hand_msg = ServerMsg::YourHand { hand: hand.clone() };
                let _ = sender
                    .send(Message::Text(serde_json::to_string(&hand_msg).unwrap()))
                    .await;
            }
        }
    }

    // Spawn a task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages from this client
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                _ => continue,
            };

            let client_msg: ClientMsg = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(e) => {
                    // Can't easily send back here without complexity; log only
                    tracing::warn!("Invalid message from {player_id}: {e}");
                    continue;
                }
            };

            process_client_msg(client_msg, game_id, player_id, &state_clone).await;
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn process_client_msg(
    msg: ClientMsg,
    game_id: Uuid,
    player_id: Uuid,
    state: &AppState,
) {
    let rooms = state.rooms.read().await;
    let room = match rooms.get(&game_id) {
        Some(r) => r,
        None => return,
    };

    let mut engine = room.engine.lock().await;

    let result: Result<ServerMsg, String> = match msg {
        ClientMsg::PlaceBid { suit, price } => {
            engine.place_bid(player_id, suit, price).map(|order_id| {
                ServerMsg::OrderPlaced {
                    order_id,
                    player_id,
                    suit,
                    side: "bid".into(),
                    price,
                }
            })
        }
        ClientMsg::PlaceAsk { suit, price } => {
            engine.place_ask(player_id, suit, price).map(|order_id| {
                ServerMsg::OrderPlaced {
                    order_id,
                    player_id,
                    suit,
                    side: "ask".into(),
                    price,
                }
            })
        }
        ClientMsg::CancelOrder { order_id } => {
            engine.cancel_order(player_id, order_id);
            Ok(ServerMsg::OrderCancelled { order_id })
        }
        ClientMsg::AcceptBid { suit } => engine
            .accept_bid(player_id, suit)
            .map(|trade| ServerMsg::TradeExecuted { trade }),
        ClientMsg::AcceptAsk { suit } => engine
            .accept_ask(player_id, suit)
            .map(|trade| ServerMsg::TradeExecuted { trade }),
        ClientMsg::Pass => {
            // A pass just broadcasts current state
            Ok(ServerMsg::GameState { game: engine.view() })
        }
        ClientMsg::EndGame => engine
            .end_game()
            .map(|result| ServerMsg::GameEnded { result }),
    };

    match result {
        Ok(server_msg) => {
            // Also persist trades to DB asynchronously
            if let ServerMsg::TradeExecuted { ref trade } = server_msg {
                let db = state.db.clone();
                let trade = trade.clone();
                let gid = game_id;
                tokio::spawn(async move {
                    let _ = persist_trade(&db, gid, &trade).await;
                });
            }
            // Broadcast to all players in the room
            let _ = room.broadcast.send(server_msg);
        }
        Err(e) => {
            tracing::debug!("Action error for player {player_id}: {e}");
            // Error is only for this player — we can't easily send it back here
            // without capturing the sender. The player will see the unchanged state.
        }
    }
}

async fn persist_trade(
    db: &PgPool,
    game_id: Uuid,
    trade: &crate::game::engine::TradeRecord,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO trades (game_id, buyer_id, seller_id, suit, price) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(game_id)
    .bind(trade.buyer_id)
    .bind(trade.seller_id)
    .bind(trade.suit.as_str())
    .bind(trade.price)
    .execute(db)
    .await?;
    Ok(())
}
