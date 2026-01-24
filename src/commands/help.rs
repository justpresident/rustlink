use super::{Command, CommandRegistry, CommandResult};
use crate::app::App;

pub struct HelpCommand;
pub struct HelpKeysCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn aliases(&self) -> &[&'static str] {
        &["?"]
    }

    fn description(&self) -> &'static str {
        "Show available commands"
    }

    fn execute(&self, app: &mut App, _args: &[&str], registry: &CommandRegistry) -> CommandResult {
        app.logs.push("Available commands:".into());

        // Dynamically generate help from all registered commands
        let mut max_usage_len = 0;
        for cmd in registry.all() {
            max_usage_len = max_usage_len.max(cmd.usage().len());
        }

        for cmd in registry.all() {
            let usage = cmd.usage();
            let padding = " ".repeat(max_usage_len - usage.len() + 2);
            let aliases = cmd.aliases();
            let alias_str = if aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", aliases.join(", "))
            };
            app.logs.push(format!(
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
        app.logs.push("".into());
        app.logs.push("Keyboard shortcuts:".into());
        app.logs.push("  Tab              - Autocomplete".into());
        app.logs.push("  Up/Down          - log history".into());
        app.logs.push("  PageUp/PageDown  - command history".into());
        app.logs
            .push("  Ctrl+A/E         - Start/End of line".into());
        app.logs.push("  Ctrl+U           - Clear line".into());
        app.logs.push("  Ctrl+W           - Delete word".into());
        app.logs.push("  Ctrl+Q           - Quit".into());
        CommandResult::Ok
    }
}
