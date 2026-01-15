use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of RAPL files available
#[derive(Debug, Clone, PartialEq, Default, Ord, Eq, PartialOrd)]
pub enum RaplType{
    #[default]
    Package, // measures CPU
    Dram, // measures RAM
    Psys, // measures entire system on chip
}
/// Represents a single RAPL domain (e.g., package-0, core, dram)
#[derive(Debug, Clone)]
pub struct RaplDomain {
    pub name: RaplType,
    pub path: PathBuf,
    pub max_energy_uj: u64,
}

#[derive(Debug)]
pub struct RaplReader {
    domains: Vec<RaplDomain>,
}

#[derive(Debug, Clone, Default)]
pub struct RaplSnapshot {
    pub readings: Vec<(RaplType, u64)>,
    pub timestamp_ns: u64,
}

impl RaplReader {
    /// Public entry point: Discover and initialise all RAPL domains
    pub fn new() -> Result<Option<Self>> {
        // Call the internal scanner. If it fails (PermissionDenied), we return the error.
        let domains = Self::scan()?;

        if domains.is_empty() {
            Ok(None) // No RAPL found
        } else {
            Ok(Some(Self { domains }))
        }
    }

    /// Internal helper to walk the directory tree
    fn scan() -> Result<Vec<RaplDomain>> {
        let rapl_path = Path::new("/sys/class/powercap/intel-rapl");

        if !rapl_path.exists() {
            return Ok(Vec::new()); // Return empty vector, don't crash
        }

        let mut domains = Vec::new();

        // iterate over /sys/class/powercap/intel-rapl/intel-rapl:*
        for entry in fs::read_dir(rapl_path)? {
            let entry = entry?; // Unwrap the directory entry
            let path = entry.path();

            if path.is_dir() {
                // Read the "name" file to know what we found
                if let Some(name) = Self::read_name(&path) {
                    if name.starts_with("package") {
                        // Add the Package itself
                        if let Some(domain) = Self::create_domain(&path, RaplType::Package) {
                            domains.push(domain);
                        }

                        // Look inside for DRAM (subdirectories)
                        Self::scan_subdomains(&path, &mut domains)?;
                    } else if name == "psys" {
                        if let Some(domain) = Self::create_domain(&path, RaplType::Psys) {
                            domains.push(domain);
                        }

                        Self::scan_subdomains(&path, &mut domains)?;
                    }
                }
            }
        }
        Ok(domains)
    }

    /// Helper to look for "dram" inside a package folder
    fn scan_subdomains(package_path: &Path, domains: &mut Vec<RaplDomain>) -> Result<()> {
        for entry in fs::read_dir(package_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = Self::read_name(&path) {
                    // Only support DRAM for now
                    if name == "dram" {
                        if let Some(domain) = Self::create_domain(&path, RaplType::Dram) {
                            domains.push(domain);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper to construct a RaplDomain from a directory
    fn create_domain(path: &Path, name: RaplType) -> Option<RaplDomain> {
        let energy_path = path.join("energy_uj");
        let max_path = path.join("max_energy_range_uj");

        // If the main energy file is missing/unreadable, do nothing for now todo handle
        if !energy_path.exists() {
            return None;
        }

        // Try to read max_energy_range
        let max_energy_uj = match fs::read_to_string(&max_path) {
            Ok(content) => {
                // If parsing fails, warn and use default.
                match content.trim().parse::<u64>() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("Warning: Could not parse '{}'. Defaulting to u64::MAX.", max_path.display());
                        u64::MAX
                    }
                }
            },
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        // expected on some systems, still tell user
                        eprintln!("Warning: No max energy uj found. Defaulting to u64::MAX.");
                        u64::MAX
                    },
                    ErrorKind::PermissionDenied => {
                        // This is a user error.
                        eprintln!("Warning: Permission denied for '{}'. Run as root for accurate overflow handling.", max_path.display());
                        u64::MAX
                    },
                    _ => {
                        // Any other I/O error (disk failure, etc)
                        eprintln!("Warning: Failed to read '{}': {}. Defaulting.", max_path.display(), e);
                        u64::MAX
                    }
                }
            }
        };

        Some(RaplDomain {
            name,
            path: energy_path,
            max_energy_uj,
        })
    }

    /// Wee helper to read the "name" file safely
    fn read_name(path: &Path) -> Option<String> {
        let name_path = path.join("name");
        fs::read_to_string(name_path).ok().map(|s| s.trim().to_string())
    }

    /// Create energy reading snapshot
    pub fn create_snapshot(&self) -> Result<RaplSnapshot> {
        let timestamp_ns = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64;

        let mut readings = Vec::with_capacity(self.domains.len());

        for domain in &self.domains {
            let energy_str = fs::read_to_string(&domain.path)?;

            let energy_uj = energy_str.trim().parse::<u64>()
                .with_context(|| format!("Failed to parse energy uj '{}'", energy_str))?;

            readings.push((domain.name.clone(), energy_uj));
        }

        Ok(RaplSnapshot {
            readings,
            timestamp_ns,
        })
    }

    /// Calculates the difference between two snapshots (handling overflow)
    pub fn snapshot_difference(&self, start: &RaplSnapshot, end: &RaplSnapshot) -> Vec<(RaplType, u64)> {
        let mut results = Vec::new();

        for (i, domain) in self.domains.iter().enumerate() {
            if let (Some((name, start_uj)), Some((_, end_uj))) =
                (start.readings.get(i), end.readings.get(i))
            {
                // Handle counter overflow
                let energy_uj = if *end_uj >= *start_uj {
                    end_uj - start_uj
                } else {
                    // Counter wrapped around
                    (domain.max_energy_uj - start_uj) + end_uj
                };

                results.push((name.clone(), energy_uj));
            }
        }
        results
    }
}