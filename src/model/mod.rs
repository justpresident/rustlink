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

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub coords: (f64, f64),
    pub fs: FileSystem,
    pub is_locked: bool,
    pub password: Option<String>,
    pub firewall: Option<Firewall>,
}
