mod config;
mod deferred_task;
mod platform;

use config::Config;
use deferred_task::DeferredTask;
use igd::search_gateway;
use igd::PortMappingProtocol;
use local_ip_address;
use platform::windows::register_windows_console_ctrl_handler;
use std::env;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use tokio::time::{self, Duration};

const LEASE_TIME: u32 = 3600;
const LEASE_RENEWAL_INTERVAL: u32 = 3000;
const CONNECTION_NAME: &str = "Rust UPnP Port Forwarder";

static TASK_OPEN_AND_MAINTAIN_CONNECTION: OnceLock<Arc<Mutex<DeferredTask>>> = OnceLock::new();

fn get_config_path() -> io::Result<std::path::PathBuf> {
    let current_dir = env::current_dir()?;
    Ok(current_dir.join("config.toml"))
}

// async fn cleanup_ports(gateway: &igd::aio::Gateway, router_port: u16) {
//     // Remove TCP port mapping
//     let _ = File::create("empty_clean.txt");
//     match gateway
//         .remove_port(PortMappingProtocol::TCP, router_port)
//         .await
//     {
//         Ok(_) => println!("TCP port mapping removed successfully."),
//         Err(e) => eprintln!("Failed to remove TCP port mapping: {}", e),
//     }

//     // Remove UDP port mapping
//     match gateway
//         .remove_port(PortMappingProtocol::UDP, router_port)
//         .await
//     {
//         Ok(_) => println!("UDP port mapping removed successfully."),
//         Err(e) => eprintln!("Failed to remove UDP port mapping: {}", e),
//     }
// }

async fn open_and_keep_active(gateway: igd::Gateway, device_port: u16, external_port: u16) {
    let local_ip = local_ip_address::local_ip()
        .unwrap_or_else(|e| {
            eprintln!("Failed to get local IP: {}", e);
            process::exit(1);
        })
        .to_string();
    let local_ip = Ipv4Addr::from_str(&local_ip).unwrap();
    let external_ip = gateway.get_external_ip().unwrap_or_else(|e| {
        eprintln!("Failed to get external IP: {}", e);
        process::exit(1);
    });
    let renewal_interval = Duration::from_secs(LEASE_RENEWAL_INTERVAL.into());
    let mut first_run = true;

    loop {
        // Add/Renew TCP Port Mapping
        match gateway.add_port(
            igd::PortMappingProtocol::TCP,
            external_port,
            SocketAddrV4::new(local_ip, device_port),
            // external IP works, router recognizes itself
            LEASE_TIME,
            &format!("{} - TCP", CONNECTION_NAME),
        ) {
            Ok(_) => {
                if first_run {
                    println!("✓ TCP port active.");
                } else {
                    println!("✓ TCP port renewed.");
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to {} TCP port mapping: {}",
                    if first_run { "add" } else { "renew" },
                    e
                );
                process::exit(1);
            }
        }

        // Add/Renew UDP Port Mapping
        match gateway.add_port(
            igd::PortMappingProtocol::UDP,
            external_port,
            SocketAddrV4::new(local_ip, device_port),
            LEASE_TIME,
            &format!("{} - UDP", CONNECTION_NAME),
        ) {
            Ok(_) => {
                if first_run {
                    println!("✓ UDP port active.");
                } else {
                    println!("✓ UDP port renewed.");
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to {} UDP port mapping: {}",
                    if first_run { "add" } else { "renew" },
                    e
                );
                process::exit(1);
            }
        }

        if first_run {
            println!("");
            println!("Port forwarding is active.");
            println!("\nLocal IP:");
            println!("{}:{}", local_ip, device_port);
            println!("\nExternal IP:");
            println!("{}:{}", external_ip, external_port);
            println!("");
            println!("Press Ctrl+C to terminate.");
        }

        first_run = false;
        time::sleep(renewal_interval).await;
    }
}

fn cleanup_ports(gateway: igd::Gateway, router_port: u16) {
    // Create or truncate the "empty_clean.txt" file

    // Remove TCP port mapping
    match gateway.remove_port(PortMappingProtocol::TCP, router_port) {
        Ok(_) => println!("TCP port mapping removed successfully."),
        Err(e) => eprintln!("Failed to remove TCP port mapping: {}", e),
    }

    // Remove UDP port mapping
    match gateway.remove_port(PortMappingProtocol::UDP, router_port) {
        Ok(_) => println!("UDP port mapping removed successfully."),
        Err(e) => eprintln!("Failed to remove UDP port mapping: {}", e),
    }
}

fn shutdown_program(gateway: igd::Gateway, external_port: u16) {
    if TASK_OPEN_AND_MAINTAIN_CONNECTION.get().is_none() {
        return;
    }

    let task = TASK_OPEN_AND_MAINTAIN_CONNECTION
        .get()
        .unwrap()
        .lock()
        .unwrap();
    task.abort_and_wait();
    cleanup_ports(gateway, external_port);
}

#[tokio::main]
async fn main() {
    let config_path = match get_config_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            process::exit(1);
        }
    };

    let config = Config::load_or_create(&config_path).unwrap();

    let device_port = config.device_port;
    let external_port = config.router_port;

    // Discover the gateway
    let gateway = match search_gateway(Default::default()) {
        Ok(gw) => gw,
        Err(e) => {
            eprintln!("Failed to discover gateway: {}", e);
            process::exit(1);
        }
    };

    // register_windows_console_ctrl_handler(|| {
    //     thread::sleep(Duration::from_secs(2));
    //     keep_active_handle.abort();
    //     cleanup_ports(gateway, external_port);
    // });

    // Two issues:
    //

    // register_windows_console_ctrl_handler(|| {
    //     thread::sleep(Duration::from_secs(4));
    // });

    let future_connection = open_and_keep_active(gateway.clone(), device_port, external_port);
    let task_connection = DeferredTask::new(future_connection);
    TASK_OPEN_AND_MAINTAIN_CONNECTION
        .set(Arc::new(Mutex::new(task_connection)))
        .unwrap();

    // Register cleanups
    let gateway_clone = gateway.clone();
    std::panic::set_hook(Box::new(move |_| {
        tokio::runtime::Handle::current().block_on(async {
            shutdown_program(gateway_clone.clone(), external_port);
        });
    }));
    let gateway_clone = gateway.clone();
    register_windows_console_ctrl_handler(move || {
        shutdown_program(gateway_clone.clone(), external_port);
    });

    // Start the connection task
    TASK_OPEN_AND_MAINTAIN_CONNECTION
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .start();

    // Handle both Ctrl+C and terminal close
    // #[cfg(target_family = "unix")]
    // let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    //     .expect("Failed to create signal handler");

    // #[cfg(windows)]
    // let mut close_handler =
    //     tokio::signal::windows::ctrl_close().expect("Failed to create ctrl_close signal handler");
    // #[cfg(windows)]
    // let close_signal = close_handler.recv();

    // let ctrl_c_signal = tokio::signal::ctrl_c();

    // tokio::select! {
    //     _ = close_signal  => {}
    //     _ = ctrl_c_signal => {}
    // }

    // Keep the program running. Shutdown handled by the ConsoleCtrlHandler
    tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
}
