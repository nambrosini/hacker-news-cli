use std::error::Error;

use serde::Deserialize;
use tokio::task::JoinSet;

const BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Deserialize)]
pub struct Item {
    by: Option<String>,
    descendants: Option<i32>,
    id: i32,
    kids: Option<Vec<i32>>,
    score: Option<i32>,
    time: i32,
    pub title: Option<String>,
    r#type: String,
    pub url: Option<String>,
    pub text: Option<String>,
}

//fn print_kid(id: &i32, tab: i32) {
//    let item = fetch_item(id).unwrap();
//    let mut tabs = String::new();
//    for _ in 0..tab {
//        tabs.push('\t');
//    }
//    println!("{}", item.text.unwrap());
//    if let Some(kids) = item.kids {
//        for k in kids {
//            print_kid(&k, tab + 1);
//        }
//    }
//}

pub async fn fetch_item(id: i32) -> Result<Item, reqwest::Error> {
    let url = format!("{}/item/{}.json", BASE_URL, id);
    let response = reqwest::get(url).await?;
    let data = response.json::<Item>().await?;
    Ok(data)
}

pub async fn fetch_best(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = format!("{}/topstories.json", BASE_URL);
    fetch_stories(&url, amount).await
}

pub async fn fetch_new(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = format!("{}/newstories.json", BASE_URL);
    fetch_stories(&url, amount).await
}

pub async fn fetch_show(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = format!("{}/showstories.json", BASE_URL);
    fetch_stories(&url, amount).await
}

pub async fn fetch_ask(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = format!("{}/askstories.json", BASE_URL);
    fetch_stories(&url, amount).await
}

pub async fn fetch_jobs(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = format!("{}/jobstories.json", BASE_URL);
    fetch_stories(&url, amount).await
}

async fn fetch_stories(url: &str, amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let response: Vec<i32> = reqwest::get(url).await?.json::<Vec<i32>>().await?;

    let mut items: Vec<Item> = Vec::new();
    let mut set = JoinSet::new();

    let response: Vec<i32> = response.into_iter().take(amount).collect();

    for id in response {
        set.spawn(fetch_item(id));
    }

    while let Some(res) = set.join_next().await {
        items.push(res??);
    }

    Ok(items)
}
