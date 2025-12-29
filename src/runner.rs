use anyhow::{Context, Result};
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

/// Result of running a command
pub struct RunResult{
    pub exit_status: ExitStatus,
    pub duration: Duration
}

/// Run a command and return its result
pub fn run_command(program: &str, args: &[String]) -> Result<RunResult> {
    let start = Instant::now();

    let exit_status = Command::new(program).args(args).status().with_context(|| format!("Failed to run program `{}`", program))?;
    let duration = start.elapsed();

    Ok(RunResult{
        exit_status,
        duration
    })
}

