use std::collections::HashSet;
use crate::network::url_utils;

/// Returns a HashSet of urls from html.
pub fn get_urls_from_html(base_url: &str, html: &str) -> HashSet<String> {
    use select::document::Document;
    use select::predicate::{Name, Or};

    Document::from(html)
        .find(Or(Name("a"), Name("link")))
        .filter_map(|el| {
            let url = match el.attr("href") {
                Some(url) => url,
                None => "",
            };
            url_utils::normalize_url(base_url, url)
        })
        .collect::<HashSet<String>>()
}
