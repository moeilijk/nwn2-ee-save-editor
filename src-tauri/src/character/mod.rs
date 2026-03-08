//! Character module - idiomatic Rust character representation
//!
//! Replaces the 11-manager architecture with a single Character struct
//! that directly owns GFF data without Arc<RwLock<>> overhead.

use indexmap::IndexMap;
use tracing::{debug, instrument};
use crate::parsers::gff::GffValue;

pub mod error;
pub mod types;
pub mod gff_helpers;
pub mod abilities;
pub mod saves;
pub mod combat;
pub mod combat_summary;
pub mod save_summary;
mod race;
mod identity;
pub mod classes;
pub(crate) mod feats;
mod skills;
mod inventory;
mod spells;
pub mod overview;

pub use error::CharacterError;
pub use types::*;
pub use race::{SizeCategory, SubraceInfo};
pub use identity::Alignment;
pub use classes::{ClassEntry, LevelHistoryEntry, SkillRankEntry, ClassInfo, BabType, XpProgress, ClassSummaryEntry, AlignmentRestriction, PrestigeRequirements, PrestigeClassValidation, PrestigeClassOption};
pub use saves::SaveBonuses;
pub use combat::{CombatStats, DamageBonuses};
pub use combat_summary::{
    ArmorClass, ACBreakdown, AttackBonuses, AttackBreakdown, Initiative,
    CombatManeuverBonus, MovementSpeed, DamageReduction, CombatSummary,
    NaturalArmorChange, InitiativeChange,
};
pub use save_summary::{
    SaveType, SavingThrows, SaveBreakdown, SaveSummary, SaveCheck, SaveChange,
};
pub use feats::{FeatEntry, FeatSource, FeatSlots, PrerequisiteResult, FeatInfo, FeatSummary, FeatCategory, FeatType, DomainInfo, FeatsState, FeatAvailability};
pub use inventory::{
    BasicItemInfo, WeightStatus, BaseItemData as CharacterBaseItemData,
    EquipmentSummary, EquipmentSlotInfo, EncumbranceInfo, EquipmentSlot,
    InventoryItem, EquipResult, UnequipResult, AddItemResult, RemoveItemResult,
    EncumbranceStatus, ItemProficiencyInfo, ProficiencyRequirement,
    FullInventorySummary, FullInventoryItem, FullEquippedItem, FullEncumbrance,
    DecodedPropertyInfo,
};
pub use spells::{
    MemorizedSpellRaw, SpellSummary, SpellDetails, SpellsState, MAX_SPELL_LEVEL,
    SpellcastingClass, KnownSpellEntry, MemorizedSpellEntry, CasterClassSummary, MetamagicFeat,
    is_displayable_spell, is_mod_prefixed_name,
};
pub use skills::{SkillSummaryEntry, SkillPointsSummary, ABLE_LEARNER_FEAT_ID};
pub use abilities::{AbilityIncrease, AbilityPointsSummary, AbilitiesState};
pub use classes::ClassesState;
pub use overview::OverviewState;

/// Character data with direct GFF ownership.
///
/// Unlike the old CharacterData/ManagerContext design, Character owns
/// its GFF fields directly without Arc<RwLock<>>. All methods are sync
/// (not async) - the single lock at AppState/SessionState level is sufficient.
pub struct Character {
    /// GFF fields - fully owned, no lazy loading
    gff: IndexMap<String, GffValue<'static>>,
    /// Track if character has been modified since load/save
    modified: bool,
}

impl Character {
    /// Create a new Character from parsed GFF fields.
    ///
    /// Uses `force_owned()` to recursively convert all LazyStruct values
    /// to StructOwned, eliminating Arc<RwLock<>> from nested data.
    #[instrument(name = "Character::from_gff", skip_all, fields(field_count = fields.len()))]
    pub fn from_gff(fields: IndexMap<String, GffValue<'static>>) -> Self {
        debug!("Converting {} GFF fields to owned values", fields.len());

        let owned_gff: IndexMap<String, GffValue<'static>> = fields
            .into_iter()
            .map(|(k, v)| (k, v.force_owned()))
            .collect();

        debug!("GFF fields converted to owned values");

        Self {
            gff: owned_gff,
            modified: false,
        }
    }

    /// Convert Character back to GFF fields for saving.
    ///
    /// Consumes the Character - caller should clone if needed.
    pub fn into_gff(self) -> IndexMap<String, GffValue<'static>> {
        self.gff
    }

    /// Get a reference to GFF fields for inspection.
    pub fn gff(&self) -> &IndexMap<String, GffValue<'static>> {
        &self.gff
    }

    /// Get a mutable reference to GFF fields.
    /// Marks the character as modified.
    pub fn gff_mut(&mut self) -> &mut IndexMap<String, GffValue<'static>> {
        self.modified = true;
        &mut self.gff
    }

    /// Check if the character has unsaved modifications.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the character as saved (clear modified flag).
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// Mark the character as modified.
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    /// Clone the GFF data (useful for saving without consuming).
    pub fn clone_gff(&self) -> IndexMap<String, GffValue<'static>> {
        self.gff.clone()
    }

    /// Basic character validation.
    ///
    /// Performs fundamental checks to ensure character integrity.
    /// Returns ValidationResult with errors/warnings.
    pub fn validate(&self, game_data: &crate::loaders::GameData) -> ValidationResult {
        let mut result = ValidationResult::ok();

        if self.first_name().is_empty() {
            result.add_warning("Character has no first name");
        }

        let race_id = self.race_id();
        if game_data.get_table("racialtypes").is_some()
            && let Some(races) = game_data.get_table("racialtypes")
                && races.get_by_id(race_id.0).is_none() {
                    result.add_error(format!("Invalid race ID: {}", race_id.0));
                }

        let level = self.total_level();
        if level < 1 {
            result.add_error("Character level is less than 1");
        }
        if level > types::MAX_TOTAL_LEVEL {
            result.add_error(format!("Character level {} exceeds maximum {}", level, types::MAX_TOTAL_LEVEL));
        }

        let alignment = self.alignment();
        if alignment.law_chaos < types::ALIGNMENT_MIN || alignment.law_chaos > types::ALIGNMENT_MAX {
            result.add_error(format!("Law/Chaos alignment {} out of range", alignment.law_chaos));
        }
        if alignment.good_evil < types::ALIGNMENT_MIN || alignment.good_evil > types::ALIGNMENT_MAX {
            result.add_error(format!("Good/Evil alignment {} out of range", alignment.good_evil));
        }

        result
    }
}

impl std::fmt::Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Character")
            .field("fields_count", &self.gff.len())
            .field("modified", &self.modified)
            .finish()
    }
}
