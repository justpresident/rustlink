//! PC Component Shop - Buy components to add to your inventory

use std::sync::LazyLock;

use crate::model::{
    COOLERS, CPUS, ComponentInventory, HardwareComponent, HardwareKind, MOTHERBOARDS, NETWORKS,
    RAMS, STORAGES,
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
    /// Checks against ALL owned items (installed + spare), not just installed
    #[allow(clippy::too_many_lines)]
    pub fn purchase_warnings(inv: &ComponentInventory, item: &HardwareComponent) -> Vec<String> {
        let mut warnings = Vec::new();

        match item {
            HardwareComponent::Cpu(cpu) => {
                // Check if ANY owned motherboard supports this CPU
                let has_compatible_mb = inv
                    .items_for(HardwareKind::Motherboard)
                    .any(|(_, c)| matches!(c, HardwareComponent::Motherboard(m) if m.socket() == Some(cpu.socket)));

                if !has_compatible_mb && inv.count_items_for(HardwareKind::Motherboard) > 0 {
                    // Get installed MB socket for the warning message
                    warnings.push(format!("Needs {} socket", cpu.socket));
                }

                // Check if ANY owned cooler supports this CPU socket and TDP
                let coolers: Vec<_> = inv
                    .items_for(HardwareKind::Cooler)
                    .filter_map(|(_, c)| match c {
                        HardwareComponent::Cooler(cooler) => Some(cooler),
                        _ => None,
                    })
                    .collect();

                if !coolers.is_empty() {
                    let has_compatible_cooler = coolers.iter().any(|c| c.is_compatible(cpu.socket));
                    if !has_compatible_cooler {
                        warnings.push(format!("Needs cooler for {} socket", cpu.socket));
                    }

                    let has_sufficient_tdp = coolers.iter().any(|c| c.max_tdp >= cpu.tdp_watts);
                    if !has_sufficient_tdp {
                        let best_tdp = coolers.iter().map(|c| c.max_tdp).max().unwrap_or(0);
                        warnings.push(format!(
                            "TDP ({} W) exceeds all owned coolers (best: {} W) - will throttle",
                            cpu.tdp_watts, best_tdp
                        ));
                    }
                }
            }
            HardwareComponent::Cooler(cooler) => {
                // Check if ANY owned motherboard is compatible
                let has_compatible_mb = inv.items_for(HardwareKind::Motherboard).any(|(_, c)| {
                    matches!(c, HardwareComponent::Motherboard(m)
                            if m.socket().is_some_and(|s| cooler.is_compatible(s)))
                });

                if !has_compatible_mb && inv.count_items_for(HardwareKind::Motherboard) > 0 {
                    let current_socket = inv
                        .installed_motherboard()
                        .and_then(super::model::hardware::Motherboard::socket)
                        .map_or_else(|| "unknown".to_string(), |s| format!("{s}"));
                    warnings.push(format!(
                        "No compatible motherboard owned (your current: {current_socket})"
                    ));
                }
            }
            HardwareComponent::Motherboard(mb) => {
                let new_socket = mb.socket();
                let new_ram_type = mb.ram_type();

                // Check if ANY owned CPU is compatible
                if let Some(socket) = new_socket {
                    let has_compatible_cpu = inv.items_for(HardwareKind::Cpu).any(
                        |(_, c)| matches!(c, HardwareComponent::Cpu(cpu) if cpu.socket == socket),
                    );

                    if !has_compatible_cpu && inv.count_items_for(HardwareKind::Cpu) > 0 {
                        let current_cpu = inv
                            .cpu()
                            .map_or_else(|| "unknown".to_string(), |c| format!("{}", c.socket));
                        warnings.push(format!(
                            "No compatible CPU owned (needs {socket}, you have {current_cpu})"
                        ));
                    }
                }

                // Check if ANY owned RAM is compatible
                if let Some(ram_type) = new_ram_type {
                    let has_compatible_ram = inv.items_for(HardwareKind::Ram).any(
                        |(_, c)| matches!(c, HardwareComponent::Ram(r) if r.ram_type == ram_type),
                    );

                    if !has_compatible_ram && inv.count_items_for(HardwareKind::Ram) > 0 {
                        let current_ram = inv
                            .rams()
                            .first()
                            .map_or_else(|| "unknown".to_string(), |r| format!("{}", r.ram_type));
                        warnings.push(format!(
                            "No compatible RAM owned (needs {ram_type}, you have {current_ram})"
                        ));
                    }
                }

                // Check if ANY owned cooler is compatible
                if let Some(socket) = new_socket {
                    let has_compatible_cooler = inv
                        .items_for(HardwareKind::Cooler)
                        .any(|(_, c)| {
                            matches!(c, HardwareComponent::Cooler(cooler) if cooler.is_compatible(socket))
                        });

                    if !has_compatible_cooler && inv.count_items_for(HardwareKind::Cooler) > 0 {
                        warnings.push(format!("No owned cooler supports {socket} socket"));
                    }
                }
            }
            HardwareComponent::Ram(ram) => {
                // Check if ANY owned motherboard supports this RAM type
                let has_compatible_mb = inv
                    .items_for(HardwareKind::Motherboard)
                    .any(|(_, c)| {
                        matches!(c, HardwareComponent::Motherboard(m) if m.ram_type() == Some(ram.ram_type))
                    });

                if !has_compatible_mb && inv.count_items_for(HardwareKind::Motherboard) > 0 {
                    let current_ram_type = inv
                        .installed_motherboard()
                        .and_then(super::model::hardware::Motherboard::ram_type)
                        .map_or_else(|| "unknown".to_string(), |r| format!("{r}"));
                    warnings.push(format!(
                        "No compatible motherboard owned (needs {}, you have {})",
                        ram.ram_type, current_ram_type
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
    /// Only blocks for insufficient credits or duplicate motherboards
    pub fn check_purchase_blocked(player: &Player, item: &HardwareComponent) -> Option<String> {
        let price = item.price();

        // Check credits
        if player.credits < price {
            return Some(format!(
                "Insufficient credits: need {}c, have {}c",
                price, player.credits
            ));
        }

        // Block duplicate motherboard (same model)
        if let HardwareComponent::Motherboard(mb) = item
            && player.owns_motherboard(mb.id)
        {
            return Some(format!("Already own motherboard: {}", mb.name));
        }

        // Don't block based on slot availability - let users buy what they want
        // Warnings will inform them about compatibility issues
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
        let warnings = Self::purchase_warnings(&player.inventory, item);

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
