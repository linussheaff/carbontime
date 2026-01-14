use crate::rapl::{RaplReader, RaplType};
use crate::nvml::{NvmlReader, NvmlAccumulator};
use anyhow::Result;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct EnergyMeasurement {
    pub cpu_energy: Vec<(RaplType, u64)>,
    pub highest_rapl_domain: RaplType,
    pub rapl_domains: Vec<RaplType>,
    pub gpu_energy: Vec<u64>,
    pub gpu_names: Vec<String>,
    pub rapl_available: bool,
    pub nvml_available: bool,
}

impl EnergyMeasurement {
    pub fn total_cpu_energy(&self) -> u64 {
        // Try to take measurements from the highest domain available
        let mut total = 0;

        for top_domain in &self.rapl_domains{
            total = self.cpu_energy
                .iter()
                .filter(|(domain, _)| *domain == *top_domain)
                .map(|(_, uj)| uj)
                .sum();
            println!("{}", total);
            if total != 0{
                return total;
            }
        }

        total
    }

    pub fn total_gpu_energy(&self) -> u64 {
        self.gpu_energy.iter().sum()
    }

    pub fn total_energy(&self) -> u64 {
        self.total_cpu_energy() + self.total_gpu_energy()
    }
}

/// Energy monitor that runs in a background thread
pub struct EnergyMonitor {
    // Use a channel to signal the stop event.
    // The empty tuple () acts as the message.
    stop_tx: mpsc::Sender<()>,
    handle: Option<JoinHandle<EnergyMeasurement>>,
}

impl EnergyMonitor {
    pub fn start(sample_interval: Duration, gpu: bool, cpu: bool) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            monitor_loop(rx, sample_interval, gpu, cpu)
        });

        Ok(Self { stop_tx: tx, handle: Some(handle) })
    }

    pub fn stop(mut self) -> EnergyMeasurement {
        let _ = self.stop_tx.send(());
        if let Some(h) = self.handle.take() {
            h.join().unwrap_or_default()
        } else{
            EnergyMeasurement::default()
        }
    }
}

fn monitor_loop(
    stop_rx: mpsc::Receiver<()>,
    sample_interval: Duration,
    gpu: bool,
    cpu: bool) -> EnergyMeasurement {
    let mut measurement = EnergyMeasurement::default();


    if cpu{
        // Initialise rapl if possible
        let rapl_reader = match RaplReader::new() {
            Ok(Some(reader)) => {
                measurement.rapl_available = true;
                Some(reader)
            },
            Ok(None) => {
                eprintln!("Warning: RAPL (CPU Power) is not supported on this machine.");
                None
            },
            Err(e) => {
                eprintln!("Warning: Failed to initialize RAPL: {}. CPU power will be 0.", e);
                None
            }
        };
        // Take initial RAPL snapshot
        let rapl_start = rapl_reader.as_ref().and_then(|r| {
            match r.create_snapshot() {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("Warning: Failed to read initial RAPL snapshot: {}", e);
                    None
                }
            }
        });

        // Stay nested
        if gpu{
            // Initialise NVML if possible
            let nvml_reader = match NvmlReader::new() {
                Ok(Some(reader)) => {
                    measurement.nvml_available = true;
                    measurement.gpu_names = reader.device_names();
                    Some(reader)
                },
                Ok(None) => {
                    eprintln!("Warning: No Nvidia GPU detected. GPU power will be 0.");
                    None
                },
                Err(e) => {
                    eprintln!("Warning: NVML Init failed: {}. GPU power will be 0.", e);
                    None
                }
            };

            // Initialize Accumulator (only if reader exists)
            let mut nvml_accumulator = nvml_reader.as_ref().map(|r| {
                NvmlAccumulator::new(r.device_count() as usize)
            });

            // Immediately take an NVML screenshot
            if let (Some(reader), Some(acc)) = (&nvml_reader, &mut nvml_accumulator) {
                acc.add_sample(reader.energy_snapshot())
            };
            // Main loop
            loop {
                match stop_rx.recv_timeout(sample_interval) {
                    // Case: we received the stop signal (Ok) or channel closed (Err::Disconnected)
                    Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    // Case: timeout elapsed so sample_interval has passed
                    Err(RecvTimeoutError::Timeout) => {
                        if let (Some(reader), Some(acc)) = (&nvml_reader, &mut nvml_accumulator) {
                            acc.add_sample(reader.energy_snapshot());
                        }
                    }
                }
            }
            // Get GPU Energy acc. sum
            if let Some(acc) = nvml_accumulator {
                measurement.gpu_energy = acc.energy;
            }

        } else {
            println!("Not measuring GPU");
        }
        // Take final RAPL reading
        if let (Some(reader), Some(start)) = (&rapl_reader, &rapl_start) {
            match reader.create_snapshot() {
                Ok(end) => {
                    measurement.cpu_energy = reader.snapshot_difference(start, &end);

                    // Record what rapl domains we've used
                    measurement.rapl_domains = start.readings.iter().map(|(domain, _)| domain.clone()).collect();
                    measurement.rapl_domains.sort();
                },
                Err(e) => eprintln!("Warning: Failed to read final RAPL snapshot: {}", e),
            }
        }
    } else{
        println!("Not measuring CPU");
    }


    measurement
}