//! carbontime - Measure energy consumption and carbon emissions of any command
//!
//! Usage: carbontime [OPTIONS] <COMMAND> [ARGS]...
//!
//! Example:
//!   carbontime python train_model.py
//!   carbontime --region uk cargo build --release
//!   carbontime --json ./my_script.sh



use anyhow::{bail, Result};
use clap::Parser;
use std::time::Duration;
use carbontime::{carbon, monitor, report, runner};

/// Measure energy consumption and estimate carbon emissions of any command
#[derive(Parser, Debug)]
#[command(
    name = "carbontime",
    version,
    about = "Measure energy consumption and estimate carbon emissions of any command",
    after_help = "EXAMPLES:\n    carbontime python train_model.py\n    carbontime --region uk cargo build --release\n    carbontime --json ./benchmark.sh"
)]
struct Args {
    /// Grid region for CO2 estimation
    #[arg(short, long, default_value = "world")]
    region: String,

    /// Sampling interval in milliseconds
    #[arg(short, long, default_value = "100")]
    interval: u64,

    /// Output format: pretty (default), json, or summary
    #[arg(short, long, default_value = "pretty")]
    format: String,

    /// Show detailed breakdown by component
    #[arg(short, long)]
    verbose: bool,

    /// List available regions and exit
    #[arg(long)]
    list_regions: bool,

    /// Ignore GPU
    #[arg(short, long, action = clap::ArgAction::SetFalse)]
    gpu: bool,

    /// Ignore CPU
    #[arg(short, long, action = clap::ArgAction::SetFalse)]
    cpu: bool,

    /// Command to run
    #[arg(trailing_var_arg = true, required_unless_present = "list_regions")]
    command: Vec<String>,


}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --list-regions
    if args.list_regions {
        println!("Available regions:");
        for region in carbon::available_regions() {
            let intensity = carbon::get_intensity(region).unwrap();
            println!("  {:<16} {} gCO2/kWh", region, intensity.gco2_per_kwh);
        }
        return Ok(());
    }

    // Validate region
    let intensity = match carbon::get_intensity(&args.region) {
        Some(i) => i,
        None => {
            eprintln!(
                "Unknown region: '{}'. Use --list-regions to see available options.",
                args.region
            );
            eprintln!("Falling back to world average.");
            carbon::get_intensity("world").unwrap()
        }
    };

    // Parse command
    if args.command.is_empty() {
        bail!("No command specified. Usage: carbontime <command> [args...]");
    }

    let program = &args.command[0];
    let cmd_args: Vec<String> = args.command[1..].to_vec();

    // Start energy monitoring
    let sample_interval = Duration::from_millis(args.interval);
    let monitor = monitor::EnergyMonitor::start(sample_interval, args.gpu, args.cpu)?;

    // Run the command
    let result = runner::run_command(program, &cmd_args)?;

    // Stop monitoring and get measurements
    let measurement = monitor.stop();

    // Print report
    match args.format.as_str() {
        "json" => {
            report::print_json(
                result.duration,
                &measurement,
                &intensity,
                result.exit_status.code().unwrap_or(-1),
            );
        }
        "summary" => {
            report::print_summary(result.duration, &measurement, &intensity);
        }
        _ => {
            report::print_report(result.duration, &measurement, &intensity, args.verbose);
        }
    }

    // Exit with the same code as the child process
    std::process::exit(result.exit_status.code().unwrap_or(1));
}

