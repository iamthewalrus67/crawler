use std::{collections::HashSet, str::FromStr, borrow::BorrowMut, time::Duration};

use select::{
    document::Document,
    predicate::{Name, Or, Element},
};
use governor::{state::{NotKeyed, InMemoryState}, clock::{QuantaClock, QuantaInstant}, middleware::NoOpMiddleware};

pub async fn parse(
    mut args: ParseArguments,
    client: &reqwest::Client,
    rate_limiter: &governor::RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
) -> std::result::Result<ParsedData, String> {
    rate_limiter.until_ready().await;
    let html = match client.get(&args.page_url).send().await {
        Ok(v) => v,
        Err(_) => return Err(args.page_url),
    };

    let html = match html.text().await {
        Ok(v) => v,
        Err(_) => return Err(args.page_url),
    };

    let html = Document::from(&html[..]);

    Ok(
        ParsedData {
            page_url: args.page_url,
            page_title: get_title_from_html(&html),
            text: get_all_text_from_html(&html),
            links: get_urls_from_html(&html, &args.base_url),
            // page_language: ,
        },)
}

fn get_title_from_html(html: &Document) -> String {
    match html.find(Name("title")).next() {
        Some(v) => v.text(),
        None => "".to_string(),
    }
}

fn get_urls_from_html(html: &Document, base_url: &str) -> HashSet<String> {
    html.find(Or(Name("a"), Name("link")))
        .filter_map(|el| el.attr("href"))
        .filter_map(|url| normalize_url(url, base_url))
        .collect::<HashSet<String>>()
}

fn get_all_text_from_html(html: &Document) -> String{
    let text_elements = vec!["p", "b", "strong", "i", "em", "mark", "small", "ins", "sub", "sup"].iter().cloned().collect::<HashSet<&str>>();

    html.find(Element)
        .filter(|el| match el.name() {
            Some(name) => text_elements.contains(&name),
            None => false,
        })
        .map(|el| el.text())
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn normalize_url(url: &str, base_url: &str) -> Option<String> {
    match url::Url::parse(url) {
        Ok(mut u) => {
            u.set_fragment(None);
            u.set_query(None);
            Some(u.to_string())
        }
        Err(parse_err) => match parse_err {
            url::ParseError::RelativeUrlWithoutBase => {
                match url::Url::from_str(base_url).unwrap().join(url) {
                    Ok(v) => Some(v.to_string()),
                    Err(_) => None,
                }
            }
            _ => None,
        },
    }
}

#[derive(Debug)]
pub struct ParseArguments {
    pub page_url: String,
    pub base_url: String,
    // pub service: tower::util::BoxService<reqwest::Request, reqwest::Response, reqwest::Error>
}

#[derive(Debug)]
pub struct ParsedData {
    pub page_url: String,
    pub page_title: String,
    pub text: String,
    pub links: HashSet<String>,
    // page_language: String,
}