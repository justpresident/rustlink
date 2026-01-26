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
// Component Slot System
// ============================================================================

/// CPU slot with socket constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuSlot {
    pub socket: CpuSocket,
    pub installed: Option<Cpu>,
}

/// Cooler slot with compatible sockets constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoolerSlot {
    pub compatible_sockets: &'static [CpuSocket],
    pub installed: Option<Cooler>,
}

/// RAM slot with type and capacity constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RamSlot {
    pub ram_type: RamType,
    pub max_capacity_mb: u32,
    pub installed: Option<Ram>,
}

/// Storage slot with interface type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSlot {
    pub slot_type: StorageSlotType,
    pub installed: Option<Storage>,
}

/// Network slot (PCI or onboard)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSlot {
    pub installed: Option<NetworkCard>,
}

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
    /// Check if this slot has a component installed
    pub const fn is_filled(&self) -> bool {
        match self {
            Self::Cpu(s) => s.installed.is_some(),
            Self::Cooler(s) => s.installed.is_some(),
            Self::Ram(s) => s.installed.is_some(),
            Self::Storage(s) => s.installed.is_some(),
            Self::Network(s) => s.installed.is_some(),
        }
    }

    /// Get the slot kind for categorization
    pub const fn kind(&self) -> ComponentSlotKind {
        match self {
            Self::Cpu(_) => ComponentSlotKind::Cpu,
            Self::Cooler(_) => ComponentSlotKind::Cooler,
            Self::Ram(_) => ComponentSlotKind::Ram,
            Self::Storage(_) => ComponentSlotKind::Storage,
            Self::Network(_) => ComponentSlotKind::Network,
        }
    }

    /// Check if a component is compatible with this slot
    pub fn is_compatible(&self, component: &OwnedComponent) -> bool {
        match (self, component) {
            (Self::Cpu(slot), OwnedComponent::Cpu(cpu)) => slot.socket == cpu.socket,
            (Self::Cooler(slot), OwnedComponent::Cooler(cooler)) => slot
                .compatible_sockets
                .iter()
                .any(|s| cooler.is_compatible(*s)),
            #[allow(clippy::suspicious_operation_groupings)]
            (Self::Ram(slot), OwnedComponent::Ram(ram)) => {
                slot.ram_type == ram.ram_type && ram.capacity_mb <= slot.max_capacity_mb
            }
            (Self::Storage(slot), OwnedComponent::Storage(storage)) => {
                // NVMe storage needs NVMe slot, others can use SATA/IDE
                matches!(
                    (slot.slot_type, storage.storage_type),
                    (StorageSlotType::NVMe, StorageType::NVMe)
                        | (StorageSlotType::SATA, StorageType::SSD | StorageType::HDD)
                        | (StorageSlotType::IDE, StorageType::HDD)
                )
            }
            (Self::Network(_), OwnedComponent::Network(_)) => true,
            _ => false,
        }
    }

    /// Install a component into this slot, returning the old component if any
    pub fn install(&mut self, component: OwnedComponent) -> Result<Option<OwnedComponent>, String> {
        if !self.is_compatible(&component) {
            return Err("Component not compatible with this slot".to_string());
        }

        match (self, component) {
            (Self::Cpu(slot), OwnedComponent::Cpu(cpu)) => {
                Ok(slot.installed.replace(cpu).map(OwnedComponent::Cpu))
            }
            (Self::Cooler(slot), OwnedComponent::Cooler(cooler)) => {
                Ok(slot.installed.replace(cooler).map(OwnedComponent::Cooler))
            }
            (Self::Ram(slot), OwnedComponent::Ram(ram)) => {
                Ok(slot.installed.replace(ram).map(OwnedComponent::Ram))
            }
            (Self::Storage(slot), OwnedComponent::Storage(storage)) => {
                Ok(slot.installed.replace(storage).map(OwnedComponent::Storage))
            }
            (Self::Network(slot), OwnedComponent::Network(network)) => {
                Ok(slot.installed.replace(network).map(OwnedComponent::Network))
            }
            _ => Err("Component type mismatch".to_string()),
        }
    }

    /// Uninstall and return the component from this slot
    pub fn uninstall(&mut self) -> Option<OwnedComponent> {
        match self {
            Self::Cpu(slot) => slot.installed.take().map(OwnedComponent::Cpu),
            Self::Cooler(slot) => slot.installed.take().map(OwnedComponent::Cooler),
            Self::Ram(slot) => slot.installed.take().map(OwnedComponent::Ram),
            Self::Storage(slot) => slot.installed.take().map(OwnedComponent::Storage),
            Self::Network(slot) => slot.installed.take().map(OwnedComponent::Network),
        }
    }
}

