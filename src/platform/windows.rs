use std::process;
use winapi::shared::minwindef::{BOOL, TRUE};
use winapi::um::consoleapi::SetConsoleCtrlHandler;

// unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
//     thread::sleep(Duration::from_secs(4));
//     TRUE
// }

// pub fn register_windows_console_ctrl_handler(_exit_flag: Option<Arc<AtomicBool>>) {
//     unsafe {
//         if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) == 0 {
//             panic!("Error setting up CTRL+C handler");
//         }
//     }
// }

use once_cell::sync::Lazy;
use std::sync::Mutex;

type CtrlHandlerFn = Box<dyn Fn() + Send + 'static>;
static CTRL_HANDLER: Lazy<Mutex<Option<CtrlHandlerFn>>> = Lazy::new(|| Mutex::new(None));

unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
    if let Ok(guard) = CTRL_HANDLER.lock() {
        if let Some(callback) = guard.as_ref() {
            callback();
        }
    }
    process::exit(0);
    // TRUE
}

pub fn register_windows_console_ctrl_handler<F>(callback: F)
where
    F: Fn() + Send + 'static,
{
    println!("Registering handler!");
    if let Ok(mut handler) = CTRL_HANDLER.lock() {
        *handler = Some(Box::new(callback));

        unsafe {
            if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) == 0 {
                panic!("Error setting up CTRL+C handler");
            }
        }
    }
}
