use std::{fs, io};
use std::io::Write;
use chrono::{DateTime, Duration, Local};
use reqwest::{Client, Error, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::Serialize;
use serde::Deserialize;

pub struct Request {
    pub is_post: bool,
    pub url: String,
    pub headers: HeaderMap,
    pub chat_request: Option<ChatRequest>,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub struct OpenAIConfig {
    pub model: Box<String>,
    pub url: Box<String>,
    pub key: Box<String>,
}

pub fn get_api_key() -> String {
    let file = fs::File::open("res/secret.json")
        .expect("file should open read only");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("file should be proper JSON");
    let key = json.get("API_KEY").expect("API_KEK not found!").to_string();

    key.replace("\"", "")
}

pub fn io_input() -> String {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Error on read line");
    input
}

pub fn format_date(date: DateTime<Local>) -> String
{
    format!("{}", date.format("%Y-%m-%d"))
}

pub async fn call_api(request: Request) -> Result<Response, Error>
{
    let client = Client::new();
    let res;
    if request.is_post && request.chat_request.is_some()
    {
        res = client.post(request.url).headers(request.headers).json(&request.chat_request.unwrap()).send().await;
    } else if request.is_post && request.chat_request.is_none() {
        res = client.post(request.url).headers(request.headers).send().await;
    } else {
        res = client.get(request.url).headers(request.headers).send().await;
    }
    res
}

pub fn generate_headers(config: OpenAIConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", *config.key).parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers
}

pub fn generate_usage_url() -> String {
    let today_date = Local::now();
    let one_month_ago_date = today_date - Duration::days(30);

    format!("https://api.openai.com/dashboard/billing/usage?start_date={}&end_date={}",
            format_date(one_month_ago_date),
            format_date(today_date))
}
pub fn init(apikey_console: Option<String>) -> OpenAIConfig {
    let apikey: String = if apikey_console.is_some()
    {
        apikey_console.unwrap()
    } else {
        get_api_key()
    };

    let config = OpenAIConfig {
        model: Box::new("gpt-3.5-turbo".to_string()),
        url: Box::new("https://api.openai.com/v1/chat/completions".to_string()),
        key: Box::from(apikey),
    };
    config
}

pub fn init_app() -> OpenAIConfig {
    let config: OpenAIConfig;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        config = init(Some(args[1].to_string()));
    } else {
        println!("Non hai passato alcun parametro. Cerco in res/secret.json");
        config = init(None);
    }
    config
}