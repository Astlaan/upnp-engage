mod config;

use config::Config;
use igd::search_gateway;
use igd::PortMappingProtocol;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::net::SocketAddrV4;
use std::path::Path;
use std::process;
use tokio::runtime::Runtime;

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
    println!("Config file created. Please edit it and rerun the program.");
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

fn cleanup_ports(gateway: &igd::Gateway, router_port: u16) {
    // Create or truncate the "empty_clean.txt" file
    if let Err(e) = File::create("empty_clean.txt") {
        eprintln!("Failed to create or truncate 'empty_clean.txt': {}", e);
        // Depending on your requirements, you might want to return early or continue
    }

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

    let router_port = config.router_port.unwrap_or(config.device_port);

    // Discover the gateway
    let gateway = match search_gateway(Default::default()) {
        Ok(gw) => gw,
        Err(e) => {
            eprintln!("Failed to discover gateway: {}", e);
            process::exit(1);
        }
    };

    // Get local IP
    let local_ip = match gateway.get_external_ip() {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to get external IP: {}", e);
            process::exit(1);
        }
    };

    // Add port mapping with a permanent lease
    // TCP Port Mapping
    match gateway.add_port(
        igd::PortMappingProtocol::TCP,
        config.device_port, // external port (router side)
        SocketAddrV4::new(local_ip, config.device_port), // internal address (your machine)
        0,                  // lease duration (0 = permanent)
        "Rust UPnP Port Forwarder - TCP", // description
    ) {
        Ok(_) => println!(
            "TCP Port {} forwarded to local port {} successfully.",
            router_port, config.device_port
        ),
        Err(e) => {
            eprintln!("Failed to add TCP port mapping: {}", e);
            process::exit(1);
        }
    }

    // UDP Port Mapping
    match gateway.add_port(
        igd::PortMappingProtocol::UDP,
        config.device_port, // external port (router side)
        SocketAddrV4::new(local_ip, config.device_port), // internal address (your machine)
        0,                  // lease duration (0 = permanent)
        "Rust UPnP Port Forwarder - UDP", // description
    ) {
        Ok(_) => println!(
            "UDP Port {} forwarded to local port {} successfully.",
            router_port, config.device_port
        ),
        Err(e) => {
            eprintln!("Failed to add UDP port mapping: {}", e);
            process::exit(1);
        }
    }

    // Setup panic handler for cleanup
    let gateway_clone = gateway.clone();
    let router_port_clone = router_port;
    std::panic::set_hook(Box::new(move |_| {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let gw = gateway_clone.clone();
        let port = router_port_clone;

        rt.block_on(async {
            cleanup_ports(&gw, port);
        });
    }));

    println!("Port forwarding is active.");
    println!("");
    println!("Press Ctrl+C to terminate or close this window to terminate.");

    // Handle both Ctrl+C and terminal close
    #[cfg(unix)]
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to create signal handler");

    #[cfg(windows)]
    let mut signal = tokio::signal::windows::ctrl_close().expect("Failed to create signal handler");

    let ctrl_c = tokio::signal::ctrl_c();

    // Test section
    // signal.recv().await;
    // let _ = File::create("empty_main.txt");
    // cleanup_ports(&gateway, router_port).await;
    // End test section

    // // Keep the program running and handle signals
    tokio::select! {
        _ = tokio::spawn(async move {
            signal.recv().await;
        }) => {
            let _ = File::create("empty_async_signal.txt");
            cleanup_ports(&gateway, router_port);
        }
        _ = ctrl_c => {
            cleanup_ports(&gateway, router_port);
            let _ = File::create("empty_async_ctrlc.txt");
        }
        // _ = tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)) => {
        //     // This will effectively never happen, but keeps the program running
        // }
    }

    // Cleanup before exit
}
