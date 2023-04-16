use std::process::exit;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use chatgpt_rust::{get_api_key, io_input};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Clone)]
struct Context {
    history: Vec<Message>,
    token: i32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Chat with ChatGPT");
    println!("Instruction\nnew \t create new session\nexit\t exit\ntoken\t token used");
    println!("---------------------------");
    let config = init();
    let mut contex = Context {
        history: init_history(),
        token: 0,
    };

    loop {
        print!("Io: ");
        let input = io_input().trim().replace("\n", "");

        match input.as_str() {
            "exit" => { exit(0) }
            "new" => {
                contex.history = init_history();
            }
            "token" => {
                println!("in this session you have used: {} token", contex.token)
            }
            _ => {
                let resp = chat_with_gpt3(config.clone(), input, &mut contex).await?;
                println!("Assistente: {}", resp);
                print_store_response(&mut contex, resp);
            }
        }
    }
}

async fn chat_with_gpt3(config: ConfigOpenAi, message: String, context: &mut Context) -> Result<String, Error> {
    let new_mes = Message {
        role: "user".to_owned(),
        content: message.to_owned(),
    };
    context.history.push(new_mes);

    let chat_request = ChatRequest {
        model: config.model,
        messages: context.history.clone(),
    };

    let client = Client::new();
    let response = client
        .post(config.url.clone())
        .header(AUTHORIZATION, format!("Bearer {}", config.key))
        .header(CONTENT_TYPE, "application/json")
        .json(&chat_request)
        .send()
        .await?;

    let response: ChatCompletion = response.json().await?;
    //println!("{:?}", response.clone()); //DEBUG
    context.token = context.token + response.usage.total_tokens;
    Ok(response.choices.first().unwrap().message.content.clone())
}

fn print_store_response(context: &mut Context, response: String) {
    let new_mg = Message {
        role: "assistant".to_owned(),
        content: response.to_owned(),
    };
    context.history.push(new_mg);
}

fn init_history() -> Vec<Message> {
    let system = Message {
        role: "system".to_owned(),
        content: "Sei un assistente cordiale che rispondi a tutte le domande che ti vengono fatte"
            .to_owned(),
    };
    vec![system]
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