#![warn(clippy::all, clippy::pedantic)]

use hacker_news_cli::terminal::Terminal;

fn main() {
    Terminal::default().run();
}
