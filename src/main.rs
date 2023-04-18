use std::process::exit;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use chatgpt_rust::{call_api, ChatRequest, generate_headers, generate_usage_url, init_app, io_input, Message, OpenAIConfig, Request};

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


#[derive(Clone)]
struct Context {
    history: Box<Vec<Message>>,
    token: i32,
}


#[derive(Deserialize)]
struct CostsData {
    total_usage: f32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = init_app();

    println!("Chat with ChatGPT");
    println!("Instruction\n\
    new \t create new session\n\
    token\t Tokens spent in this runtime\n\
    usage\t $$$ spent in this month\n\
    exit\t exit");
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
            "usage" => {
                usage_amount_gpt3(config.clone()).await?;
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

async fn usage_amount_gpt3(config: OpenAIConfig) -> Result<String, Error> {
    let response = call_api(Request {
        is_post: false,
        url: generate_usage_url(),
        headers: generate_headers(config),
        chat_request: None,
    }).await?;
    //println!("{:?}", response.text().await); //DEBUG
    if response.status() == 200
    {
        let cost_data: CostsData = response.json().await?;
        let res = format!("$ {:.3}", (cost_data.total_usage / 100.0));
        println!("{}", res);
        Ok(res)
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