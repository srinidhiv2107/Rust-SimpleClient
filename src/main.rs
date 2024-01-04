use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::blocking::Client;
use tokio::time::Duration;

#[derive(Debug, Deserialize)]
struct CoinbaseResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    amount: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "--mode=cache" => {
            println!("Selected mode: Cache");
            cache_mode(&args).await?;
        }
        "--mode=read" => {
            println!("Selected mode: Read");
            read_mode().await?;
        }
        _ => {
            println!("Invalid mode. Use cache or read.");
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  ./simple --mode=<cache|read> [--times=<seconds>]");
}

async fn cache_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let times: u64 = args
        .iter()
        .find(|s| s.starts_with("--times="))
        .and_then(|s| s.split('=').nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";

    let client = Client::new();
    let file = Arc::new(Mutex::new(OpenOptions::new().write(true).create(true).open("result.txt")?));

    let mut sum = 0.0;
    let mut count = 0;

    let delay = Duration::from_secs(1); // Adjust the delay duration as needed

    while count < times {
        if let Ok(response) = client.get(url).send() {
            if let Ok(message) = response.json::<CoinbaseResponse>() {
                let amount = message.data.amount.parse::<f64>().unwrap_or(0.0);
                sum += amount;

                // Print each received amount
                println!("Received amount: {}", amount);

                count += 1;
            }
        }

        // Introduce a delay between requests
        tokio::time::sleep(delay).await;
    }

    let average = sum / times as f64;

    let mut file = file.lock().await;
    writeln!(&mut file, "Cache complete. The average USD price of BTC on Coinbase is: {}", average)?;

    println!("Cache complete. The average USD price of BTC on Coinbase is: {}", average);

    Ok(())
}

async fn read_mode() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "result.txt";

    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                println!("The result.txt file is empty. Run in cache mode first.");
            } else {
                let file = File::open(file_path)?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    println!("{}", line?);
                }
            }

            Ok(())
        }
        Err(_) => {
            println!("The result.txt file does not exist. Run in cache mode first.");
            Ok(())
        }
    }
}
