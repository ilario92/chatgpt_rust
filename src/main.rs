use std::process::exit;
use reqwest::{Error};
use serde::{Deserialize, Serialize};
use termion::{color, style};
use chatgpt_rust::{call_api, change_context, ChatRequest, from_config_json_or_default, generate_headers, generate_usage_url, init_app, init_history, io_input, Message, OpenAIConfig, Request, store_to_history};

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
    println!("\t\tInstruction\n\
    new \t\t Create new session\n\
    context \t Set context for new session\n\
    token\t\t Tokens spent in this runtime\n\
    usage\t\t $$$ spent in this month\n\
    exit\t\t exit");
    println!("---------------------------");

    let mut context = Context {
        history: init_history(),
        token: 0,
    };

    loop {
        //println!("{:?}", context.history); //DEBUG
        print!("Io: ");
        let input = io_input().trim().replace("\n", "");

        match input.as_str() {
            "exit" => { exit(0) }
            "new" => {
                context.history = init_history();
                println!("-- New Session --");
            }
            "context" => {
                println!("-- Change Context --");
                context.history = change_context(io_input().trim().replace("\n", ""));
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
                println!("{}Assistente: {}{}{}",
                         style::Italic,
                         color::Fg(color::Green),
                         response,
                         style::Reset,
                );
                context.history = store_to_history(*context.history.clone(), "assistant", response);
            }
        }
    }
}

async fn chat_with_gpt3(config: OpenAIConfig, mut context: &mut Context) -> Result<String, Error> {
    let chat_request = ChatRequest {
        model: config.model.to_string(),
        messages: *context.history.clone(),
    };

    let response = call_api(Request {
        is_post: true,
        url: from_config_json_or_default("url_chat", "https://api.openai.com/v1/chat/completions"),
        headers: generate_headers(config),
        chat_request: Some(chat_request),
    }).await?;
    //println!("{:?}", response); //DEBUG
    if response.status() == 200
    {
        let chat_completion: ChatCompletion = response.json().await?;
        context.token = context.token + chat_completion.usage.total_tokens;
        Ok(chat_completion.choices.first().unwrap().message.content.clone())
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