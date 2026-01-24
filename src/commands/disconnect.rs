use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;

pub struct DisconnectCommand;

impl Command for DisconnectCommand {
    fn name(&self) -> &'static str {
        "disconnect"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Disconnect from current server"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if app.target_ip.is_some() {
            app.logs.push("Disconnected.".into());
        }
        app.connection_path = vec!["127.0.0.1".into()];
        app.target_ip = None;
        app.is_tracing = false;
        app.trace_percentage = 0.0;
        CommandResult::Ok
    }
}
