use std::error::Error;

use serde::Deserialize;

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

pub fn fetch_item(id: &i32) -> Result<Item, Box<dyn Error>> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let response = reqwest::blocking::get(url)?.json::<Item>()?;
    Ok(response)
}

pub fn fetch_top_stories(amount: usize) -> Result<Vec<Item>, Box<dyn Error>> {
    let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let response: Vec<i32> = reqwest::blocking::get(url)?.json::<Vec<i32>>()?;

    let mut items = Vec::new();

    for id in response.iter().take(amount) {
        let item = fetch_item(id)?;
        items.push(item);
    }

    Ok(items)
}
