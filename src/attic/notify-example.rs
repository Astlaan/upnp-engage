use std::sync::Arc;
use tokio::sync::{Notify, oneshot};
use tokio::task;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a shared notify object for cancellation
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    // Define the future (task) ahead of time
    let future_task = async move {
        loop {
            tokio::select! {
                _ = notify_clone.notified() => {
                    println!("Task received shutdown signal. Cleaning up...");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    println!("Task is running...");
                }
            }
        }
    };

    // Register the shutdown hook before spawning the task
    let hook_notify = notify.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, sending shutdown signal...");
        hook_notify.notify_one();
    }).expect("Error setting Ctrl+C handler");

    // Now, spawn the task after hooks are registered
    let handle = task::spawn(future_task);

    // Wait for the task to finish
    handle.await.expect("Task failed");
}
