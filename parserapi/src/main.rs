use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::error::Error;

/// Configuration
#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind_address: String,
}

/// Data model for the query parameter.
#[derive(Deserialize)]
struct TweetQuery {
    tweet_url: String,
}

/// Joins text nodes, preserving spaces between them, and removes occurrences of "https://" and "http://".
fn join_text_nodes(text_nodes: Vec<String>) -> String {
    let joined = text_nodes
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");
    joined.replace("https://", "").replace("http://", "")
}

/// Formats the text so that there is no space before colons and exactly one space after each colon.
fn fix_colon_spacing(text: &str) -> String {
    // This regex matches spaces around the colon.
    let re = Regex::new(r"\s*:\s*").unwrap();
    re.replace_all(text, ": ").to_string()
}

/// Navigates to the given tweet URL and processes tweet text and media links.
///
/// Note: We use `Box<dyn Error + Send + Sync>` as the error type.
fn process_tweet(tweet_url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Set browser options.
    let options = LaunchOptionsBuilder::default().headless(false).build()?;
    let browser = Browser::new(options)?;

    // Open a new tab.
    let tab = browser.new_tab()?;

    // Navigate to the provided tweet URL.
    tab.navigate_to(tweet_url)?;

    // Wait for the tweet text element to load.
    tab.wait_for_element(r#"div[data-testid="tweetText"]"#)?;

    // Get the HTML content of the page.
    let html_content = tab.get_content()?;

    // Parse the HTML content.
    let document = Html::parse_document(&html_content);

    // Selector for tweet text elements.
    let tweet_text_selector = Selector::parse(r#"div[data-testid="tweetText"]"#).unwrap();
    // Selector for media elements.
    let image_selector = Selector::parse("img.css-9pa8cd").unwrap();

    let mut tweet_texts = Vec::new();
    // Process tweet texts.
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

        let tweet_text = fix_colon_spacing(&raw_text);

        if !tweet_text.is_empty() {
            tweet_texts.push(tweet_text);
        }
    }

    let mut media_links = Vec::new();
    // Process media links, exclude profile image links.
    for element in document.select(&image_selector) {
        if let Some(src) = element.value().attr("src") {
            if !src.starts_with("https://pbs.twimg.com/profile_images") {
                media_links.push(src.to_string());
            }
        }
    }

    // Result: Tweet text + newline + Media links.
    let result = format!("{}\n{}", tweet_texts.join(" "), media_links.join(" "));
    Ok(result)
}

/// GET /tweet_url endpoint.
/// Calls the `process_tweet` function with the tweet_url query parameter.
async fn tweet_handler(query: web::Query<TweetQuery>) -> impl Responder {
    let tweet_url = query.tweet_url.clone();

    // Execute the process_tweet function in a blocking thread.
    let result = web::block(move || process_tweet(&tweet_url)).await;

    match result {
        Ok(Ok(res)) => HttpResponse::Ok().body(res),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(format!("Processing error: {}", e)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Server error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load server settings from the config.toml file.
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .expect("Failed to read configuration file!");
    let app_config: AppConfig = settings
        .try_deserialize()
        .expect("Failed to deserialize configuration!");
    let bind_address = app_config.server.bind_address;

    println!("Server is running on {}...", bind_address);

    HttpServer::new(|| App::new().route("/tweet_url", web::get().to(tweet_handler)))
        .bind(bind_address)?
        .run()
        .await
}
