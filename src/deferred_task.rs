use core::fmt;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::runtime::Builder;
use tokio::task;

/// Struct representing a deferred task with manual start and abort control.
pub struct DeferredTask {
    handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,
    future: Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}

impl DeferredTask {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
            future: Some(Box::pin(future)),
        }
    }

    /// Start the task manually by spawning it onto the tokio runtime.
    pub fn start(&mut self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if handle_lock.is_none() {
            println!("Starting the task...");
            if let Some(future) = self.future.take() {
                let handle = task::spawn(future);
                *handle_lock = Some(handle);
            }
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
                // let rt = tokio::runtime::Runtime::new().unwrap();
                let rt = Builder::new_current_thread().build().unwrap();
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

impl fmt::Debug for DeferredTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeferredTask")
            .field("handle", &self.handle)
            .field("future", &"dyn Future<Output = ()>")
            .finish()
    }
}
