use crate::model::{
    COOLERS, CPUS, ComponentInventory, File, HardwareComponent, HardwareKind, MOTHERBOARDS, Mail,
    Motherboard, NETWORKS, PC, RAMS, STORAGES,
};

/// Player state for economy, inventory, and owned items
pub struct Player {
    pub credits: u32,
    pub local_files: Vec<File>,
    pub inbox: Vec<Mail>,
    pub pc: PC,                        // Currently assembled PC (motherboard with slots)
    pub inventory: ComponentInventory, // Owned components not yet installed
    pub owned_tools: Vec<String>,      // Software tools player owns
}

impl Player {
    /// Get the motherboard if the assembled PC is functional
    pub fn working_motherboard(&self) -> Option<&Motherboard> {
        self.pc.functional_motherboard()
    }

    /// Get compute power (0 if PC is incomplete/invalid)
    pub fn compute_power(&self) -> u32 {
        self.pc.compute_power()
    }

    /// Check if the PC is functional
    pub fn pc_functional(&self) -> bool {
        self.pc.is_functional()
    }

    /// Get warnings when installing a new motherboard over existing parts
    fn motherboard_install_warnings(&self, new_mb: &Motherboard) -> Vec<String> {
        let mut warnings = Vec::new();
        let new_socket = new_mb.socket();
        let new_ram_type = new_mb.ram_type();

        if let Some(cpu) = self.pc.cpu()
            && let Some(socket) = new_socket
            && cpu.socket != socket
        {
            warnings.push(format!(
                "Warning: Installed CPU {} is incompatible",
                cpu.name
            ));
        }
        if let Some(cooler) = self.pc.cooler()
            && let Some(socket) = new_socket
            && !cooler.is_compatible(socket)
        {
            warnings.push(format!(
                "Warning: Installed cooler {} is incompatible",
                cooler.name
            ));
        }
        for ram in self.pc.rams() {
            if let Some(ram_type) = new_ram_type
                && ram.ram_type != ram_type
            {
                warnings.push(format!(
                    "Warning: Installed RAM {} is incompatible",
                    ram.name
                ));
            }
        }
        warnings
    }

    /// Install a component from inventory by index
    /// Returns warnings (informational only - install always succeeds if Ok is returned)
    pub fn install_from_inventory(
        &mut self,
        kind: HardwareKind,
        index: usize,
    ) -> Result<Vec<String>, String> {
        // Get the component to check if it can be installed (without removing yet)
        let component = self
            .inventory
            .get_at(kind, index)
            .ok_or("Invalid inventory index")?;

        // Check if it can be installed
        self.pc.can_install(component)?;

        // Now we know it will succeed - remove from inventory and install
        let component = self
            .inventory
            .remove_at(kind, index)
            .expect("index was valid");

        // Get warnings for motherboard installs
        let mut warnings = if let HardwareComponent::Motherboard(ref new_mb) = component {
            self.motherboard_install_warnings(new_mb)
        } else {
            Vec::new()
        };

        // Install the component (old one goes to inventory if any)
        if let Some(old) = self.pc.install_component(component) {
            self.inventory.add(old);
        }

        // Auto-populate empty slots with compatible items
        self.auto_populate_slots(&mut warnings);

        Ok(warnings)
    }

    /// Auto-populate empty PC slots with best compatible items from inventory
    /// Motherboard is the base and determines compatibility - it's never auto-installed
    fn auto_populate_slots(&mut self, warnings: &mut Vec<String>) {
        // Get motherboard specs - return early if no motherboard
        let Some(mb) = &self.pc.motherboard else {
            return;
        };
        let (mb_socket, mb_ram_type) = (mb.socket(), mb.ram_type());

        // Try to fill CPU slot
        if self.mb_needs(HardwareKind::Cpu) {
            self.try_auto_install(warnings, HardwareKind::Cpu, |inv| {
                inv.best_cpu_index(mb_socket)
            });
        }

        // Try to fill Cooler slot
        if self.mb_needs(HardwareKind::Cooler) {
            self.try_auto_install(warnings, HardwareKind::Cooler, |inv| {
                inv.best_cooler_index(mb_socket)
            });
        }

        // Try to fill RAM slots
        while self.mb_has_empty(HardwareKind::Ram) {
            if !self.try_auto_install(warnings, HardwareKind::Ram, |inv| {
                inv.best_ram_index(mb_ram_type)
            }) {
                break;
            }
        }

        // Try to fill Storage slots
        while self.mb_has_empty(HardwareKind::Storage) {
            if !self.try_auto_install(warnings, HardwareKind::Storage, |inv| {
                inv.best_storage_index()
            }) {
                break;
            }
        }

        // Try to fill Network slots
        while self.mb_has_empty(HardwareKind::Network) {
            if !self.try_auto_install(warnings, HardwareKind::Network, |inv| {
                inv.best_network_index()
            }) {
                break;
            }
        }
    }

