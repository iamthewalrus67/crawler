use std::{collections::{HashMap, HashSet, BTreeSet}, sync::{Mutex, Arc}};

use tokio::sync::mpsc;

use crate::{parsing::{ParseArguments, ParsedData}, crawling::AgentManager};

mod parsing;
mod crawling;

enum Url {
    Sent,
    NotSent,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json_string_from_server = reqwest::get("http://18.222.66.143/crawlerManager/getAddress").await?.text().await?;
    let json: serde_json::Value = serde_json::from_str(&json_string_from_server)?;
    let start_url = json.get("address").unwrap().as_str().unwrap().to_string();
    println!("Start url: {}", start_url);

    let mut urls: HashMap<String, Url> = HashMap::new();

    urls.insert(start_url, Url::NotSent);

    let pages_limit: u64 = 20;
    let mut visited_pages: u64 = 0;

    let agent_manager = AgentManager::new(1, "test_crawler".to_string());

    while visited_pages < pages_limit {
        let mut urls_for_one_agent = vec![];
        let mut urls_iter = urls.iter_mut();

        while urls_for_one_agent.len() < 10 {
            if let Some(url) = urls_iter.next() {
                match url.1 {
                    Url::NotSent => {
                        urls_for_one_agent.push(url.0.clone());
                        *url.1 = Url::Sent;
                    },
                    // TODO: Create an error state to resend after some time
                    _ => continue,
                }
            } else {
                break;
            }
        }
        agent_manager.url_sender.send(urls_for_one_agent).unwrap();

        if let Ok(res) = agent_manager.data_receiver.try_recv() {
            match res {
                Ok(parsed_data) => {
                    println!("{} {}: {}", visited_pages, parsed_data.page_url, parsed_data.page_title);
                    // println!("{}", parsed_data.text);
                    for link in parsed_data.links {
                        urls.entry(link).or_insert(Url::NotSent);
                    }
                    urls.remove(&parsed_data.page_url);
                    visited_pages += 1;
                },
                Err(page_url) => {
                    // TODO: Rerty send

                    urls.remove(&page_url);
                }
            }
        }
    }

    Ok(())
}
