pub mod firewall;
pub mod hardware;

pub use hardware::{
    COOLERS, CPUS, ComponentInventory, ComponentSlotType, Cooler, CoolerSlot, CoolerType, Cpu,
    CpuSlot, CpuSocket, DisplayGroup, HardwareComponent, HardwareKind, HardwareMaximums,
    MOTHERBOARDS, Motherboard, MotherboardTier, NETWORKS, NetworkCard, NetworkSlot, NetworkType,
    OwnedComponent, RAMS, Ram, RamSlot, RamType, STORAGES, Storage, StorageSlot, StorageSlotType,
    StorageType, format_bytes, format_speed,
};

use crate::model::firewall::Firewall;

// ============================================================================
// File System
// ============================================================================

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

// ============================================================================
// Mail System
// ============================================================================

#[derive(Debug, Clone)]
pub struct Mail {
    pub id: u32,
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub is_read: bool,
    pub mission_id: Option<u32>, // Associated mission ID if this is a mission mail
}

// ============================================================================
// Server System
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerType {
    Home,
    PublicDNS,
    Bank,
    Data,
}

impl ServerType {
    pub const fn associated_commands(&self) -> &'static [&'static str] {
        match self {
            Self::Bank => &["account_info", "transfer"],
            Self::Home | Self::PublicDNS | Self::Data => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account_number: String,
    pub balance: i32,
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
    pub accounts: Option<Vec<Account>>,
    pub logs: Vec<String>,
}

impl Server {
    pub const fn is_illegal(&self) -> bool {
        self.is_locked || self.firewall.is_some()
    }

    pub fn log<S: AsRef<str>>(&mut self, message: S) {
        self.logs.push(message.as_ref().to_string());
    }
}
