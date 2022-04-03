use reqwest;

const API_KEY: &str = "lip_UQmtRzDmwvdDLkGYZZhT";
const BASE_URL: &str = "https://lichess.org/api";

fn test_api() {
    let body = reqwest::get("https://icanhasip.com/")?
        .text()?;

    println!("body = {:?}", body);
}



