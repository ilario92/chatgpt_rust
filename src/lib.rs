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
    pub model: String,
    pub url: String,
    pub key: String,
}

pub fn from_config_json(string: &str) -> String {
    let file = fs::File::open("res/config.json")
        .expect("file exception");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("file should be proper JSON");
    json.get(string)
        .expect(format!("{} not found!", string).as_str()).to_string().replace("\"", "")
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
    headers.insert(AUTHORIZATION, format!("Bearer {}", config.key).parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers
}

pub fn generate_usage_url() -> String {
    let today_date = Local::now();
    let one_month_ago_date = today_date - Duration::days(30);

    format!("{}start_date={}&end_date={}",
            from_config_json("url_usage"),
            format_date(one_month_ago_date),
            format_date(today_date))
}

pub fn init(apikey_console: Option<String>) -> OpenAIConfig {
    let apikey: String = if apikey_console.is_some()
    {
        apikey_console.unwrap()
    } else {
        from_config_json("api_key")
    };

    let config = OpenAIConfig {
        model: from_config_json("model"),
        url: from_config_json("url_chat"),
        key: apikey,
    };
    config
}

pub fn store_to_history(mut history: Vec<Message>, role: &str, message: String) -> Box<Vec<Message>> {
    history.push(Message {
        role: role.to_owned(),
        content: message.to_owned(),
    });
    Box::new(history)
}

pub fn init_history() -> Box<Vec<Message>> {
    store_to_history(vec![],
                     "system",
                     "Sei un assistente cordiale che rispondi a tutte le domande che ti vengono fatte".to_string())
}

pub fn init_app() -> OpenAIConfig {
    let config: OpenAIConfig;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        config = init(Some(args[1].to_string()));
    } else {
        println!("API KEY not found in args. Search res/config.json");
        config = init(None);
    }
    config
}