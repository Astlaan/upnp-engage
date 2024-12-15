use std::sync::{atomic::AtomicBool, Arc};
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{BOOL, TRUE};
use winapi::um::consoleapi::SetConsoleCtrlHandler;

unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
    thread::sleep(Duration::from_secs(4));
    TRUE
}

pub fn register_ctrl_handler(_exit_flag: Arc<AtomicBool>) {
    unsafe {
        if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) == 0 {
            panic!("Error setting up CTRL+C handler");
        }
    }
}
