# Integration Tests

This directory contains integration tests organized by domain.

## Test Suites

| Suite | Command | Purpose |
|-------|---------|---------|
| **character** | `cargo test --features integration-tests --test character` | Character business logic (leveling, feats, skills, etc.) |
| **parsing** | `cargo test --features integration-tests --test parsing` | File format parsing (GFF, TLK, 2DA, ERF, XML) |
| **gamedata** | `cargo test --features integration-tests --test gamedata` | Game data inspection and validation |
| **services** | `cargo test --features integration-tests --test services` | Service layer (ResourceManager, SavegameHandler, etc.) |
| **utils** | `cargo test --features integration-tests --test utils` | Utility functions (zip, caching, path discovery) |
| **debugging** | `cargo test --features integration-tests --test debugging` | Local diagnostics for fixtures and binary format investigations |
| **file_operations** | `cargo test --features integration-tests --test file_operations` | Save/path file operation coverage |
| **save_pipeline** | `cargo test --features integration-tests --test save_pipeline` | End-to-end save parsing and write flow coverage |

## Running Tests

```bash
# Run the default test suite on a clean checkout
cargo test

# Compile and run all integration tests (requires local fixture data)
cargo test --features integration-tests

# Run a specific suite
cargo test --features integration-tests --test character

# Run a specific test file within a suite
cargo test --features integration-tests --test character classes

# Run with output visible (for gamedata inspection tests)
cargo test --features integration-tests --test gamedata player -- --nocapture

# Run a single test by name
cargo test --features integration-tests --test character test_abilities_and_cascades
```

By default, the integration test targets are disabled on clean checkouts because they depend on local NWN2 data and fixture files under `tests/fixtures/`.

## Directory Structure

```
tests/
├── character.rs              # Entry point
├── character/
│   ├── abilities.rs          # Ability scores, cascades to HP
│   ├── classes.rs            # Class management, level up/down
│   ├── combat.rs             # Attack bonuses, AC, damage
│   ├── feats.rs              # Feat management, prerequisites
│   ├── identity.rs           # Name, portrait, alignment
│   ├── inventory.rs          # Equipment, items, weight
│   ├── race.rs               # Racial bonuses, size, speed
│   ├── saves.rs              # Saving throws
│   ├── skills.rs             # Skill ranks, modifiers
│   └── spells.rs             # Spellcasting, metamagic
│
├── parsing.rs                # Entry point
├── parsers/
│   ├── gff.rs                # GFF V3.2 binary format
│   ├── tlk.rs                # Talk table (dialog.tlk)
│   ├── tda.rs                # 2DA tabular data
│   ├── erf.rs                # ERF/HAK/MOD archives
│   └── xml.rs                # XML files (globals.xml, etc.)
│
├── gamedata.rs               # Entry point
├── gamedata/
│   ├── campaign.rs           # Campaign/module data inspection
│   ├── classes.rs            # classes.2da inspection
│   ├── feats.rs              # feat.2da, cls_feat_*.2da
│   ├── items.rs              # baseitems.2da, item properties
│   ├── player.rs             # .bic GFF structure (LvlStatList, etc.)
│   ├── races.rs              # racialtypes.2da
│   ├── skills.rs             # skills.2da
│   └── spells.rs             # spells.2da
│
├── services.rs               # Entry point
├── services/
│   ├── campaign.rs           # Campaign service
│   ├── field_mapper.rs       # GFF field name translation
│   ├── item_property_decoder.rs  # Item property decoding
│   ├── playerinfo.rs         # Player info parsing
│   ├── resource_manager.rs   # Resource loading, override chain
│   ├── rule_detector.rs      # Ruleset detection (OC/MotB/SoZ)
│   └── savegame_handler.rs   # Save file I/O, backups
│
├── utils.rs                  # Entry point
├── utils/
│   ├── directory_walker.rs   # Directory traversal
│   ├── path_discovery.rs     # NWN2 path detection
│   ├── precompiled_cache.rs  # Cache serialization
│   ├── prerequisite_graph.rs # Feat dependency graphs
│   ├── resource_scanner.rs   # Resource file scanning
│   ├── zip_content_reader.rs # Zip file reading
│   └── zip_indexer.rs        # Zip archive indexing
│
├── debugging.rs              # Entry point
├── file_operations.rs        # Entry point
├── save_pipeline.rs          # Entry point
│
├── common/                   # Shared test utilities
│   └── mod.rs                # TestContext, fixtures helpers
│
├── fixtures/                 # Test data files
```

## Test Categories

### Parser Tests (`--test parsing`)
Low-level format correctness. "Can we read/write this file format correctly?"

### Character Tests (`--test character`)
Business logic correctness. "Does `level_up()` grant the right feats?"

### Gamedata Tests (`--test gamedata`)
Data inspection and validation using the **loaders** (`GameDataLoader`, `DataModelLoader`).
Run with `--nocapture` to see output.
"What does the loaded data look like? Is it correct?"

### Services Tests (`--test services`)
Service layer behavior. "Does ResourceManager load overrides correctly?"

### Utils Tests (`--test utils`)
Utility function behavior. "Does path discovery find the NWN2 install?"

## Adding New Tests

1. Add your test function to the appropriate file in the subdirectory
2. Use `super::super::common::create_test_context` for full context
3. Use `super::super::common::load_test_gff` for fixture loading
4. Run the target with `--features integration-tests`

```rust
use super::super::common::create_test_context;

#[tokio::test]
async fn test_my_feature() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");
    
    // Your test here
}
```

## Fixtures

Place test data files in `tests/fixtures/`:
- `fixtures/gff/` - Sample .bic character files
- `fixtures/saves/` - Sample save game data

Load with:
```rust
let bytes = load_test_gff("character_name/character_name.bic");
```
