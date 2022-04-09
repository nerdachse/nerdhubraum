use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, tungstenite::Error as TError, WebSocketStream,
};
use tracing::info;

type WebSocketWrite =
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>;
type WebSocketRead =
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;

const TWITCH_PUBSUB_URL: &str = "wss://pubsub-edge.twitch.tv";

pub async fn listen_to_twitch_follows() {
    let (ws_stream, _) = connect_async(TWITCH_PUBSUB_URL)
        .await
        .expect("Failed to connect to TWITCH_PUBSUB_URL");

    let (mut write, mut read) = ws_stream.split();

    let topic = format!("following.{channel}", channel = crate::TWITCH_USER_ID);
    let auth_token = crate::TWITCH_AUTH_TOKEN;

    let request = "{
      \"type\": \"LISTEN\",
      \"nonce\": \"1234\",
      \"data\": {
        \"topics\": [\"{topic}\"],
        \"auth_token\": \"{auth_token}\"
      }
    }";

    write
        .send(Message::Text(request.to_owned()))
        .await
        .expect("Failed to subscribe to channel follows");

    while let Some(msg) = read.next().await {
        let msg = msg.expect("failed to get msg");
        info!("msg: {msg}");
    }
}