/// Simple enum for categorizing slot types (no data)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSlotKind {
    Cpu,
    Cooler,
    Ram,
    Storage,
    Network,
}

impl ComponentSlotKind {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Cooler => "Cooler",
            Self::Ram => "RAM",
            Self::Storage => "Storage",
            Self::Network => "Network",
        }
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

    /// Count total slots of a specific kind
    pub fn slot_count(&self, kind: ComponentSlotKind) -> usize {
        self.slots.iter().filter(|s| s.kind() == kind).count()
    }

    /// Count filled slots of a specific kind
    pub fn filled_count(&self, kind: ComponentSlotKind) -> usize {
        self.slots
            .iter()
            .filter(|s| s.kind() == kind && s.is_filled())
            .count()
    }

    /// Count empty slots of a specific kind
    pub fn empty_count(&self, kind: ComponentSlotKind) -> usize {
        self.slot_count(kind) - self.filled_count(kind)
    }

    /// Check if PC is functional (has at least 1 of each required component installed)
    pub fn is_functional(&self) -> bool {
        let has_cpu = self
            .slots
            .iter()
            .any(|s| matches!(s, ComponentSlotType::Cpu(c) if c.installed.is_some()));
        let has_cooler = self
            .slots
            .iter()
            .any(|s| matches!(s, ComponentSlotType::Cooler(c) if c.installed.is_some()));
        let has_ram = self
            .slots
            .iter()
            .any(|s| matches!(s, ComponentSlotType::Ram(r) if r.installed.is_some()));
        let has_storage = self
            .slots
            .iter()
            .any(|s| matches!(s, ComponentSlotType::Storage(st) if st.installed.is_some()));
        let has_network = self
            .slots
            .iter()
            .any(|s| matches!(s, ComponentSlotType::Network(n) if n.installed.is_some()));

        has_cpu && has_cooler && has_ram && has_storage && has_network
    }

    /// Find first empty slot compatible with the given component
    pub fn find_empty_compatible_slot(
        &mut self,
        component: &OwnedComponent,
    ) -> Option<&mut ComponentSlotType> {
        self.slots
            .iter_mut()
            .find(|s| !s.is_filled() && s.is_compatible(component))
    }

    /// Count empty slots compatible with a component
    pub fn count_empty_compatible_slots(&self, component: &OwnedComponent) -> usize {
        self.slots
            .iter()
            .filter(|s| !s.is_filled() && s.is_compatible(component))
            .count()
    }

    /// Install a component into the first compatible empty slot
    pub fn install(&mut self, component: OwnedComponent) -> Result<(), String> {
        let slot = self
            .find_empty_compatible_slot(&component)
            .ok_or_else(|| format!("No compatible empty slot for {}", component.name()))?;
        slot.install(component)?;
        Ok(())
    }

    /// Install a component into a specific slot index, returning old component if any
    pub fn install_at(
        &mut self,
        index: usize,
        component: OwnedComponent,
    ) -> Result<Option<OwnedComponent>, String> {
        let slot = self
            .slots
            .get_mut(index)
            .ok_or_else(|| format!("Invalid slot index: {index}"))?;
        slot.install(component)
    }

    /// Uninstall component from a specific slot index
    pub fn uninstall_at(&mut self, index: usize) -> Option<OwnedComponent> {
        self.slots
            .get_mut(index)
            .and_then(ComponentSlotType::uninstall)
    }

    /// Get installed CPU (if any)
    pub fn cpu(&self) -> Option<&Cpu> {
        self.slots.iter().find_map(|s| match s {
            ComponentSlotType::Cpu(slot) => slot.installed.as_ref(),
            _ => None,
        })
    }

    /// Get installed cooler (if any)
    pub fn cooler(&self) -> Option<&Cooler> {
        self.slots.iter().find_map(|s| match s {
            ComponentSlotType::Cooler(slot) => slot.installed.as_ref(),
            _ => None,
        })
    }

    /// Get all installed RAM modules
    pub fn rams(&self) -> Vec<&Ram> {
        self.slots
            .iter()
            .filter_map(|s| match s {
                ComponentSlotType::Ram(slot) => slot.installed.as_ref(),
                _ => None,
            })
            .collect()
    }

    /// Get total installed RAM capacity in MB
    pub fn total_ram_mb(&self) -> u32 {
        self.rams().iter().map(|r| r.capacity_mb).sum()
    }

    /// Get all installed storage devices
    pub fn storages(&self) -> Vec<&Storage> {
        self.slots
            .iter()
            .filter_map(|s| match s {
                ComponentSlotType::Storage(slot) => slot.installed.as_ref(),
                _ => None,
            })
            .collect()
    }

    /// Get total storage capacity in MB
    pub fn total_storage_mb(&self) -> u32 {
        self.storages().iter().map(|s| s.capacity_mb).sum()
    }

    /// Get all installed network cards
    pub fn networks(&self) -> Vec<&NetworkCard> {
        self.slots
            .iter()
            .filter_map(|s| match s {
                ComponentSlotType::Network(slot) => slot.installed.as_ref(),
                _ => None,
            })
            .collect()
    }

    /// Get best network speed in kbps
    pub fn best_network_speed(&self) -> u32 {
        self.networks()
            .iter()
            .map(|n| n.speed_kbps)
            .max()
            .unwrap_or(0)
    }

    /// Get effective CPU frequency considering cooler
    pub fn effective_cpu_freq(&self) -> u32 {
        let Some(cpu) = self.cpu() else { return 0 };
        let cooler_tdp = self.cooler().map_or(0, |c| c.max_tdp);
        cpu.effective_freq(cooler_tdp)
    }

    /// Get compute power score
    pub fn compute_power(&self) -> u32 {
        let Some(cpu) = self.cpu() else { return 0 };
        let cooler_tdp = self.cooler().map_or(0, |c| c.max_tdp);
        cpu.compute_power(cooler_tdp)
    }
}

