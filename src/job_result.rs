use crate::host::Host;

/// Struct to hold the result of a job execution.
#[derive(Debug)]
pub struct JobResult {
    pub name: String,
    pub packages: usize,
    pub steps: usize,
    pub failures: usize,
    pub dry_run: bool,
}

impl JobResult {
    #[must_use]
    pub const fn new(name: String, packages: usize, steps: usize, failures: usize, dry_run: bool) -> Self {
        Self {
            name,
            packages,
            steps,
            failures,
            dry_run,
        }
    }
}

pub fn print_summary(results: &[JobResult], host: &dyn Host) {
    host.println("\nSummary:");
    let mut total_jobs = 0usize;
    let mut total_pkgs = 0usize;
    let mut total_steps = 0usize;
    let mut total_failures = 0usize;

    for r in results {
        total_jobs += 1;
        total_pkgs += r.packages;
        total_steps += r.packages * r.steps; // expands per package except parallel prints aggregated steps; approximate
        total_failures += r.failures;
        let status = if r.failures == 0 { "OK" } else { "FAIL" };
        let mode = if r.dry_run { "dry-run" } else { "run" };
        host.println_fmt(format_args!(
            "  Job {:<15} status={status} packages={} steps={} failures={} mode={mode}",
            r.name, r.packages, r.steps, r.failures
        ));
    }
    host.println_fmt(format_args!(
        "  Total jobs={total_jobs} packages={total_pkgs} steps={total_steps} failures={total_failures}"
    ));
    if total_failures > 0 {
        host.println("Overall: FAILURE");
    } else {
        host.println("Overall: SUCCESS");
    }
}
