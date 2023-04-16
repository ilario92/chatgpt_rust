use serde::Deserialize;
use serde::Serialize;
use reqwest::{Client, Response, Error};

const API_URL: &str = "https://www.ilario.me/index.json";

#[derive(Serialize)]
struct Post {
    title: String,
    body: String,
    userId: i32,
}

async fn send_post_to_api(post: Post) -> Result<Response, Error> {
    let client = Client::new();
    let response = client.get(API_URL)
        //.json(&post)
        .send()
        .await?;
    Ok(response)
}

async fn receive_api_response(response: Response) -> Result<(), Error> {
    let api_response = response.text().await?;
    println!("{:#?}", api_response);
    Ok(())
}

#[tokio::main]
async fn main() {
    let post = Post {
        title: String::from("Il mio nuovo post"),
        body: String::from("Questo è il corpo del mio nuovo post"),
        userId: 1,
    };


    let response = send_post_to_api(post).await.unwrap();
    println!("res: {:?}", response);
    receive_api_response(response).await.unwrap();
}