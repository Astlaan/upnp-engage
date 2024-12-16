use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{task, time};

/// Struct representing a deferred task with manual start and abort control.
#[derive(Debug)]
pub struct DeferredTask {
    handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,
}

impl DeferredTask {
    /// Create a new DeferredTask with a given future.
    pub fn new<F>(future: F) -> Self
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            handle: Arc::new(Mutex::new(Some(task::spawn(future)))),
        }
    }

    /// Start the task manually by spawning it onto the tokio runtime.
    pub fn start(&self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if handle_lock.is_none() {
            let handle = task::spawn(async {
                time::sleep(Duration::from_secs(5)).await;
            });
            *handle_lock = Some(handle);
        }
    }

    /// Abort the task and wait for it to stop.
    pub fn abort_and_wait(&self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if let Some(handle) = handle_lock.take() {
            println!("Aborting the task...");

            handle.abort();

            // Check if there is a current Tokio runtime
            if let Ok(runtime_handle) = tokio::runtime::Handle::try_current() {
                // If in a runtime, use it
                runtime_handle.block_on(async {
                    let _ = handle.await; // Await termination of the task
                });
            } else {
                // Otherwise, create a temporary runtime
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let _ = handle.await; // Await termination of the task
                });
            }

            // Create runtime without checking if it exists
            // let rt = tokio::runtime::Runtime::new().unwrap();
            // rt.block_on(async {
            //     let _ = handle.await; // Await termination of the task
            // });

            // tokio::runtime::Handle::current().block_on(async {
            //     match handle.await {
            //         Ok(_) => println!("Task completed successfully."),
            //         Err(e) if e.is_cancelled() => println!("Task was cancelled."),
            //         Err(e) => println!("Task encountered an error: {:?}", e),
            //     }
            // });
        } else {
            println!("No task to abort.");
        }
    }
}
