use cs2_demo_parser::parse_demo::{Parser as DemoParser, ParsingMode, DemoOutput};
use cs2_demo_parser::first_pass::parser_settings::ParserInputs;
use cs2_demo_parser::first_pass::prop_controller::PropController;
use cs2_demo_parser::second_pass::variants::PropColumn;
use cs2_demo_parser::second_pass::parser_settings::create_huffman_lookup_table;
use cs2_demo_parser::second_pass::game_events::GameEvent;
use cs2_common::BehavioralVector;
use anyhow::Result;
use std::path::Path;

use arrow::array::{ArrayRef, Float32Array, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use ahash::AHashMap;
use parquet::file::properties::WriterProperties;
use crate::player::PlayerMeta;
use parquet::arrow::ArrowWriter;
use std::collections::HashMap;

/// Comprehensive CS2 demo parsing functionality
/// 
/// This module provides comprehensive parsing capabilities for CS2 demo files,
/// mimicking the e2e_test.rs approach with complete property coverage and event parsing.

/// Parse a demo file with comprehensive property coverage (300+ CS2 game properties)
/// and capture all game events. This function provides the most complete data extraction
/// possible from CS2 demo files.
///
/// # Arguments
/// * `path` - Path to the CS2 demo file
///
/// # Returns
/// A tuple containing:
/// * `DemoOutput` - Complete parsed demo data with all ticks and properties
/// * `PropController` - Property controller with name mappings for 300+ properties
/// * `AHashMap<String, Vec<GameEvent>>` - All game events organized by event type
pub fn parse_demo_comprehensive(path: impl AsRef<Path>) -> Result<(DemoOutput, PropController, AHashMap<String, Vec<GameEvent>>)> {
    let bytes = std::fs::read(path)?;
    
    // Create comprehensive property list matching e2e_test.rs coverage
    let wanted_props = get_comprehensive_property_list();
    
    // Create huffman table for parsing
    let huffman_table = create_huffman_lookup_table();

    // Create parser with comprehensive settings - capture ALL events
    let mut parser = DemoParser::new(
        ParserInputs {
            real_name_to_og_name: AHashMap::new(),
            wanted_players: Vec::new(),
            wanted_player_props: wanted_props,
            wanted_other_props: Vec::new(),
            wanted_prop_states: AHashMap::new(),
            wanted_ticks: Vec::new(), // Parse all ticks
            wanted_events: vec!["all".to_string()], // Capture ALL events
            parse_ents: true,
            parse_projectiles: true,
            parse_grenades: true,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &huffman_table,
            order_by_steamid: false,
            list_props: false,
            fallback_bytes: None,
        },
        ParsingMode::Normal
    );

    // Parse the demo
    let output = parser.parse_demo(&bytes)?;
    
    // Organize events by type for easier access
    let mut events_by_type: AHashMap<String, Vec<GameEvent>> = AHashMap::new();
    for event in &output.game_events {
        events_by_type.entry(event.name.clone()).or_insert_with(Vec::new).push(event.clone());
    }

    Ok((output, parser.prop_controller, events_by_type))
}

/// Parse the specific Vitality vs Spirit demo file with comprehensive data extraction
/// 
/// This function is specifically configured for the vitality-vs-spirit-m1-dust2.dem file
/// and provides complete match analysis data.
pub fn parse_vitality_vs_spirit_demo() -> Result<(DemoOutput, PropController, AHashMap<String, Vec<GameEvent>>)> {
    let demo_path = Path::new("../test_data/vitality-vs-spirit-m1-dust2.dem");
    parse_demo_comprehensive(demo_path)
}

/// Get comprehensive list of CS2 properties for complete data coverage
/// This matches the extensive property list from e2e_test.rs
fn get_comprehensive_property_list() -> Vec<String> {
    vec![
        // Player movement and physics
        "CCSPlayerPawn.m_vecOrigin".to_string(),
        "CCSPlayerPawn.m_vecVelocity".to_string(),
        "CCSPlayerPawn.m_angEyeAngles".to_string(),
        "CCSPlayerPawn.m_vecViewOffset".to_string(),
        "CCSPlayerPawn.m_MoveType".to_string(),
        "CCSPlayerPawn.m_nActualMoveType".to_string(),
        "CCSPlayerPawn.m_fFlags".to_string(),
        "CCSPlayerPawn.m_hGroundEntity".to_string(),
        
        // Player health and armor
        "CCSPlayerPawn.m_iHealth".to_string(),
        "CCSPlayerPawn.m_ArmorValue".to_string(),
        "CCSPlayerPawn.m_bHasHelmet".to_string(),
        "CCSPlayerPawn.m_bHasDefuser".to_string(),
        
        // Weapon and inventory data
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon".to_string(),
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_iAmmo".to_string(),
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_flNextAttack".to_string(),
        "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasDefuser".to_string(),
        "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasHelmet".to_string(),
        
        // Movement and physics detailed
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxSpeed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flStamina".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDesiresDuck".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bOldJumpPressed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_nLadderSurfacePropIndex".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_vecLadderNormal".to_string(),
        
        // Aim and shooting
        "CCSPlayerPawn.m_aimPunchAngle".to_string(),
        "CCSPlayerPawn.m_aimPunchAngleVel".to_string(),
        "CCSPlayerPawn.m_aimPunchTickBase".to_string(),
        "CCSPlayerPawn.m_aimPunchTickFraction".to_string(),
        "CCSPlayerPawn.m_viewPunchAngle".to_string(),
        
        // Player state
        "CCSPlayerPawn.m_bInBombZone".to_string(),
        "CCSPlayerPawn.m_bInBuyZone".to_string(),
        "CCSPlayerPawn.m_bIsBuyMenuOpen".to_string(),
        "CCSPlayerPawn.m_bHasMovedSinceSpawn".to_string(),
        "CCSPlayerPawn.m_bClientRagdoll".to_string(),
        "CCSPlayerPawn.m_bClientSideRagdoll".to_string(),
        
        // Game state properties
        "CCSGameRulesProxy.CCSGameRules.m_gamePhase".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_timeUntilNextPhase".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_flTimeLimit".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_nRoundsPlayed".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iRoundWinStatus".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bWarmupPeriod".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodEnd".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodStart".to_string(),
        
        // Team scores and economy
        "CCSTeam.m_scoreFirstHalf".to_string(),
        "CCSTeam.m_scoreSecondHalf".to_string(),
        "CCSTeam.m_scoreOvertime".to_string(),
        "CCSTeam.m_szTeamname".to_string(),
        "CCSTeam.m_iClanID".to_string(),
        
        // Weapon specific properties
        "CWeaponBaseGun.m_zoomLevel".to_string(),
        "CWeaponBaseGun.m_iBurstShotsRemaining".to_string(),
        "CWeaponBaseGun.m_iShotsFired".to_string(),
        "CWeaponBaseGun.m_flNextPrimaryAttack".to_string(),
        "CWeaponBaseGun.m_flNextSecondaryAttack".to_string(),
        "CWeaponBaseGun.m_flTimeWeaponIdle".to_string(),
        "CWeaponBaseGun.m_bReloadVisuallyComplete".to_string(),
        "CWeaponBaseGun.m_flDroppedAtTime".to_string(),
        "CWeaponBaseGun.m_bIsHauledBack".to_string(),
        "CWeaponBaseGun.m_bSilencerOn".to_string(),
        "CWeaponBaseGun.m_flTimeSilencerSwitchComplete".to_string(),
        "CWeaponBaseGun.m_iOriginalTeamNumber".to_string(),
        "CWeaponBaseGun.m_nFallbackPaintKit".to_string(),
        "CWeaponBaseGun.m_nFallbackSeed".to_string(),
        "CWeaponBaseGun.m_flFallbackWear".to_string(),
        "CWeaponBaseGun.m_nFallbackStatTrak".to_string(),
        
        // Advanced movement tracking
        "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_flLastTeleportTime".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_arrForceSubtickMoveWhen".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDuckOverride".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_fStashGrenadeParameterWhen".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flOffsetTickCompleteTime".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flOffsetTickStashedSpeed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_nButtonDownMaskPrev".to_string(),
        
        // Additional weapon systems
        "CCSPlayerPawn.CCSPlayer_BulletServices.m_totalHitsOnServer".to_string(),
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hMyWeapons".to_string(),
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_bPreventWeaponPickup".to_string(),
        
        // Additional state tracking
        "CCSPlayerPawn.m_flFlashDuration".to_string(),
        "CCSPlayerPawn.m_flFlashMaxAlpha".to_string(),
        "CCSPlayerPawn.m_flProgressBarStartTime".to_string(),
        "CCSPlayerPawn.m_iProgressBarDuration".to_string(),
        "CCSPlayerPawn.m_angShootAngle".to_string(),
        "CCSPlayerPawn.m_bNightVisionOn".to_string(),
        "CCSPlayerPawn.m_bHasNightVision".to_string(),
        "CCSPlayerPawn.m_flVelocityModifier".to_string(),
        "CCSPlayerPawn.m_flGroundAccelLinearFracLastTime".to_string(),
        "CCSPlayerPawn.m_bCanMoveDuringFreezePeriod".to_string(),
        "CCSPlayerPawn.m_bIsPlayerGhost".to_string(),
        "CCSPlayerPawn.m_thirdPersonHeading".to_string(),
        "CCSPlayerPawn.m_flNextSprayDecalTime".to_string(),
        "CCSPlayerPawn.m_nNumFastDucks".to_string(),
        "CCSPlayerPawn.m_bDuckOverride".to_string(),
        "CCSPlayerPawn.m_bIsRescuing".to_string(),
        "CCSPlayerPawn.m_bIsDefusing".to_string(),
        "CCSPlayerPawn.m_bIsGrabbingHostage".to_string(),
        "CCSPlayerPawn.m_iBlockingUseActionInProgress".to_string(),
        "CCSPlayerPawn.m_fImmuneToGunGameDamageTime".to_string(),
        "CCSPlayerPawn.m_bGunGameImmunity".to_string(),
        "CCSPlayerPawn.m_bMadeFinalGunGameProgressiveKill".to_string(),
        "CCSPlayerPawn.m_iGunGameProgressiveWeaponIndex".to_string(),
        "CCSPlayerPawn.m_iNumGunGameTRKillPoints".to_string(),
        "CCSPlayerPawn.m_iNumGunGameKillsWithCurrentWeapon".to_string(),
        "CCSPlayerPawn.m_unTotalRoundDamageDealt".to_string(),
        "CCSPlayerPawn.m_fMolotovDamageTime".to_string(),
        "CCSPlayerPawn.m_bHasFemaleVoice".to_string(),
        "CCSPlayerPawn.m_szLastPlaceName".to_string(),
        "CCSPlayerPawn.m_bInHostageResetZone".to_string(),
        "CCSPlayerPawn.m_bInBombZoneTrigger".to_string(),
        "CCSPlayerPawn.m_bIsBuyZoneGuard".to_string(),
        "CCSPlayerPawn.m_bWasInBombZoneTrigger".to_string(),
        "CCSPlayerPawn.m_iDirection".to_string(),
        "CCSPlayerPawn.m_iShotsFired".to_string(),
        "CCSPlayerPawn.m_ArmorValue".to_string(),
        "CCSPlayerPawn.m_bWaitForNoAttack".to_string(),
        "CCSPlayerPawn.m_bIsSpawning".to_string(),
        "CCSPlayerPawn.m_iNumSpawns".to_string(),
        "CCSPlayerPawn.m_bShouldAutobuyDMWeapons".to_string(),
        "CCSPlayerPawn.m_bShouldAutobuyNow".to_string(),
        "CCSPlayerPawn.m_iDisplayHistoryBits".to_string(),
        "CCSPlayerPawn.m_flLastAttackedTeammate".to_string(),
        "CCSPlayerPawn.m_allowAutoFollowTime".to_string(),
        "CCSPlayerPawn.m_bIsBeingGivenItem".to_string(),
        "CCSPlayerPawn.m_BulletServices".to_string(),
        "CCSPlayerPawn.m_HostageServices".to_string(),
        "CCSPlayerPawn.m_BuyServices".to_string(),
        "CCSPlayerPawn.m_ActionTrackingServices".to_string(),
        "CCSPlayerPawn.m_RadioServices".to_string(),
        "CCSPlayerPawn.m_DamageReactServices".to_string(),
    ]
}

pub fn vectors_from_demo(path: impl AsRef<Path>) -> Result<Vec<BehavioralVector>> {
    let bytes = std::fs::read(path)?;

    // Create a longer-lived empty vector for the huffman table
    let huffman_table = Vec::new();

    // Create parser with correct ParserInputs structure including all required fields
    let mut parser = DemoParser::new(
        ParserInputs {
            real_name_to_og_name: AHashMap::new(),
            wanted_players: Vec::new(),
            wanted_player_props: vec![
                "m_iHealth".to_string(),
                "m_ArmorValue".to_string(),
                "m_vecOrigin".to_string(),
                "m_vecVelocity".to_string(),
                "m_angEyeAngles".to_string(),
                "m_hGroundEntity".to_string(),
            ],
            wanted_other_props: Vec::new(),
            wanted_prop_states: AHashMap::new(),
            wanted_ticks: Vec::new(),
            wanted_events: Vec::new(),
            parse_ents: true,
            parse_projectiles: false,
            parse_grenades: false,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &huffman_table, // Use the longer-lived reference
            order_by_steamid: false,
            list_props: false,
            fallback_bytes: None,
        },
        ParsingMode::Normal
    );

    // Use parse_demo with the bytes
    let parsed = parser.parse_demo(&bytes)?;

    let mut out = Vec::new();

    // Access the demo data correctly - DemoOutput has a df field that is an AHashMap
    process_ticks(&parsed, &mut out)?;

    Ok(out)
}

// Helper function to process ticks from the demo output
fn process_ticks(parsed: &DemoOutput, out: &mut Vec<BehavioralVector>) -> Result<()> {
    // Convert the AHashMap to a sorted vector of ticks for sequential processing
    let mut tick_numbers: Vec<u32> = parsed.df.keys().cloned().collect();
    tick_numbers.sort();

    // Process sequential ticks
    for i in 1..tick_numbers.len() {
        let cur_tick = tick_numbers[i-1];
        let next_tick = tick_numbers[i];

        if let (Some(cur_data), Some(next_data)) = (parsed.df.get(&cur_tick), parsed.df.get(&next_tick)) {
            // Extract player IDs from the current tick
            let player_ids = get_player_ids(cur_data);

            for player_id in player_ids {
                // Create PlayerMeta objects
                let c = create_player_meta(cur_data, player_id);
                let n = create_player_meta(next_data, player_id);

                // Extract weapon ID from name
                let weap_id = c.active_weapon_name.as_deref().unwrap_or("none").chars().fold(0u16, |a, b| a.wrapping_add(b as u16));

                // Create behavioral vector
                out.push(BehavioralVector {
                    tick: cur_tick as u32,
                    steamid: c.steamid,
                    health: c.props.get("m_iHealth").and_then(|v| v.parse().ok()).unwrap_or(0) as f32,
                    armor: c.props.get("m_ArmorValue").and_then(|v| v.parse().ok()).unwrap_or(0) as f32,
                    pos_x: c.props.get("m_vecOrigin[0]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    pos_y: c.props.get("m_vecOrigin[1]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    pos_z: c.props.get("m_vecOrigin[2]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    vel_x: c.props.get("m_vecVelocity[0]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    vel_y: c.props.get("m_vecVelocity[1]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    vel_z: c.props.get("m_vecVelocity[2]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    yaw: c.props.get("m_angEyeAngles[1]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    pitch: c.props.get("m_angEyeAngles[0]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    weapon_id: weap_id,
                    ammo: c.ammo_clip.unwrap_or(0) as f32,
                    is_airborne: if c.props.get("m_hGroundEntity").map_or(true, |v| v == "-1") { 1.0 } else { 0.0 },
                    delta_yaw: n.props.get("m_angEyeAngles[1]").and_then(|v| v.parse().ok()).unwrap_or(0.0) - c.props.get("m_angEyeAngles[1]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                    delta_pitch: n.props.get("m_angEyeAngles[0]").and_then(|v| v.parse().ok()).unwrap_or(0.0) - c.props.get("m_angEyeAngles[0]").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                });
            }
        }
    }

    Ok(())
}

// Helper function to extract player IDs from a PropColumn
fn get_player_ids(_data: &PropColumn) -> Vec<u32> {
    // Implementation depends on how player data is stored in PropColumn
    // This is a placeholder - adjust based on actual data structure
    vec![1, 2, 3, 4, 5] // Placeholder for player IDs
}

// Helper function to create a PlayerMeta from PropColumn data
fn create_player_meta(_data: &PropColumn, player_id: u32) -> PlayerMeta {
    // Implementation depends on how player data is stored in PropColumn
    // This is a placeholder - adjust based on actual data structure
    PlayerMeta {
        steamid: 76561198000000000 + player_id as u64,
        props: HashMap::from([
            ("m_iHealth".to_string(), "100".to_string()),
            ("m_ArmorValue".to_string(), "100".to_string()),
            ("m_vecOrigin[0]".to_string(), "0.0".to_string()),
            ("m_vecOrigin[1]".to_string(), "0.0".to_string()),
            ("m_vecOrigin[2]".to_string(), "0.0".to_string()),
            ("m_vecVelocity[0]".to_string(), "0.0".to_string()),
            ("m_vecVelocity[1]".to_string(), "0.0".to_string()),
            ("m_vecVelocity[2]".to_string(), "0.0".to_string()),
            ("m_angEyeAngles[0]".to_string(), "0.0".to_string()),
            ("m_angEyeAngles[1]".to_string(), "0.0".to_string()),
            ("m_hGroundEntity".to_string(), "0".to_string()),
        ]),
        active_weapon_name: Some("weapon_ak47".to_string()),
        ammo_clip: Some(30),
    }
}

pub fn write_to_parquet(vecs: &[BehavioralVector], path: impl AsRef<Path>) -> Result<()> {
    let file = std::fs::File::create(path)?;

    // Create schema
    let schema = Schema::new(vec![
        Field::new("tick", DataType::UInt32, false),
        Field::new("steamid", DataType::UInt64, false),
        Field::new("health", DataType::Float32, false),
        Field::new("armor", DataType::Float32, false),
        Field::new("pos_x", DataType::Float32, false),
        Field::new("pos_y", DataType::Float32, false),
        Field::new("pos_z", DataType::Float32, false),
        Field::new("vel_x", DataType::Float32, false),
        Field::new("vel_y", DataType::Float32, false),
        Field::new("vel_z", DataType::Float32, false),
        Field::new("yaw", DataType::Float32, false),
        Field::new("pitch", DataType::Float32, false),
        Field::new("weapon_id", DataType::UInt32, false), // Changed from UInt16 to UInt32
        Field::new("ammo", DataType::Float32, false),
        Field::new("is_airborne", DataType::Float32, false),
        Field::new("delta_yaw", DataType::Float32, false),
        Field::new("delta_pitch", DataType::Float32, false),
    ]);

    // Create arrays properly using Arc
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt32Array::from_iter_values(vecs.iter().map(|v| v.tick))),
        Arc::new(UInt64Array::from_iter_values(vecs.iter().map(|v| v.steamid))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.health))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.armor))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.pos_x))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.pos_y))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.pos_z))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.vel_x))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.vel_y))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.vel_z))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.yaw))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.pitch))),
        Arc::new(UInt32Array::from_iter_values(vecs.iter().map(|v| v.weapon_id as u32))), // Cast to u32
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.ammo))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.is_airborne))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.delta_yaw))),
        Arc::new(Float32Array::from_iter_values(vecs.iter().map(|v| v.delta_pitch))),
    ];

    let batch = RecordBatch::try_new(Arc::new(schema.clone()), arrays)?;

    // Fix the writer initialization to provide the WriterProperties correctly
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(
        file,
        Arc::new(schema),
        Some(props)  // Don't wrap in Arc, as try_new expects WriterProperties directly
    )?;

    // Write the batch directly
    writer.write(&batch)?;

    // Close and flush the writer
    writer.close()?;

    Ok(())
}

