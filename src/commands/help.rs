use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;

pub struct HelpCommand;
pub struct HelpKeysCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Show available commands"
    }

    fn execute(&self, app: &mut App, _args: &[&str], registry: &CommandRegistry) -> CommandResult {
        app.log("Available commands:");

        // Dynamically generate help from all registered commands
        let mut max_usage_len = 0;
        for cmd in registry.all_active() {
            max_usage_len = max_usage_len.max(cmd.usage().len());
        }

        for cmd in registry.all_active() {
            let usage = cmd.usage();
            let padding = " ".repeat(max_usage_len - usage.len() + 2);
            let aliases = cmd.aliases();
            let alias_str = if aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", aliases.join(", "))
            };
            app.log(format!(
                "  {}{}- {}{}",
                usage,
                padding,
                cmd.description(),
                alias_str
            ));
        }

        CommandResult::Ok
    }
}

impl Command for HelpKeysCommand {
    fn name(&self) -> &'static str {
        "keys"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Show available shortcuts"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        app.log("");
        app.log("Keyboard shortcuts:");
        app.log("  Tab              - Autocomplete");
        app.log("  Up/Down          - Scroll Up/Down");
        app.log("  PageUp/PageDown  - Command history");
        app.log("  Ctrl+A/E         - Start/End of line");
        app.log("  Ctrl+U           - Clear line");
        app.log("  Ctrl+W           - Delete word");
        app.log("  Ctrl+Q           - Quit");
        CommandResult::Ok
    }
}
