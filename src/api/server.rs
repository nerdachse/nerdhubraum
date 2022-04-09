use serde::Serialize;
use std::convert::Infallible;
use warp::{Filter, Rejection};

use sqlx::SqlitePool;

#[derive(sqlx::FromRow, Serialize)]
pub struct Follower {
    pub followed_at: String,
    pub follower_id: i64,
    pub name: String,
}

async fn get_twitch_followers(pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    let followers = sqlx::query_as::<_, Follower>("SELECT * FROM followers")
        // https://docs.rs/sqlx/0.5.10/sqlx/macro.query_as.html
        .fetch_all(&pool)
        .await
        .expect("failed to select followers");

    Ok(warp::reply::json(&followers))
}

pub async fn start(pool: SqlitePool) {
    let api = warp::path("api");
    let twitch = api.and(warp::path("twitch"));

    let get_twitch_followers = twitch
        .and(warp::path("followers"))
        .and(with_db(pool.clone()))
        .and_then(get_twitch_followers);

    let ws = super::websocket::distributor_websocket();
    let overlays = overlays();

    let routes = ws
        .or(get_twitch_followers)
        .or(overlays);

    warp::serve(routes).run(([127, 0, 0, 1], 13337)).await;
}

fn overlays() -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + Send { 
    let overlays = warp::path("overlays");
    let follower_alert = overlays
        .and(warp::path("follower_alert"))
        // Note: warp::fs::dir starts from the src folder
        .and(warp::fs::dir("frontend/overlays/follower_alert"));

    let memes = overlays
        .and(warp::path("memes"))
        // Note: warp::fs::dir starts from the src folder
        .and(warp::fs::dir("frontend/overlays/memes"));

    memes.or(follower_alert)
}

fn with_db(pool: SqlitePool) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
