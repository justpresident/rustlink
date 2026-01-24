pub mod firewall;

use crate::model::firewall::Firewall;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub size: u32,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct FileSystem {
    pub files: Vec<File>,
}

#[derive(Debug, Clone)]
pub struct Mission {
    pub id: u32,
    pub description: String,
    pub target_ip: String,
    pub target_file: String,
    pub reward: u32,
    pub is_complete: bool,
}

#[derive(Debug, Clone)]
pub struct Mail {
    pub id: u32,
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub is_read: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerType {
    Home,
    PublicDNS,
    Bank,
    Data,
    // Add more as needed
}

impl ServerType {
    pub fn associated_commands(&self) -> &'static [&'static str] {
        match self {
            ServerType::Home => &[],      // Example: Home might have basic file ops
            ServerType::PublicDNS => &[], // Public DNS might have no special commands
            ServerType::Bank => &["account_info", "transfer"], // Bank has file ops and bank commands
            ServerType::Data => &[],                           // Data servers have file ops
        }
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account_number: String,
    pub balance: i32, // Can be negative for debts
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub coords: (f64, f64),
    pub fs: FileSystem,
    pub is_locked: bool,
    pub password: Option<String>,
    pub firewall: Option<Firewall>,
    pub server_type: ServerType,
    pub accounts: Option<Vec<Account>>, // Only for Bank servers
}

impl Server {
    /// Returns true if connecting to this server triggers a trace
    pub fn is_illegal(&self) -> bool {
        self.is_locked || self.firewall.is_some()
    }
}
