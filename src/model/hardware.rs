//! PC Hardware component system with compatibility and performance modeling
//!
//! The motherboard is a container of component slots. Each slot type defines
//! its constraints (socket type, RAM type, etc.) and holds an optional
//! installed component.

use std::fmt;
use std::sync::LazyLock;

// ============================================================================
// CPU Generation & Socket System
// ============================================================================

/// CPU socket type - determines motherboard/cooler compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuSocket {
    Socket478, // Early Pentium 4
    LGA775,    // Later Pentium 4, Core 2
    LGA1156,   // First gen Core i
    LGA1200,   // 10th/11th gen
    LGA1700,   // 12th/13th gen
    AM4,       // AMD Ryzen
    AM5,       // AMD Ryzen 7000+
}

impl fmt::Display for CpuSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Socket478 => write!(f, "Socket 478"),
            Self::LGA775 => write!(f, "LGA 775"),
            Self::LGA1156 => write!(f, "LGA 1156"),
            Self::LGA1200 => write!(f, "LGA 1200"),
            Self::LGA1700 => write!(f, "LGA 1700"),
            Self::AM4 => write!(f, "AM4"),
            Self::AM5 => write!(f, "AM5"),
        }
    }
}

/// CPU component
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpu {
    pub id: &'static str,
    pub name: &'static str,
    pub socket: CpuSocket,
    pub cores: u8,
    pub threads: u8,
    pub base_freq_mhz: u32, // Base frequency in MHz
    pub max_freq_mhz: u32,  // Maximum turbo frequency (requires adequate cooling)
    pub tdp_watts: u32,     // Thermal Design Power
    pub price: u32,
}

impl Cpu {
    /// Calculate effective frequency based on cooler capability
    pub const fn effective_freq(&self, cooler_max_tdp: u32) -> u32 {
        if cooler_max_tdp >= self.tdp_watts {
            self.max_freq_mhz
        } else {
            // Throttle frequency proportionally
            let ratio = cooler_max_tdp * 100 / self.tdp_watts;
            let freq_range = self.max_freq_mhz - self.base_freq_mhz;
            self.base_freq_mhz + (freq_range * ratio / 100)
        }
    }

    /// Calculate compute power (arbitrary units for tool speed)
    pub const fn compute_power(&self, cooler_max_tdp: u32) -> u32 {
        let freq = self.effective_freq(cooler_max_tdp);
        (self.cores as u32) * (self.threads as u32) * freq / 1000
    }
}

// ============================================================================
// CPU Cooler
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolerType {
    Stock,
    Tower,
    AIO,
}

/// CPU Cooler component
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cooler {
    pub id: &'static str,
    pub name: &'static str,
    pub cooler_type: CoolerType,
    pub max_tdp: u32, // Maximum TDP it can handle
    pub compatible_sockets: &'static [CpuSocket],
    pub price: u32,
}

impl Cooler {
    pub fn is_compatible(&self, socket: CpuSocket) -> bool {
        self.compatible_sockets.contains(&socket)
    }
}

// ============================================================================
// RAM
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RamType {
    DDR3,
    DDR4,
    DDR5,
}

impl fmt::Display for RamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DDR3 => write!(f, "DDR3"),
            Self::DDR4 => write!(f, "DDR4"),
            Self::DDR5 => write!(f, "DDR5"),
        }
    }
}

/// RAM module
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ram {
    pub id: &'static str,
    pub name: &'static str,
    pub ram_type: RamType,
    pub capacity_mb: u32,
    pub speed_mhz: u32,
    pub price: u32,
}

// ============================================================================
// Storage
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    HDD,
    SSD,
    NVMe,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HDD => write!(f, "HDD"),
            Self::SSD => write!(f, "SSD"),
            Self::NVMe => write!(f, "NVMe"),
        }
    }
}

/// Storage drive
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Storage {
    pub id: &'static str,
    pub name: &'static str,
    pub storage_type: StorageType,
    pub capacity_mb: u32,
    pub read_speed_mbps: u32, // MB/s
    pub write_speed_mbps: u32,
    pub price: u32,
}

// ============================================================================
// Network
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Dialup,
    DSL,
    Cable,
    Fiber,
}

impl fmt::Display for NetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dialup => write!(f, "Dial-up"),
            Self::DSL => write!(f, "DSL"),
            Self::Cable => write!(f, "Cable"),
            Self::Fiber => write!(f, "Fiber"),
        }
    }
}

/// Network adapter
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkCard {
    pub id: &'static str,
    pub name: &'static str,
    pub network_type: NetworkType,
    pub speed_kbps: u32,
    pub price: u32,
}

// ============================================================================
// Storage Slot Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageSlotType {
    IDE,
    SATA,
    NVMe,
}

impl fmt::Display for StorageSlotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IDE => write!(f, "IDE"),
            Self::SATA => write!(f, "SATA"),
            Self::NVMe => write!(f, "NVMe"),
        }
    }
}

// ============================================================================
// Component Slot System (defines constraints, not installed components)
// ============================================================================

/// CPU slot constraint - defines socket type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuSlot {
    pub socket: CpuSocket,
}

