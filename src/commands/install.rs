use crate::color_modes::ColorModes;
use crate::config::{Config, Tool, ToolId};
use crate::host::Host;
use crate::log::Log;
use crate::outputter::Outputter;
use cargo_metadata::Metadata;
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser, Debug, Clone)]
pub struct InstallArgs {
    /// Send log output to the specified file.
    #[arg(short = 'l', long, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Number of log files to retain (default: 16).
    #[arg(long, default_value_t = 16, value_name = "COUNT")]
    log_file_retention_count: usize,

    /// Colorize output.
    #[arg(long, value_name = "WHEN", default_value_t = ColorModes::Auto, value_enum)]
    color: ColorModes,
}

pub fn install_tools<H: Host>(args: &InstallArgs, host: &mut H, cfg: &Config, metadata: &Metadata) -> anyhow::Result<()> {
    let log = Log::new(
        metadata.target_directory.as_std_path(),
        "install",
        args.log_file.as_deref(),
        args.log_file_retention_count,
    )?;

    // after this point, thia code takes care of error reporting itself
    host.fail_silently();

    let outputter = Outputter::new(host, &log, args.color);
    outputter.start_activity("Installing/Updating");

    let mut tools: Vec<_> = cfg.tools().iter().collect();
    tools.sort_by(|x, y| x.0.cmp(y.0));

    for (tool_id, tool) in &tools {
        install_tool(host, tool_id, tool, &outputter)?;
    }

    outputter.complete_activity(format!("installed or updated {} tool(s)", tools.len()));
    Ok(())
}

fn install_tool<H: Host>(host: &H, tool_id: &ToolId, tool: &Tool, outputter: &Outputter<H>) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

    _ = cmd.current_dir(std::env::current_dir().unwrap_or_default());
    _ = cmd.arg("install").arg(tool_id.to_string());
    _ = cmd.arg("--version").arg(tool.version().to_string());

    if let Some(index) = tool.index() {
        _ = cmd.arg("--index").arg(index);
    }

    if let Some(registry) = tool.registry() {
        _ = cmd.arg("--registry").arg(registry);
    }

    if let Some(git) = tool.git() {
        _ = cmd.arg("--git").arg(git);
    }

    if let Some(branch) = tool.branch() {
        _ = cmd.arg("--branch").arg(branch);
    }

    if let Some(tag) = tool.tag() {
        _ = cmd.arg("--tag").arg(tag);
    }

    if let Some(rev) = tool.rev() {
        _ = cmd.arg("--rev").arg(rev);
    }

    if let Some(path) = tool.path() {
        _ = cmd.arg("--path").arg(path);
    }

    if let Some(root) = tool.root() {
        _ = cmd.arg("--root").arg(root);
    }

    _ = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    outputter.message(format!("{} {}", tool_id, tool.version()));
    outputter.run_command(&cmd);

    match host.spawn(&mut cmd) {
        Ok(child) => match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    outputter.command_error("unable to install", Some(output.status), Some(&output), true);
                    Err(anyhow::anyhow!(format!(
                        "unable to install '{} {}': {}",
                        tool_id,
                        tool.version(),
                        output.status
                    )))
                }
            }

            Err(e) => {
                outputter.command_error(format!("unable to wait for 'cargo install': {e}"), None, None, true);
                Err(anyhow::anyhow!(format!("unable to wait for 'cargo install': {e}")))
            }
        },

        Err(e) => {
            outputter.command_error(format!("unable to start 'cargo install': {e}"), None, None, true);
            Err(anyhow::anyhow!(format!("unable to start 'cargo install': {e}")))
        }
    }
}
