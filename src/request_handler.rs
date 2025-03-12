use reqwest::blocking::{Client, Response};
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::configuration::Settings;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonResult {
    pub readable_product: ReadableProduct,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReadableProduct {
    pub type_name: String,
    pub page_structure: PageStructure,
    pub number: u32,
    pub next_readable_product_uri: Option<String>,
    pub has_purchased: bool,
    pub finish_reading_notification_uri: Option<String>,
    pub permalink: String,
    pub show_square_thumbnail_in_recommendation: bool,
    pub image_uris_digest: String,
    pub prev_readable_product_uri: Option<String>,
    pub title: String,
    pub is_public: bool,
    pub id: String,
    pub toc: Option<String>,
    pub series: Series,
    pub published_at: String,
    pub point_gettable_episode_when_complete_reading: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageStructure {
    pub reading_direction: String,
    pub start_position: String,
    pub cho_ju_giga: String,
    pub pages: Vec<Page>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type",)]
pub enum Page {
    #[serde(rename = "link")]
    Link {
         #[serde(rename = "linkPosition")]
         link_position: String,
    },
    #[serde(rename = "main")]
    Main {
         height: Option<u32>,
         width: Option<u32>,
         #[serde(rename = "contentStart")]
         content_start: Option<String>,
         src: Option<String>,
    },
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "backMatter")]
    BackMatter,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub title: String,
    pub thumbnail_uri: String,
    pub id: String,
}

impl JsonResult {
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

pub fn get_request_builder_for_url(url: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let client = Client::new();
    let headers = reqwest::header::HeaderMap::new();
    let response = client.get(url)
        .headers(headers)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:130.0) Gecko/20100101 Firefox/130.0")
        .send()?
        .error_for_status()?;

    Ok(response)
}

pub fn extract_json_from_website(config: &Settings) -> Result<String, Box<dyn std::error::Error>> {
    let body = get_request_builder_for_url(config.source.as_str()).unwrap().text()?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("script#episode-json").unwrap();

    let element = document.select(&selector).next().unwrap();
    let json_data = element.value().attr("data-value").unwrap();
    Ok(json_data.to_string())
}


