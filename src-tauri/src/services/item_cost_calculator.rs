use indexmap::IndexMap;
use tracing::{trace, warn};

use crate::character::gff_helpers::gff_value_to_i32;
use crate::loaders::GameData;
use crate::parsers::gff::GffValue;
use crate::utils::parsing::row_str;

/// NWN2 cost tables use multipliers that represent thousands of gold.
const GOLD_MULTIPLIER: f64 = 1000.0;

/// Calculates item costs from 2DA tables rather than relying on stored GFF values.
///
/// Formula: TotalCost = (BaseCost * ItemMultiplier + PropertyCosts * GOLD_MULTIPLIER) * StackSize
pub struct ItemCostCalculator;

impl ItemCostCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate the total cost of an item based on its base type and properties.
    /// Returns the calculated cost, or None if calculation fails.
    pub fn calculate_item_cost(
        &self,
        item_struct: &IndexMap<String, GffValue<'static>>,
        game_data: &GameData,
    ) -> Option<u32> {
        let base_item_id = item_struct
            .get("BaseItem")
            .and_then(gff_value_to_i32)
            .unwrap_or(0);

        let stack_size = f64::from(
            item_struct
                .get("StackSize")
                .and_then(gff_value_to_i32)
                .unwrap_or(1)
                .max(1),
        );

        let Some((base_cost, item_multiplier)) = self.get_base_item_costs(base_item_id, game_data)
        else {
            trace!(base_item_id, "Failed to get base item costs");
            return None;
        };

        let property_costs = self.calculate_all_property_costs(item_struct, game_data);

        let total = (base_cost * item_multiplier + property_costs * GOLD_MULTIPLIER) * stack_size;

        Some(total.round() as u32)
    }

    fn get_base_item_costs(&self, base_item_id: i32, game_data: &GameData) -> Option<(f64, f64)> {
        let baseitems = game_data.get_table("baseitems")?;
        let row = baseitems.get_by_id(base_item_id)?;

        let base_cost = row_str(&row, "basecost")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let item_multiplier = row_str(&row, "itemmultiplier")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);

        Some((base_cost, item_multiplier))
    }

    fn calculate_all_property_costs(
        &self,
        item_struct: &IndexMap<String, GffValue<'static>>,
        game_data: &GameData,
    ) -> f64 {
        let Some(GffValue::ListOwned(props)) = item_struct.get("PropertiesList") else {
            return 0.0;
        };

        let mut total = 0.0;
        for prop in props {
            if let Some(cost) = self.calculate_property_cost(prop, game_data) {
                total += cost;
            }
        }
        total
    }

    fn calculate_property_cost(
        &self,
        prop: &IndexMap<String, GffValue<'static>>,
        game_data: &GameData,
    ) -> Option<f64> {
        let property_type = prop.get("PropertyName").and_then(gff_value_to_i32)? as u32;

        let cost_value = prop
            .get("CostValue")
            .and_then(gff_value_to_i32)
            .unwrap_or(0) as u32;

        let itempropdef = game_data.get_table("itempropdef")?;
        let prop_def_row = itempropdef.get_by_id(property_type as i32)?;

        let prop_def_cost = row_str(&prop_def_row, "cost")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);

        let cost_table_ref = row_str(&prop_def_row, "costtableresref")
            .and_then(|s| s.parse::<u32>().ok());

        let cost_multiplier = cost_table_ref
            .map(|table_id| self.lookup_cost_table_value(table_id, cost_value, game_data))
            .unwrap_or(0.0);

        Some(prop_def_cost * cost_multiplier)
    }

    fn lookup_cost_table_value(&self, table_id: u32, cost_value: u32, game_data: &GameData) -> f64 {
        let Some(iprp_costtable) = game_data.get_table("iprp_costtable") else {
            warn!("iprp_costtable.2da not loaded");
            return 0.0;
        };

        let Some(table_row) = iprp_costtable.get_by_id(table_id as i32) else {
            return 0.0;
        };

        let Some(name) = row_str(&table_row, "name") else {
            return 0.0;
        };

        if name == "****" {
            return 0.0;
        }

        let table_name = name.to_lowercase();

        let Some(cost_table) = game_data.get_table(&table_name) else {
            trace!(table_name, "Cost table not loaded");
            return 0.0;
        };

        let Some(row) = cost_table.get_by_id(cost_value as i32) else {
            return 0.0;
        };

        row_str(&row, "cost")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
    }
}

impl Default for ItemCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_creation() {
        let calc = ItemCostCalculator::new();
        assert!(std::mem::size_of_val(&calc) == 0);
    }
}
