use std::{collections::HashMap, error::Error, ops::Range};

use serde::Deserialize;
use tokio::task::JoinSet;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
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

const TOP_STORIES: &str = "topstories.json";
const BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Default)]
pub struct HttpClient {
    top_ids: Option<Vec<i32>>,
    new_ids: Option<Vec<i32>>,
    show_ids: Option<Vec<i32>>,
    ask_ids: Option<Vec<i32>>,
    jobs_ids: Option<Vec<i32>>,
    items: HashMap<i32, Item>,
}

impl HttpClient {
    pub async fn get_top(&mut self, range: Range<usize>) -> Result<Vec<Item>, Box<dyn Error>> {
        if self.top_ids.is_none() {
            self.top_ids = Some(Self::get_ids(TOP_STORIES).await?);
        }

        let top_ids = &self.top_ids.clone().unwrap()[range];
        let stories = self.fetch_stories(top_ids).await?;
        stories.iter().for_each(|s| {
            self.items.insert(s.id, s.clone());
        });
        Ok(stories)
    }

    pub async fn get_new(&mut self, range: Range<usize>) -> Result<Vec<Item>, Box<dyn Error>> {
        if self.new_ids.is_none() {
            self.new_ids = Some(Self::get_ids(TOP_STORIES).await?);
        }

        let new_ids = &self.new_ids.clone().unwrap()[range];
        let stories = self.fetch_stories(new_ids).await?;
        self.add_to_cache(&stories);
        Ok(stories)
    }

    pub async fn get_show(&mut self, range: Range<usize>) -> Result<Vec<Item>, Box<dyn Error>> {
        if self.show_ids.is_none() {
            self.show_ids = Some(Self::get_ids(TOP_STORIES).await?);
        }

        let show_ids = &self.show_ids.clone().unwrap()[range];
        let stories = self.fetch_stories(show_ids).await?;
        self.add_to_cache(&stories);
        Ok(stories)
    }

    pub async fn get_ask(&mut self, range: Range<usize>) -> Result<Vec<Item>, Box<dyn Error>> {
        if self.ask_ids.is_none() {
            self.ask_ids = Some(Self::get_ids(TOP_STORIES).await?);
        }

        let ask_ids = &self.ask_ids.clone().unwrap()[range];
        let stories = self.fetch_stories(ask_ids).await?;
        self.add_to_cache(&stories);
        Ok(stories)
    }

    pub async fn get_jobs(&mut self, range: Range<usize>) -> Result<Vec<Item>, Box<dyn Error>> {
        if self.jobs_ids.is_none() {
            self.jobs_ids = Some(Self::get_ids(TOP_STORIES).await?);
        }

        let jobs_ids = &self.jobs_ids.clone().unwrap()[range];
        let stories = self.fetch_stories(jobs_ids).await?;
        self.add_to_cache(&stories);
        Ok(stories)
    }

    fn add_to_cache(&mut self, stories: &[Item]) {
        stories.iter().for_each(|s| {
            self.items.insert(s.id, s.clone());
        })
    }

    async fn get_ids(url_path: &str) -> Result<Vec<i32>, reqwest::Error> {
        let url = format!("{BASE_URL}/{url_path}");

        reqwest::get(url).await?.json::<Vec<i32>>().await
    }

    async fn fetch_stories(&self, ids: &[i32]) -> Result<Vec<Item>, Box<dyn Error>> {
        let mut items: Vec<Item> = Vec::new();
        let mut set = JoinSet::new();

        for id in ids {
            match self.items.get(id) {
                Some(item) => items.push(item.clone()),
                None => {
                    set.spawn(Self::fetch_item(*id));
                }
            }
        }

        while let Some(res) = set.join_next().await {
            items.push(res??);
        }

        Ok(items)
    }

    async fn fetch_item(id: i32) -> Result<Item, reqwest::Error> {
        let url = format!("{}/item/{}.json", BASE_URL, id);
        let response = reqwest::get(url).await?;
        let data = response.json::<Item>().await?;
        Ok(data)
    }
}
