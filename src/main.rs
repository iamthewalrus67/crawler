use std::{collections::HashSet, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/";
    let links = get_links_from_url(url).await?;

    println!("{:#?}", links);

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
        .map(|el| {
            let link = el.attr("href").unwrap();
            normalize_link(url, link)
        })
        .flatten()
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
