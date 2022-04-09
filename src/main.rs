use std::env;
use std::time::Duration;
use tokio::{task, time};

use std::error::Error;
use tracing::{debug, error, info};

mod api;
mod platforms;

use platforms::twitch;

use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;

mod experiments;

const TWITCH_CLIENT_ID: &str = "twitch_client_id";
pub const TWITCH_AUTH_TOKEN: &str = "twitch_auth_token";
pub const TWITCH_USER_ID: &str = "twitch_user_id";
const DATABASE_URL: &str = "DATABASE_URL";

fn read_env_variable_or_fail(var_name: &str) -> String {
    env::var(var_name).expect(&format!("Missing env variable \"{var_name}\""))
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

    //let pool_cloned = pool.clone();
    let get_and_save_followers = task::spawn(get_and_save_followers(
        user_id,
        client_id,
        auth_token,
        pool.clone(),
    ));

    let api_server = task::spawn(api::start(pool.clone()));
    //let pubsub_follows = task::spawn(experiments::listen_to_twitch_follows());
    // Await the result of the spawned task.
    let _result = get_and_save_followers.await?;
    let _result = api_server.await?;
    //let _result = pubsub_follows.await?;

    Ok(())
}

async fn get_and_save_followers(
    user_id: String,
    client_id: String,
    auth_token: String,
    pool: SqlitePool,
) {
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

        for follower in response.data.iter() {
            let insert_follower = sqlx::query!(
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

            if let Err(e) = insert_follower {
                if let Some(e) = e.into_database_error() {
                    if !e
                        .message()
                        .contains("UNIQUE constraint failed: followers.follower_id")
                    {
                        error!("Failed to write follower to db: {:?}", follower)
                    }
                } else {
                    error!("Failed to write follower to db: {:?}", follower)
                }
            }
        }
    }
}
