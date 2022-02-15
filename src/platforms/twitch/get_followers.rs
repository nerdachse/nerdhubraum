use serde::{Deserialize, Serialize};
use ureq::get;

use super::TWITCH_HELIX_API_ENDPOINT;

#[derive(Deserialize, Serialize, Debug)]
pub struct SingleFollower {
    pub followed_at: String,
    pub from_id: String,
    from_login: String,
    pub from_name: String,
    to_id: String,
    to_login: String,
    to_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetFollowersResponse {
    pub data: Vec<SingleFollower>,
    //pagination: Value, // I don't care
    pub total: u32,
}

pub async fn get_followers(
    user_id: String,
    client_id: String,
    auth_token: String,
) -> Result<GetFollowersResponse, ureq::Error> {
    let url = format!("{TWITCH_HELIX_API_ENDPOINT}/users/follows?to_id={user_id}&first=100");
    let response: String = get(&url)
        .set("client-id", &client_id)
        .set("Authorization", &format!("Bearer {auth_token}"))
        .call()?
        .into_string()?;

    let response: GetFollowersResponse =
        serde_json::from_str(&response).expect("couldn't deserialize response from twitch");

    Ok(response)
}
