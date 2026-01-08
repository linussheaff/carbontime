use nvml_wrapper::Nvml;
use anyhow::{Result, Context};

pub struct NvmlReader{
    nvml: Nvml,
    device_count: u32
}

pub struct NvmlSnapshot{
    pub readings: Vec<u32>,
    pub timestamp_ns: u64
}

impl NvmlReader {
    pub fn new() -> Result<Option<Self>> {
        let nvml = match Nvml::init(){
            Ok(n) => n,
            Err(e) => {
                eprintln!("Warning: NVML not available due to ({e}), running in CPU only mode (ignore GPU measurements with TODO flag)");
                return Ok(None);
            }
        };

        let device_count = match nvml.device_count(){
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: could not get NVML device count due to ({e}), running in CPU only mode");
                return Ok(None);
            }
        };

        if device_count == 0 {
            return Ok(None);
        }

        Ok(Some(NvmlReader{nvml, device_count}))
    }

    pub fn device_count(&self) -> u32 {self.device_count}

    pub fn energy_snapshot(&self) -> NvmlSnapshot {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let mut power = Vec::with_capacity(self.device_count as usize);

        // Get readings, don't panic if you can't get reading for any reason but do warn
        for i in 0..self.device_count {
            let device = match self.nvml.device_by_index(i){
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Warning: Could  not get power usage for device ({i}) due to error ({e}), defaulting to 0");
                    power.push(0);
                    continue;
                }
            };


            let power_i = device.power_usage().unwrap_or_else(|e| {
                    eprintln!("Warning: Could  not get power usage for device ({i}) due to error ({e}), defaulting to 0");
                    0
                });

            power.push(power_i);
        }
        NvmlSnapshot{readings: power, timestamp_ns: time}

    }

    /// Get GPU names for reporting, rust silliness
    pub fn device_names(&self) -> Vec<String> {
        (0..self.device_count)
            .map(|i| {
                self.nvml
                    .device_by_index(i)
                    .ok()
                    .and_then(|d| d.name().ok())
                    .unwrap_or_else(|| format!("GPU {i}"))
            })
            .collect()
    }
}

pub struct NvmlAccumulator{
    pub energy: Vec<u64>,
    last_snapshot: Option<NvmlSnapshot>,
}

impl NvmlAccumulator {
    pub fn new(gpu_count: usize) -> NvmlAccumulator {
        Self {
            energy: vec![0; gpu_count],
            last_snapshot: None,
        }
    }

    /// Accumulate energy based on an NVML snapshot
    pub fn add_sample(&mut self, snapshot: NvmlSnapshot) {
        // Unpack the last snapshot if it exists
        if let Some(last_snapshot) = &self.last_snapshot {
            // Calculate time delta
            let time_delta_ns = snapshot.timestamp_ns.saturating_sub(last_snapshot.timestamp_ns);
            let dt_s = time_delta_ns as f64 / 1_000_000_000.0;


            // Calculate energy. Power is in mW, we want µJ
            // mW * s = mJ, mJ * 1000 = µJ
            for (i, &power) in snapshot.readings.iter().enumerate() {
                // Check we haven't gotten more readings from last snapshot
                if i < self.energy.len(){
                    let energy = (power as f64 * dt_s * 1000.0) as u64;
                    self.energy[i] += energy;
                }
                else{
                    eprintln!("Warning: more GPUs in last snapshot than we are tracking");
                }
            }
        }
        self.last_snapshot = Some(snapshot);
    }
}