pub mod url_utils {
    /// Returns an absolute url.
    pub fn normalize_url(base_url: &str, url: &str) -> Option<String> {
        use std::str::FromStr;
        use url::{ParseError, Url};

        match Url::parse(url) {
            Ok(new_url) => {
                if new_url.has_host() {
                    Some(new_url.to_string())
                } else {
                    None
                }
            }
            Err(parse_err) => match parse_err {
                ParseError::RelativeUrlWithoutBase => Some({
                    Url::from_str(base_url)
                        .unwrap()
                        .join(url)
                        .unwrap()
                        .to_string()
                }),
                _ => None,
            },
        }
    }
}

/// Returns html from url.
pub async fn get_html_from_url(url: &str) -> Result<String, reqwest::Error> {
    let resp = reqwest::get(url).await?;
    let content = resp.text().await?;

    Ok(content)
}