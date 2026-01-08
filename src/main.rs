mod runner;
mod rapl;
mod monitor;
mod nvml;

use clap::Parser;
use crate::rapl::RaplReader;

#[derive(Debug, Parser)]
struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>
}
fn main() {
    // Parse command
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("No command specified. Usage carbontime <command> [args...]");
    }

    let reader = RaplReader::new();
    println!("WEE")
}
