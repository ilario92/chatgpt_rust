use std::{fs, io};
use std::io::Write;
use reqwest::{Client, Error};

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize)]
struct ChatCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Deserialize, Serialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Deserialize, Serialize)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: i32,
}

#[derive(Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Clone)]
struct ConfigOpenAi {
    model: String,
    url: String,
    key: String,
}

async fn chat_with_gpt3(config: ConfigOpenAi, message: String) -> Result<String, Error> {
    let chat_request = ChatRequest {
        model: config.model,
        messages: vec![Message {
            role: "user".to_owned(),
            content: message.to_owned(),
        }],
    };

    let client = Client::new();
    let response = client
        .post(config.url)
        .header(AUTHORIZATION, format!("Bearer {}", config.key))
        .header(CONTENT_TYPE, "application/json")
        .json(&chat_request)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;

    Ok(chat_response.choices.first().unwrap().message.content.clone())
}

fn get_api_key() -> String {
    let file = fs::File::open("res/secret.json")
        .expect("file should open read only");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("file should be proper JSON");
    let key = json.get("API_KEY").expect("API_KEK not found!").to_string();

    key.replace("\"", "")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Chat with ChatGPT");

    let contex = init();

    let mut input = String::new();
    loop {
        print!("Io: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("Errore nella lettura dell'input");
        let response = chat_with_gpt3(contex.clone(), input.clone()).await?;
        println!("Assistente: {}", response);
    }
}

fn init() -> ConfigOpenAi {
    let open_ia = get_api_key();

    let config = ConfigOpenAi {
        model: "gpt-3.5-turbo".to_string(),
        url: "https://api.openai.com/v1/chat/completions".to_string(),
        key: open_ia,
    };
    config
}

