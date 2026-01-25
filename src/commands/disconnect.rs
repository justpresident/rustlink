use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;
use crate::model::ServerType;

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
        let old_server_type = app.connection.connected_server_type.clone();
        let new_server_type = Some(ServerType::Home);

        if app.connection.target_ip.is_some() {
            app.log("Disconnected.");
        }
        app.connection.reset();

        if old_server_type == new_server_type {
            CommandResult::Ok
        } else {
            CommandResult::ConnectionChanged {
                old_server_type,
                new_server_type,
            }
        }
    }
}