/// Cooler slot constraint - defines compatible sockets
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoolerSlot {
    pub compatible_sockets: &'static [CpuSocket],
}

/// RAM slot constraint - defines type and max capacity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RamSlot {
    pub ram_type: RamType,
    pub max_capacity_mb: u32,
}

/// Storage slot constraint - defines interface type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSlot {
    pub slot_type: StorageSlotType,
}

/// Network slot (no constraints)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSlot;

/// Unified component slot enum - motherboard contains a vector of these
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentSlotType {
    Cpu(CpuSlot),
    Cooler(CoolerSlot),
    Ram(RamSlot),
    Storage(StorageSlot),
    Network(NetworkSlot),
}

impl ComponentSlotType {
    /// Get the hardware kind for categorization
    pub const fn kind(&self) -> HardwareKind {
        match self {
            Self::Cpu(_) => HardwareKind::Cpu,
            Self::Cooler(_) => HardwareKind::Cooler,
            Self::Ram(_) => HardwareKind::Ram,
            Self::Storage(_) => HardwareKind::Storage,
            Self::Network(_) => HardwareKind::Network,
        }
    }

    /// Check if a component is compatible with this slot
    pub fn is_compatible(&self, component: &HardwareComponent) -> bool {
        match (self, component) {
            (Self::Cpu(slot), HardwareComponent::Cpu(cpu)) => slot.socket == cpu.socket,
            (Self::Cooler(slot), HardwareComponent::Cooler(cooler)) => slot
                .compatible_sockets
                .iter()
                .any(|s| cooler.is_compatible(*s)),
            #[allow(clippy::suspicious_operation_groupings)]
            (Self::Ram(slot), HardwareComponent::Ram(ram)) => {
                slot.ram_type == ram.ram_type && ram.capacity_mb <= slot.max_capacity_mb
            }
            (Self::Storage(slot), HardwareComponent::Storage(storage)) => {
                // NVMe storage needs NVMe slot, others can use SATA/IDE
                matches!(
                    (slot.slot_type, storage.storage_type),
                    (StorageSlotType::NVMe, StorageType::NVMe)
                        | (StorageSlotType::SATA, StorageType::SSD | StorageType::HDD)
                        | (StorageSlotType::IDE, StorageType::HDD)
                )
            }
            (Self::Network(_), HardwareComponent::Network(_)) => true,
            _ => false,
        }
    }
}

/// Unified enum for all hardware categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HardwareKind {
    #[default]
    Cpu,
    Cooler,
    Motherboard,
    Ram,
    Storage,
    Network,
}

impl HardwareKind {
    /// All hardware kinds in display order
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

    /// Whether this kind represents a slot type within a motherboard
    pub const fn is_slot_kind(&self) -> bool {
        !matches!(self, Self::Motherboard)
    }
}

// ============================================================================
// Motherboard
// ============================================================================

/// Motherboard tier for visual differentiation in ASCII art
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotherboardTier {
    Basic,       // Socket478 - simple ASCII with +--+
    Standard,    // LGA775 - moderate with ┌──┐
    Performance, // LGA1200, AM4 - detailed with ╭──╮
    Enthusiast,  // LGA1700, AM5 - full colors with ╔══╗
}

/// Motherboard - container of component slots
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Motherboard {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: MotherboardTier,
    pub slots: Vec<ComponentSlotType>,
    pub price: u32,
}

impl Motherboard {
    /// Get the CPU socket type from the CPU slot
    pub fn socket(&self) -> Option<CpuSocket> {
        self.slots.iter().find_map(|s| match s {
            ComponentSlotType::Cpu(cpu_slot) => Some(cpu_slot.socket),
            _ => None,
        })
    }

    /// Get the RAM type from the first RAM slot
    pub fn ram_type(&self) -> Option<RamType> {
        self.slots.iter().find_map(|s| match s {
            ComponentSlotType::Ram(ram_slot) => Some(ram_slot.ram_type),
            _ => None,
        })
    }

    /// Count total slots of a specific kind (0 for Motherboard)
    pub fn slot_count(&self, kind: HardwareKind) -> usize {
        if !kind.is_slot_kind() {
            return 0;
        }
        self.slots.iter().filter(|s| s.kind() == kind).count()
    }

    /// Count slots compatible with a specific component
    pub fn count_compatible_slots(&self, component: &HardwareComponent) -> usize {
        self.slots
            .iter()
            .filter(|s| s.is_compatible(component))
            .count()
    }
}

// ============================================================================
// Component Inventory & Assembly System
// ============================================================================

/// A component that can be stored in inventory
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareComponent {
    Cpu(Cpu),
    Cooler(Cooler),
    Motherboard(Motherboard),
    Ram(Ram),
    Storage(Storage),
    Network(NetworkCard),
}

impl HardwareComponent {
    pub const fn name(&self) -> &str {
        match self {
            Self::Cpu(c) => c.name,
            Self::Cooler(c) => c.name,
            Self::Motherboard(m) => m.name,
            Self::Ram(r) => r.name,
            Self::Storage(s) => s.name,
            Self::Network(n) => n.name,
        }
    }

