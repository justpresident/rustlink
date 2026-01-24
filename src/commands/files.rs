use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn aliases(&self) -> &[&'static str] {
        &["dir", "list"]
    }

    fn description(&self) -> &'static str {
        "List files on the current server"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if let Some(target) = &app.target_ip {
            let server = &app.servers[target];

            // Check firewall first
            if let Some(firewall) = &server.firewall {
                if firewall.is_active {
                    app.logs
                        .push("Access Denied: Firewall active. Run FirewallBuster first.".into());
                    return CommandResult::Ok;
                }
            }

            if server.is_locked {
                app.logs
                    .push("Access Denied: Server Locked. Run PasswordBreaker first.".into());
            } else if server.fs.files.is_empty() {
                app.logs.push("No files found.".into());
            } else {
                app.logs.push(format!("Files on {}:", server.name));
                for file in &server.fs.files {
                    app.logs
                        .push(format!("  {} ({} bytes)", file.name, file.size));
                }
            }
        } else {
            app.logs
                .push("Not connected to any server. Use 'connect <ip>' first.".into());
        }
        CommandResult::Ok
    }
}

pub struct ScpCommand;

impl Command for ScpCommand {
    fn name(&self) -> &'static str {
        "scp"
    }

    fn aliases(&self) -> &[&'static str] {
        &["download", "get", "cp"]
    }

    fn description(&self) -> &'static str {
        "Download a file from the current server"
    }

    fn usage(&self) -> &'static str {
        "scp <filename>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(&filename) = args.first() else {
            app.logs.push("Usage: scp <filename>".into());
            return CommandResult::Ok;
        };

        let Some(target) = app.target_ip.clone() else {
            app.logs.push("Not connected to any server.".into());
            return CommandResult::Ok;
        };

        let server = &app.servers[&target];

        // Check firewall
        if let Some(firewall) = &server.firewall {
            if firewall.is_active {
                app.logs.push("Access Denied: Firewall active.".into());
                return CommandResult::Ok;
            }
        }

        if server.is_locked {
            app.logs.push("Access Denied: Server Locked.".into());
            return CommandResult::Ok;
        }

        if let Some(file) = server.fs.files.iter().find(|f| f.name == filename) {
            app.local_files.push(file.clone());
            app.logs
                .push(format!("Downloaded '{}' ({} bytes)", filename, file.size));
            app.check_missions(filename);
        } else {
            app.logs.push(format!("File '{}' not found.", filename));
        }

        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        if let Some(target) = &app.target_ip {
            if let Some(server) = app.servers.get(target) {
                return server
                    .fs
                    .files
                    .iter()
                    .map(|f| &f.name)
                    .filter(|name| name.starts_with(prefix))
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }
}
