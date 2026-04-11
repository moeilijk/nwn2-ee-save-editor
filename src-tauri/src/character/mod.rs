//! Character module - idiomatic Rust character representation
//!
//! Replaces the 11-manager architecture with a single Character struct
//! that directly owns GFF data without Arc<RwLock<>> overhead.

use crate::parsers::gff::GffValue;
use indexmap::IndexMap;
use tracing::{debug, instrument};

pub mod abilities;
pub mod classes;
pub mod combat;
pub mod combat_summary;
pub mod error;
pub(crate) mod feats;
pub mod gff_helpers;
mod identity;
mod inventory;
pub mod overview;
mod race;
pub mod save_summary;
pub mod saves;
mod skills;
mod spells;
pub mod types;

pub use abilities::{AbilitiesState, AbilityIncrease, AbilityPointsSummary};
pub use classes::ClassesState;
pub use classes::{
    AlignmentRestriction, BabType, ClassEntry, ClassInfo, ClassSummaryEntry, LevelHistoryEntry,
    PrestigeClassOption, PrestigeClassValidation, PrestigeRequirements, SkillRankEntry, XpProgress,
};
pub use combat::{CombatStats, DamageBonuses};
pub use combat_summary::{
    ACBreakdown, ArmorClass, AttackBonuses, AttackBreakdown, CombatManeuverBonus, CombatSummary,
    DamageReduction, Initiative, InitiativeChange, MovementSpeed, NaturalArmorChange,
};
pub use error::CharacterError;
pub use feats::{
    DomainInfo, FeatAvailability, FeatCategory, FeatEntry, FeatInfo, FeatSlots, FeatSource,
    FeatSummary, FeatType, FeatsState, PrerequisiteResult,
};
pub use identity::Alignment;
pub use inventory::{
    AddItemResult, BaseItemData as CharacterBaseItemData, BasicItemInfo, DecodedPropertyInfo,
    EncumbranceInfo, EncumbranceStatus, EquipResult, EquipmentSlot, EquipmentSlotInfo,
    EquipmentSummary, FullEncumbrance, FullEquippedItem, FullInventoryItem, FullInventorySummary,
    InventoryItem, ItemProficiencyInfo, ProficiencyRequirement, RemoveItemResult, UnequipResult,
    WeightStatus,
};
pub use overview::OverviewState;
pub use race::{RacialProperties, SizeCategory, SubraceInfo};
pub use save_summary::{SaveBreakdown, SaveChange, SaveCheck, SaveSummary, SaveType, SavingThrows};
pub use saves::SaveBonuses;
pub use skills::{ABLE_LEARNER_FEAT_ID, SkillPointsSummary, SkillSummaryEntry};
pub use spells::{
    CasterClassSummary, KnownSpellEntry, MAX_SPELL_LEVEL, MemorizedSpellEntry, MemorizedSpellRaw,
    MetamagicFeat, SpellDetails, SpellSummary, SpellcastingClass, SpellsState,
    is_displayable_spell, is_mod_prefixed_name,
};
pub use types::*;

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
            && races.get_by_id(race_id.0).is_none()
        {
            result.add_error(format!("Invalid race ID: {}", race_id.0));
        }

        let level = self.total_level();
        if level < 1 {
            result.add_error("Character level is less than 1");
        }
        if level > types::MAX_TOTAL_LEVEL {
            result.add_error(format!(
                "Character level {} exceeds maximum {}",
                level,
                types::MAX_TOTAL_LEVEL
            ));
        }

        let alignment = self.alignment();
        if alignment.law_chaos < types::ALIGNMENT_MIN || alignment.law_chaos > types::ALIGNMENT_MAX
        {
            result.add_error(format!(
                "Law/Chaos alignment {} out of range",
                alignment.law_chaos
            ));
        }
        if alignment.good_evil < types::ALIGNMENT_MIN || alignment.good_evil > types::ALIGNMENT_MAX
        {
            result.add_error(format!(
                "Good/Evil alignment {} out of range",
                alignment.good_evil
            ));
        }

        result
    }

    /// Comprehensive pre-save legality validation across all character domains.
    pub fn validate_for_save(&self, game_data: &crate::loaders::GameData) -> SaveLegalityReport {
        let mut report = SaveLegalityReport::ok();

        let core = self.validate(game_data);
        for warning in core.warnings {
            report.add_warning(SaveLegalityDomain::Core, "core_warning", warning, None);
        }
        for error in core.errors {
            report.add_error(SaveLegalityDomain::Core, "core_invalid", error, None);
        }

        let race_validation = self.validate_race(game_data);
        for error in race_validation.errors {
            report.add_error(SaveLegalityDomain::Race, "race_invalid", error, Some("Race".to_string()));
        }

        let class_entries = self.class_entries();
        if class_entries.len() > types::MAX_CLASSES {
            report.add_error(
                SaveLegalityDomain::Class,
                "class_count_exceeded",
                format!(
                    "Character has {} classes; maximum allowed is {}",
                    class_entries.len(),
                    types::MAX_CLASSES
                ),
                Some("ClassList".to_string()),
            );
        }

        let total_level: i32 = class_entries.iter().map(|entry| entry.level).sum();
        if total_level > types::MAX_TOTAL_LEVEL {
            report.add_error(
                SaveLegalityDomain::Class,
                "total_level_exceeded",
                format!(
                    "Character level {} exceeds maximum {}",
                    total_level,
                    types::MAX_TOTAL_LEVEL
                ),
                Some("ClassList".to_string()),
            );
        }

        for class_entry in &class_entries {
            let prestige = self.validate_prestige_class_requirements(class_entry.class_id, game_data);
            if !prestige.can_take && !prestige.missing_requirements.is_empty() {
                report.add_error(
                    SaveLegalityDomain::Class,
                    "prestige_requirements_not_met",
                    format!(
                        "Class {} fails prerequisites: {}",
                        class_entry.class_id.0,
                        prestige.missing_requirements.join(", ")
                    ),
                    Some("ClassList".to_string()),
                );
            }
        }

        for feat_id in self.feat_ids() {
            let prereq = self.validate_feat_prerequisites(feat_id, game_data);
            if !prereq.can_take {
                report.add_error(
                    SaveLegalityDomain::Feat,
                    "feat_prerequisites_not_met",
                    format!(
                        "Feat {} is invalid: {}",
                        feat_id.0,
                        prereq.missing_requirements.join(", ")
                    ),
                    Some("FeatList".to_string()),
                );
            }
        }

        for error in self.validate_skills() {
            report.add_error(
                SaveLegalityDomain::Skills,
                "skill_structure_invalid",
                error,
                Some("SkillList".to_string()),
            );
        }

        report.merge(self.validate_level_history_consistency());
        report.merge(self.validate_skill_budget_consistency(game_data));

        for error in self.validate_spells(game_data) {
            report.add_error(
                SaveLegalityDomain::Spells,
                "spell_invalid",
                error,
                Some("ClassList[].KnownList".to_string()),
            );
        }

        let inventory = self.validate_inventory();
        for error in inventory.errors {
            report.add_error(
                SaveLegalityDomain::Inventory,
                "inventory_invalid",
                error,
                Some("ItemList".to_string()),
            );
        }
        for warning in inventory.warnings {
            report.add_warning(
                SaveLegalityDomain::Inventory,
                "inventory_warning",
                warning,
                Some("ItemList".to_string()),
            );
        }

        report
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
