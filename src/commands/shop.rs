use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct ShopCommand;

impl Command for ShopCommand {
    fn name(&self) -> &'static str {
        "shop"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Open the PC hardware/software shop"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if !app.can_open_shop() {
            if app.connection.target_ip.is_some() {
                app.log("Cannot open shop while connected to a server. Disconnect first.");
            } else {
                app.log("Cannot open shop while being traced.");
            }
            return CommandResult::Ok;
        }

        app.open_shop();
        app.log("Opening shop... (Press Esc to close)");
        CommandResult::Ok
    }
}