// ============================================================================
// Complete PC Configuration (Legacy - for compatibility)
// ============================================================================

/// A complete PC configuration with all components (legacy struct)
/// Used primarily for computing aggregate stats from a functional motherboard
#[derive(Debug, Clone)]
pub struct PC {
    pub motherboard: Motherboard,
}

impl PC {
    /// Create PC from a functional motherboard
    pub fn from_motherboard(motherboard: Motherboard) -> Option<Self> {
        if motherboard.is_functional() {
            Some(Self { motherboard })
        } else {
            None
        }
    }

    /// Get effective CPU frequency considering cooler
    pub fn effective_cpu_freq(&self) -> u32 {
        self.motherboard.effective_cpu_freq()
    }

    /// Get compute power score
    pub fn compute_power(&self) -> u32 {
        self.motherboard.compute_power()
    }

    /// Get memory bandwidth score
    #[allow(clippy::cast_possible_truncation)]
    pub fn memory_score(&self) -> u32 {
        let rams = self.motherboard.rams();
        if rams.is_empty() {
            return 0;
        }
        let total_capacity: u32 = rams.iter().map(|r| r.capacity_mb).sum();
        let avg_speed: u32 = rams.iter().map(|r| r.speed_mhz).sum::<u32>() / rams.len() as u32;
        total_capacity / 256 * avg_speed / 1000
    }

