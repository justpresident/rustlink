use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;

pub struct ConnectCommand;

impl Command for ConnectCommand {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn aliases(&self) -> &[&'static str] {
        &["ssh"]
    }

    fn description(&self) -> &'static str {
        "Connect to a server by IP address"
    }

    fn usage(&self) -> &'static str {
        "connect <ip>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if let Some(&ip) = args.first() {
            // Check if already in connection path
            if app.connection_path.contains(&ip.to_string()) {
                app.log(format!("Error: Already connected through {}", ip));
                return CommandResult::Ok;
            }

            if let Some(server) = app.servers.get(ip) {
                app.connection_path.push(ip.to_string());
                app.target_ip = Some(ip.to_string());
                // Only start trace for illegal servers (locked or with firewall)
                let is_illegal = server.is_locked || server.firewall.is_some();
                if is_illegal {
                    app.is_tracing = true;
                }
                app.log(format!("Connected to {}", ip));
            } else {
                app.log(format!("Error: Unknown IP {}", ip));
            }
        } else {
            app.log("Usage: connect <ip>");
        }
        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        app.servers
            .keys()
            .filter(|ip| ip.starts_with(prefix))
            .cloned()
            .collect()
    }
}
