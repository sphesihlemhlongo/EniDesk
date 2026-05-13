use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc, RwLock};

type ClientId = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum SignalMessage {
    Register { id: ClientId },
    Offer { target: ClientId, sdp: String },
    Answer { target: ClientId, sdp: String },
    IceCandidate { target: ClientId, candidate: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ServerMessage {
    Registered { id: ClientId },
    Offer { from: ClientId, sdp: String },
    Answer { from: ClientId, sdp: String },
    IceCandidate { from: ClientId, candidate: String },
    Error { message: String },
}

type ClientSender = mpsc::UnboundedSender<Message>;

struct AppState {
    clients: RwLock<HashMap<ClientId, ClientSender>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        clients: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Signaling server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Task to forward messages from the channel to the websocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut current_client_id: Option<ClientId> = None;

    // Task to handle incoming websocket messages
    let recv_state = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {
                    match signal {
                        SignalMessage::Register { id } => {
                            let mut clients = recv_state.clients.write().await;
                            clients.insert(id.clone(), tx.clone());
                            current_client_id = Some(id.clone());
                            let reply = ServerMessage::Registered { id };
                            let _ = tx.send(Message::Text(serde_json::to_string(&reply).unwrap()));
                            println!("Client registered: {}", current_client_id.as_ref().unwrap());
                        }
                        SignalMessage::Offer { target, sdp } => {
                            if let Some(from_id) = &current_client_id {
                                let clients = recv_state.clients.read().await;
                                if let Some(target_tx) = clients.get(&target) {
                                    let fwd = ServerMessage::Offer { from: from_id.clone(), sdp };
                                    let _ = target_tx.send(Message::Text(serde_json::to_string(&fwd).unwrap()));
                                }
                            }
                        }
                        SignalMessage::Answer { target, sdp } => {
                            if let Some(from_id) = &current_client_id {
                                let clients = recv_state.clients.read().await;
                                if let Some(target_tx) = clients.get(&target) {
                                    let fwd = ServerMessage::Answer { from: from_id.clone(), sdp };
                                    let _ = target_tx.send(Message::Text(serde_json::to_string(&fwd).unwrap()));
                                }
                            }
                        }
                        SignalMessage::IceCandidate { target, candidate } => {
                            if let Some(from_id) = &current_client_id {
                                let clients = recv_state.clients.read().await;
                                if let Some(target_tx) = clients.get(&target) {
                                    let fwd = ServerMessage::IceCandidate { from: from_id.clone(), candidate };
                                    let _ = target_tx.send(Message::Text(serde_json::to_string(&fwd).unwrap()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Disconnect logic
        if let Some(id) = current_client_id {
            let mut clients = recv_state.clients.write().await;
            clients.remove(&id);
            println!("Client disconnected: {}", id);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