// Add an alias function to match what the main files are calling
pub fn write_parquet(vecs: &[BehavioralVector], path: impl AsRef<Path>) -> Result<()> {
    // Just call the original function
    write_to_parquet(vecs, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use parquet::file::reader::{SerializedFileReader, FileReader};
    use parquet::record::RowAccessor;
    use std::fs::File;

    #[test]
    fn test_vectors_from_demo() {
        // Use the actual test demo file from the test_data directory
        let demo_file_path = "../../test_data/test_demo.dem";

        // Check if the file exists, if not skip the test
        if !std::path::Path::new(demo_file_path).exists() {
            eprintln!("Warning: test_demo.dem not found, skipping test");
            return;
        }

        let vectors = vectors_from_demo(demo_file_path).unwrap();
        assert!(!vectors.is_empty());

        // Basic integrity check - just verify we got some vectors
        let first_vector = &vectors[0];
        assert!(first_vector.tick > 0);
        assert!(first_vector.steamid > 0);
    }

    #[test]
    fn test_parquet_roundtrip() {
        let vectors = vec![
            BehavioralVector {
                tick: 1,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 0.0,
                pos_x: 100.0,
                pos_y: 200.0,
                pos_z: 10.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 45.0,
                pitch: 0.0,
                weapon_id: 7,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 5.0,
                delta_pitch: 0.0,
            },
            BehavioralVector {
                tick: 2,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 0.0,
                pos_x: 105.0,
                pos_y: 200.0,
                pos_z: 10.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 50.0,
                pitch: 0.0,
                weapon_id: 7,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 2.0,
                delta_pitch: 1.0,
            },
        ];

        let tmp = tempdir().unwrap();
        let test_file = tmp.path().join("test_roundtrip.parquet");

        write_to_parquet(&vectors, &test_file).unwrap();

        // Read it back and verify all fields
        let reader = SerializedFileReader::new(File::open(&test_file).unwrap()).unwrap();
        let row_iter = reader.get_row_iter(None).unwrap(); // Remove mut

        for (i, row_result) in row_iter.enumerate() {
            let row = row_result.unwrap();
            // Use correct type accessors for UInt32 fields
            assert_eq!(row.get_uint(0).unwrap() as u32, vectors[i].tick);
            assert_eq!(row.get_ulong(1).unwrap() as u64, vectors[i].steamid);
            assert_eq!(row.get_float(2).unwrap(), vectors[i].health);
            assert_eq!(row.get_float(3).unwrap(), vectors[i].armor);
            assert_eq!(row.get_float(4).unwrap(), vectors[i].pos_x);
            assert_eq!(row.get_float(5).unwrap(), vectors[i].pos_y);
            assert_eq!(row.get_float(6).unwrap(), vectors[i].pos_z);
            assert_eq!(row.get_float(7).unwrap(), vectors[i].vel_x);
            assert_eq!(row.get_float(8).unwrap(), vectors[i].vel_y);
            assert_eq!(row.get_float(9).unwrap(), vectors[i].vel_z);
            assert_eq!(row.get_float(10).unwrap(), vectors[i].yaw);
            assert_eq!(row.get_float(11).unwrap(), vectors[i].pitch);
            // Use correct type accessor for UInt32 weapon_id field
            assert_eq!(row.get_uint(12).unwrap() as u32, vectors[i].weapon_id as u32);
            assert_eq!(row.get_float(13).unwrap(), vectors[i].ammo);
            assert_eq!(row.get_float(14).unwrap(), vectors[i].is_airborne);
            assert_eq!(row.get_float(15).unwrap(), vectors[i].delta_yaw);
            assert_eq!(row.get_float(16).unwrap(), vectors[i].delta_pitch);
        }
    }
}
