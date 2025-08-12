# Idiomatic parsing interface: features and presets

This module centralizes event/prop selection behind:
- `bitflags`-backed `ParsingFeatures` (AIM, MOVEMENT, UTILITY, OBJECTIVE, ECONOMY, RULES, INFO, VALIDATION)
- `ParsingPreset` enum: `Minimal`, `Standard`, `Rich`
- `build_wanted(features)` -> `Wanted { player_props, other_props, events }`

Usage:
```rust
let features = ParsingPreset::Rich.to_features();
let wanted = build_wanted(features);
// feed wanted.* into ParserInputs
```

Rationale:
- Keeps call sites clean and Rust-idiomatic
- Ensures consistency between pipeline and ML crates
- Allows flipping feature groups without editing string lists at multiple sites

Notes:
- No DB schema changes: new behavior metrics stay in JSON `features`
- `parse_grenades = true` for utility (flash/smoke/molly/HE) effectiveness
- Heavy per-tick computations are deferred to moment windows for performance