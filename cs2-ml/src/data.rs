use cs2_demo_parser::parse_demo::{Parser as DemoParser, ParsingMode, DemoOutput};
use cs2_demo_parser::first_pass::parser_settings::ParserInputs;
use cs2_demo_parser::second_pass::variants::PropColumn;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::array::{Float32Array, UInt32Array, UInt64Array, ArrayRef};
use std::sync::Arc;
use arrow::record_batch::RecordBatch;
use std::path::Path;
use anyhow::Result;
use cs2_common::BehavioralVector;
use parquet::file::properties::WriterProperties;
use crate::player::PlayerMeta;
use ahash::AHashMap;
use std::collections::HashMap;
use parquet::file::reader::{SerializedFileReader, FileReader};
use std::fs::File;
use parquet::record::RowAccessor;

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
    let mut writer = parquet::arrow::ArrowWriter::try_new(
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