    /// Helper: check if motherboard needs a component (has none installed)
    fn mb_needs(&self, kind: HardwareKind) -> bool {
        self.pc
            .motherboard
            .as_ref()
            .is_some_and(|mb| mb.needs_component(kind))
    }

    /// Helper: check if motherboard has empty slots for a component kind
    fn mb_has_empty(&self, kind: HardwareKind) -> bool {
        self.pc
            .motherboard
            .as_ref()
            .is_some_and(|mb| mb.has_empty_slot(kind))
    }

    /// Try to auto-install a component. Returns true if installed, false if nothing to install.
    fn try_auto_install(
        &mut self,
        warnings: &mut Vec<String>,
        kind: HardwareKind,
        get_best_index: impl FnOnce(&ComponentInventory) -> Option<usize>,
    ) -> bool {
        let Some(index) = get_best_index(&self.inventory) else {
            return false;
        };

        // Get component and check if it can be installed
        let Some(component) = self.inventory.get_at(kind, index) else {
            return false;
        };

        if self.pc.can_install(component).is_err() {
            return false;
        }

        // Remove and install
        let component = self.inventory.remove_at(kind, index).expect("valid index");
        let name = component.name().to_string();
        self.pc.install_component(component);
        warnings.push(format!("Auto-installed {}: {name}", kind.name()));
        true
    }

    /// Remove a component from assembled PC slot at given index and put it in inventory
    pub fn uninstall_component_at(&mut self, slot_index: usize) -> Option<String> {
        let component = self.pc.uninstall_at(slot_index)?;
        let name = component.name().to_string();
        self.inventory.add(component);
        Some(name)
    }

    /// Uninstall first component of a given category and put it in inventory
    pub fn uninstall_component(&mut self, kind: HardwareKind) -> Option<String> {
        // For motherboard category, uninstall the entire motherboard
        if kind == HardwareKind::Motherboard {
            let old_mb = self.pc.motherboard.take()?;
            let name = old_mb.name.to_string();
            self.inventory.add(HardwareComponent::Motherboard(old_mb));
            return Some(name);
        }

        let mb = self.pc.motherboard.as_mut()?;

        // Find first filled slot of this category
        let slot_idx = mb
            .slots
            .iter()
            .position(|s| s.kind() == kind && s.is_filled())?;

        let component = mb.uninstall_at(slot_idx)?;
        let name = component.name().to_string();
        self.inventory.add(component);
        Some(name)
    }

    /// Check if player owns a specific motherboard (installed or in inventory)
    pub fn owns_motherboard(&self, mb_id: &str) -> bool {
        if let Some(mb) = &self.pc.motherboard
            && mb.id == mb_id
        {
            return true;
        }
        self.inventory.has_motherboard(mb_id)
    }

    /// Count total owned components of a specific item
    pub fn count_owned_of(&self, item: &HardwareComponent) -> usize {
        self.inventory.count_matching(item)
    }

    /// Get the current motherboard (installed or first in inventory)
    pub fn get_motherboard(&self) -> Option<&Motherboard> {
        self.pc
            .motherboard
            .as_ref()
            .or_else(|| self.inventory.first_motherboard())
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Player {
    pub fn new(rich_player: bool) -> Self {
        // Create starter motherboard with basic components installed
        let mut starter_mb = MOTHERBOARDS[0].clone();

        // Install starter components into the motherboard slots
        for slot in &mut starter_mb.slots {
            match slot {
                crate::model::ComponentSlotType::Cpu(cpu_slot) => {
                    cpu_slot.installed = Some(CPUS[0].clone());
                }
                crate::model::ComponentSlotType::Cooler(cooler_slot) => {
                    cooler_slot.installed = Some(COOLERS[0].clone());
                }
                crate::model::ComponentSlotType::Ram(ram_slot) => {
                    // Only install in first RAM slot
                    if ram_slot.installed.is_none() {
                        ram_slot.installed = Some(RAMS[0].clone());
                        break; // Only install one RAM module initially
                    }
                }
                _ => {}
            }
        }
        // Install storage and network
        for slot in &mut starter_mb.slots {
            if let crate::model::ComponentSlotType::Storage(storage_slot) = slot
                && storage_slot.installed.is_none()
            {
                storage_slot.installed = Some(STORAGES[0].clone());
                break;
            }
        }
        for slot in &mut starter_mb.slots {
            if let crate::model::ComponentSlotType::Network(network_slot) = slot
                && network_slot.installed.is_none()
            {
                network_slot.installed = Some(NETWORKS[0].clone());
                break;
            }
        }

        Self {
            credits: if rich_player {500_000} else {500},
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
            pc: PC::from_motherboard(starter_mb),
            inventory: ComponentInventory::default(),
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
        if let Some(mb) = self.working_motherboard() {
            mb.compute_power() >= min_compute && mb.total_ram_mb() >= min_memory
        } else {
            false // PC not functional
        }
    }

    pub fn unread_mail_count(&self) -> usize {
        self.inbox.iter().filter(|m| !m.is_read).count()
    }
}
