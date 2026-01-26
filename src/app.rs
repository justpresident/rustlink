use crate::connection::Connection;
use crate::missions::{ActionContext, ActionType, MissionEffect, MissionSystem};
use crate::player::Player;
use crate::terminal::Terminal;
use crate::world::GameWorld;
use std::time::Instant;

/// UI mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UIMode {
    #[default]
    Normal,
    Shop,
}

/// Which tab is active in the shop view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopTab {
    #[default]
    Available,
    Owned,
}

/// A popup dialog for confirmations
#[derive(Debug, Clone)]
pub struct Dialog {
    pub title: String,
    pub messages: Vec<String>,
    pub confirm_text: String,
    pub cancel_text: String,
    pub on_confirm: DialogAction,
}

/// What to do when a dialog is confirmed
#[derive(Debug, Clone)]
pub enum DialogAction {
    /// Purchase item at index in current shop category
    PurchaseItem(usize),
    /// Install component from inventory (category as slot, component id)
    InstallComponent(ShopCategory, String),
    /// Just close the dialog
    None,
}

impl Dialog {
    pub fn purchase_confirmation(item_name: &str, price: u32, warnings: Vec<String>) -> Self {
        let mut messages = vec![format!("Purchase {} for {}c?", item_name, price)];
        if !warnings.is_empty() {
            messages.push(String::new());
            messages.push("⚠ Compatibility warnings:".into());
            for w in warnings {
                messages.push(format!("  • {w}"));
            }
        }
        Self {
            title: "Confirm Purchase".into(),
            messages,
            confirm_text: "Buy".into(),
            cancel_text: "Cancel".into(),
            on_confirm: DialogAction::None, // Will be set by caller
        }
    }

    pub fn install_confirmation(
        component_name: &str,
        removed: &[String],
        warnings: &[String],
        pc_functional: bool,
    ) -> Self {
        let mut messages = vec![format!("Install {}?", component_name)];
        if !removed.is_empty() {
            messages.push(String::new());
            messages.push("⚠ The following will be removed:".into());
            for r in removed {
                messages.push(format!("  • {r}"));
            }
        }
        if !warnings.is_empty() {
            messages.push(String::new());
            for w in warnings {
                messages.push(format!("  ⚠ {w}"));
            }
        }
        if !pc_functional {
            messages.push(String::new());
            messages.push("❌ PC will NOT be functional after this!".into());
        }
        Self {
            title: "Confirm Installation".into(),
            messages,
            confirm_text: "Install".into(),
            cancel_text: "Cancel".into(),
            on_confirm: DialogAction::None,
        }
    }
}

/// Shop category being viewed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopCategory {
    #[default]
    Cpu,
    Cooler,
    Motherboard,
    Ram,
    Storage,
    Network,
}

impl ShopCategory {
    pub const fn all() -> &'static [Self] {
        &[
            Self::Cpu,
            Self::Cooler,
            Self::Motherboard,
            Self::Ram,
            Self::Storage,
            Self::Network,
        ]
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Cooler => "Cooler",
            Self::Motherboard => "Motherboard",
            Self::Ram => "RAM",
            Self::Storage => "Storage",
            Self::Network => "Network",
        }
    }
}

pub struct App {
    pub world: GameWorld,
    pub connection: Connection,
    pub player: Player,
    pub terminal: Terminal,
    pub missions: MissionSystem,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub animation_tick: u64,
    pub ui_mode: UIMode,
    // Shop state (merged with assembly)
    pub shop_category: ShopCategory,
    pub shop_selection: usize,
    pub shop_tab: ShopTab, // Available vs Owned items
    // Dialog state
    pub dialog: Option<Dialog>,
    pub dialog_confirmed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new(false)
    }
}

impl App {
    pub fn new(rich_player: bool) -> Self {
        Self {
            world: GameWorld::new(),
            connection: Connection::new(),
            player: Player::new(rich_player),
            terminal: Terminal::new(),
            missions: MissionSystem::new(),
            last_tick: Instant::now(),
            should_quit: false,
            animation_tick: 0,
            ui_mode: UIMode::Normal,
            shop_category: ShopCategory::Cpu,
            shop_selection: 0,
            shop_tab: ShopTab::Available,
            dialog: None,
            dialog_confirmed: false,
        }
    }

