mod get_followers;

pub(in crate::platforms::twitch) const TWITCH_HELIX_API_ENDPOINT: &'static str =
    "https://api.twitch.tv/helix";

pub use get_followers::{get_followers, GetFollowersResponse, SingleFollower};
