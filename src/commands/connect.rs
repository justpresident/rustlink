use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;

pub struct ConnectCommand;

impl Command for ConnectCommand {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn aliases(&self) -> &[&'static str] {
        &["ssh", "cn"]
    }

    fn description(&self) -> &'static str {
        "Connect to a server by IP address"
    }

    fn usage(&self) -> &'static str {
        "connect <ip>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if let Some(&ip) = args.first() {
            if app.servers.contains_key(ip) {
                app.connection_path.push(ip.to_string());
                app.target_ip = Some(ip.to_string());
                app.is_tracing = true;
                app.logs.push(format!("Connected to {}", ip));
            } else {
                app.logs.push(format!("Error: Unknown IP {}", ip));
            }
        } else {
            app.logs.push("Usage: connect <ip>".into());
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
