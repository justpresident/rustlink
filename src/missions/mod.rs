//! Generic mission system with stateful missions and pluggable objectives
//!
//! This system is separate from Player and can trigger any game effects:
//! - Send/receive emails
//! - Award credits
//! - Unlock new missions
//! - Switch UI screens
//! - Modify game state

use std::collections::HashMap;

use crate::model::Mail;

// ============================================================================
// Game Actions - All possible player actions that can trigger mission updates
// ============================================================================

/// Types of actions the player can perform
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    /// Downloaded a file from a server
    FileDownload,
    /// Deleted a file from a server
    FileDelete,
    /// Uploaded a file to a server
    FileUpload,
    /// Transferred money between accounts
    BankTransfer,
    /// Connected to a server
    ServerConnect,
    /// Disconnected from a server
    ServerDisconnect,
    /// Ran a tool (e.g., `PasswordBreaker`)
    ToolRun,
    /// Tool completed successfully
    ToolComplete,
    /// Deleted a log entry
    LogDelete,
    /// Read a mail
    MailRead,
    /// Accepted a mission from mail
    MissionAccept,
    /// Custom action for extensibility
    Custom(String),
}

/// Context for an action - contains all relevant information
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub action_type: ActionType,
    /// Target server IP (if applicable)
    pub server_ip: Option<String>,
    /// Filename involved (if applicable)
    pub filename: Option<String>,
    /// Amount involved (for bank transfers, etc.)
    pub amount: Option<i64>,
    /// Source account (for transfers)
    pub source_account: Option<String>,
    /// Target account (for transfers)
    pub target_account: Option<String>,
    /// Tool name (if applicable)
    pub tool_name: Option<String>,
    /// Mail ID (if applicable)
    pub mail_id: Option<u32>,
    /// Mission ID (if applicable)
    pub mission_id: Option<u32>,
    /// Custom data for extensibility
    pub custom_data: HashMap<String, String>,
}

impl ActionContext {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            action_type,
            server_ip: None,
            filename: None,
            amount: None,
            source_account: None,
            target_account: None,
            tool_name: None,
            mail_id: None,
            mission_id: None,
            custom_data: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_server(mut self, ip: impl Into<String>) -> Self {
        self.server_ip = Some(ip.into());
        self
    }

    #[must_use]
    pub fn with_file(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    #[must_use]
    pub const fn with_amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    #[must_use]
    pub fn with_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    #[must_use]
    pub const fn with_mail(mut self, mail_id: u32) -> Self {
        self.mail_id = Some(mail_id);
        self
    }
}

// ============================================================================
// Mission Objectives - What needs to happen to progress/complete a mission
// ============================================================================

/// A condition that must be met
#[derive(Debug, Clone)]
pub enum ObjectiveCondition {
    /// Download specific file from specific server
    DownloadFile { server_ip: String, filename: String },
    /// Delete specific file from specific server
    DeleteFile { server_ip: String, filename: String },
    /// Transfer money from one account to another
    TransferMoney {
        target_account: String,
        min_amount: i64,
    },
    /// Connect to a specific server
    ConnectToServer { server_ip: String },
    /// Run a specific tool on a server
    RunTool {
        server_ip: String,
        tool_name: String,
    },
    /// Delete all logs on a server
    ClearLogs { server_ip: String },
    /// Custom condition with a matcher function name
    Custom { condition_id: String },
}

impl ObjectiveCondition {
    /// Check if an action satisfies this condition
    pub fn matches(&self, ctx: &ActionContext) -> bool {
        match self {
            Self::DownloadFile {
                server_ip,
                filename,
            } => {
                ctx.action_type == ActionType::FileDownload
                    && ctx.server_ip.as_ref() == Some(server_ip)
                    && ctx.filename.as_ref() == Some(filename)
            }
            Self::DeleteFile {
                server_ip,
                filename,
            } => {
                ctx.action_type == ActionType::FileDelete
                    && ctx.server_ip.as_ref() == Some(server_ip)
                    && ctx.filename.as_ref() == Some(filename)
            }
            Self::TransferMoney {
                target_account,
                min_amount,
            } => {
                ctx.action_type == ActionType::BankTransfer
                    && ctx.target_account.as_ref() == Some(target_account)
                    && ctx.amount.unwrap_or(0) >= *min_amount
            }
            Self::ConnectToServer { server_ip } => {
                ctx.action_type == ActionType::ServerConnect
                    && ctx.server_ip.as_ref() == Some(server_ip)
            }
            Self::RunTool {
                server_ip,
                tool_name,
            } => {
                ctx.action_type == ActionType::ToolComplete
                    && ctx.server_ip.as_ref() == Some(server_ip)
                    && ctx.tool_name.as_ref() == Some(tool_name)
            }
            Self::ClearLogs { server_ip } => {
                ctx.action_type == ActionType::LogDelete
                    && ctx.server_ip.as_ref() == Some(server_ip)
            }
            Self::Custom { .. } => false, // Custom conditions need external evaluation
        }
    }
}

// ============================================================================
// Mission Effects - What happens when a mission progresses/completes
// ============================================================================

/// Effects that occur when a mission state changes
#[derive(Debug, Clone)]
pub enum MissionEffect {
    /// Award credits to player
    AwardCredits(u32),
    /// Penalize player credits
    PenalizeCredits(u32),
    /// Send a mail to player
    SendMail(Mail),
    /// Unlock a new mission (make it available)
    UnlockMission(u32),
    /// Log a message to terminal
    LogMessage(String),
    /// Switch to a different UI mode
    SwitchUI(String),
    /// Modify mission state (set a variable)
    SetMissionVar {
        mission_id: u32,
        key: String,
        value: String,
    },
    /// Custom effect for extensibility
    Custom {
        effect_id: String,
        data: HashMap<String, String>,
    },
}

// ============================================================================
// Mission State
// ============================================================================

/// State of a mission
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionStatus {
    /// Mission is available but not started
    Available,
    /// Mission is accepted and in progress
    Active,
    /// Mission is completed successfully
    Completed,
    /// Mission failed
    Failed,
    /// Mission expired
    Expired,
}

/// A single objective within a mission
#[derive(Debug, Clone)]
pub struct MissionObjective {
    pub id: u32,
    pub description: String,
    pub condition: ObjectiveCondition,
    pub is_complete: bool,
    pub is_optional: bool,
}

/// A mission with its full state
#[derive(Debug, Clone)]
pub struct Mission {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub client: String,
    pub status: MissionStatus,
    pub objectives: Vec<MissionObjective>,
    pub reward: u32,
    /// Effects triggered on mission completion
    pub on_complete: Vec<MissionEffect>,
    /// Effects triggered on mission failure
    pub on_fail: Vec<MissionEffect>,
    /// Effects triggered when mission is accepted
    pub on_accept: Vec<MissionEffect>,
    /// Custom state variables for complex missions
    pub state: HashMap<String, String>,
    /// Mail ID that originated this mission (if any)
    pub source_mail_id: Option<u32>,
}

impl Mission {
    pub fn new(id: u32, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            client: "Anonymous".into(),
            status: MissionStatus::Available,
            objectives: Vec::new(),
            reward: 0,
            on_complete: Vec::new(),
            on_fail: Vec::new(),
            on_accept: Vec::new(),
            state: HashMap::new(),
            source_mail_id: None,
        }
    }

