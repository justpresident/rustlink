use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn aliases(&self) -> &[&'static str] {
        &["cls"]
    }

    fn description(&self) -> &'static str {
        "Clear the log display"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        app.logs.clear();
        CommandResult::Ok
    }
}

pub struct ExitCommand;

impl Command for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn aliases(&self) -> &[&'static str] {
        &["quit", "q"]
    }

    fn description(&self) -> &'static str {
        "Exit the program"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        app.should_quit = true;
        CommandResult::Quit
    }
}
