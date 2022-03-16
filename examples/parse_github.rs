use std::sync::{Arc, Mutex};
use std::{collections::HashSet, str::FromStr};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/";
    let mut links_to_visit = get_links_from_url(url).await?;

    println!("{:#?}", links_to_visit);

    // Consider using parking_lot::Mutex
    let visited_links = Arc::new(Mutex::new(HashSet::new()));

    println!("links_to_visit.len(): {:?}", links_to_visit.len());
    println!(
        "visited_links.len(): {:?}",
        visited_links.lock().unwrap().len()
    );

    let mut i = 3;

    loop {
        let now = std::time::Instant::now();

        // Multi-producer, single-consumer channel for sending and receiving new urls from parsed web pages
        let (tx, mut rx) = mpsc::channel(links_to_visit.len());

        let mut tasks = Vec::new();

        let mut j = 20;

        loop {
            if j == 0 {
                break;
            }

            j -= 1;

            let link = match links_to_visit.iter().next() {
                Some(link) => link.clone(),
                None => break,
            };
            links_to_visit.remove(&link);

            let visited_links_arc_clone = Arc::clone(&visited_links);
            let tx_clone = tx.clone();

            tasks.push(tokio::spawn(async move {
                match get_links_from_url(&link).await {
                    Ok(parsed_links) => {
                        tx_clone.send(parsed_links).await.unwrap();
                        visited_links_arc_clone.lock().unwrap().insert(link.clone());
                    }
                    Err(e) => eprintln!("An error occured: {}", e),
                };
            }));
        }

        for task in tasks {
            task.await?;
        }

        drop(tx);

        while let Some(parsed_links) = rx.recv().await {
            links_to_visit.extend(parsed_links);
        }

        links_to_visit = &links_to_visit - &visited_links.lock().unwrap();

        println!("links_to_visit.len(): {:?}", links_to_visit.len());
        println!(
            "visited_links.len(): {:?}",
            visited_links.lock().unwrap().len()
        );
        println!("{}", now.elapsed().as_micros());

        if i == 0 {
            break;
        }

        i -= 1;
    }

    // println!("{:#?}", links_to_visit);
    // println!("links_to_visit.len(): {:?}", links_to_visit.len());
    // println!("visited_links.len(): {:?}", visited_links.lock().unwrap().len());

    Ok(())
}

/// Returns a HashSet of links from url.
/// Returns an error if it couldn't make a request or parse the html page.
async fn get_links_from_url(url: &str) -> Result<HashSet<String>, reqwest::Error> {
    use select::document::Document;
    use select::predicate::Name;

    let resp = reqwest::get(url).await?;
    let content = resp.text().await?;

    let links = Document::from(content.as_str())
        .find(Name("a"))
        .filter_map(|el| {
            let link = match el.attr("href") {
                Some(link) => link,
                None => "",
            };
            normalize_link(url, link)
        })
        .collect::<HashSet<String>>();

    return Ok(links);
}

/// Returns an absolute url.
fn normalize_link(base_link: &str, link: &str) -> Option<String> {
    use url::{ParseError, Url};

    match Url::parse(link) {
        Ok(new_url) => {
            if new_url.has_host() {
                Some(new_url.to_string())
            } else {
                None
            }
        }
        Err(parse_err) => match parse_err {
            ParseError::RelativeUrlWithoutBase => Some({
                Url::from_str(base_link)
                    .unwrap()
                    .join(link)
                    .unwrap()
                    .to_string()
            }),
            _ => None,
        },
    }
}
