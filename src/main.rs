mod config;
mod platform{
    pub mod windows;
}

use config::Config;
use igd::search_gateway;
use igd::PortMappingProtocol;
use std::env;
use std::fs;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::path::Path;
use std::process;
use tokio::runtime::Runtime;
use tokio::time::{self, Duration};
use platform::windows::register_windows_console_ctrl_handler;

const LEASE_TIME: u32 = 3600;
const LEASE_RENEWAL_INTERVAL: u32 = 3000;

fn get_config_path() -> io::Result<std::path::PathBuf> {
    let current_dir = env::current_dir()?;
    Ok(current_dir.join("config.toml"))
}

fn prompt(config: &mut Config, path: &Path) -> io::Result<()> {
    println!("Please set the device port in the config file at `config.toml`, in this directory.",);
    println!("Example:");
    println!("device_port = 8080");
    fs::write(
        path,
        toml::to_string_pretty(&config).expect("Failed to serialize config"),
    )?;
    println!("\nPlease edit it and rerun the program.");
    println!("Press Enter to close.");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(())
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

async fn open_and_keep_active(
    gateway: igd::Gateway,
    local_ip: Ipv4Addr,
    device_port: u16,
    router_port: u16,
) {
    let renewal_interval = Duration::from_secs(LEASE_RENEWAL_INTERVAL.into());
    let mut first_run = true;

    loop {
        // Add/Renew TCP Port Mapping
        match gateway.add_port(
            igd::PortMappingProtocol::TCP,
            router_port,
            SocketAddrV4::new(local_ip, device_port),
            LEASE_TIME,
            "Rust UPnP Port Forwarder - TCP",
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
            device_port,
            SocketAddrV4::new(local_ip, device_port),
            LEASE_TIME,
            "Rust UPnP Port Forwarder - UDP",
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

        first_run = false;
        time::sleep(renewal_interval).await;
    }
}

fn cleanup_ports(gateway: &igd::Gateway, router_port: u16) {
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

#[tokio::main]
async fn main() {
    let config_path = match get_config_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            process::exit(1);
        }
    };

    let mut config = match Config::load_or_create(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            process::exit(1);
        }
    };

    if !config.is_complete() {
        if let Err(e) = prompt(&mut config, &config_path) {
            eprintln!("Error creating config: {}", e);
            process::exit(1);
        }
        process::exit(0);
    }

    let device_port = config.device_port;
    let external_port = config.router_port.unwrap_or(config.device_port);

    // Discover the gateway
    let gateway = match search_gateway(Default::default()) {
        Ok(gw) => gw,
        Err(e) => {
            eprintln!("Failed to discover gateway: {}", e);
            process::exit(1);
        }
    };

    // Get local IP
    let external_ip = match gateway.get_external_ip() {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to get external IP: {}", e);
            process::exit(1);
        }
    };
    let local_ip = external_ip; // Router recognizes itself

    let gateway_clone = gateway.clone();
    std::panic::set_hook(Box::new(move |_| {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            cleanup_ports(&gateway_clone, external_port);
        });
    }));

    register_windows_console_ctrl_handler(None);

    // Spawn the port mapping task to run in the background
    let keep_active_handle = tokio::spawn(open_and_keep_active(
        gateway.clone(),
        local_ip,
        device_port,
        external_port,
    ));

    println!("");
    println!("Port forwarding is active. External IP:");
    println!("{}:{}", external_ip, external_port);
    println!("");
    println!("Press Ctrl+C to terminate.");

    // Handle both Ctrl+C and terminal close
    // #[cfg(target_family = "unix")]
    // let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    //     .expect("Failed to create signal handler");

    #[cfg(windows)]
    let mut close_handler =
        tokio::signal::windows::ctrl_close().expect("Failed to create ctrl_close signal handler");
    #[cfg(windows)]
    let close_signal = close_handler.recv();

    let ctrl_c_signal = tokio::signal::ctrl_c();

    tokio::select! {
        _ = close_signal  => {}
        _ = ctrl_c_signal => {}
    }

    // Wait for the background task to finish (it should be aborted by now)
    keep_active_handle.abort();
    cleanup_ports(&gateway, external_port);
}