    /// Get storage speed score
    pub fn storage_score(&self) -> u32 {
        self.motherboard
            .storages()
            .iter()
            .map(|s| u32::midpoint(s.read_speed_mbps, s.write_speed_mbps))
            .max()
            .unwrap_or(0)
    }

    /// Get network speed
    pub fn network_speed(&self) -> u32 {
        self.motherboard.best_network_speed()
    }

    /// Overall power score for tool effectiveness
    pub fn overall_power(&self) -> u32 {
        self.compute_power() + self.memory_score() + self.storage_score() / 10
    }
}

// ============================================================================
// Component Inventory & Assembly System
// ============================================================================

/// Identifies which component category we're working with (includes Motherboard)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSlot {
    Cpu,
    Cooler,
    Motherboard,
    Ram,
    Storage,
    Network,
}

impl ComponentSlot {
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

    /// Convert to `ComponentSlotKind` (for non-motherboard slots)
    pub const fn to_kind(&self) -> Option<ComponentSlotKind> {
        match self {
            Self::Cpu => Some(ComponentSlotKind::Cpu),
            Self::Cooler => Some(ComponentSlotKind::Cooler),
            Self::Ram => Some(ComponentSlotKind::Ram),
            Self::Storage => Some(ComponentSlotKind::Storage),
            Self::Network => Some(ComponentSlotKind::Network),
            Self::Motherboard => None,
        }
    }
}

/// A component that can be stored in inventory
#[derive(Debug, Clone)]
pub enum OwnedComponent {
    Cpu(Cpu),
    Cooler(Cooler),
    Motherboard(Motherboard),
    Ram(Ram),
    Storage(Storage),
    Network(NetworkCard),
}

impl OwnedComponent {
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

