//! Social media and platform integration tools
//!
//! Tools for interacting with Twitter, Discord, GitHub, and other platforms.

mod discord;
mod discord_lookup;
mod github_user;
mod twitter_post;
pub mod twitter_oauth;

pub use discord::DiscordTool;
pub use discord_lookup::DiscordLookupTool;
pub use github_user::GithubUserTool;
pub use twitter_oauth::{generate_oauth_header, percent_encode, TwitterCredentials};
pub use twitter_post::TwitterPostTool;
