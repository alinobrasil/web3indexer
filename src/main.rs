#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use std::collections::HashMap;
use std::env;
use tokio::sync::Mutex;
use uuid::Uuid;
mod chain_data;
use chain_data::{fetch_chain_data, Log};

use dotenv::dotenv;
use serde::Serialize;

type TaskId = Uuid;
type TaskStatus = Result<Vec<Log>, String>;

struct Config {
    infura_url: String,
    target_address: String,
}

struct AppState {
    client: reqwest::Client,
    config: Config,
    tasks: Mutex<HashMap<TaskId, TaskStatus>>,
}

#[derive(Serialize, Deserialize)]
struct ResultNumber {
    result: i32,
}

#[derive(Serialize, Debug)]
pub struct ApiError {
    message: String,
}

#[get("/fetchdata?<start_block>&<end_block>")]
async fn fetchdata(
    state: &State<AppState>,
    start_block: u64,
    end_block: u64,
) -> Result<Json<Vec<Log>>, (Status, String)> {
    let client = &state.client;
    let config = &state.config;

    let target_address: String = config.target_address.clone();

    match fetch_chain_data(start_block, end_block, target_address, client).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => Err((Status::BadRequest, e.to_string())),
    }
}

#[get("/checkdata?<task_id>")]
async fn checkdata(task_id: Uuid, state: &State<AppState>) -> String {
    let task_map = state.tasks.lock().await;
    match task_map.get(&task_id) {
        Some(Ok(logs)) => format!("Status: Complete\nData: {:?}", logs),
        Some(Err(e)) => format!("Status: Error\nError: {}", e),
        None => "Status: Running".to_string(),
    }
}

#[get("/add?<a>&<b>")]
fn add(a: i32, b: i32) -> Json<ResultNumber> {
    Json(ResultNumber { result: a + b })
}

fn main() {
    dotenv().ok();

    let client = reqwest::Client::new();

    let tasks = Mutex::new(HashMap::new());

    let config = Config {
        infura_url: env::var("INFURA_URL")
            .expect("INFURA_URL must be set")
            .to_string(),
        target_address: env::var("TARGET_ADDRESS")
            .expect("TARGET_ADDRESS must be set")
            .to_string()
            .to_lowercase(),
    };

    let app_state = AppState {
        client,
        config,
        tasks,
    };

    rocket::build()
        .manage(app_state)
        .mount("/", routes![add, fetchdata, checkdata])
        .launch();
}
