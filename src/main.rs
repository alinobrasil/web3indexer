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
use tokio::sync::Mutex;
use uuid::Uuid;

mod chain_data;
use chain_data::fetch_chain_data;
use chain_data::Log;

use dotenv::dotenv;
use serde::Serialize;

type TaskId = Uuid;
type TaskStatus = Result<Vec<Log>, String>;

#[derive(Serialize, Deserialize)]
struct ResultNumber {
    result: i32,
}

#[derive(Serialize, Debug)]
pub struct ApiError {
    message: String,
}

#[derive(Clone)]
struct AppState {
    tasks: Mutex<HashMap<TaskId, TaskStatus>>,
}

#[get("/fetchdata?<target_address>&<start_block>&<end_block>")]
async fn fetchdata(
    target_address: String,
    start_block: u64,
    end_block: u64,
) -> Result<Json<Vec<Log>>, (Status, String)> {
    match fetch_chain_data(start_block, end_block, target_address).await {
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

    let app_state = AppState {
        tasks: Mutex::new(HashMap::new()),
    };

    rocket::build()
        .manage(app_state)
        .mount("/", routes![add, fetchdata, checkdata])
        .launch();
}
