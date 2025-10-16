use crate::host::Host;
use crate::log::Log;
use crate::ws_config::{Tool, ToolId, Tools};
use clap::Parser;
use console::style;
use core::time::Duration;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Parser, Debug)]
pub struct InstallArgs {
    /// Send log output to the specified file.
    #[arg(short = 'l', long, value_name = "FILE")]
    pub log_file: Option<PathBuf>,
}

pub fn install_tools<H: Host>(args: &InstallArgs, host: &mut H, workspace_dir: &Path, tools: &Tools) -> anyhow::Result<()> {
    let mut log = Log::new(workspace_dir, "install", Option::from(&args.log_file))?;

    for (tool_id, tool) in tools.iter() {
        install_tool(host, tool_id, tool, &mut log)?;
    }

    Ok(())
}

fn install_tool<H: Host>(host: &mut H, tool_id: &ToolId, tool: &Tool, log: &mut Log) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

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

    log.info(&format!("Executing '{cmd:?}'"));
    host.print(format!("{} {} ", tool_id, tool.version()));

    let mut child = host.spawn(&mut cmd)?;

    let spinner = ["|", "/", "-", "\\"];
    let mut i = 0;
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            if status.success() {
                host.println(format!("\r{} {}... Done", tool_id, tool.version()));
            } else {
                host.println(format!("\r{} {}... {}", tool_id, tool.version(), style("Failed").red()));
                host.println(String::from_utf8_lossy(&output.stdout).to_string());
                host.eprintln(String::from_utf8_lossy(&output.stderr).to_string());

                log.error(&format!("'cargo install' failed with status code {}", status.code().unwrap_or(-1)));

                return Err(anyhow::anyhow!("'cargo install' returned status {}", status.code().unwrap_or(0)));
            }
            break;
        }

        host.print(format!("\r{} {} {}", tool_id, tool.version(), spinner[i]));
        i = (i + 1) % spinner.len();
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
