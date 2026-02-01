use crate::model::{
    COOLERS, CPUS, ComponentInventory, File, HardwareComponent, HardwareKind, MOTHERBOARDS, Mail,
    Motherboard, NETWORKS, RAMS, STORAGES,
};

/// Player state for economy, inventory, and owned items
pub struct Player {
    pub credits: u32,
    pub local_files: Vec<File>,
    pub inbox: Vec<Mail>,
    pub inventory: ComponentInventory, // All owned components, tracks what's installed
    pub owned_tools: Vec<String>,      // Software tools player owns
}

impl Player {
    /// Get the installed motherboard if the PC is functional
    pub fn working_motherboard(&self) -> Option<&Motherboard> {
        if self.inventory.is_functional() {
            self.inventory.installed_motherboard()
        } else {
            None
        }
    }

    /// Get compute power (0 if PC is incomplete/invalid)
    pub fn compute_power(&self) -> u32 {
        self.inventory.compute_power()
    }

    /// Check if the PC is functional
    pub fn pc_functional(&self) -> bool {
        self.inventory.is_functional()
    }

    /// Toggle install state of a component by its absolute index in inventory
    pub fn toggle_install(&mut self, index: usize) -> Result<String, String> {
        let component = self.inventory.get(index).ok_or("Invalid index")?;
        let name = component.name().to_string();

        if self.inventory.is_installed(index) {
            self.inventory.uninstall(index);
            Ok(format!("Uninstalled {name}"))
        } else {
            self.inventory.install_item(index)?;
            Ok(format!("Installed {name}"))
        }
    }

    /// Install a component by its absolute index
    pub fn install_component(&mut self, index: usize) -> Result<Vec<String>, String> {
        let component = self.inventory.get(index).ok_or("Invalid index")?;
        let name = component.name().to_string();
        let kind = component.kind();

        let uninstalled = self.inventory.install_item(index)?;

        let mut warnings = Vec::new();
        if !uninstalled.is_empty() {
            for idx in uninstalled {
                if let Some(c) = self.inventory.get(idx) {
                    warnings.push(format!("Uninstalled {} (incompatible)", c.name()));
                }
            }
        }

        // Auto-populate if we just installed a motherboard
        if kind == HardwareKind::Motherboard {
            self.auto_populate_slots(&mut warnings);
        }

        warnings.insert(0, format!("Installed {name}"));
        Ok(warnings)
    }

    /// Uninstall a component by its absolute index
    pub fn uninstall_component(&mut self, index: usize) -> Result<String, String> {
        let component = self.inventory.get(index).ok_or("Invalid index")?;
        let name = component.name().to_string();

        if !self.inventory.is_installed(index) {
            return Err("Component is not installed".to_string());
        }

        self.inventory.uninstall(index);
        Ok(format!("Uninstalled {name}"))
    }