    /// Check if the shop can be opened (disconnected and not being traced)
    pub fn can_open_shop(&self) -> bool {
        self.connection.target_ip.is_none() && self.connection.trace_percentage < 1.0
    }

    /// Open the shop UI
    pub fn open_shop(&mut self) {
        if self.can_open_shop() {
            self.ui_mode = UIMode::Shop;
            self.shop_category = ShopCategory::Cpu;
            self.shop_selection = 0;
            self.shop_tab = ShopTab::Available;
        }
    }

    /// Try to close the shop UI - only works if PC is functional
    pub fn try_close_shop(&mut self) -> bool {
        if self.player.pc_functional() {
            self.ui_mode = UIMode::Normal;
            self.dialog = None;
            true
        } else {
            false
        }
    }

    /// Toggle between Available and Owned tabs in shop
    pub const fn toggle_shop_tab(&mut self) {
        self.shop_tab = match self.shop_tab {
            ShopTab::Available => ShopTab::Owned,
            ShopTab::Owned => ShopTab::Available,
        };
        self.shop_selection = 0;
    }

    /// Show a dialog
    pub fn show_dialog(&mut self, dialog: Dialog) {
        self.dialog = Some(dialog);
        self.dialog_confirmed = false;
    }

    /// Close the current dialog
    pub fn close_dialog(&mut self) {
        self.dialog = None;
        self.dialog_confirmed = false;
    }

    /// Confirm the current dialog and return the action
    pub fn confirm_dialog(&mut self) -> Option<DialogAction> {
        if let Some(dialog) = self.dialog.take() {
            self.dialog_confirmed = true;
            Some(dialog.on_confirm)
        } else {
            None
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

                // Process tool completion action
                let ctx = ActionContext::new(ActionType::ToolComplete)
                    .with_server(&target_ip)
                    .with_tool(&tool_name);
                self.process_action(&ctx);
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

    /// Process a game action through the mission system
    pub fn process_action(&mut self, ctx: &ActionContext) {
        let effects = self.missions.process_action(ctx);
        self.apply_effects(effects);
    }

    /// Apply mission effects to the game state
    fn apply_effects(&mut self, effects: Vec<MissionEffect>) {
        for effect in effects {
            match effect {
                MissionEffect::AwardCredits(amount) => {
                    self.player.award(amount);
                    self.terminal.log(format!("Received {amount}c"));
                }
                MissionEffect::PenalizeCredits(amount) => {
                    self.player.penalize(amount);
                    self.terminal.log(format!("Lost {amount}c"));
                }
                MissionEffect::SendMail(mail) => {
                    self.player.add_mail(mail);
                    self.terminal.log("New mail received!");
                }
                MissionEffect::UnlockMission(_mission_id) => {
                    // Mission is now available in templates
                }
                MissionEffect::LogMessage(msg) => {
                    self.terminal.log(msg);
                }
                MissionEffect::SwitchUI(_ui_name) => {
                    // Can be extended for custom UI screens
                }
                MissionEffect::SetMissionVar { .. } | MissionEffect::Custom { .. } => {
                    // For complex mission state tracking / extensibility
                }
            }
        }
    }

    /// Accept a mission by mail ID
    pub fn accept_mission_from_mail(&mut self, mail_id: u32) -> Result<(), String> {
        // Find the mail
        let mail = self.player.inbox.iter().find(|m| m.id == mail_id);
        let Some(mail) = mail else {
            return Err(format!("Mail {mail_id} not found"));
        };

        let Some(mission_id) = mail.mission_id else {
            return Err("This mail does not contain a mission offer".into());
        };

        // Accept the mission
        let effects = self.missions.accept_mission(mission_id)?;
        self.apply_effects(effects);

        // Log mission details
        if let Some(mission) = self.missions.current_mission() {
            self.terminal
                .log(format!("Mission accepted: {}", mission.title));
            for obj in &mission.objectives {
                self.terminal.log(format!("  - {}", obj.description));
            }
            self.terminal.log(format!("Reward: {}c", mission.reward));
        }

        Ok(())
    }

    // Convenience methods that delegate to sub-structs
    pub fn log<S: AsRef<str>>(&mut self, message: S) {
        self.terminal.log(message);
    }
}
