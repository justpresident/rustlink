use super::{Command, CommandResult};
use crate::app::App;
use crate::commands::CommandRegistry;
use crate::tools::{ActiveTool, ToolRegistry};

pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn aliases(&self) -> &[&'static str] {
        &["exec"]
    }

    fn description(&self) -> &'static str {
        "Run a hacking tool on the current target"
    }

    fn usage(&self) -> &'static str {
        "run <tool>"
    }

    fn execute(&self, app: &mut App, args: &[&str], registry: &CommandRegistry) -> CommandResult {
        let tool_registry = &registry.tool_registry;

        let Some(&tool_name) = args.first() else {
            app.log("Usage: run <tool>");
            app.log("Available tools:");
            for tool in tool_registry.all() {
                app.log(format!("  {} - {}", tool.name(), tool.description()));
            }
            return CommandResult::Ok;
        };

        let Some(target) = app.connection.target_ip.clone() else {
            app.log("Not connected to any server.");
            return CommandResult::Ok;
        };

        // Find the tool in the registry
        let Some(tool) = tool_registry.find(tool_name) else {
            app.log(format!("Tool '{tool_name}' not found."));
            app.log("Available tools:");
            for t in tool_registry.all() {
                app.log(format!("  {} - {}", t.name(), t.description()));
            }
            return CommandResult::Ok;
        };

        // Check if player owns this tool
        if !app.player.owns_tool(tool.name()) {
            app.log(format!(
                "You don't own '{tool_name}'. Purchase it from the shop."
            ));
            return CommandResult::Ok;
        }

        // Check if PC is functional
        let Some(pc) = app.player.working_pc() else {
            app.log("Your PC is not functional! Use 'assemble' to complete your build.");
            return CommandResult::Ok;
        };

        // Check hardware requirements (using compute power and memory)
        let min_compute = tool.min_compute_power();
        let min_memory = tool.min_memory_mb();
        if !app.player.can_use_tool(min_compute, min_memory) {
            app.log(format!(
                "Insufficient hardware. {} requires: {} compute power, {} MB RAM",
                tool.name(),
                min_compute,
                min_memory
            ));
            app.log(format!(
                "Your PC: {} compute power, {} MB RAM",
                pc.compute_power(),
                pc.motherboard.total_ram_mb()
            ));
            app.log("Upgrade your PC at the shop.");
            return CommandResult::Ok;
        }

        // Check if already running a tool
        if app.connection.active_tool.is_some() {
            app.log("A tool is already running. Wait for it to complete.");
            return CommandResult::Ok;
        }

        // Check if tool can run on this target
        if let Err(msg) = tool.can_run(app, &target) {
            app.log(msg);
            return CommandResult::Ok;
        }

        // Start the tool
        tool.on_start(app, &target);
        app.connection.active_tool = Some(ActiveTool::new(tool_name, &target));

        CommandResult::Ok
    }

    fn completions(&self, _app: &App, _arg_index: usize, _prefix: &str) -> Vec<String> {
        Vec::new()
    }

    fn completions_with_tools(
        &self,
        _app: &App,
        _arg_index: usize,
        prefix: &str,
        tool_registry: &ToolRegistry,
    ) -> Vec<String> {
        tool_registry.completions(prefix)
    }
}
