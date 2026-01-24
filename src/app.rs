use crate::connection::Connection;
use crate::player::Player;
use crate::terminal::Terminal;
use crate::world::GameWorld;
use std::time::Instant;

pub struct App {
    pub world: GameWorld,
    pub connection: Connection,
    pub player: Player,
    pub terminal: Terminal,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub animation_tick: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            world: GameWorld::new(),
            connection: Connection::new(),
            player: Player::new(),
            terminal: Terminal::new(),
            last_tick: Instant::now(),
            should_quit: false,
            animation_tick: 0,
        }
    }

    pub fn on_tick(&mut self, tool_registry: &crate::tools::ToolRegistry) {
        // Handle Tool Progress
        let tool_info = self.connection.active_tool.as_ref().map(|active| {
            (
                active.tool_name.clone(),
                active.target_ip.clone(),
                active.progress,
            )
        });

        if let Some((tool_name, target_ip, progress)) = tool_info
            && let Some(tool) = tool_registry.find(&tool_name)
        {
            let new_progress = tool.on_tick(self, &target_ip, progress);
            if let Some(ref mut active) = self.connection.active_tool {
                active.progress = new_progress;
            }
            if new_progress >= 100.0 {
                tool.on_complete(self, &target_ip);
                self.connection.active_tool = None;
            }
        }

        // Handle Trace
        if self.connection.tick_trace(&self.world.servers) {
            self.terminal
                .log("!!! TERMINAL COMPROMISED - DISCONNECTING !!!");
            self.connection.reset();
            self.player.penalize(100);
        }

        self.animation_tick = self.animation_tick.wrapping_add(1);
    }

    // Convenience methods that delegate to sub-structs
    pub fn log<S: AsRef<str>>(&mut self, message: S) {
        self.terminal.log(message);
    }

    pub fn check_missions(&mut self, filename: &str) {
        let rewards = self.player.check_missions(filename);
        for reward in rewards {
            self.terminal.log(format!("MISSION COMPLETE: +{}c", reward));
        }
    }
}
