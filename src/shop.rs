//! PC Component Shop - Buy components to add to your inventory

use crate::app::ShopCategory;
use crate::model::{
    AssembledPC, COOLERS, CPUS, ComponentSlotKind, Cooler, Cpu, MOTHERBOARDS, Motherboard,
    NETWORKS, NetworkCard, OwnedComponent, RAMS, Ram, STORAGES, Storage,
};
use crate::player::Player;

/// The PC Component Shop
pub struct Shop;

impl Shop {
    /// Get ALL CPUs available for purchase (no filtering)
    pub fn all_cpus() -> Vec<&'static Cpu> {
        CPUS.iter().filter(|c| c.price > 0).collect()
    }

    /// Get ALL coolers available for purchase
    pub fn all_coolers() -> Vec<&'static Cooler> {
        COOLERS.iter().filter(|c| c.price > 0).collect()
    }

    /// Get ALL motherboards available for purchase
    pub fn all_motherboards() -> Vec<Motherboard> {
        MOTHERBOARDS
            .iter()
            .filter(|m| m.price > 0)
            .cloned()
            .collect()
    }

    /// Get ALL RAM available for purchase
    pub fn all_ram() -> Vec<&'static Ram> {
        RAMS.iter().filter(|r| r.price > 0).collect()
    }

    /// Get ALL storage available for purchase
    pub fn all_storage() -> Vec<&'static Storage> {
        STORAGES.iter().filter(|s| s.price > 0).collect()
    }

    /// Get ALL network cards available for purchase
    pub fn all_network() -> Vec<&'static NetworkCard> {
        NETWORKS.iter().filter(|n| n.price > 0).collect()
    }

    /// Get items for a specific category (all items, no filtering)
    pub fn items_for_category(category: ShopCategory) -> Vec<ShopItem> {
        match category {
            ShopCategory::Cpu => Self::all_cpus().into_iter().map(ShopItem::Cpu).collect(),
            ShopCategory::Cooler => Self::all_coolers()
                .into_iter()
                .map(ShopItem::Cooler)
                .collect(),
            ShopCategory::Motherboard => Self::all_motherboards()
                .into_iter()
                .map(ShopItem::Motherboard)
                .collect(),
            ShopCategory::Ram => Self::all_ram().into_iter().map(ShopItem::Ram).collect(),
            ShopCategory::Storage => Self::all_storage()
                .into_iter()
                .map(ShopItem::Storage)
                .collect(),
            ShopCategory::Network => Self::all_network()
                .into_iter()
                .map(ShopItem::Network)
                .collect(),
        }
    }

    /// Get compatibility warnings for purchasing a component
    /// These are informational - they don't block purchase
    pub fn purchase_warnings(pc: &AssembledPC, item: &ShopItem) -> Vec<String> {
        let mut warnings = Vec::new();

        match item {
            ShopItem::Cpu(cpu) => {
                if let Some(mb) = &pc.motherboard
                    && let Some(mb_socket) = mb.socket()
                    && cpu.socket != mb_socket
                {
                    warnings.push(format!(
                        "Requires {} motherboard (you have {})",
                        cpu.socket, mb_socket
                    ));
                }
                if let Some(cooler) = pc.cooler() {
                    if !cooler.is_compatible(cpu.socket) {
                        warnings.push(format!("Your cooler doesn't support {} socket", cpu.socket));
                    }
                    if cpu.tdp_watts > cooler.max_tdp {
                        warnings.push(format!(
                            "TDP ({} W) exceeds your cooler ({} W) - will throttle",
                            cpu.tdp_watts, cooler.max_tdp
                        ));
                    }
                }
            }
            ShopItem::Cooler(cooler) => {
                if let Some(mb) = &pc.motherboard
                    && let Some(mb_socket) = mb.socket()
                    && !cooler.is_compatible(mb_socket)
                {
                    warnings.push(format!("Doesn't support your {mb_socket} socket"));
                }
            }
            ShopItem::Motherboard(mb) => {
                let new_socket = mb.socket();
                let new_ram_type = mb.ram_type();

                if let Some(cpu) = pc.cpu()
                    && let Some(socket) = new_socket
                    && socket != cpu.socket
                {
                    warnings.push(format!("Requires {} CPU (you have {})", socket, cpu.socket));
                }
                for ram in pc.rams() {
                    if let Some(ram_type) = new_ram_type
                        && ram_type != ram.ram_type
                    {
                        warnings.push(format!(
                            "Requires {} RAM (you have {})",
                            ram_type, ram.ram_type
                        ));
                        break; // Only warn once
                    }
                }
                if let Some(cooler) = pc.cooler()
                    && let Some(socket) = new_socket
                    && !cooler.is_compatible(socket)
                {
                    warnings.push(format!("Your cooler doesn't support {socket} socket"));
                }
            }
            ShopItem::Ram(ram) => {
                if let Some(mb) = &pc.motherboard
                    && let Some(mb_ram_type) = mb.ram_type()
                    && ram.ram_type != mb_ram_type
                {
                    warnings.push(format!(
                        "Requires {} motherboard (you have {})",
                        ram.ram_type, mb_ram_type
                    ));
                }
            }
            ShopItem::Storage(_) | ShopItem::Network(_) => {
                // Always compatible
            }
        }

        warnings
    }

    /// Check if a purchase should be blocked
    pub fn check_purchase_blocked(player: &Player, item: &ShopItem) -> Option<String> {
        let price = item.price();

        // Check credits
        if player.credits < price {
            return Some(format!(
                "Insufficient credits: need {}c, have {}c",
                price, player.credits
            ));
        }

        // Block duplicate motherboard
        if let ShopItem::Motherboard(mb) = item
            && player.owns_motherboard(mb.id)
        {
            return Some(format!("Already own motherboard: {}", mb.name));
        }

        // Block if no slots available for component type
        if let Some(mb) = player.get_motherboard() {
            let kind = match item {
                ShopItem::Cpu(_) => Some(ComponentSlotKind::Cpu),
                ShopItem::Cooler(_) => Some(ComponentSlotKind::Cooler),
                ShopItem::Ram(_) => Some(ComponentSlotKind::Ram),
                ShopItem::Storage(_) => Some(ComponentSlotKind::Storage),
                ShopItem::Network(_) => Some(ComponentSlotKind::Network),
                ShopItem::Motherboard(_) => None,
            };

            if let Some(k) = kind {
                let max_slots = mb.slot_count(k);
                let owned = player.count_owned_of(item);
                if owned >= max_slots {
                    return Some(format!(
                        "No {} slots available (max: {})",
                        k.name(),
                        max_slots
                    ));
                }
            }
        }

        None
    }

    /// Purchase a component - adds to inventory
    pub fn purchase(player: &mut Player, item: &ShopItem) -> Result<PurchaseResult, String> {
        // Check for blocking conditions
        if let Some(block_reason) = Self::check_purchase_blocked(player, item) {
            return Err(block_reason);
        }

        let price = item.price();

        // Get warnings (informational only)
        let warnings = Self::purchase_warnings(&player.assembled_pc, item);

        // Deduct credits
        player.credits -= price;

        // Add to inventory
        let component = item.to_owned_component();
        let name = component.name().to_string();
        player.inventory.add(component);

        Ok(PurchaseResult {
            item_name: name,
            price,
            warnings,
        })
    }
}

/// Result of a purchase
#[derive(Debug)]
pub struct PurchaseResult {
    pub item_name: String,
    pub price: u32,
    pub warnings: Vec<String>,
}

/// A shop item wrapping any component type
#[derive(Debug, Clone)]
pub enum ShopItem {
    Cpu(&'static Cpu),
    Cooler(&'static Cooler),
    Motherboard(Motherboard),
    Ram(&'static Ram),
    Storage(&'static Storage),
    Network(&'static NetworkCard),
}

impl ShopItem {
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
                let ram_slots = m.slot_count(ComponentSlotKind::Ram);
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
            _ => None,
        }
    }

    /// Convert to an owned component
    pub fn to_owned_component(&self) -> OwnedComponent {
        match self {
            Self::Cpu(c) => OwnedComponent::Cpu((*c).clone()),
            Self::Cooler(c) => OwnedComponent::Cooler((*c).clone()),
            Self::Motherboard(m) => OwnedComponent::Motherboard(m.clone()),
            Self::Ram(r) => OwnedComponent::Ram((*r).clone()),
            Self::Storage(s) => OwnedComponent::Storage((*s).clone()),
            Self::Network(n) => OwnedComponent::Network((*n).clone()),
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
