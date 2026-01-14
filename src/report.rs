//! Report generation for energy measurements

use crate::carbon::{self, CarbonIntensity};
use crate::monitor::EnergyMeasurement;
use colored::*;
use std::time::Duration;

/// Format a duration in a human-readable way
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = d.subsec_millis();

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// Format energy with appropriate units
fn format_energy(uj: u64) -> String {
    let joules = uj as f64 / 1_000_000.0;

    if joules >= 1_000_000.0 {
        format!("{:.2} MJ ({:.2} kWh)", joules / 1_000_000.0, joules / 3_600_000.0)
    } else if joules >= 1_000.0 {
        format!("{:.2} kJ ({:.2} Wh)", joules / 1_000.0, joules / 3_600.0)
    } else {
        format!("{:.2} J", joules)
    }
}

/// Format CO2 with appropriate units
fn format_co2(grams: f64) -> String {
    if grams >= 1000.0 {
        format!("{:.2} kg", grams / 1000.0)
    } else {
        format!("{:.0}g", grams)
    }
}

/// Print the energy report
pub fn print_report(
    duration: Duration,
    measurement: &EnergyMeasurement,
    intensity: &CarbonIntensity,
    verbose: bool,
) {
    let separator = "─".repeat(55);

    println!();
    println!("{}", separator.bright_blue());
    println!("{}", "carbontime report".bright_green().bold());
    println!("{}", separator.bright_blue());

    // Duration
    println!(
        "{:<16} {}",
        "Duration:".white(),
        format_duration(duration).bright_white()
    );

    // CPU Energy
    if measurement.rapl_available {
        if verbose && !measurement.cpu_energy.is_empty() {
            println!("{}", "CPU Energy:".white());
            for (domain, uj) in &measurement.cpu_energy {
                println!("  {:<14} {}", format!("{:?}:", domain), format_energy(*uj));
            }
            println!(
                "  {:<14} {}",
                "Total:".white(),
                format_energy(measurement.total_cpu_energy()).bright_white()
            );
        } else {
            println!(
                "{:<16} {}",
                "CPU Energy:".white(),
                format_energy(measurement.total_cpu_energy()).bright_white()
            );
        }
    } else {
        println!(
            "{:<16} {}",
            "CPU Energy:".white(),
            "(RAPL not available)".dimmed()
        );
    }

    // GPU Energy
    if measurement.nvml_available {
        if verbose && measurement.gpu_energy.len() > 1 {
            println!("{}", "GPU Energy:".white());
            for (i, uj) in measurement.gpu_energy.iter().enumerate() {
                let name = measurement
                    .gpu_names
                    .get(i)
                    .map(|s| s.as_str())
                    .unwrap_or("GPU");
                println!("  {:<14} {}", format!("{}:", name), format_energy(*uj));
            }
            println!(
                "  {:<14} {}",
                "Total:".white(),
                format_energy(measurement.total_gpu_energy()).bright_white()
            );
        } else if measurement.total_gpu_energy() > 0 {
            println!(
                "{:<16} {}",
                "GPU Energy:".white(),
                format_energy(measurement.total_gpu_energy()).bright_white()
            );
        } else {
            println!(
                "{:<16} {}",
                "GPU Energy:".white(),
                "(no GPU activity detected)".dimmed()
            );
        }
    } else {
        println!(
            "{:<16} {}",
            "GPU Energy:".white(),
            "0.0".dimmed()
        );
    }

    // Total Energy
    let total_uj = measurement.total_energy();
    if total_uj > 0 {
        println!("{}", separator.dimmed());
        println!(
            "{:<16} {}",
            "Total Energy:".white().bold(),
            format_energy(total_uj).bright_white().bold()
        );

        // CO2 estimation
        let co2_grams = carbon::calculate_co2_grams(total_uj, intensity);
        println!(
            "{:<16} {} {}",
            "Est. CO2:".white(),
            format!("~{}", format_co2(co2_grams)).bright_yellow(),
            format!("({}: {} gCO2/kWh)", intensity.name, intensity.gco2_per_kwh).dimmed()
        );

        // Power average
        let duration_hours = duration.as_secs_f64() / 3600.0;
        if duration_hours > 0.0 {
            let avg_watts = (total_uj as f64 / 1_000_000.0) / duration.as_secs_f64();
            println!(
                "{:<16} {}",
                "Avg Power:".white(),
                format!("{:.1} W", avg_watts).bright_white()
            );
        }
    } else {
        println!("{}", separator.dimmed());
        println!(
            "{}",
            "No energy data available. This may be because:".yellow()
        );
        println!("{}", "  • RAPL requires root or appropriate permissions".dimmed());
        println!("{}", "  • NVML requires NVIDIA drivers and GPUs".dimmed());
        println!("{}", "  • The process ran too briefly to measure".dimmed());
    }

    println!("{}", separator.bright_blue());
}

/// Print a minimal one-line summary
pub fn print_summary(duration: Duration, measurement: &EnergyMeasurement, intensity: &CarbonIntensity) {
    let total_uj = measurement.total_energy();
    let co2_grams = carbon::calculate_co2_grams(total_uj, intensity);

    if total_uj > 0 {
        println!(
            "{} | {} | ~{}",
            format_duration(duration),
            format_energy(total_uj),
            format_co2(co2_grams)
        );
    } else {
        println!(" {} | (no energy data)", format_duration(duration));
    }
}

/// Output as JSON for machine consumption
pub fn print_json(
    duration: Duration,
    measurement: &EnergyMeasurement,
    intensity: &CarbonIntensity,
    exit_code: i32,
) {
    let total_uj = measurement.total_energy();
    let co2_grams = carbon::calculate_co2_grams(total_uj, intensity);

    println!(
        r#"{{"duration_ms": {}, "cpu_energy_uj": {}, "gpu_energy_uj": {}, "total_energy_uj": {}, "co2_grams": {:.2}, "region": "{}", "exit_code": {}}}"#,
        duration.as_millis(),
        measurement.total_cpu_energy(),
        measurement.total_gpu_energy(),
        total_uj,
        co2_grams,
        intensity.name,
        exit_code
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.500s");
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
    }

    #[test]
    fn test_format_energy() {
        assert!(format_energy(1_000_000).contains("1.00 J"));
        assert!(format_energy(1_000_000_000).contains("1.00 kJ"));
        assert!(format_energy(1_000_000_000_000).contains("1.00 MJ"));
    }
}
