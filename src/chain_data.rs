use reqwest::Error as ReqwestError;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use tokio::task;
use std::env;
use dotenv::dotenv;

#[derive(Debug, Deserialize)]
struct Response {
    jsonrpc: String,
    id: u32,
    result: Vec<Log>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Log {
    address: String,
    data: String,
    topics: Vec<String>,
}

async fn fetch_logs_from_blocks(start_block: String, end_block: String) -> Result<Vec<Log>, ReqwestError> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getLogs",
        "params": [{
            "fromBlock": start_block,
            "toBlock": end_block
        }]
    });


    let client = reqwest::Client::new();
    let infura_url = env::var("INFURA_URL").expect("INFURA_URL must be set");
    let res: Response = client.post(&infura_url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.result)
}

fn block_to_hex(block: u64) -> String {
    format!("0x{:x}", block) 
  }

  
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let target_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string().to_lowercase();
    let start_block = 18277200;
    let end_block   = 18277210;    

    let logs = fetch_chain_data(start_block, end_block, target_address).await?;

    // println!("Logs: {:?}", logs);

    Ok(())

}

pub async fn fetch_chain_data(
    start_block: u64, 
    end_block: u64,
    target_address: String,
) -> Result<Vec<Log>, Box<dyn Error>> {

    

    let target_address = target_address.to_lowercase();


    let mut handles = Vec::new();
    let mut current_start = start_block;

    while current_start <= end_block {
        // Define the range for this batch 
        let batchsize = 3;
        let current_end = std::cmp::min(current_start + (batchsize - 1), end_block);

        println!("spawning task for blocks {} to {}", current_start, current_end);

        let handle = task::spawn(fetch_logs_from_blocks(
            block_to_hex(current_start).to_string(),
            block_to_hex(current_end).to_string(),
        ));
        handles.push(handle);

        current_start += batchsize;
    }

    let mut matching_logs = Vec::new();

    // Await on the handles to get the results
    for handle in handles {
        match handle.await {
            Ok(Ok(logs)) => {
                // Filter logs based on the specified address and collect them
                let filtered_logs: Vec<_> = logs
                .into_iter()
                .filter(|log| log.address == target_address)
                .collect();

                matching_logs.extend(filtered_logs.clone());
                println!("# matching entries: {}", filtered_logs.len());
            },
            Ok(Err(e)) => {
                println!("Error fetching logs: {}", e);
                return Err(Box::new(e));
            },
            Err(e) => {
                println!("Task error: {}", e);
                return Err(Box::new(e));
            },
        }
    }

    Ok(matching_logs)
}
