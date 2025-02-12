use headless_chrome::{Browser, LaunchOptionsBuilder};
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;
use std::io::{self, Write};

/// Joins text nodes with a space, then removes any occurrences of "https://" and "http://".
fn join_text_nodes(text_nodes: Vec<String>) -> String {
    let joined = text_nodes
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");
    joined.replace("https://", "").replace("http://", "")
}

/// Post-processes the given text so that every colon (":") is attached to the previous word
/// (i.e. no space before the colon and is followed by a single space, for example: "a: b").
fn fix_colon_spacing(text: &str) -> String {
    // This regex matches any amount of whitespace before and after a colon.
    let re = Regex::new(r"\s*:\s*").unwrap();
    re.replace_all(text, ": ").to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    // Prompt the user for the tweet URL.
    print!("Enter the tweet URL: ");
    io::stdout().flush()?;
    let mut tweet_url = String::new();
    io::stdin().read_line(&mut tweet_url)?;
    let tweet_url = tweet_url.trim(); // Remove extra whitespace

    // Launch the browser in visible mode.
    let options = LaunchOptionsBuilder::default().headless(false).build()?;
    let browser = Browser::new(options)?;

    // Open a new tab.
    let tab = browser.new_tab()?;

    // Navigate to the provided tweet URL.
    tab.navigate_to(tweet_url)?;

    // Wait for the tweet text element to load.
    // On Twitter/X, tweet text is usually contained in a div with data-testid="tweetText".
    tab.wait_for_element(r#"div[data-testid="tweetText"]"#)?;

    // Retrieve the page's HTML content.
    let html_content = tab.get_content()?;

    // Parse the HTML content.
    let document = Html::parse_document(&html_content);

    // Selector for the tweet text element.
    let tweet_text_selector = Selector::parse(r#"div[data-testid="tweetText"]"#).unwrap();
    // Selector for media elements (the class name may change over time).
    let image_selector = Selector::parse("img.css-9pa8cd").unwrap();

    let mut tweet_texts = Vec::new();
    println!("\nTweet Text:");
    // Iterate over tweet text elements and join their text nodes.
    for tweet in document.select(&tweet_text_selector) {
        let text_nodes: Vec<String> = tweet
            .children()
            .filter_map(|child| child.value().as_text().map(|t| t.to_string()))
            .collect();

        let raw_text = if !text_nodes.is_empty() {
            join_text_nodes(text_nodes)
        } else {
            tweet
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
                .replace("https://", "")
                .replace("http://", "")
        };

        // Colon (:) should be attached to the preceding text and followed by a space.
        let tweet_text = fix_colon_spacing(&raw_text);

        if !tweet_text.is_empty() {
            println!("{}", tweet_text);
            tweet_texts.push(tweet_text);
        }
    }

    let mut media_links = Vec::new();
    println!("\nMedia Links:");
    // Print media links only if they don't start with
    // "https://pbs.twimg.com/profile_images".
    for element in document.select(&image_selector) {
        if let Some(src) = element.value().attr("src") {
            if !src.starts_with("https://pbs.twimg.com/profile_images") {
                println!("{}", src);
                media_links.push(src.to_string());
            }
        }
    }

    // Total result = Tweet Text  + (new line) Media Links
    let result = format!("{}\n{}", tweet_texts.join(" "), media_links.join(" "));
    println!("\nResult:\n{}", result);

    Ok(())
}
