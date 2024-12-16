use std::{
    sync::{Arc, Mutex},
    time::Duration,
    thread,
};
use tokio::{task, time};
use winapi::um::consoleapi::SetConsoleCtrlHandler;
use winapi::um::wincon::{CTRL_C_EVENT, PHANDLER_ROUTINE};

/// Struct representing a deferred task with manual start and abort control.
struct DeferredTask {
    handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,
}

impl DeferredTask {
    /// Create a new DeferredTask with a given future.
    fn new<F>(future: F) -> Self
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            handle: Arc::new(Mutex::new(Some(task::spawn(future)))),
        }
    }

    /// Start the task manually by spawning it onto the tokio runtime.
    fn start(&self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if handle_lock.is_none() {
            println!("Starting the task...");
            let handle = task::spawn(async {
                println!("Task is running...");
                time::sleep(Duration::from_secs(5)).await;
                println!("Task completed!");
            });
            *handle_lock = Some(handle);
        }
    }

    /// Abort the task and wait for it to stop.
    fn abort_and_wait(&self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if let Some(handle) = handle_lock.take() {
            println!("Aborting the task...");
            handle.abort();
            // Wait for the task to confirm it has been aborted
            let result = tokio::runtime::Handle::current().block_on(async {
                match handle.await {
                    Ok(_) => println!("Task completed successfully."),
                    Err(e) if e.is_cancelled() => println!("Task was cancelled."),
                    Err(e) => println!("Task encountered an error: {:?}", e),
                }
            });
        } else {
            println!("No task to abort.");
        }
    }
}

/// Global static mutable reference for the console handler to access the DeferredTask.
static mut TASK: Option<Arc<DeferredTask>> = None;

/// Console handler routine for handling CTRL+C.
extern "system" fn console_handler(ctrl_type: u32) -> i32 {
    unsafe {
        if ctrl_type == CTRL_C_EVENT {
            if let Some(ref task) = TASK {
                task.abort_and_wait();
            }
            return 1; // Signal handled
        }
    }
    0 // Signal not handled
}

#[tokio::main]
async fn main() {
    // Create a deferred task
    let deferred_task = Arc::new(DeferredTask::new(async {
        println!("Task execution started!");
        // Simulate work that takes a while
        for i in 0..10 {
            println!("Working... {}", i);
            time::sleep(Duration::from_secs(1)).await;
        }
        println!("Task execution finished!");
    }));

    // Set the global task reference for the console handler
    unsafe {
        TASK = Some(deferred_task.clone());
    }

    // Set the console control handler to catch CTRL+C
    unsafe {
        if SetConsoleCtrlHandler(Some(console_handler), 1) == 0 {
            eprintln!("Failed to set console control handler");
            return;
        }
    }

    println!("Press CTRL+C to abort the task.");

    // Start the task
    deferred_task.start();

    // Keep the main thread alive until the task completes or is aborted
    loop {
        time::sleep(Duration::from_secs(1)).await;
    }
}
