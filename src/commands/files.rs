use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn aliases(&self) -> &[&'static str] {
        &["dir"]
    }

    fn description(&self) -> &'static str {
        "List files on the current server"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if let Some(target) = &app.target_ip {
            let server = &app.servers[target];

            // Check firewall first
            if let Some(firewall) = &server.firewall
                && firewall.is_active
            {
                app.log("Access Denied: Firewall active. Run FirewallBuster first.");
                return CommandResult::Ok;
            }

            if server.is_locked {
                app.log("Access Denied: Server Locked. Run PasswordBreaker first.");
            } else if server.fs.files.is_empty() {
                app.log("No files found.");
            } else {
                // Collect file info first to avoid borrow issues
                let server_name = server.name.clone();
                let file_info: Vec<_> = server
                    .fs
                    .files
                    .iter()
                    .map(|f| (f.name.clone(), f.size))
                    .collect();

                app.log(format!("Files on {}:", server_name));
                for (name, size) in file_info {
                    app.log(format!("  {} ({} bytes)", name, size));
                }
            }
        } else {
            app.log("Not connected to any server. Use 'connect <ip>' first.");
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
        &["download"]
    }

    fn description(&self) -> &'static str {
        "Download a file from the current server"
    }

    fn usage(&self) -> &'static str {
        "scp <filename>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(&filename) = args.first() else {
            app.log("Usage: scp <filename>");
            return CommandResult::Ok;
        };

        let Some(target) = app.target_ip.clone() else {
            app.log("Not connected to any server.");
            return CommandResult::Ok;
        };

        let server = &app.servers[&target];

        // Check firewall
        if let Some(firewall) = &server.firewall
            && firewall.is_active
        {
            app.log("Access Denied: Firewall active.");
            return CommandResult::Ok;
        }

        if server.is_locked {
            app.log("Access Denied: Server Locked.");
            return CommandResult::Ok;
        }

        if let Some(file) = server.fs.files.iter().find(|f| f.name == filename) {
            app.local_files.push(file.clone());
            app.log(format!("Downloaded '{}' ({} bytes)", filename, file.size));
            app.check_missions(filename);
        } else {
            app.log(format!("File '{}' not found.", filename));
        }

        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        if let Some(target) = &app.target_ip
            && let Some(server) = app.servers.get(target)
            && !server.is_locked
        {
            return server
                .fs
                .files
                .iter()
                .map(|f| &f.name)
                .filter(|name| name.starts_with(prefix))
                .cloned()
                .collect();
        }
        Vec::new()
    }
}
