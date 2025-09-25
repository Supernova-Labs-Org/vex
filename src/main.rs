
use std::{
    time::Instant,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering}
    }
};

use futures::future::join_all;
use reqwest::Client;


// need to replace reqwest client to the h3 client
pub mod client;



#[tokio::main]
async fn main(){

    let server_url = "http://127.0.0.1:7777";
    let total_request_send = 5;
    let concurrency = 2;

    let request_client = Client::builder()
        .http3_prior_knowledge()
        .danger_accept_invalid_certs(true) // for self-signed certificates
        .build()
        .unwrap();

    // Shared atomic counters
    let success = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut task_list = Vec::new();

    for _ in 0..concurrency {
        let client = request_client.clone();
        let url = server_url;

        let success = Arc::new(AtomicUsize::new(0));
        let errors = Arc::new(AtomicUsize::new(0));

        task_list.push(tokio::spawn(async move{
            let request_per_thread = total_request_send / concurrency;
            for _ in 0..request_per_thread {
                let t0 = Instant::now();

                match client.get(&*url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            success.fetch_add(1, Ordering::Relaxed);
                        }
                        else{
                            eprintln!("Non-200 status: {}", response.status());
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                        // if want to record latency 
                        let _latency = t0.elapsed();
                    },
                    Err(e) => {
                        eprintln!("Request failed: {:?}", e);
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    join_all(task_list).await;
    let time_elaspsed = start_time.elapsed();


    println!("Total request send: {}", total_request_send);
    println!("Total success request: {}", success.load(Ordering::Relaxed));
    println!("Total failed request: {}", errors.load(Ordering::Relaxed));
    println!("Time elapsed: {:?}", time_elaspsed);
    println!("Req/sec: {:.2}", total_request_send as f64 / time_elaspsed.as_secs_f64());
}