use crate::config::Config;
use crate::host::Host;
use clap::ArgAction;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct ListJobArgs {
    /// Show the steps defined for each job
    #[arg(short = 's', long, action = ArgAction::SetTrue)]
    show_steps: bool,
}

pub fn list_jobs<H: Host>(args: &ListJobArgs, host: &H, cfg: &Config) {
    if cfg.jobs().is_empty() {
        host.println("No jobs defined in the workspace configuration.");
        return;
    }

    for (job_id, job) in cfg.jobs().iter() {
        host.println(job_id.as_str());

        if args.show_steps {
            for step in job.steps() {
                if let Some(id) = step.id() {
                    host.println(format!("  {}", id.as_str()));
                } else {
                    host.println(format!("  {}", step.name()));
                }
            }
        }
    }
}
