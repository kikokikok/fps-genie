# CS2 Demo Parser - Data Module and Real Demo Tests

This implementation provides comprehensive parsing capabilities for CS2 demo files, specifically the Vitality vs Spirit match demo. The code mimics the approach used in `e2e_test.rs` but is structured for parsing real demo files with all events, properties, and players.

## Files Created

### `data.rs`
Core data parsing functionality that provides:
- Comprehensive property list (300+ CS2 game properties)
- Full event parsing with "all" events configuration  
- Custom property mapping for easier data access
- Support for parsing complete matches with all players and ticks

Key functions:
- `parse_demo_comprehensive()` - Parse any demo file comprehensively
- `parse_vitality_vs_spirit_demo()` - Parse the specific Vitality vs Spirit demo
- `create_custom_property_mapping()` - Create custom property mappings

### `real_demo_tests.rs`
Comprehensive test suite for validating the Vitality vs Spirit demo parsing:
- Test all events are captured (round_start, player_death, weapon_fire, etc.)
- Test all player properties are parsed (health, position, weapons, etc.)
- Test all players from both teams are tracked
- Test event data structures and completeness
- Test parsing performance and data coverage

## Usage

### Running Tests

Most tests are marked with `#[ignore]` because parsing the large demo file (541MB) takes significant time:

```bash
# Run all real demo tests (takes several minutes)
cargo test -- --ignored

# Run a specific test
cargo test test_parse_vitality_vs_spirit_demo -- --ignored

# Run only the fast tests (no demo parsing)
cargo test test_data_module_functions
```

### Using the Data Module

```rust
use cs2_demo_parser::data::{parse_vitality_vs_spirit_demo, create_custom_property_mapping};

// Parse the Vitality vs Spirit demo
let (output, prop_controller, events) = parse_vitality_vs_spirit_demo()?;

// Access parsed data
println!("Event types: {}", events.len());
println!("Properties: {}", prop_controller.name_to_id.len());
println!("Total events: {}", events.values().map(|v| v.len()).sum::<usize>());

// Use custom property mappings
let custom_mapping = create_custom_property_mapping();
// Access position data, weapon names, etc.
```

## Implementation Details

The implementation follows the same pattern as `e2e_test.rs`:

1. **Comprehensive Property List**: Includes all player, game rule, and weapon properties
2. **All Events Parsing**: Uses `["all"]` to capture every game event
3. **No Filtering**: Captures all players, all ticks, includes projectiles and grenades
4. **Event Grouping**: Groups events by type in a BTreeMap for easy access
5. **Custom Mappings**: Provides human-readable names for common properties

## Performance Notes

- The Vitality vs Spirit demo file is 541MB
- Parsing takes several minutes on first access
- Results are cached using `lazy_static` for subsequent test runs
- Tests use `#[ignore]` to avoid blocking regular test runs

## Data Captured

The parser captures:
- **All game events**: 40+ event types including deaths, weapon fires, bomb events
- **All player properties**: 300+ properties including position, health, weapons, movement
- **All players**: Both Team Vitality and Team Spirit players
- **Complete timeline**: All ticks and game states throughout the match
- **Projectiles and grenades**: Full ballistics and explosive data

This provides comprehensive data for CS2 match analysis and AI training systems.