    pub const fn slot(&self) -> ComponentSlot {
        match self {
            Self::Cpu(_) => ComponentSlot::Cpu,
            Self::Cooler(_) => ComponentSlot::Cooler,
            Self::Motherboard(_) => ComponentSlot::Motherboard,
            Self::Ram(_) => ComponentSlot::Ram,
            Self::Storage(_) => ComponentSlot::Storage,
            Self::Network(_) => ComponentSlot::Network,
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
}

/// Player's component inventory (purchased but not installed)
#[derive(Debug, Clone, Default)]
pub struct ComponentInventory {
    pub cpus: Vec<Cpu>,
    pub coolers: Vec<Cooler>,
    pub motherboards: Vec<Motherboard>,
    pub rams: Vec<Ram>,
    pub storage: Vec<Storage>,
    pub networks: Vec<NetworkCard>,
}

impl ComponentInventory {
    pub fn add(&mut self, component: OwnedComponent) {
        match component {
            OwnedComponent::Cpu(c) => self.cpus.push(c),
            OwnedComponent::Cooler(c) => self.coolers.push(c),
            OwnedComponent::Motherboard(m) => self.motherboards.push(m),
            OwnedComponent::Ram(r) => self.rams.push(r),
            OwnedComponent::Storage(s) => self.storage.push(s),
            OwnedComponent::Network(n) => self.networks.push(n),
        }
    }

    pub fn remove_by_id(&mut self, slot: ComponentSlot, id: &str) -> Option<OwnedComponent> {
        match slot {
            ComponentSlot::Cpu => {
                let idx = self.cpus.iter().position(|c| c.id == id)?;
                Some(OwnedComponent::Cpu(self.cpus.remove(idx)))
            }
            ComponentSlot::Cooler => {
                let idx = self.coolers.iter().position(|c| c.id == id)?;
                Some(OwnedComponent::Cooler(self.coolers.remove(idx)))
            }
            ComponentSlot::Motherboard => {
                let idx = self.motherboards.iter().position(|m| m.id == id)?;
                Some(OwnedComponent::Motherboard(self.motherboards.remove(idx)))
            }
            ComponentSlot::Ram => {
                let idx = self.rams.iter().position(|r| r.id == id)?;
                Some(OwnedComponent::Ram(self.rams.remove(idx)))
            }
            ComponentSlot::Storage => {
                let idx = self.storage.iter().position(|s| s.id == id)?;
                Some(OwnedComponent::Storage(self.storage.remove(idx)))
            }
            ComponentSlot::Network => {
                let idx = self.networks.iter().position(|n| n.id == id)?;
                Some(OwnedComponent::Network(self.networks.remove(idx)))
            }
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.cpus.is_empty()
            && self.coolers.is_empty()
            && self.motherboards.is_empty()
            && self.rams.is_empty()
            && self.storage.is_empty()
            && self.networks.is_empty()
    }

    pub const fn total_count(&self) -> usize {
        self.cpus.len()
            + self.coolers.len()
            + self.motherboards.len()
            + self.rams.len()
            + self.storage.len()
            + self.networks.len()
    }
}

/// PC with optional motherboard (motherboard contains all component slots)
#[derive(Debug, Clone, Default)]
pub struct AssembledPC {
    pub motherboard: Option<Motherboard>,
}

/// Result of attempting to install a component
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Warning messages about the installation
    pub warnings: Vec<String>,
    /// Whether the PC will be functional after install
    pub pc_functional: bool,
    /// Whether the install is blocked (incompatible)
    pub blocked: bool,
}

impl AssembledPC {
    /// Create from a motherboard
    pub const fn from_motherboard(motherboard: Motherboard) -> Self {
        Self {
            motherboard: Some(motherboard),
        }
    }

    /// Try to convert to a complete PC (returns None if not functional)
    pub fn to_complete_pc(&self) -> Option<PC> {
        let mb = self.motherboard.as_ref()?;
        PC::from_motherboard(mb.clone())
    }

    /// Check if PC is functional (motherboard exists and has required components)
    pub fn is_functional(&self) -> bool {
        self.motherboard
            .as_ref()
            .is_some_and(Motherboard::is_functional)
    }

    /// Check if the current configuration is valid
    pub fn is_valid(&self) -> bool {
        self.is_functional()
    }

    /// Get list of component categories that need at least one component
    pub fn empty_slot_kinds(&self) -> Vec<ComponentSlotKind> {
        let Some(mb) = &self.motherboard else {
            return vec![
                ComponentSlotKind::Cpu,
                ComponentSlotKind::Cooler,
                ComponentSlotKind::Ram,
                ComponentSlotKind::Storage,
                ComponentSlotKind::Network,
            ];
        };

        let mut empty = Vec::new();
        if mb.filled_count(ComponentSlotKind::Cpu) == 0 {
            empty.push(ComponentSlotKind::Cpu);
        }
        if mb.filled_count(ComponentSlotKind::Cooler) == 0 {
            empty.push(ComponentSlotKind::Cooler);
        }
        if mb.filled_count(ComponentSlotKind::Ram) == 0 {
            empty.push(ComponentSlotKind::Ram);
        }
        if mb.filled_count(ComponentSlotKind::Storage) == 0 {
            empty.push(ComponentSlotKind::Storage);
        }
        if mb.filled_count(ComponentSlotKind::Network) == 0 {
            empty.push(ComponentSlotKind::Network);
        }
        empty
    }

    /// Preview what happens when installing a component
    pub fn preview_install(&self, component: &OwnedComponent) -> InstallResult {
        let mut warnings = Vec::new();
        let mut blocked = false;

        // Installing a new motherboard
        if let OwnedComponent::Motherboard(new_mb) = component {
            // Check compatibility with inventory components that might be installed later
            // For now, motherboard can always be installed
            if self.motherboard.is_some() {
                warnings.push("Will replace current motherboard".to_string());
            }
            return InstallResult {
                warnings,
                pc_functional: new_mb.is_functional(),
                blocked: false,
            };
        }

        // For other components, need a motherboard first
        let Some(mb) = &self.motherboard else {
            return InstallResult {
                warnings: vec!["✗ BLOCKED: No motherboard installed".to_string()],
                pc_functional: false,
                blocked: true,
            };
        };

        // Check if there's a compatible slot
        if mb.count_empty_compatible_slots(component) == 0 {
            // Check if it's a compatibility issue or just no slots
            let kind = component.slot().to_kind();
            if let Some(k) = kind {
                if mb.empty_count(k) == 0 {
                    warnings.push(format!("✗ BLOCKED: All {} slots are full", k.name()));
                } else {
                    warnings.push(format!(
                        "✗ BLOCKED: Not compatible with any {} slot",
                        k.name()
                    ));
                }
            }
            blocked = true;
        }

        // Check if PC will be functional after install
        let pc_functional = if blocked {
            false
        } else {
            let mut test_mb = mb.clone();
            test_mb.install(component.clone()).is_ok() && test_mb.is_functional()
        };

        InstallResult {
            warnings,
            pc_functional,
            blocked,
        }
    }

    /// Install a component into the motherboard
    pub fn install_component(
        &mut self,
        component: OwnedComponent,
    ) -> Result<Option<OwnedComponent>, String> {
        // Special case: installing a motherboard
        if let OwnedComponent::Motherboard(new_mb) = component {
            let old = self.motherboard.replace(new_mb);
            return Ok(old.map(OwnedComponent::Motherboard));
        }

        // For other components, install into motherboard
        let mb = self
            .motherboard
            .as_mut()
            .ok_or("No motherboard installed")?;

        // Find compatible empty slot and install
        let slot = mb
            .find_empty_compatible_slot(&component)
            .ok_or_else(|| format!("No compatible empty slot for {}", component.name()))?;

        slot.install(component)
    }

    /// Uninstall a component from a specific slot index
    pub fn uninstall_at(&mut self, slot_index: usize) -> Option<OwnedComponent> {
        self.motherboard.as_mut()?.uninstall_at(slot_index)
    }

    /// Get compute power (0 if not functional)
    pub fn compute_power(&self) -> u32 {
        self.motherboard
            .as_ref()
            .map_or(0, Motherboard::compute_power)
    }

    /// Get the current socket (from motherboard)
    pub fn current_socket(&self) -> Option<CpuSocket> {
        self.motherboard.as_ref().and_then(Motherboard::socket)
    }

    /// Get the current RAM type (from motherboard)
    pub fn current_ram_type(&self) -> Option<RamType> {
        self.motherboard.as_ref().and_then(Motherboard::ram_type)
    }

    /// Get CPU if installed
    pub fn cpu(&self) -> Option<&Cpu> {
        self.motherboard.as_ref().and_then(|mb| mb.cpu())
    }

    /// Get cooler if installed
    pub fn cooler(&self) -> Option<&Cooler> {
        self.motherboard.as_ref().and_then(|mb| mb.cooler())
    }

    /// Get all installed RAM
    pub fn rams(&self) -> Vec<&Ram> {
        self.motherboard
            .as_ref()
            .map_or_else(Vec::new, |mb| mb.rams())
    }

    /// Get total RAM in MB
    pub fn total_ram_mb(&self) -> u32 {
        self.motherboard
            .as_ref()
            .map_or(0, Motherboard::total_ram_mb)
    }

    /// Get all installed storage
    pub fn storages(&self) -> Vec<&Storage> {
        self.motherboard
            .as_ref()
            .map_or_else(Vec::new, |mb| mb.storages())
    }

    /// Get total storage in MB
    pub fn total_storage_mb(&self) -> u32 {
        self.motherboard
            .as_ref()
            .map_or(0, Motherboard::total_storage_mb)
    }

    /// Get all installed network cards
    pub fn networks(&self) -> Vec<&NetworkCard> {
        self.motherboard
            .as_ref()
            .map_or_else(Vec::new, |mb| mb.networks())
    }

    /// Get best network speed
    pub fn best_network_speed(&self) -> u32 {
        self.motherboard
            .as_ref()
            .map_or(0, Motherboard::best_network_speed)
    }
}

impl ComponentInventory {
    /// Find best compatible CPU for the current PC config (highest compute power)
    pub fn best_cpu(&self, socket: Option<CpuSocket>) -> Option<&Cpu> {
        self.cpus
            .iter()
            .filter(|c| socket.is_none() || socket == Some(c.socket))
            .max_by_key(|c| u32::from(c.cores) * c.max_freq_mhz)
    }

    /// Find best compatible cooler (highest TDP support)
    pub fn best_cooler(&self, socket: Option<CpuSocket>) -> Option<&Cooler> {
        self.coolers
            .iter()
            .filter(|c| socket.is_none() || socket.is_some_and(|s| c.is_compatible(s)))
            .max_by_key(|c| c.max_tdp)
    }

    /// Find best compatible motherboard for a socket
    pub fn best_motherboard(&self, socket: Option<CpuSocket>) -> Option<&Motherboard> {
        self.motherboards
            .iter()
            .filter(|m| socket.is_none() || m.socket() == socket)
            .max_by_key(|m| m.slot_count(ComponentSlotKind::Ram))
    }

    /// Find best compatible RAM (highest capacity)
    pub fn best_ram(&self, ram_type: Option<RamType>) -> Option<&Ram> {
        self.rams
            .iter()
            .filter(|r| ram_type.is_none() || ram_type == Some(r.ram_type))
            .max_by_key(|r| r.capacity_mb)
    }

    /// Find best storage (highest capacity)
    pub fn best_storage(&self) -> Option<&Storage> {
        self.storage.iter().max_by_key(|s| s.capacity_mb)
    }

    /// Find best network (highest speed)
    pub fn best_network(&self) -> Option<&NetworkCard> {
        self.networks.iter().max_by_key(|n| n.speed_kbps)
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
                    installed: None,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::Socket478],
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 1024,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 1024,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::IDE,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::IDE,
                    installed: None,
                }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
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
                    installed: None,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA775],
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR3,
                    max_capacity_mb: 2048,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
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
                    installed: None,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA1200],
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
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
                    installed: None,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::LGA1700],
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR5,
                    max_capacity_mb: 65536,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
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
                    installed: None,
                }),
                ComponentSlotType::Cooler(CoolerSlot {
                    compatible_sockets: &[CpuSocket::AM4],
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Ram(RamSlot {
                    ram_type: RamType::DDR4,
                    max_capacity_mb: 32768,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::NVMe,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Storage(StorageSlot {
                    slot_type: StorageSlotType::SATA,
                    installed: None,
                }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
                ComponentSlotType::Network(NetworkSlot { installed: None }),
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
        price: 0,
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
        price: 0,
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

// ============================================================================
// Helper functions
// ============================================================================

pub fn find_cpu(id: &str) -> Option<&'static Cpu> {
    CPUS.iter().find(|c| c.id == id)
}

pub fn find_cooler(id: &str) -> Option<&'static Cooler> {
    COOLERS.iter().find(|c| c.id == id)
}

pub fn find_motherboard(id: &str) -> Option<Motherboard> {
    MOTHERBOARDS.iter().find(|m| m.id == id).cloned()
}

pub fn find_ram(id: &str) -> Option<&'static Ram> {
    RAMS.iter().find(|r| r.id == id)
}

pub fn find_storage(id: &str) -> Option<&'static Storage> {
    STORAGES.iter().find(|s| s.id == id)
}

pub fn find_network(id: &str) -> Option<&'static NetworkCard> {
    NETWORKS.iter().find(|n| n.id == id)
}

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
