# carbontime 

A lightweight command wrapper that measures energy consumption and estimates carbon emissions.

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

## Features

- **CPU energy** via Intel/AMD RAPL (Running Average Power Limit)
- **GPU energy** via NVIDIA NVML (Management Library)
- **Carbon estimation** with regional grid intensity data

## Installation

### From source

```bash
git clone https://github.com/yourusername/carbontime
cd carbontime
cargo build --release
sudo cp target/release/carbontime /usr/local/bin/
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

### Basic usage

```bash
carbontime <command> [args...]
```

### Options

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

### Examples

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

### Available regions

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

## JSON Output

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

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