    #[must_use]
    pub fn with_client(mut self, client: impl Into<String>) -> Self {
        self.client = client.into();
        self
    }

    #[must_use]
    pub fn with_reward(mut self, reward: u32) -> Self {
        self.reward = reward;
        self.on_complete.push(MissionEffect::AwardCredits(reward));
        self
    }

    #[must_use]
    pub fn with_objective(mut self, objective: MissionObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    #[must_use]
    pub fn with_on_complete(mut self, effect: MissionEffect) -> Self {
        self.on_complete.push(effect);
        self
    }

    #[must_use]
    pub const fn with_source_mail(mut self, mail_id: u32) -> Self {
        self.source_mail_id = Some(mail_id);
        self
    }

    /// Check if all required objectives are complete
    pub fn is_complete(&self) -> bool {
        self.objectives
            .iter()
            .filter(|o| !o.is_optional)
            .all(|o| o.is_complete)
    }

    /// Process an action and return any triggered effects
    pub fn process_action(&mut self, ctx: &ActionContext) -> Vec<MissionEffect> {
        if self.status != MissionStatus::Active {
            return Vec::new();
        }

        let mut effects = Vec::new();

        // Check each objective
        for objective in &mut self.objectives {
            if !objective.is_complete && objective.condition.matches(ctx) {
                objective.is_complete = true;
                effects.push(MissionEffect::LogMessage(format!(
                    "Objective complete: {}",
                    objective.description
                )));
            }
        }

        // Check if mission is now complete
        if self.is_complete() {
            self.status = MissionStatus::Completed;
            effects.extend(self.on_complete.clone());
        }

        effects
    }
}

// ============================================================================
// Mission System - Manages all missions
// ============================================================================

/// The mission system - separate from Player
pub struct MissionSystem {
    /// All mission templates (available missions)
    templates: HashMap<u32, Mission>,
    /// Active missions (player has accepted)
    pub active_missions: HashMap<u32, Mission>,
    /// Completed mission IDs
    pub completed_missions: Vec<u32>,
    /// Currently selected active mission
    pub current_mission_id: Option<u32>,
    /// Next mail ID for generated mails
    next_mail_id: u32,
}

impl MissionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            templates: HashMap::new(),
            active_missions: HashMap::new(),
            completed_missions: Vec::new(),
            current_mission_id: None,
            next_mail_id: 3,
        };
        system.register_default_missions();
        system
    }

    fn register_default_missions(&mut self) {
        // Mission 1: Steal research.doc
        let mission1 = Mission::new(
            1,
            "Corporate Data Retrieval",
            "Retrieve 'research.doc' from Central Data servers",
        )
        .with_client("anonymous@darknet.org")
        .with_reward(2000)
        .with_objective(MissionObjective {
            id: 1,
            description: "Download 'research.doc' from 172.16.0.4".into(),
            condition: ObjectiveCondition::DownloadFile {
                server_ip: "172.16.0.4".into(),
                filename: "research.doc".into(),
            },
            is_complete: false,
            is_optional: false,
        })
        .with_on_complete(MissionEffect::UnlockMission(2))
        .with_on_complete(MissionEffect::SendMail(Mail {
            id: 0, // Will be assigned
            sender: "anonymous@darknet.org".into(),
            subject: "[JOB] Bank Records Extraction".into(),
            body: "Good work on the Central Data job.\n\n\
                   We have a more lucrative opportunity: retrieve 'accounts.dat' from \
                   Global Trust Bank (212.43.10.5).\n\n\
                   Payment: 5000 credits.\n\n\
                   Use 'mail accept' to take this job."
                .into(),
            is_read: false,
            mission_id: Some(2),
        }));

        // Mission 2: Bank records
        let mission2 = Mission::new(
            2,
            "Bank Records Extraction",
            "Download sensitive account data from Global Trust Bank",
        )
        .with_client("anonymous@darknet.org")
        .with_reward(5000)
        .with_objective(MissionObjective {
            id: 1,
            description: "Download 'accounts.dat' from 212.43.10.5".into(),
            condition: ObjectiveCondition::DownloadFile {
                server_ip: "212.43.10.5".into(),
                filename: "accounts.dat".into(),
            },
            is_complete: false,
            is_optional: false,
        });

        self.templates.insert(1, mission1);
        self.templates.insert(2, mission2);
    }

    /// Register a new mission template
    pub fn register_mission(&mut self, mission: Mission) {
        self.templates.insert(mission.id, mission);
    }

    /// Get available missions (not yet accepted)
    pub fn available_missions(&self) -> Vec<&Mission> {
        self.templates
            .values()
            .filter(|m| {
                m.status == MissionStatus::Available
                    && !self.active_missions.contains_key(&m.id)
                    && !self.completed_missions.contains(&m.id)
            })
            .collect()
    }

    /// Accept a mission
    pub fn accept_mission(&mut self, mission_id: u32) -> Result<Vec<MissionEffect>, String> {
        // Check if already have an active mission
        if self.current_mission_id.is_some() {
            return Err("You already have an active mission. Complete it first.".into());
        }

        let template = self
            .templates
            .get(&mission_id)
            .ok_or_else(|| format!("Mission {mission_id} not found"))?;

        if self.active_missions.contains_key(&mission_id) {
            return Err("Mission already accepted".into());
        }

        if self.completed_missions.contains(&mission_id) {
            return Err("Mission already completed".into());
        }

        let mut mission = template.clone();
        mission.status = MissionStatus::Active;

        let effects = mission.on_accept.clone();
        self.active_missions.insert(mission_id, mission);
        self.current_mission_id = Some(mission_id);

        Ok(effects)
    }

    /// Process an action through all active missions
    pub fn process_action(&mut self, ctx: &ActionContext) -> Vec<MissionEffect> {
        let mut all_effects = Vec::new();
        let mut completed_ids = Vec::new();

        for (id, mission) in &mut self.active_missions {
            let effects = mission.process_action(ctx);
            if mission.status == MissionStatus::Completed {
                completed_ids.push(*id);
            }
            all_effects.extend(effects);
        }

        // Move completed missions
        for id in completed_ids {
            self.active_missions.remove(&id);
            self.completed_missions.push(id);
            if self.current_mission_id == Some(id) {
                self.current_mission_id = None;
            }
        }

        // Process mail effects - assign IDs
        for effect in &mut all_effects {
            if let MissionEffect::SendMail(mail) = effect {
                mail.id = self.next_mail_id;
                self.next_mail_id += 1;
            }
        }

        all_effects
    }

    /// Get the current active mission
    pub fn current_mission(&self) -> Option<&Mission> {
        self.current_mission_id
            .and_then(|id| self.active_missions.get(&id))
    }

    /// Get all active missions
    pub fn all_active(&self) -> Vec<&Mission> {
        self.active_missions.values().collect()
    }

    /// Check if a mission ID corresponds to a mail
    pub fn get_mission_for_mail(&self, mail_mission_id: u32) -> Option<&Mission> {
        self.templates.get(&mail_mission_id)
    }
}

impl Default for MissionSystem {
    fn default() -> Self {
        Self::new()
    }
}
