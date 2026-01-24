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
        let old_server_type = app.connection.connected_server_type.clone();

        let Some(&ip) = args.first() else {
            app.log("Usage: connect <ip>");
            return CommandResult::Ok;
        };

        if app.connection.is_in_path(ip) {
            app.log(format!("Error: Already connected through {}", ip));
            return CommandResult::Ok;
        }

        let Some(server) = app.world.get(ip) else {
            app.log(format!("Error: Unknown IP {}", ip));
            return CommandResult::Ok;
        };

        let new_server_type = Some(server.server_type.clone());
        app.connection.connect_to(server);
        app.log(format!("Connected to {}", ip));

        if old_server_type != new_server_type {
            CommandResult::ConnectionChanged {
                old_server_type,
                new_server_type,
            }
        } else {
            CommandResult::Ok
        }
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        app.world
            .servers
            .keys()
            .filter(|ip: &&String| ip.starts_with(prefix))
            .cloned()
            .collect()
    }
}
