use crate::{
    model::{
        AssembledPC, COOLERS, CPUS, ComponentInventory, ComponentSlotKind, File, MOTHERBOARDS,
        Mail, Motherboard, NETWORKS, OwnedComponent, PC, RAMS, STORAGES,
    },
    shop::ShopItem,
};

/// Player state for economy, inventory, and owned items
pub struct Player {
    pub credits: u32,
    pub local_files: Vec<File>,
    pub inbox: Vec<Mail>,
    pub assembled_pc: AssembledPC, // Currently assembled PC (motherboard with slots)
    pub inventory: ComponentInventory, // Owned components not yet installed
    pub owned_tools: Vec<String>,  // Software tools player owns
}

impl Player {
    /// Get a working PC if the assembled PC is functional
    pub fn working_pc(&self) -> Option<PC> {
        self.assembled_pc.to_complete_pc()
    }

    /// Get compute power (0 if PC is incomplete/invalid)
    pub fn compute_power(&self) -> u32 {
        self.assembled_pc.compute_power()
    }

    /// Check if the PC is functional
    pub fn pc_functional(&self) -> bool {
        self.assembled_pc.is_functional()
    }

    /// Check if a component can be installed (compatible with motherboard)
    fn check_install_compatibility(
        &self,
        slot: crate::model::ComponentSlot,
        id: &str,
    ) -> Result<(), String> {
        use crate::model::ComponentSlot;

        let Some(mb) = &self.assembled_pc.motherboard else {
            return Ok(()); // No motherboard = anything goes for motherboard install
        };

        let mb_socket = mb.socket();
        let mb_ram_type = mb.ram_type();

        match slot {
            ComponentSlot::Cpu => {
                if let Some(cpu) = self.inventory.cpus.iter().find(|c| c.id == id)
                    && let Some(socket) = mb_socket
                    && cpu.socket != socket
                {
                    return Err(format!(
                        "Cannot install: CPU needs {} socket, motherboard has {}",
                        cpu.socket, socket
                    ));
                }
            }
            ComponentSlot::Cooler => {
                if let Some(cooler) = self.inventory.coolers.iter().find(|c| c.id == id)
                    && let Some(socket) = mb_socket
                    && !cooler.is_compatible(socket)
                {
                    return Err(format!(
                        "Cannot install: Cooler doesn't support {socket} socket"
                    ));
                }
            }
            ComponentSlot::Ram => {
                if let Some(ram) = self.inventory.rams.iter().find(|r| r.id == id)
                    && let Some(ram_type) = mb_ram_type
                    && ram.ram_type != ram_type
                {
                    return Err(format!(
                        "Cannot install: RAM is {} but motherboard requires {}",
                        ram.ram_type, ram_type
                    ));
                }
            }
            _ => {} // Motherboard, Storage, Network are always allowed
        }
        Ok(())
    }

