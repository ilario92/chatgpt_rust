use std::{fs, io};
use std::io::Write;

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

