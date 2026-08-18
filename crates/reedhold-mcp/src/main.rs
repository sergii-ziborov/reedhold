//! `reedhold` binary: MCP stdio for agents.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match run(&arguments) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("reedhold: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<ExitCode, String> {
    match arguments.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("reedhold {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Some("--help" | "-h") | None => {
            print_help();
            Ok(ExitCode::SUCCESS)
        }
        Some("mcp") => {
            let mut server = reedhold_mcp::build_server();
            mcport::serve(&mut server).map_err(|error| error.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        Some(other) => Err(format!("unknown command {other}")),
    }
}

fn print_help() {
    println!("reedhold {version}", version = env!("CARGO_PKG_VERSION"));
    println!("Usage: reedhold mcp");
    println!("       reedhold --version");
}