    /// Auto-populate empty PC slots with best compatible spare items from inventory
    fn auto_populate_slots(&mut self, warnings: &mut Vec<String>) {
        let Some(mb) = self.inventory.installed_motherboard() else {
            return;
        };
        let mb_socket = mb.socket();
        let mb_ram_type = mb.ram_type();

        // Try to fill CPU slot
        if self.inventory.count_installed(HardwareKind::Cpu) == 0
            && let Some(idx) = self.inventory.best_spare_cpu_index(mb_socket)
            && self.inventory.install_item(idx).is_ok()
            && let Some(c) = self.inventory.get(idx)
        {
            warnings.push(format!("Auto-installed CPU: {}", c.name()));
        }

        // Try to fill Cooler slot
        if self.inventory.count_installed(HardwareKind::Cooler) == 0
            && let Some(idx) = self.inventory.best_spare_cooler_index(mb_socket)
            && self.inventory.install_item(idx).is_ok()
            && let Some(c) = self.inventory.get(idx)
        {
            warnings.push(format!("Auto-installed Cooler: {}", c.name()));
        }

        // Try to fill RAM slots
        let mb = self.inventory.installed_motherboard();
        let ram_slots = mb.map_or(0, |m| m.slot_count(HardwareKind::Ram));
        while self.inventory.count_installed(HardwareKind::Ram) < ram_slots {
            if let Some(idx) = self.inventory.best_spare_ram_index(mb_ram_type) {
                if self.inventory.install_item(idx).is_ok() {
                    if let Some(c) = self.inventory.get(idx) {
                        warnings.push(format!("Auto-installed RAM: {}", c.name()));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Try to fill Storage slots
        let mb = self.inventory.installed_motherboard();
        let storage_slots = mb.map_or(0, |m| m.slot_count(HardwareKind::Storage));
        while self.inventory.count_installed(HardwareKind::Storage) < storage_slots {
            if let Some(idx) = self.inventory.best_spare_storage_index() {
                if self.inventory.install_item(idx).is_ok() {
                    if let Some(c) = self.inventory.get(idx) {
                        warnings.push(format!("Auto-installed Storage: {}", c.name()));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Try to fill Network slots
        let mb = self.inventory.installed_motherboard();
        let network_slots = mb.map_or(0, |m| m.slot_count(HardwareKind::Network));
        while self.inventory.count_installed(HardwareKind::Network) < network_slots {
            if let Some(idx) = self.inventory.best_spare_network_index() {
                if self.inventory.install_item(idx).is_ok() {
                    if let Some(c) = self.inventory.get(idx) {
                        warnings.push(format!("Auto-installed Network: {}", c.name()));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Check if player owns a specific motherboard
    pub fn owns_motherboard(&self, mb_id: &str) -> bool {
        self.inventory.has_motherboard(mb_id)
    }

    /// Count total owned components of a specific item
    pub fn count_owned_of(&self, item: &HardwareComponent) -> usize {
        self.inventory.count_items_for(item.kind())
    }

    /// Get the current motherboard (installed or first in inventory)
    pub fn get_motherboard(&self) -> Option<&Motherboard> {
        self.inventory.installed_motherboard().or_else(|| {
            self.inventory
                .items_for(HardwareKind::Motherboard)
                .next()
                .and_then(|(_, c)| match c {
                    HardwareComponent::Motherboard(m) => Some(m),
                    _ => None,
                })
        })
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Player {
    pub fn new(rich_player: bool) -> Self {
        let mut inventory = ComponentInventory::default();

        // Add starter components to inventory
        inventory.add(HardwareComponent::Motherboard(MOTHERBOARDS[0].clone()));
        inventory.add(HardwareComponent::Cpu(CPUS[0].clone()));
        inventory.add(HardwareComponent::Cooler(COOLERS[0].clone()));
        inventory.add(HardwareComponent::Ram(RAMS[0].clone()));
        inventory.add(HardwareComponent::Storage(STORAGES[0].clone()));
        inventory.add(HardwareComponent::Network(NETWORKS[0].clone()));

        for i in 0..inventory.total_count() {
            let _ = inventory.install_item(i);
        }

        Self {
            credits: if rich_player { 500_000 } else { 500 },
            local_files: Vec::new(),
            inbox: vec![
                Mail {
                    id: 1,
                    sender: "admin@uplink.net".into(),
                    subject: "Welcome to Uplink!".into(),
                    body: "Welcome, Agent. Your journey into the digital underworld begins now.\n\n\
                           Your gateway PC has basic specs - check the shop to upgrade when you have credits.\n\n\
                           Use 'shop' to open the PC shop. Press Tab to switch between Available and Owned items.\n\n\
                           Good luck.".into(),
                    is_read: false,
                    mission_id: None,
                },
                Mail {
                    id: 2,
                    sender: "anonymous@darknet.org".into(),
                    subject: "[JOB] Corporate Data Retrieval".into(),
                    body: "We need someone to retrieve 'research.doc' from Central Data (172.16.0.4).\n\n\
                           Payment: 2000 credits upon delivery.\n\n\
                           Reply with 'mail accept 2' to take this job.".into(),
                    is_read: false,
                    mission_id: Some(1),
                },
            ],
            inventory,
            owned_tools: vec!["PasswordBreaker".into(), "FirewallBuster".into()],
        }
    }

    pub fn add_file(&mut self, file: File) {
        self.local_files.push(file);
    }

    pub fn add_mail(&mut self, mail: Mail) {
        self.inbox.push(mail);
    }

    pub const fn penalize(&mut self, amount: u32) {
        self.credits = self.credits.saturating_sub(amount);
    }

    pub const fn award(&mut self, amount: u32) {
        self.credits += amount;
    }

    /// Check if player owns a specific tool
    pub fn owns_tool(&self, tool_name: &str) -> bool {
        self.owned_tools.iter().any(|t| t == tool_name)
    }

    /// Check if player's PC meets minimum requirements
    pub fn can_use_tool(&self, min_compute: u32, min_memory: u32) -> bool {
        self.inventory.is_functional()
            && self.inventory.compute_power() >= min_compute
            && self.inventory.total_ram_mb() >= min_memory
    }

    pub fn unread_mail_count(&self) -> usize {
        self.inbox.iter().filter(|m| !m.is_read).count()
    }
}
