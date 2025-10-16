use crate::host::Host;
use crate::ws_config::WorkspaceConfig;
use clap::ArgAction;
use clap::Parser;

// TODO: implement the ability to display job steps

#[derive(Parser, Debug)]
pub struct ListJobArgs {
    /// Show the steps defined for each job
    #[arg(short = 's', long, action = ArgAction::SetTrue)]
    pub show_steps: bool,
}

pub fn list_jobs<H: Host>(_: &ListJobArgs, host: &mut H, ws_config: &WorkspaceConfig) {
    for (job_id, job) in ws_config.jobs().iter() {
        let name = job.name().unwrap_or(job_id.as_str());
        host.println(format!("Job: {name}"));
    }
}
