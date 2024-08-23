use std::{collections::HashMap, sync::{atomic::{AtomicU64, Ordering}, Arc}};

use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::Response, routing::get, Router};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::emojicoin_dot_fun::EmojicoinDbEvent;

struct Connection {
    socket: WebSocket,
    id: u64,
}

struct AppState {
    connections: Mutex<HashMap<u64, Connection>>,
}

async fn healthcheck() {}

pub async fn start(mut receiver: UnboundedReceiver<EmojicoinDbEvent>) {
    let port = std::env::var("WS_PORT");
    if port.is_err() {
        tracing::error!("Environment variable WS_PORT is not set.");
        return;
    }
    let port: Result<u16,_> = port.unwrap().parse();
    if port.is_err() {
        tracing::error!("Environment variable WS_PORT is not a valid port number.");
        return;
    }
    let port = port.unwrap();

    let app_state = AppState {
        connections: Mutex::new(HashMap::new())
    };
    let app_state = Arc::new(app_state);
    let app_state_clone = app_state.clone();
    let app = Router::new()
        .route("/ws", get(handler))
        .route("/", get(healthcheck))
        .with_state(app_state);

    let sender_handler = tokio::spawn(async move {
        let app_state = app_state_clone;
        while let Some(value) = receiver.recv().await {
            let value_string = serde_json::to_string(&value).unwrap();
            let mut to_remove = vec![];
            let mut connections_mut = app_state.connections.lock().await;
            for connection in connections_mut.values_mut() {
                let res = connection.socket.send(Message::Text(value_string.clone())).await;
                if res.is_err() {
                    to_remove.push(connection.id);
                }
            }
            for id in to_remove {
                tracing::info!("Removing connection with ID {id}");
                let connection = connections_mut.remove(&id);
                if let Some(connection) = connection {
                    let _ = connection.socket.close();
                }
            }
        }
    });

    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{port}")).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    tokio::select! {
        _ = sender_handler => {
            tracing::error!("Sender error.")
        }
        _ = server_handle => {
            tracing::error!("Server error")
        }
    };
}

async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state.clone()))
}

static NEXT_USER_ID: AtomicU64 = AtomicU64::new(0);

async fn handle_websocket(socket: WebSocket, app_state: Arc<AppState>) {
    let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!("New connection with ID {user_id}");
    app_state.connections.lock().await.insert(user_id, Connection {
        socket,
        id: user_id,
    });
}

