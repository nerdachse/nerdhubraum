use serde::Serialize;
use std::convert::Infallible;
use warp::Filter;

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
    // FIXME Don't do this, stupid!
    // else you're opening the box of pandora
    let cors = warp::cors().allow_any_origin();

    let api = warp::path("api");
    let twitch = api.and(warp::path("twitch"));

    let get_twitch_followers = twitch
        .and(warp::path("followers"))
        .and(with_db(pool.clone()))
        .and_then(get_twitch_followers)
        .with(cors);

    warp::serve(get_twitch_followers)
        .run(([127, 0, 0, 1], 13337))
        .await;
}

fn with_db(pool: SqlitePool) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
