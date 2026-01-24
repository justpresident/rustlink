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
        if app.connection.target_ip.is_some() {
            app.log("Disconnected.");
        }
        app.connection.reset();
        CommandResult::Ok
    }
}
