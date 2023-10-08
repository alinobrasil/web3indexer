#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rocket_contrib;
mod chain_data;
use chain_data::fetch_chain_data;
use chain_data::{Log};

use rocket_contrib::json::Json;
use rocket::response::Responder;
use rocket::{response, Response};
use rocket::http::Status;
use rocket::Request;
use std::io::Cursor;
use std::fmt;
use serde::Serialize;

use dotenv::dotenv;

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "API Error: {}", self.message)
    }
}


enum Either<A, B> {
    A(A),
    B(B),
}

impl<'r, A: Responder<'r>, B: Responder<'r>> Responder<'r> for Either<A, B> {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'r> {
        match self {
            Either::A(a) => a.respond_to(req),
            Either::B(b) => b.respond_to(req),
        }
    }
}


// Define the data structures for our endpoints
#[derive(Serialize, Deserialize)]
struct ResultNumber {
    result: i32,
}

#[derive(Serialize, Deserialize)]
struct ResultString {
    result: String,
}


#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
pub struct ApiError {
    message: String,
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, _: &Request) -> rocket::response::Result<'r> {
        // For simplicity, we'll convert the error to a string and return it as a plain text response with a 400 status code
        // Customize as needed!
        response::Response::build()
            .status(rocket::http::Status::BadRequest)
            .header(rocket::http::ContentType::Plain)
            .sized_body(std::io::Cursor::new(self.to_string()))
            .ok()
    }
}






#[get("/fetchdata")]
fn fetchdata() -> Either<Json<Vec<Log>>, ApiError> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let target_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string().to_lowercase();
    let start_block = 18277200;
    let end_block = 18277210;

    let result = runtime.block_on(fetch_chain_data(start_block, end_block, target_address));

    match result {
        Ok(logs) => Either::A(Json(logs)),
        Err(e) => Either::B(ApiError { message: e.to_string() }),
    }
}




// Endpoint to add two numbers (sample, to see that API is functional)
#[get("/add?<a>&<b>")]
fn add(a: i32, b: i32) -> Json<ResultNumber> {
    Json(ResultNumber {
        result: a + b,
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    rocket::ignite()
        .mount("/", routes![add, fetchdata])
        .launch();
}
