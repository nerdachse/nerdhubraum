use warp::{Filter, Rejection};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use tracing::info;

/// Our global unique id counter.
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

type Clients = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

pub fn distributor_websocket() -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + Send { 
    let clients = Clients::default();
    let clients = warp::any().map(move || clients.clone());

    warp::path("ws")
        .and(warp::ws())
        .and(clients)
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| client_connected(socket, clients))
        })
}

async fn client_connected(ws: WebSocket, clients: Clients) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    info!("new client: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut client_ws_tx, mut client_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            client_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected clients.
    clients.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific client's connection.

    // When any client sends a message, broadcast it to
    // all other users...
    while let Some(result) = client_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        handle_client_message(my_id, msg, &clients).await;
    }

    // client_ws_rx stream will keep processing as long as the client stays
    // connected. Once they disconnect, then...
    client_disconnected(my_id, &clients).await;
}

async fn handle_client_message(my_id: usize, msg: Message, users: &Clients) {
    info!("Got msg: {:?}", msg);
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    //let new_msg = format!("<Client#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn client_disconnected(my_id: usize, clients: &Clients) {
    info!("client {} left", my_id);

    // Stream closed up, so remove from the user list
    clients.write().await.remove(&my_id);
}
