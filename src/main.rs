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
struct OpenAIConfig {
    model: Box<String>,
    url: Box<String>,
    key: Box<String>,
}

#[derive(Clone)]
struct Context {
    history: Box<Vec<Message>>,
    token: i32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = init_openai();

    println!("Chat with ChatGPT");
    println!("Instruction\nnew \t create new session\nexit\t exit\ntoken\t token used");
    println!("---------------------------");

    let mut context = Context {
        history: init_history(),
        token: 0,
    };

    loop {
        //println!("{:?}", context.history); for debug
        print!("Io: ");
        let input = io_input().trim().replace("\n", "");

        match input.as_str() {
            "exit" => { exit(0) }
            "new" => {
                context.history = init_history();
            }
            "token" => {
                println!("in this session you have used: {} token", context.token)
            }
            _ => {
                context.history = store_to_history(*context.history.clone(), "user", input);
                let response = chat_with_gpt3(config.clone(), &mut context).await?;
                println!("Assistente: {}", response);
                context.history = store_to_history(*context.history.clone(), "assistant", response);
            }
        }
    }
}

fn init_openai() -> OpenAIConfig {
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

async fn chat_with_gpt3(config: OpenAIConfig, mut context: &mut Context) -> Result<String, Error> {
    let chat_request = ChatRequest {
        model: *config.model,
        messages: *context.history.clone(),
    };

    let client = Client::new();
    let response = client
        .post(*config.url)
        .header(AUTHORIZATION, format!("Bearer {}", *config.key))
        .header(CONTENT_TYPE, "application/json")
        .json(&chat_request)
        .send()
        .await?;
    //println!("{:?}", response); //DEBUG
    if response.status() == 200
    {
        let response: ChatCompletion = response.json().await?;
        context.token = context.token + response.usage.total_tokens;
        Ok(response.choices.first().unwrap().message.content.clone())
    } else {
        Err(response.status()).expect("Status code")
    }
}

fn store_to_history(mut history: Vec<Message>, role: &str, message: String) -> Box<Vec<Message>> {
    history.push(Message {
        role: role.to_owned(),
        content: message.to_owned(),
    });
    Box::new(history)
}

fn init_history() -> Box<Vec<Message>> {
    store_to_history(vec![],
                     "system",
                     "Sei un assistente cordiale che rispondi a tutte le domande che ti vengono fatte".to_string())
}

fn init(apikey_console: Option<String>) -> OpenAIConfig {
    let apikey: String;
    if apikey_console.is_some()
    {
        apikey = apikey_console.unwrap();
    } else {
        apikey = get_api_key();
    }

    let config = OpenAIConfig {
        model: Box::new("gpt-3.5-turbo".to_string()),
        url: Box::new("https://api.openai.com/v1/chat/completions".to_string()),
        key: Box::from(apikey),
    };
    config
}