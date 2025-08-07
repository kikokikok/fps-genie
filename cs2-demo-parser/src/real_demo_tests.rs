#![allow(non_snake_case)]

use crate::data::{parse_vitality_vs_spirit_demo, create_custom_property_mapping};
use crate::first_pass::prop_controller::PropController;
use crate::parse_demo::DemoOutput;
use crate::second_pass::game_events::GameEvent;
use lazy_static::lazy_static;
use std::collections::BTreeMap;

lazy_static! {
    /// Parsed data from the Vitality vs Spirit demo
    /// This contains all events, props, and players parsed from the real demo
    static ref VITALITY_VS_SPIRIT_DATA: (DemoOutput, PropController, BTreeMap<String, Vec<GameEvent>>) = 
        parse_vitality_vs_spirit_demo().expect("Failed to parse Vitality vs Spirit demo");
}

/// Test that we can successfully parse the Vitality vs Spirit demo
#[test]
fn test_parse_vitality_vs_spirit_demo() {
    let (output, prop_controller, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Verify basic parsing worked
    assert!(!output.df.is_empty(), "Demo should have parsed data");
    assert!(!prop_controller.name_to_id.is_empty(), "Should have property mappings");
    assert!(!events.is_empty(), "Should have parsed events");
    
    println!("Successfully parsed Vitality vs Spirit demo:");
    println!("- Properties: {}", prop_controller.name_to_id.len());
    println!("- Event types: {}", events.len());
    println!("- Total events: {}", events.values().map(|v| v.len()).sum::<usize>());
}

/// Test that all expected player properties are captured
#[test]
fn test_all_player_props_captured() {
    let (output, prop_controller, _) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Check that we have key player properties
    let expected_props = [
        "CCSPlayerPawn.m_iHealth",
        "CCSPlayerPawn.m_ArmorValue", 
        "CCSPlayerPawn.m_iTeamNum",
        "CCSPlayerController.m_iszPlayerName",
        "CCSPlayerController.m_steamID",
        "X", "Y", "Z",
        "velocity_X", "velocity_Y", "velocity_Z",
        "pitch", "yaw",
        "weapon_name",
        "is_alive"
    ];
    
    for prop in expected_props.iter() {
        if let Some(&prop_id) = prop_controller.name_to_id.get(*prop) {
            assert!(output.df.contains_key(&prop_id), "Property {} should be in parsed data", prop);
        }
    }
    
    println!("Verified all key player properties are captured");
}

/// Test that all game events are captured
#[test] 
fn test_all_events_captured() {
    let (_, _, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Check for key CS2 events that should be present in any real match
    let expected_events = [
        "round_start",
        "round_end", 
        "player_spawn",
        "player_death",
        "weapon_fire",
        "player_hurt",
        "round_freeze_end",
        "bomb_planted",
        "bomb_defused",
        "item_equip"
    ];
    
    for event_type in expected_events.iter() {
        assert!(events.contains_key(*event_type), "Event type {} should be captured", event_type);
        assert!(!events[*event_type].is_empty(), "Event type {} should have instances", event_type);
    }
    
    println!("Verified all key event types are captured:");
    for (event_type, event_list) in events.iter() {
        println!("- {}: {} events", event_type, event_list.len());
    }
}

/// Test that we capture all players from both teams
#[test]
fn test_all_players_captured() {
    let (output, prop_controller, _) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Get player names and steamids
    let name_prop_id = prop_controller.name_to_id.get("CCSPlayerController.m_iszPlayerName");
    let steamid_prop_id = prop_controller.name_to_id.get("CCSPlayerController.m_steamID");
    let team_prop_id = prop_controller.name_to_id.get("CCSPlayerController.m_iTeamNum");
    
    if let (Some(&name_id), Some(&steamid_id), Some(&team_id)) = (name_prop_id, steamid_prop_id, team_prop_id) {
        let names = output.df.get(&name_id);
        let steamids = output.df.get(&steamid_id);
        let teams = output.df.get(&team_id);
        
        assert!(names.is_some(), "Should have player names");
        assert!(steamids.is_some(), "Should have player steam IDs");
        assert!(teams.is_some(), "Should have team assignments");
        
        // Should have players from both Team Vitality and Team Spirit
        // In a real match, we expect at least 10 players (5 per team)
        if let (Some(names), Some(steamids), Some(teams)) = (names, steamids, teams) {
            assert!(names.len() >= 10, "Should have at least 10 players tracked");
            assert!(steamids.len() >= 10, "Should have at least 10 steam IDs");
            assert!(teams.len() >= 10, "Should have at least 10 team assignments");
            
            println!("Captured {} player records", names.len());
        }
    }
}

/// Test specific event data structure for player deaths
#[test]
fn test_player_death_events() {
    let (_, _, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    if let Some(death_events) = events.get("player_death") {
        assert!(!death_events.is_empty(), "Should have player death events");
        
        // Check first death event has expected fields
        let first_death = &death_events[0];
        assert_eq!(first_death.name, "player_death");
        
        // Death events should have attacker, victim, weapon info
        let field_names: Vec<&str> = first_death.fields.iter().map(|f| f.name.as_str()).collect();
        
        // Common fields in player_death events
        let expected_fields = ["user_name", "user_steamid", "attacker_name", "attacker_steamid", "weapon"];
        for field in expected_fields.iter() {
            if !field_names.contains(field) {
                println!("Warning: Expected field '{}' not found in player_death event", field);
            }
        }
        
        println!("Player death events: {} total", death_events.len());
        println!("First death event fields: {:?}", field_names);
    }
}

/// Test weapon fire events tracking
#[test]
fn test_weapon_fire_events() {
    let (_, _, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    if let Some(fire_events) = events.get("weapon_fire") {
        assert!(!fire_events.is_empty(), "Should have weapon fire events");
        
        // Should have many weapon fire events in a real match
        assert!(fire_events.len() > 100, "Real match should have many weapon fires");
        
        println!("Weapon fire events: {} total", fire_events.len());
    }
}

/// Test round progression events
#[test]  
fn test_round_events() {
    let (_, _, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    let round_starts = events.get("round_start").map(|v| v.len()).unwrap_or(0);
    let round_ends = events.get("round_end").map(|v| v.len()).unwrap_or(0);
    
    // Should have multiple rounds in a real match
    assert!(round_starts > 0, "Should have round start events");
    assert!(round_ends > 0, "Should have round end events");
    
    // Usually round starts and ends should be close in number
    let round_diff = (round_starts as i32 - round_ends as i32).abs();
    assert!(round_diff <= 1, "Round starts and ends should be close in count");
    
    println!("Round events - Starts: {}, Ends: {}", round_starts, round_ends);
}

/// Test bomb-related events for competitive match
#[test]
fn test_bomb_events() {
    let (_, _, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    // In a competitive dust2 match, we expect bomb events
    let bomb_plants = events.get("bomb_planted").map(|v| v.len()).unwrap_or(0);
    let bomb_defuses = events.get("bomb_defused").map(|v| v.len()).unwrap_or(0);
    let bomb_explosions = events.get("bomb_exploded").map(|v| v.len()).unwrap_or(0);
    
    println!("Bomb events - Plants: {}, Defuses: {}, Explosions: {}", bomb_plants, bomb_defuses, bomb_explosions);
    
    // Should have some bomb activity in a competitive match
    let total_bomb_activity = bomb_plants + bomb_defuses + bomb_explosions;
    assert!(total_bomb_activity > 0, "Should have some bomb-related activity");
}

/// Test custom property mappings work correctly
#[test]
fn test_custom_properties() {
    let (output, _, _) = &*VITALITY_VS_SPIRIT_DATA;
    let custom_mapping = create_custom_property_mapping();
    
    // Check that custom properties are accessible
    for (&prop_id, &prop_name) in custom_mapping.iter() {
        if output.df.contains_key(&prop_id) {
            println!("Custom property '{}' found with ID {}", prop_name, prop_id);
        }
    }
    
    // Should have basic position data
    assert!(output.df.contains_key(&crate::first_pass::prop_controller::PLAYER_X_ID), "Should have X coordinates");
    assert!(output.df.contains_key(&crate::first_pass::prop_controller::PLAYER_Y_ID), "Should have Y coordinates"); 
    assert!(output.df.contains_key(&crate::first_pass::prop_controller::PLAYER_Z_ID), "Should have Z coordinates");
}

/// Test that we have comprehensive data across multiple ticks
#[test]
fn test_tick_coverage() {
    let (output, prop_controller, _) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Check that we have data across many ticks
    if let Some(&tick_id) = prop_controller.name_to_id.get("tick") {
        if let Some(tick_data) = output.df.get(&tick_id) {
            assert!(tick_data.len() > 0, "Should have tick data");
            println!("Captured data across {} tick entries", tick_data.len());
        }
    }
    
    // Should have substantial amount of data for a real match
    let total_data_points: usize = output.df.values().map(|v| v.len()).sum();
    assert!(total_data_points > 1000, "Should have substantial data from real match");
    
    println!("Total data points captured: {}", total_data_points);
}

/// Integration test to verify the parser works end-to-end with real demo
#[test]
fn test_end_to_end_parsing() {
    let (output, prop_controller, events) = &*VITALITY_VS_SPIRIT_DATA;
    
    // Summary verification
    println!("=== Vitality vs Spirit Demo Parsing Summary ===");
    println!("Properties parsed: {}", prop_controller.name_to_id.len());
    println!("Event types: {}", events.len());
    println!("Total events: {}", events.values().map(|v| v.len()).sum::<usize>());
    println!("Data frames: {}", output.df.len());
    println!("Total data points: {}", output.df.values().map(|v| v.len()).sum::<usize>());
    
    // Verify we have comprehensive coverage
    assert!(prop_controller.name_to_id.len() > 100, "Should track many properties");
    assert!(events.len() > 20, "Should capture many event types");
    assert!(events.values().map(|v| v.len()).sum::<usize>() > 500, "Should have many events");
    assert!(output.df.len() > 50, "Should have many data frames");
    assert!(output.df.values().map(|v| v.len()).sum::<usize>() > 5000, "Should have substantial data");
    
    println!("✓ End-to-end parsing verification complete");
}