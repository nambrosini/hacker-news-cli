use std::error::Error;

use serde::Deserialize;

fn print_kid(id: &i32, tab: i32) {
    let item = fetch_item(id).unwrap();
    let mut tabs = String::new();
    for _ in 0..tab {
        tabs.push('\t');
    }
    println!("{}", item.text.unwrap());
    if let Some(kids) = item.kids {
        for k in kids {
            print_kid(&k, tab + 1);
        }
    }
}

fn fetch_item(id: &i32) -> Result<Item, Box<dyn Error>> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let response = reqwest::blocking::get(url)?.json::<Item>()?;
    Ok(response)
}

#[derive(Debug, Deserialize)]
struct Item {
    by: Option<String>,
    descendants: Option<i32>,
    id: i32,
    kids: Option<Vec<i32>>,
    score: Option<i32>,
    time: i32,
    title: Option<String>,
    r#type: String,
    url: Option<String>,
    text: Option<String>,
}

fn fetch_top_stories() -> Result<Vec<i32>, Box<dyn Error>> {
    let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let response = reqwest::blocking::get(url)?.json::<Vec<i32>>()?;
    Ok(response)
}
