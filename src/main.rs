use std::env;
use std::time::Duration;
use tokio::{task, time};

use std::error::Error;
use tracing::{debug, error, info};

mod api;
mod platforms;

use platforms::twitch;

use tokio::fs::read;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use sqlx::sqlite::SqlitePool;
use dotenv::dotenv;

pub const TWITCH_FOLLOWERS_FILE: &'static str = "twitch_followers.json";

const TWITCH_CLIENT_ID: &str = "twitch_client_id";
const TWITCH_AUTH_TOKEN: &str = "twitch_auth_token";
const TWITCH_USER_ID: &str = "twitch_user_id";
const DATABASE_URL: &str = "DATABASE_URL";

fn read_env_variable_or_fail(var_name: &str) -> String {
    env::var(var_name).expect(&format!(
        "Missing env variable \"{var_name}\""
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let client_id = read_env_variable_or_fail(TWITCH_CLIENT_ID);
    let auth_token = read_env_variable_or_fail(TWITCH_AUTH_TOKEN);
    let user_id = read_env_variable_or_fail(TWITCH_USER_ID);
    let db_url = read_env_variable_or_fail(DATABASE_URL);

    pretty_env_logger::init_timed();

    let pool = SqlitePool::connect(&db_url).await?;

    let background_task = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));

        let mut number_of_calls = 1;
        loop {
            debug!("This is the {number_of_calls} call to fetch the followers");
            interval.tick().await;
            let response =
                twitch::get_followers(user_id.clone(), client_id.clone(), auth_token.clone())
                    .await
                    .expect("Couldn't get followers");
            info!(
                "Congrats, you have {total} followers",
                total = response.total
            );
            number_of_calls += 1;
            let serialized = serde_json::to_vec(&response).expect("Failed to serialize response");
            // TODO use sqlx and write to a sqlite database, probably?
            // Or use sled?
            let mut saved = File::create(TWITCH_FOLLOWERS_FILE)
                .await
                .expect("Failed to open file");
            if let Err(_) = saved.write_all(&serialized).await {
                error!("Failed to write followers to file")
            }
        }
    });

    let api_server = task::spawn(api::start(pool.clone()));
    let followers_db_task = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        let mut number_of_calls = 1;
        loop {
            info!("This is the {number_of_calls} call to write the followers to the db");
            interval.tick().await;
            persist_followers_to_db(pool.clone()).await;
            number_of_calls += 1;
        }
    });
    // Await the result of the spawned task.
    let _result = background_task.await?;
    let _result = api_server.await?;
    let _result = followers_db_task.await?;

    Ok(())
}

async fn persist_followers_to_db(pool: SqlitePool) {
    let followers = read(crate::TWITCH_FOLLOWERS_FILE)
        .await
        .expect("Failed to read the followers from disk");
    let followers = String::from_utf8_lossy(&followers);
    let followers: twitch::GetFollowersResponse =
        serde_json::from_str(&followers).expect("couldn't deserialize followers from file");

    for follower in followers.data.iter() {
        let _insert_follower = sqlx::query!(
            r#"
                INSERT INTO followers (name, follower_id, followed_at)
                VALUES (?1, ?2, ?3)
            "#,
            follower.from_name,
            follower.from_id,
            follower.followed_at,
        )
        .execute(&pool)
        .await;
    }
}
