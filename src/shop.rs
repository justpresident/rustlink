//! PC Component Shop - Buy components to add to your inventory

use std::sync::LazyLock;

use crate::model::{
    COOLERS, CPUS, HardwareComponent, HardwareKind, MOTHERBOARDS, NETWORKS, PC, RAMS, STORAGES,
};
use crate::player::Player;

/// All purchasable shop items (price > 0) as a single unified list
static SHOP_ITEMS: LazyLock<Vec<HardwareComponent>> = LazyLock::new(|| {
    let mut items = Vec::new();
    items.extend(
        CPUS.iter()
            .filter(|c| c.price > 0)
            .map(|c| HardwareComponent::Cpu(c.clone())),
    );
    items.extend(
        COOLERS
            .iter()
            .filter(|c| c.price > 0)
            .map(|c| HardwareComponent::Cooler(c.clone())),
    );
    items.extend(
        MOTHERBOARDS
            .iter()
            .filter(|m| m.price > 0)
            .map(|m| HardwareComponent::Motherboard(m.clone())),
    );
    items.extend(
        RAMS.iter()
            .filter(|r| r.price > 0)
            .map(|r| HardwareComponent::Ram(r.clone())),
    );
    items.extend(
        STORAGES
            .iter()
            .filter(|s| s.price > 0)
            .map(|s| HardwareComponent::Storage(s.clone())),
    );
    items.extend(
        NETWORKS
            .iter()
            .filter(|n| n.price > 0)
            .map(|n| HardwareComponent::Network(n.clone())),
    );
    items
});

/// The PC Component Shop
pub struct Shop;

impl Shop {
    pub fn count_items_for(kind: HardwareKind) -> usize {
        SHOP_ITEMS.iter().filter(|c| c.kind() == kind).count()
    }

    /// Get an iterator over items for a specific hardware kind
    pub fn items_for(kind: HardwareKind) -> impl Iterator<Item = &'static HardwareComponent> {
        SHOP_ITEMS.iter().filter(move |c| c.kind() == kind)
    }

    /// Get a specific item by kind and index (within that kind)
    pub fn get_at(kind: HardwareKind, index: usize) -> Option<&'static HardwareComponent> {
        Self::items_for(kind).nth(index)
    }

    /// Get compatibility warnings for purchasing a component
    /// These are informational - they don't block purchase
    pub fn purchase_warnings(pc: &PC, item: &HardwareComponent) -> Vec<String> {
        let mut warnings = Vec::new();

        match item {
            HardwareComponent::Cpu(cpu) => {
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
            HardwareComponent::Cooler(cooler) => {
                if let Some(mb) = &pc.motherboard
                    && let Some(mb_socket) = mb.socket()
                    && !cooler.is_compatible(mb_socket)
                {
                    warnings.push(format!("Doesn't support your {mb_socket} socket"));
                }
            }
            HardwareComponent::Motherboard(mb) => {
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
            HardwareComponent::Ram(ram) => {
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
            HardwareComponent::Storage(_) | HardwareComponent::Network(_) => {
                // Always compatible
            }
        }

        warnings
    }

    /// Check if a purchase should be blocked
    pub fn check_purchase_blocked(player: &Player, item: &HardwareComponent) -> Option<String> {
        let price = item.price();

        // Check credits
        if player.credits < price {
            return Some(format!(
                "Insufficient credits: need {}c, have {}c",
                price, player.credits
            ));
        }

        // Block duplicate motherboard
        if let HardwareComponent::Motherboard(mb) = item
            && player.owns_motherboard(mb.id)
        {
            return Some(format!("Already own motherboard: {}", mb.name));
        }

        // Block if no slots available for component type
        if let Some(mb) = player.get_motherboard() {
            let kind = item.kind();
            if kind.is_slot_kind() {
                let max_slots = mb.slot_count(kind);
                let owned = player.count_owned_of(item);
                if owned >= max_slots {
                    return Some(format!(
                        "No {} slots available (max: {})",
                        kind.name(),
                        max_slots
                    ));
                }
            }
        }

        None
    }

    /// Purchase a component - adds to inventory
    pub fn purchase(
        player: &mut Player,
        item: &HardwareComponent,
    ) -> Result<PurchaseResult, String> {
        // Check for blocking conditions
        if let Some(block_reason) = Self::check_purchase_blocked(player, item) {
            return Err(block_reason);
        }

        let price = item.price();

        // Get warnings (informational only)
        let warnings = Self::purchase_warnings(&player.pc, item);

        // Deduct credits
        player.credits -= price;

        // Add to inventory
        let name = item.name().to_string();
        player.inventory.add(item.clone());

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
