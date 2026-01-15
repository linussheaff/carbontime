pub mod carbon;
pub mod monitor;
pub mod nvml;
pub mod rapl;
pub mod report;
pub mod runner;

use pyo3::prelude::*;
use pyo3::types::PyDict; 
use std::time::Duration;
use crate::monitor::EnergyMonitor;

/// Python class: CarbonTracker
#[pyclass]
struct CarbonTracker {
    monitor: Option<EnergyMonitor>,
    interval_ms: u64,
    gpu: bool,
    cpu: bool,
}

#[pymethods]
impl CarbonTracker {
    #[new]
    #[pyo3(signature = (interval_ms=100, gpu=true, cpu=true))]
    fn new(interval_ms: u64, gpu: bool, cpu: bool) -> Self {
        CarbonTracker {
            monitor: None,
            interval_ms,
            gpu,
            cpu,
        }
    }

    /// Enter context: `with tracker:`
    fn __enter__(mut self_: PyRefMut<Self>) -> PyResult<()> {
        let interval = Duration::from_millis(self_.interval_ms);

        // Call existing Rust start logic
        let monitor = EnergyMonitor::start(interval, self_.gpu, self_.cpu)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        self_.monitor = Some(monitor);
        Ok(())
    }

    /// Exit context: block ends
    fn __exit__(
        &mut self,
        py: Python<'_>,
        _exc_type: Option<Py<PyAny>>, 
        _exc_value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>
    ) -> PyResult<Py<PyAny>> { 

        if let Some(monitor) = self.monitor.take() {
            // Stop the monitor
            let measurement = monitor.stop();

            // Create a Bound dictionary
            let dict = PyDict::new(py);

            // Set items 
            dict.set_item("total_energy_uj", measurement.total_energy())?;
            dict.set_item("cpu_energy_uj", measurement.total_cpu_energy())?;
            dict.set_item("gpu_energy_uj", measurement.total_gpu_energy())?;

            // Convert Bound<PyDict> into Py<PyAny> (Generic Python Object)
            Ok(dict.into())
        } else {
            Ok(py.None())
        }
    }
}

/// The Python Module Definition
/// Note: The signature uses `Bound<'_, PyModule>` now.
#[pymodule]
fn carbontime(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CarbonTracker>()?;
    Ok(())
}