use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "cctmog-combined")]
#[command(about = "CCTMOG Poker - Combined server and client launcher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run both server and multiple clients
    Both {
        /// Number of clients to start
        #[arg(short, long, default_value = "2")]
        clients: u32,
        /// Port for the server
        #[arg(short, long, default_value = "9001")]
        port: u16,
    },
    /// Run only the server
    Server {
        /// Port for the server
        #[arg(short, long, default_value = "9001")]
        port: u16,
    },
    /// Run only a client
    Client,
    /// Run clients only (no server initially)
    ClientsOnly {
        /// Number of clients to start
        #[arg(short, long, default_value = "2")]
        clients: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Both { clients, port } => {
            run_both(clients, port);
        }
        Commands::Server { port } => {
            run_server(port);
        }
        Commands::Client => {
            run_client();
        }
        Commands::ClientsOnly { clients } => {
            run_clients_only(clients);
        }
    }
}

fn run_both(clients: u32, port: u16) {
    println!("🚀 Starting CCTMOG Poker - Lounge server + {} clients on port {}", clients, port);

    // Start lounge/discovery server in background on port 9001
    println!("📡 Starting lounge server on port 9001...");
    let server_handle = thread::spawn(move || {
        run_server(9001);  // Always use 9001 for lounge server
    });

    // Wait a moment for server to start
    thread::sleep(Duration::from_millis(1500));

    // Start clients
    let mut client_handles = Vec::new();
    for i in 1..=clients {
        println!("🎮 Starting client {}...", i);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(500 * i as u64)); // Stagger client starts
            run_client_with_index(i);
        });
        client_handles.push(handle);
    }

    println!("✅ All processes started.");
    println!("💡 Note: Each client can volunteer to host games (embedded servers on port 9002+)");
    println!("Press Ctrl+C to stop.");

    // Wait for all clients to finish (they won't unless killed)
    for handle in client_handles {
        let _ = handle.join();
    }

    // Wait for server to finish
    let _ = server_handle.join();
}

fn run_server(port: u16) {
    let status = Command::new("cargo")
        .args(&["run", "-p", "cctmog-server"])
        .env("PORT", port.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("❌ Server exited with error: {}", exit_status);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_clients_only(clients: u32) {
    println!("🚀 Starting {} CCTMOG clients (no server)", clients);

    // Start clients
    let mut client_handles = Vec::new();
    for i in 1..=clients {
        println!("🎮 Starting client {}...", i);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(500 * i as u64)); // Stagger client starts
            run_client_with_index(i);
        });
        client_handles.push(handle);
    }

    println!("✅ All clients started. Press Ctrl+C to stop.");

    // Wait for all clients to finish (they won't unless killed)
    for handle in client_handles {
        let _ = handle.join();
    }
}

fn run_client() {
    let status = Command::new("cargo")
        .args(&["run", "-p", "cctmog"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("❌ Client exited with error: {}", exit_status);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to start client: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_client_with_index(index: u32) {
    let status = Command::new("cargo")
        .args(&["run", "-p", "cctmog"])
        .env("CLIENT_INDEX", index.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("❌ Client exited with error: {}", exit_status);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to start client: {}", e);
            std::process::exit(1);
        }
    }
}