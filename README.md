# carbontime

A lightweight command wrapper and Python library that measures energy consumption and estimates carbon emissions.

**CLI Mode:**
```bash
$ carbontime python train_model.py
```bash
$ carbontime python train_model.py

Training epoch 1/10...
Training complete!

───────────────────────────────────────────────────
 carbontime report
───────────────────────────────────────────────────
Duration:        10m 23s
CPU Energy:      142.3 kJ (39.5 Wh)
GPU Energy:      891.2 kJ (247.6 Wh)
Total Energy:    1033.5 kJ (287.1 Wh)
Est. CO2:        ~104g (uk: 207 gCO2/kWh)
Avg Power:       165.8 W
───────────────────────────────────────────────────
```

**Python Library mode**
```python
import carbontime

tracker = carbontime.CarbonTracker(interval_ms=100)
with tracker as measurements:
    # Your training code here...
    train_model()

print(measurements) 
# {'total_energy_uj': 1023400, 'cpu_energy_uj': ...}
```


## Features

- **CPU energy** via Intel/AMD RAPL (Running Average Power Limit)
- **GPU energy** via NVIDIA NVML (Management Library)
- **Carbon estimation** with regional grid intensity data
- **Dual Mode** use as a a standalone CLI tool or import directly into Python

## Installation

### CLI Tool (Binary)

```bash
git clone https://github.com/yourusername/carbontime
cd carbontime
cargo build --release
sudo cp target/release/carbontime /usr/local/bin/
```

### Python Library
Install with your favourite package manager 
```bash
pip install .
```

### Permissions

RAPL energy counters require either:
- Root access, OR
- Read permission on `/sys/class/powercap/intel-rapl:*/energy_uj`

To allow non-root access:

```bash
# Option 1: Add user to a group with access (recommended)
sudo groupadd rapl
sudo chown -R root:rapl /sys/class/powercap/intel-rapl*
sudo chmod -R g+r /sys/class/powercap/intel-rapl*
sudo usermod -aG rapl $USER

# Option 2: Use capabilities (per-binary)
sudo setcap cap_sys_rawio+ep /usr/local/bin/carbontime
```

## Usage

### CLI usage

```bash
carbontime <command> [args...]
```

#### Options

```
 -r, --region <REGION>      Grid region for CO2 estimation [default: world]
 -i, --interval <INTERVAL>  Sampling interval in milliseconds [default: 100]
 -f, --format <FORMAT>      Output format: pretty (default), json, or summary [default: pretty]
 -v, --verbose              Show detailed breakdown by component
     --list-regions         List available regions and exit
 -g, --gpu                  Ignore GPU
 -c, --cpu                  Ignore CPU
 -h, --help                 Print help
 -V, --version              Print version
```

#### Examples

```bash
# Basic usage
carbontime python train.py

# Specify region for accurate CO2 estimation
carbontime --region uk python train.py

# JSON output for scripting
carbontime --format json ./benchmark.sh > results.json

# One-line summary
carbontime --format summary make build

# Verbose breakdown
carbontime --verbose python train.py
```

#### JSON Output

```json
{
  "duration_ms": 623000,
  "cpu_energy_uj": 142300000000,
  "gpu_energy_uj": 891200000000,
  "total_energy_uj": 1033500000000,
  "co2_grams": 104.23,
  "region": "uk",
  "exit_code": 0
}
```

### Python Library Usage 
You can use `carbontime` directly in your Python scripts using the `CarbonTracker` context manager. This runs the Rust monitor in a background thread.

#### Basic Example 
```python 
import carbontime
import time

# Initialize tracker (default: 100ms interval, GPU enabled, CPU enabled)
tracker = carbontime.CarbonTracker(interval_ms=100)

print("Starting measurement...")

# The monitor starts when you enter the 'with' block
with tracker as results:
    # Simulate workload
    time.sleep(2)
    _ = [x**2 for x in range(1_000_000)]

# The monitor stops automatically. Results are available in a dictionary.
print(f"Total Energy: {results['total_energy_uj']} µJ")
print(f"CPU Energy:   {results['cpu_energy_uj']} µJ")
print(f"GPU Energy:   {results['gpu_energy_uj']} µJ")
```

#### Configuration
Configure the tracker by passing arguments to the constructor
```python 
# Disable GPU monitoring, set interval to 500ms
tracker = carbontime.CarbonTracker(interval_ms=500, gpu=False, cpu=True)
```



## Available regions

```bash
$ carbontime --list-regions
Available regions:
  uk               207 gCO2/kWh
  us-avg           390 gCO2/kWh
  us-california    230 gCO2/kWh
  germany          350 gCO2/kWh
  france           56 gCO2/kWh
  sweden           45 gCO2/kWh
  china            555 gCO2/kWh
  world            436 gCO2/kWh
  ...
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
