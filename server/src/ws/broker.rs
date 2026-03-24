// The broker lives inside GameRoom.broadcast (a tokio broadcast::Sender<ServerMsg>).
//
// Usage pattern:
//   room.broadcast.send(msg) — sends to all active WebSocket sessions for that game.
//
// Sessions subscribe via room.broadcast.subscribe() when they connect.
// This module is intentionally thin; real logic is in state.rs and session.rs.
