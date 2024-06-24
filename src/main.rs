#![warn(clippy::all, clippy::pedantic)]

use hacker_news_cli::terminal::Terminal;

#[tokio::main]
async fn main() {
    Terminal::default().run().await;
}