    pub const fn id(&self) -> &str {
        match self {
            Self::Cpu(c) => c.id,
            Self::Cooler(c) => c.id,
            Self::Motherboard(m) => m.id,
            Self::Ram(r) => r.id,
            Self::Storage(s) => s.id,
            Self::Network(n) => n.id,
        }
    }

    pub const fn kind(&self) -> HardwareKind {
        match self {
            Self::Cpu(_) => HardwareKind::Cpu,
            Self::Cooler(_) => HardwareKind::Cooler,
            Self::Motherboard(_) => HardwareKind::Motherboard,
            Self::Ram(_) => HardwareKind::Ram,
            Self::Storage(_) => HardwareKind::Storage,
            Self::Network(_) => HardwareKind::Network,
        }
    }

    pub const fn price(&self) -> u32 {
        match self {
            Self::Cpu(c) => c.price,
            Self::Cooler(c) => c.price,
            Self::Motherboard(m) => m.price,
            Self::Ram(r) => r.price,
            Self::Storage(s) => s.price,
            Self::Network(n) => n.price,
        }
    }

    /// Get a human-readable description of the component
    pub fn description(&self) -> String {
        match self {
            Self::Cpu(c) => format!(
                "{} cores / {} threads, {}-{} MHz, {} W TDP",
                c.cores, c.threads, c.base_freq_mhz, c.max_freq_mhz, c.tdp_watts
            ),
            Self::Cooler(c) => format!("{:?} cooler, up to {} W TDP", c.cooler_type, c.max_tdp),
            Self::Motherboard(m) => {
                let socket = m
                    .socket()
                    .map_or_else(|| "N/A".to_string(), |s| format!("{s}"));
                let ram_type = m
                    .ram_type()
                    .map_or_else(|| "N/A".to_string(), |r| format!("{r}"));
                let ram_slots = m.slot_count(HardwareKind::Ram);
                format!("{socket} socket, {ram_type} support, {ram_slots} RAM slots")
            }
            Self::Ram(r) => format!(
                "{} {} @ {} MHz",
                format_bytes(r.capacity_mb),
                r.ram_type,
                r.speed_mhz
            ),
            Self::Storage(s) => format!(
                "{} {:?}, {}/{} MB/s R/W",
                format_bytes(s.capacity_mb),
                s.storage_type,
                s.read_speed_mbps,
                s.write_speed_mbps
            ),
            Self::Network(n) => format!("{:?} @ {}", n.network_type, format_speed(n.speed_kbps)),
        }
    }

    /// Get socket/type compatibility info for the component
    pub fn socket_info(&self) -> Option<String> {
        match self {
            Self::Cpu(c) => Some(format!("{}", c.socket)),
            Self::Cooler(c) => {
                let sockets: Vec<_> = c
                    .compatible_sockets
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect();
                Some(sockets.join(", "))
            }
            Self::Motherboard(m) => m.socket().map(|s| format!("{s}")),
            Self::Ram(r) => Some(format!("{}", r.ram_type)),
            Self::Storage(_) | Self::Network(_) => None,
        }
    }
}

/// Format bytes to human readable
pub fn format_bytes(mb: u32) -> String {
    if mb >= 1_048_576 {
        format!("{:.1} TB", f64::from(mb) / 1_048_576.0)
    } else if mb >= 1024 {
        format!("{:.1} GB", f64::from(mb) / 1024.0)
    } else {
        format!("{mb} MB")
    }
}

/// Format speed to human readable
pub fn format_speed(kbps: u32) -> String {
    if kbps >= 1_048_576 {
        format!("{:.1} Gbps", f64::from(kbps) / 1_048_576.0)
    } else if kbps >= 1024 {
        format!("{:.1} Mbps", f64::from(kbps) / 1024.0)
    } else {
        format!("{kbps} Kbps")
    }
}

/// A component owned by the player with installation state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedComponent {
    pub component: HardwareComponent,
    pub installed: bool,
}

/// Represents a grouped view of inventory items for display
/// Installed items are shown individually, spare items of same model are grouped
#[derive(Debug, Clone)]
pub struct DisplayGroup {
    /// The absolute indices of items in this group
    pub indices: Vec<usize>,
    /// The component (representative, all items in group have same id)
    pub component: HardwareComponent,
    /// Whether these items are installed (groups only contain items with same install state)
    pub installed: bool,
}

impl DisplayGroup {
    /// Get the count of items in this group
    pub const fn count(&self) -> usize {
        self.indices.len()
    }

    /// Get the first index (for operations on the group)
    pub fn first_index(&self) -> usize {
        self.indices[0]
    }
}

/// Player's component inventory - single source of truth for all owned items
/// Items are never removed, only added.
#[derive(Debug, Clone, Default)]
pub struct ComponentInventory {
    items: Vec<OwnedComponent>,
}

impl ComponentInventory {
    /// Add a new component to inventory (purchased from shop)
    pub fn add(&mut self, component: HardwareComponent) {
        self.items.push(OwnedComponent {
            component,
            installed: false,
        });
    }