    /// Get warnings when installing a new motherboard over existing parts
    fn motherboard_install_warnings(&self, new_mb: &Motherboard) -> Vec<String> {
        let mut warnings = Vec::new();
        let new_socket = new_mb.socket();
        let new_ram_type = new_mb.ram_type();

        if let Some(cpu) = self.assembled_pc.cpu()
            && let Some(socket) = new_socket
            && cpu.socket != socket
        {
            warnings.push(format!(
                "Warning: Installed CPU {} is incompatible",
                cpu.name
            ));
        }
        if let Some(cooler) = self.assembled_pc.cooler()
            && let Some(socket) = new_socket
            && !cooler.is_compatible(socket)
        {
            warnings.push(format!(
                "Warning: Installed cooler {} is incompatible",
                cooler.name
            ));
        }
        for ram in self.assembled_pc.rams() {
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

    /// Install a component from inventory
    /// Rejects incompatible parts (motherboard determines compatibility)
    /// Auto-populates empty slots with compatible items after successful install
    pub fn install_from_inventory(
        &mut self,
        slot: crate::model::ComponentSlot,
        id: &str,
    ) -> Result<Vec<String>, String> {
        // Check compatibility first
        self.check_install_compatibility(slot, id)?;

        // Remove from inventory
        let component = self
            .inventory
            .remove_by_id(slot, id)
            .ok_or_else(|| format!("Component {id} not found in inventory"))?;

        // Get warnings for motherboard installs
        let mut warnings = if let OwnedComponent::Motherboard(ref new_mb) = component {
            self.motherboard_install_warnings(new_mb)
        } else {
            Vec::new()
        };

        // Install the new component (old one goes to inventory if exists)
        // We'll recover it into inventory if install fails
        let component_backup = component.clone();
        match self.assembled_pc.install_component(component) {
            Ok(Some(old)) => {
                self.inventory.add(old);
            }
            Ok(None) => {}
            Err(e) => {
                self.inventory.add(component_backup);
                return Err(e);
            }
        }

        // Auto-populate empty slots with compatible items
        self.auto_populate_slots(&mut warnings);

        Ok(warnings)
    }

    /// Auto-populate empty PC slots with best compatible items from inventory
    /// Motherboard is the base and determines compatibility - it's never auto-installed
    #[allow(clippy::too_many_lines)]
    fn auto_populate_slots(&mut self, warnings: &mut Vec<String>) {
        use crate::model::ComponentSlot;

        // Get motherboard specs - return early if no motherboard
        let (mb_socket, mb_ram_type) = {
            let Some(mb) = &self.assembled_pc.motherboard else {
                return;
            };
            (mb.socket(), mb.ram_type())
        };

        // Try to fill CPU slot (must match motherboard socket)
        let needs_cpu = self
            .assembled_pc
            .motherboard
            .as_ref()
            .is_some_and(|mb| mb.filled_count(ComponentSlotKind::Cpu) == 0);
        if needs_cpu {
            let best = self
                .inventory
                .best_cpu(mb_socket)
                .map(|c| (c.id.to_string(), c.name.to_string()));
            if let Some((id, name)) = best
                && let Some(c) = self.inventory.remove_by_id(ComponentSlot::Cpu, &id)
            {
                let _ = self.assembled_pc.install_component(c);
                warnings.push(format!("Auto-installed CPU: {name}"));
            }
        }

        // Try to fill Cooler slot (must support motherboard socket)
        let needs_cooler = self
            .assembled_pc
            .motherboard
            .as_ref()
            .is_some_and(|mb| mb.filled_count(ComponentSlotKind::Cooler) == 0);
        if needs_cooler {
            let best = self
                .inventory
                .best_cooler(mb_socket)
                .map(|c| (c.id.to_string(), c.name.to_string()));
            if let Some((id, name)) = best
                && let Some(c) = self.inventory.remove_by_id(ComponentSlot::Cooler, &id)
            {
                let _ = self.assembled_pc.install_component(c);
                warnings.push(format!("Auto-installed Cooler: {name}"));
            }
        }

        // Try to fill RAM slots (must match motherboard RAM type)
        loop {
            let has_empty_ram = self
                .assembled_pc
                .motherboard
                .as_ref()
                .is_some_and(|mb| mb.empty_count(ComponentSlotKind::Ram) > 0);
            if !has_empty_ram {
                break;
            }
            let best = self
                .inventory
                .best_ram(mb_ram_type)
                .map(|r| (r.id.to_string(), r.name.to_string()));
            if let Some((id, name)) = best {
                if let Some(r) = self.inventory.remove_by_id(ComponentSlot::Ram, &id) {
                    if self.assembled_pc.install_component(r).is_ok() {
                        warnings.push(format!("Auto-installed RAM: {name}"));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Try to fill Storage slots
        loop {
            let has_empty_storage = self
                .assembled_pc
                .motherboard
                .as_ref()
                .is_some_and(|mb| mb.empty_count(ComponentSlotKind::Storage) > 0);
            if !has_empty_storage {
                break;
            }
            let best = self
                .inventory
                .best_storage()
                .map(|s| (s.id.to_string(), s.name.to_string()));
            if let Some((id, name)) = best {
                if let Some(s) = self.inventory.remove_by_id(ComponentSlot::Storage, &id) {
                    if self.assembled_pc.install_component(s).is_ok() {
                        warnings.push(format!("Auto-installed Storage: {name}"));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Try to fill Network slots
        loop {
            let has_empty_network = self
                .assembled_pc
                .motherboard
                .as_ref()
                .is_some_and(|mb| mb.empty_count(ComponentSlotKind::Network) > 0);
            if !has_empty_network {
                break;
            }
            let best = self
                .inventory
                .best_network()
                .map(|n| (n.id.to_string(), n.name.to_string()));
            if let Some((id, name)) = best {
                if let Some(n) = self.inventory.remove_by_id(ComponentSlot::Network, &id) {
                    if self.assembled_pc.install_component(n).is_ok() {
                        warnings.push(format!("Auto-installed Network: {name}"));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Remove a component from assembled PC slot at given index and put it in inventory
    pub fn uninstall_component_at(&mut self, slot_index: usize) -> Option<String> {
        let component = self.assembled_pc.uninstall_at(slot_index)?;
        let name = component.name().to_string();
        self.inventory.add(component);
        Some(name)
    }

    /// Uninstall first component of a given category and put it in inventory
    pub fn uninstall_component(&mut self, slot: crate::model::ComponentSlot) -> Option<String> {
        use crate::model::ComponentSlot;

        let mb = self.assembled_pc.motherboard.as_mut()?;

        // For motherboard category, uninstall the entire motherboard
        if slot == ComponentSlot::Motherboard {
            let old_mb = self.assembled_pc.motherboard.take()?;
            let name = old_mb.name.to_string();
            self.inventory.add(OwnedComponent::Motherboard(old_mb));
            return Some(name);
        }

        // Find first filled slot of this category
        let kind = slot.to_kind()?;
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
        if let Some(mb) = &self.assembled_pc.motherboard
            && mb.id == mb_id
        {
            return true;
        }
        self.inventory.motherboards.iter().any(|m| m.id == mb_id)
    }

    /// Count total owned components of a specific item
    pub fn count_owned_of(&self, item: &ShopItem) -> usize {
        match item {
            ShopItem::Cpu(real) => self.inventory.cpus.iter().filter(|c| **c == **real).count(),
            ShopItem::Cooler(real) => self
                .inventory
                .coolers
                .iter()
                .filter(|c| **c == **real)
                .count(),
            ShopItem::Ram(real) => self.inventory.rams.iter().filter(|c| **c == **real).count(),
            ShopItem::Storage(real) => self
                .inventory
                .storage
                .iter()
                .filter(|c| **c == **real)
                .count(),
            ShopItem::Network(real) => self
                .inventory
                .networks
                .iter()
                .filter(|c| **c == **real)
                .count(),
            ShopItem::Motherboard(real) => self
                .inventory
                .motherboards
                .iter()
                .filter(|c| **c == *real)
                .count(),
        }
    }

    /// Get the current motherboard (installed or first in inventory)
    pub fn get_motherboard(&self) -> Option<&Motherboard> {
        self.assembled_pc
            .motherboard
            .as_ref()
            .or_else(|| self.inventory.motherboards.first())
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
            assembled_pc: AssembledPC::from_motherboard(starter_mb),
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
        if let Some(pc) = self.working_pc() {
            pc.compute_power() >= min_compute && pc.motherboard.total_ram_mb() >= min_memory
        } else {
            false // PC not functional
        }
    }

    pub fn unread_mail_count(&self) -> usize {
        self.inbox.iter().filter(|m| !m.is_read).count()
    }
}