    /// Get an item by absolute index
    pub fn get(&self, index: usize) -> Option<&HardwareComponent> {
        self.items.get(index).map(|o| &o.component)
    }

    /// Check if an item at the given absolute index is installed
    pub fn is_installed(&self, index: usize) -> bool {
        self.items.get(index).is_some_and(|o| o.installed)
    }

    /// Mark an item as installed
    pub fn install(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.installed = true;
        }
    }

    /// Mark an item as not installed (spare)
    pub fn uninstall(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.installed = false;
        }
    }

    /// Toggle installed state
    pub fn toggle_installed(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.installed = !item.installed;
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub const fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn count_items_for(&self, kind: HardwareKind) -> usize {
        self.items
            .iter()
            .filter(|o| o.component.kind() == kind)
            .count()
    }

    /// Get an iterator over (`absolute_index`, item) for a hardware kind
    pub fn items_for(
        &self,
        kind: HardwareKind,
    ) -> impl Iterator<Item = (usize, &HardwareComponent)> {
        self.items
            .iter()
            .enumerate()
            .filter(move |(_, o)| o.component.kind() == kind)
            .map(|(i, o)| (i, &o.component))
    }

    /// Get an iterator over (`absolute_index`, item, `is_installed`) for a hardware kind
    pub fn items_for_display(
        &self,
        kind: HardwareKind,
    ) -> impl Iterator<Item = (usize, &HardwareComponent, bool)> {
        self.items
            .iter()
            .enumerate()
            .filter(move |(_, o)| o.component.kind() == kind)
            .map(|(i, o)| (i, &o.component, o.installed))
    }

    /// Get a specific item by kind and index within that kind
    pub fn get_at(&self, kind: HardwareKind, index: usize) -> Option<(usize, &HardwareComponent)> {
        self.items_for(kind).nth(index)
    }

    /// Check if a motherboard with the given id exists in inventory
    pub fn has_motherboard(&self, mb_id: &str) -> bool {
        self.items
            .iter()
            .any(|o| matches!(&o.component, HardwareComponent::Motherboard(m) if m.id == mb_id))
    }

    /// Get the installed motherboard (if any)
    pub fn installed_motherboard(&self) -> Option<&Motherboard> {
        self.items.iter().find_map(|o| {
            if o.installed {
                match &o.component {
                    HardwareComponent::Motherboard(m) => Some(m),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Get the index of the installed motherboard (if any)
    pub fn installed_motherboard_idx(&self) -> Option<usize> {
        self.items.iter().enumerate().find_map(|(i, o)| {
            if o.installed && matches!(&o.component, HardwareComponent::Motherboard(_)) {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Count installed items of a specific kind
    pub fn count_installed(&self, kind: HardwareKind) -> usize {
        self.items
            .iter()
            .filter(|o| o.installed && o.component.kind() == kind)
            .count()
    }

    /// Get all installed items of a specific kind
    pub fn installed_of_kind(
        &self,
        kind: HardwareKind,
    ) -> impl Iterator<Item = &HardwareComponent> {
        self.items
            .iter()
            .filter(move |o| o.installed && o.component.kind() == kind)
            .map(|o| &o.component)
    }

    /// Check if PC is functional (has motherboard and required components installed)
    pub fn is_functional(&self) -> bool {
        self.installed_motherboard().is_some()
            && self.count_installed(HardwareKind::Cpu) >= 1
            && self.count_installed(HardwareKind::Cooler) >= 1
            && self.count_installed(HardwareKind::Ram) >= 1
            && self.count_installed(HardwareKind::Storage) >= 1
            && self.count_installed(HardwareKind::Network) >= 1
    }

    /// Check if a component can be installed
    /// Returns Ok(()) if installable, Err with reason if not
    pub fn can_install(&self, index: usize) -> Result<(), String> {
        let owned = self.items.get(index).ok_or("Invalid index")?;

        if owned.installed {
            return Err("Already installed".to_string());
        }

        // Motherboards can always be installed (will uninstall existing)
        if matches!(&owned.component, HardwareComponent::Motherboard(_)) {
            return Ok(());
        }

        // For other components, need a motherboard first
        let mb = self
            .installed_motherboard()
            .ok_or("No motherboard installed")?;

        // Check if there's a compatible slot available
        let kind = owned.component.kind();
        let installed_count = self.count_installed(kind);
        let compatible_slots = mb.count_compatible_slots(&owned.component);

        if compatible_slots == 0 {
            return Err("Not compatible with this motherboard".to_string());
        }

        if installed_count >= compatible_slots {
            return Err(format!("All compatible {} slots are full", kind.name()));
        }

        Ok(())
    }

    /// Install an item, uninstalling conflicting items if needed
    /// Returns list of uninstalled item indices
    pub fn install_item(&mut self, index: usize) -> Result<Vec<usize>, String> {
        self.can_install(index)?;

        let is_motherboard = matches!(
            &self.items[index].component,
            HardwareComponent::Motherboard(_)
        );
        let mut uninstalled = Vec::new();

        // If installing a motherboard, uninstall all other components first
        if is_motherboard {
            // Uninstall existing motherboard and all other components
            for (i, item) in self.items.iter_mut().enumerate() {
                if item.installed {
                    item.installed = false;
                    uninstalled.push(i);
                }
            }
        }

        self.items[index].installed = true;
        Ok(uninstalled)
    }

    /// Get installed CPU (if any)
    pub fn cpu(&self) -> Option<&Cpu> {
        self.installed_of_kind(HardwareKind::Cpu)
            .find_map(|c| match c {
                HardwareComponent::Cpu(cpu) => Some(cpu),
                _ => None,
            })
    }

    /// Get installed cooler (if any)
    pub fn cooler(&self) -> Option<&Cooler> {
        self.installed_of_kind(HardwareKind::Cooler)
            .find_map(|c| match c {
                HardwareComponent::Cooler(cooler) => Some(cooler),
                _ => None,
            })
    }

    /// Get all installed RAM
    pub fn rams(&self) -> Vec<&Ram> {
        self.installed_of_kind(HardwareKind::Ram)
            .filter_map(|c| match c {
                HardwareComponent::Ram(r) => Some(r),
                _ => None,
            })
            .collect()
    }

    /// Get all installed storage
    pub fn storages(&self) -> Vec<&Storage> {
        self.installed_of_kind(HardwareKind::Storage)
            .filter_map(|c| match c {
                HardwareComponent::Storage(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    /// Get all installed network cards
    pub fn networks(&self) -> Vec<&NetworkCard> {
        self.installed_of_kind(HardwareKind::Network)
            .filter_map(|c| match c {
                HardwareComponent::Network(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Get total installed RAM capacity in MB
    pub fn total_ram_mb(&self) -> u32 {
        self.rams().iter().map(|r| r.capacity_mb).sum()
    }

    /// Get total storage capacity in MB
    pub fn total_storage_mb(&self) -> u32 {
        self.storages().iter().map(|s| s.capacity_mb).sum()
    }

    /// Get best network speed in kbps
    pub fn best_network_speed(&self) -> u32 {
        self.networks()
            .iter()
            .map(|n| n.speed_kbps)
            .max()
            .unwrap_or(0)
    }

    /// Get compute power score
    pub fn compute_power(&self) -> u32 {
        let Some(cpu) = self.cpu() else { return 0 };
        let cooler_tdp = self.cooler().map_or(0, |c| c.max_tdp);
        cpu.compute_power(cooler_tdp)
    }

    /// Get memory bandwidth score
    #[allow(clippy::cast_possible_truncation)]
    pub fn memory_score(&self) -> u32 {
        let rams = self.rams();
        if rams.is_empty() {
            return 0;
        }
        let total_capacity: u32 = rams.iter().map(|r| r.capacity_mb).sum();
        let avg_speed: u32 = rams.iter().map(|r| r.speed_mhz).sum::<u32>() / rams.len() as u32;
        total_capacity / 256 * avg_speed / 1000
    }

    /// Get storage speed score
    pub fn storage_score(&self) -> u32 {
        self.storages()
            .iter()
            .map(|s| u32::midpoint(s.read_speed_mbps, s.write_speed_mbps))
            .max()
            .unwrap_or(0)
    }

    /// Overall power score for tool effectiveness
    pub fn overall_power(&self) -> u32 {
        self.compute_power() + self.memory_score() + self.storage_score() / 10
    }

    /// Get effective CPU frequency considering cooler
    pub fn effective_cpu_freq(&self) -> u32 {
        let Some(cpu) = self.cpu() else { return 0 };
        let cooler_tdp = self.cooler().map_or(0, |c| c.max_tdp);
        cpu.effective_freq(cooler_tdp)
    }

    /// Get a sorted and grouped view for display
    /// - Installed items shown individually (sorted by name)
    /// - Non-installed items grouped by component ID (sorted by name)
    /// - Installed items come first, then spare groups
    pub fn grouped_display(&self, kind: HardwareKind) -> Vec<DisplayGroup> {
        use std::collections::HashMap;

        let mut result = Vec::new();
        let mut spare_groups: HashMap<&str, Vec<usize>> = HashMap::new();

        for (i, owned) in self.items.iter().enumerate() {
            if owned.component.kind() != kind {
                continue;
            }

            if owned.installed {
                result.push(DisplayGroup {
                    indices: vec![i],
                    component: owned.component.clone(),
                    installed: true,
                });
            } else {
                spare_groups
                    .entry(owned.component.id())
                    .or_default()
                    .push(i);
            }
        }

        // Convert spare groups to DisplayGroups
        let spare: Vec<DisplayGroup> = spare_groups
            .into_values()
            .map(|indices| {
                let component = self.items[indices[0]].component.clone();
                DisplayGroup {
                    indices,
                    component,
                    installed: false,
                }
            })
            .collect();

        // Sort spare by name

        // Installed first, then spare
        result.extend(spare);
        result.sort_by(|a, b| a.component.price().cmp(&b.component.price()));
        result
    }
}

impl ComponentInventory {
    /// Find absolute index of best compatible spare CPU (highest compute power)
    pub fn best_spare_cpu_index(&self, socket: Option<CpuSocket>) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.installed && o.component.kind() == HardwareKind::Cpu)
            .filter_map(|(i, o)| match &o.component {
                HardwareComponent::Cpu(cpu) if socket.is_none() || socket == Some(cpu.socket) => {
                    Some((i, cpu))
                }
                _ => None,
            })
            .max_by_key(|(_, c)| u32::from(c.cores) * c.max_freq_mhz)
            .map(|(i, _)| i)
    }

    /// Find absolute index of best compatible spare cooler (highest TDP support)
    pub fn best_spare_cooler_index(&self, socket: Option<CpuSocket>) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.installed && o.component.kind() == HardwareKind::Cooler)
            .filter_map(|(i, o)| match &o.component {
                HardwareComponent::Cooler(cooler)
                    if socket.is_none() || socket.is_some_and(|s| cooler.is_compatible(s)) =>
                {
                    Some((i, cooler))
                }
                _ => None,
            })
            .max_by_key(|(_, c)| c.max_tdp)
            .map(|(i, _)| i)
    }

    /// Find absolute index of best compatible spare RAM (highest capacity)
    pub fn best_spare_ram_index(&self, ram_type: Option<RamType>) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.installed && o.component.kind() == HardwareKind::Ram)
            .filter_map(|(i, o)| match &o.component {
                HardwareComponent::Ram(ram)
                    if ram_type.is_none() || ram_type == Some(ram.ram_type) =>
                {
                    Some((i, ram))
                }
                _ => None,
            })
            .max_by_key(|(_, r)| r.capacity_mb)
            .map(|(i, _)| i)
    }

    /// Find absolute index of best spare storage (highest capacity)
    pub fn best_spare_storage_index(&self) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.installed && o.component.kind() == HardwareKind::Storage)
            .filter_map(|(i, o)| match &o.component {
                HardwareComponent::Storage(s) => Some((i, s)),
                _ => None,
            })
            .max_by_key(|(_, s)| s.capacity_mb)
            .map(|(i, _)| i)
    }

    /// Find absolute index of best spare network (highest speed)
    pub fn best_spare_network_index(&self) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.installed && o.component.kind() == HardwareKind::Network)
            .filter_map(|(i, o)| match &o.component {
                HardwareComponent::Network(n) => Some((i, n)),
                _ => None,
            })
            .max_by_key(|(_, n)| n.speed_kbps)
            .map(|(i, _)| i)
    }
}

// ============================================================================
// Component Catalogs (static data)
// ============================================================================

pub static CPUS: &[Cpu] = &[
    Cpu {
        id: "p4_2400",
        name: "Pentium 4 2.4GHz",
        socket: CpuSocket::Socket478,
        cores: 1,
        threads: 1,
        base_freq_mhz: 2400,
        max_freq_mhz: 2400,
        tdp_watts: 68,
        price: 0, // starter
    },
    Cpu {
        id: "p4_3200",
        name: "Pentium 4 3.2GHz HT",
        socket: CpuSocket::Socket478,
        cores: 1,
        threads: 2,
        base_freq_mhz: 3200,
        max_freq_mhz: 3200,
        tdp_watts: 82,
        price: 800,
    },
    Cpu {
        id: "c2d_e6600",
        name: "Core 2 Duo E6600",
        socket: CpuSocket::LGA775,
        cores: 2,
        threads: 2,
        base_freq_mhz: 2400,
        max_freq_mhz: 2400,
        tdp_watts: 65,
        price: 1500,
    },
    Cpu {
        id: "c2q_q6600",
        name: "Core 2 Quad Q6600",
        socket: CpuSocket::LGA775,
        cores: 4,
        threads: 4,
        base_freq_mhz: 2400,
        max_freq_mhz: 2400,
        tdp_watts: 95,
        price: 2500,
    },
    Cpu {
        id: "i5_10400",
        name: "Core i5-10400",
        socket: CpuSocket::LGA1200,
        cores: 6,
        threads: 12,
        base_freq_mhz: 2900,
        max_freq_mhz: 4300,
        tdp_watts: 65,
        price: 4000,
    },
    Cpu {
        id: "i7_10700k",
        name: "Core i7-10700K",
        socket: CpuSocket::LGA1200,
        cores: 8,
        threads: 16,
        base_freq_mhz: 3800,
        max_freq_mhz: 5100,
        tdp_watts: 125,
        price: 7000,
    },
    Cpu {
        id: "i9_12900k",
        name: "Core i9-12900K",
        socket: CpuSocket::LGA1700,
        cores: 16,
        threads: 24,
        base_freq_mhz: 3200,
        max_freq_mhz: 5200,
        tdp_watts: 241,
        price: 12000,
    },
    Cpu {
        id: "r5_5600x",
        name: "Ryzen 5 5600X",
        socket: CpuSocket::AM4,
        cores: 6,
        threads: 12,
        base_freq_mhz: 3700,
        max_freq_mhz: 4600,
        tdp_watts: 65,
        price: 4500,
    },
    Cpu {
        id: "r9_5950x",
        name: "Ryzen 9 5950X",
        socket: CpuSocket::AM4,
        cores: 16,
        threads: 32,
        base_freq_mhz: 3400,
        max_freq_mhz: 4900,
        tdp_watts: 105,
        price: 10000,
    },
];

pub static COOLERS: &[Cooler] = &[
    Cooler {
        id: "stock_478",
        name: "Intel Stock Cooler (478)",
        cooler_type: CoolerType::Stock,
        max_tdp: 70,
        compatible_sockets: &[CpuSocket::Socket478],
        price: 0,
    },
    Cooler {
        id: "stock_775",
        name: "Intel Stock Cooler (775)",
        cooler_type: CoolerType::Stock,
        max_tdp: 65,
        compatible_sockets: &[CpuSocket::LGA775],
        price: 50,
    },
    Cooler {
        id: "cm_212",
        name: "Cooler Master Hyper 212",
        cooler_type: CoolerType::Tower,
        max_tdp: 150,
        compatible_sockets: &[
            CpuSocket::LGA775,
            CpuSocket::LGA1156,
            CpuSocket::LGA1200,
            CpuSocket::AM4,
        ],
        price: 400,
    },
    Cooler {
        id: "nh_d15",
        name: "Noctua NH-D15",
        cooler_type: CoolerType::Tower,
        max_tdp: 250,
        compatible_sockets: &[
            CpuSocket::LGA1200,
            CpuSocket::LGA1700,
            CpuSocket::AM4,
            CpuSocket::AM5,
        ],
        price: 1000,
    },
    Cooler {
        id: "kraken_x63",
        name: "NZXT Kraken X63 AIO",
        cooler_type: CoolerType::AIO,
        max_tdp: 300,
        compatible_sockets: &[
            CpuSocket::LGA1200,
            CpuSocket::LGA1700,
            CpuSocket::AM4,
            CpuSocket::AM5,
        ],
        price: 1800,
    },
];

/// Static motherboard catalog - lazily initialized
pub static MOTHERBOARDS: LazyLock<Vec<Motherboard>> = LazyLock::new(|| {
    vec![
        Motherboard {
            id: "mb_478",
            name: "Generic Socket 478 Board",
            tier: MotherboardTier::Basic,
            slots: vec![
                ComponentSlotType::Cpu(CpuSlot {
                    socket: CpuSocket::Socket478,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::Socket478],
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 1024,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 1024,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::IDE,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::IDE,
                }),
                ComponentSlotType::Network(NetworkSlot),
            ],
            price: 0,
        },
        Motherboard {
            id: "mb_775",
            name: "Intel P35 Chipset",
            tier: MotherboardTier::Standard,
            slots: vec![
                ComponentSlotType::Cpu(CpuSlot {
                    socket: CpuSocket::LGA775,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA775],
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Network(NetworkSlot),
            ],
            price: 800,
        },
        Motherboard {
            id: "mb_z490",
            name: "ASUS ROG Z490",
            tier: MotherboardTier::Performance,
            slots: vec![
                ComponentSlotType::Cpu(CpuSlot {
                    socket: CpuSocket::LGA1200,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA1200],
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Network(NetworkSlot),
                ComponentSlotType::Network(NetworkSlot),
            ],
            price: 2500,
        },
        Motherboard {
            id: "mb_z690",
            name: "MSI MEG Z690",
            tier: MotherboardTier::Enthusiast,
            slots: vec![
                ComponentSlotType::Cpu(CpuSlot {
                    socket: CpuSocket::LGA1700,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA1700],
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Network(NetworkSlot),
                ComponentSlotType::Network(NetworkSlot),
            ],
            price: 4000,
        },
        Motherboard {
            id: "mb_b550",
            name: "ASUS ROG B550-F",
            tier: MotherboardTier::Performance,
            slots: vec![
                ComponentSlotType::Cpu(CpuSlot {
                    socket: CpuSocket::AM4,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::AM4],
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                }),
                ComponentSlotType::Network(NetworkSlot),
                ComponentSlotType::Network(NetworkSlot),
            ],
            price: 2000,
        },
    ]
});

pub static RAMS: &[Ram] = &[
    Ram {
        id: "ddr3_512",
        name: "512MB DDR3-800",
        ram_type: RamType::DDR3,
        capacity_mb: 512,
        speed_mhz: 800,
        price: 250,
    },
    Ram {
        id: "ddr3_2048",
        name: "2GB DDR3-1333",
        ram_type: RamType::DDR3,
        capacity_mb: 2048,
        speed_mhz: 1333,
        price: 500,
    },
    Ram {
        id: "ddr3_4096",
        name: "4GB DDR3-1600",
        ram_type: RamType::DDR3,
        capacity_mb: 4096,
        speed_mhz: 1600,
        price: 1000,
    },
    Ram {
        id: "ddr4_8192",
        name: "8GB DDR4-2666",
        ram_type: RamType::DDR4,
        capacity_mb: 8192,
        speed_mhz: 2666,
        price: 1500,
    },
    Ram {
        id: "ddr4_16384",
        name: "16GB DDR4-3200",
        ram_type: RamType::DDR4,
        capacity_mb: 16384,
        speed_mhz: 3200,
        price: 3000,
    },
    Ram {
        id: "ddr4_32768",
        name: "32GB DDR4-3600",
        ram_type: RamType::DDR4,
        capacity_mb: 32768,
        speed_mhz: 3600,
        price: 6000,
    },
    Ram {
        id: "ddr5_32768",
        name: "32GB DDR5-5600",
        ram_type: RamType::DDR5,
        capacity_mb: 32768,
        speed_mhz: 5600,
        price: 8000,
    },
    Ram {
        id: "ddr5_65536",
        name: "64GB DDR5-6000",
        ram_type: RamType::DDR5,
        capacity_mb: 65536,
        speed_mhz: 6000,
        price: 15000,
    },
];

pub static STORAGES: &[Storage] = &[
    Storage {
        id: "hdd_1gb",
        name: "1GB IDE HDD",
        storage_type: StorageType::HDD,
        capacity_mb: 1024,
        read_speed_mbps: 10,
        write_speed_mbps: 8,
        price: 100,
    },
    Storage {
        id: "hdd_20gb",
        name: "20GB HDD 7200RPM",
        storage_type: StorageType::HDD,
        capacity_mb: 20480,
        read_speed_mbps: 80,
        write_speed_mbps: 60,
        price: 300,
    },
    Storage {
        id: "hdd_250gb",
        name: "250GB HDD",
        storage_type: StorageType::HDD,
        capacity_mb: 256_000,
        read_speed_mbps: 120,
        write_speed_mbps: 100,
        price: 800,
    },
    Storage {
        id: "ssd_120gb",
        name: "120GB SATA SSD",
        storage_type: StorageType::SSD,
        capacity_mb: 122_880,
        read_speed_mbps: 500,
        write_speed_mbps: 400,
        price: 1500,
    },
    Storage {
        id: "ssd_500gb",
        name: "500GB SATA SSD",
        storage_type: StorageType::SSD,
        capacity_mb: 512_000,
        read_speed_mbps: 550,
        write_speed_mbps: 500,
        price: 3000,
    },
    Storage {
        id: "nvme_1tb",
        name: "1TB NVMe SSD",
        storage_type: StorageType::NVMe,
        capacity_mb: 1_048_576,
        read_speed_mbps: 3500,
        write_speed_mbps: 3000,
        price: 5000,
    },
    Storage {
        id: "nvme_2tb",
        name: "2TB NVMe Gen4",
        storage_type: StorageType::NVMe,
        capacity_mb: 2_097_152,
        read_speed_mbps: 7000,
        write_speed_mbps: 5000,
        price: 10000,
    },
];

pub static NETWORKS: &[NetworkCard] = &[
    NetworkCard {
        id: "dialup_56k",
        name: "56K Modem",
        network_type: NetworkType::Dialup,
        speed_kbps: 56,
        price: 0,
    },
    NetworkCard {
        id: "dsl_256",
        name: "DSL 256Kbps",
        network_type: NetworkType::DSL,
        speed_kbps: 256,
        price: 500,
    },
    NetworkCard {
        id: "dsl_1m",
        name: "DSL 1Mbps",
        network_type: NetworkType::DSL,
        speed_kbps: 1024,
        price: 1000,
    },
    NetworkCard {
        id: "cable_10m",
        name: "Cable 10Mbps",
        network_type: NetworkType::Cable,
        speed_kbps: 10240,
        price: 2000,
    },
    NetworkCard {
        id: "cable_100m",
        name: "Cable 100Mbps",
        network_type: NetworkType::Cable,
        speed_kbps: 102_400,
        price: 4000,
    },
    NetworkCard {
        id: "fiber_1g",
        name: "Fiber 1Gbps",
        network_type: NetworkType::Fiber,
        speed_kbps: 1_048_576,
        price: 8000,
    },
];

/// Get maximum values for UI progress bars
pub struct HardwareMaximums {
    pub cpu_freq: u32,
    pub cpu_cores: u8,
    pub cpu_threads: u8,
    pub ram_capacity: u32,
    pub ram_speed: u32,
    pub storage_capacity: u32,
    pub storage_speed: u32,
    pub network_speed: u32,
}

impl HardwareMaximums {
    pub fn calculate() -> Self {
        Self {
            cpu_freq: CPUS.iter().map(|c| c.max_freq_mhz).max().unwrap_or(5200),
            cpu_cores: CPUS.iter().map(|c| c.cores).max().unwrap_or(16),
            cpu_threads: CPUS.iter().map(|c| c.threads).max().unwrap_or(32),
            ram_capacity: RAMS.iter().map(|r| r.capacity_mb).max().unwrap_or(65536),
            ram_speed: RAMS.iter().map(|r| r.speed_mhz).max().unwrap_or(6000),
            storage_capacity: STORAGES
                .iter()
                .map(|s| s.capacity_mb)
                .max()
                .unwrap_or(2_097_152),
            storage_speed: STORAGES
                .iter()
                .map(|s| s.read_speed_mbps)
                .max()
                .unwrap_or(7000),
            network_speed: NETWORKS
                .iter()
                .map(|n| n.speed_kbps)
                .max()
                .unwrap_or(1_048_576),
        }
    }
}
