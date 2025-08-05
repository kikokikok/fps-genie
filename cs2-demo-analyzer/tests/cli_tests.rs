#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::path::Path;
    use tempfile::tempdir;
    use cs2_common::BehavioralVector;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("cs2-demo-analyzer").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("CS2 Demo Analyzer"));
    }

    #[test]
    #[ignore] // Requires a real demo file
    fn test_analyze_command() {
        // Create a temporary directory for output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        
        // To run this test, place a test demo file at this path
        let demo_path = Path::new("test_data/sample.dem");
        if !demo_path.exists() {
            println!("Skipping test_analyze_command: no demo file found");
            return;
        }
        
        let mut cmd = Command::cargo_bin("cs2-demo-analyzer").unwrap();
        cmd.arg("analyze")
            .arg("--demo").arg(demo_path)
            .arg("--output-dir").arg(output_path)
            .assert()
            .success();
        
        // Verify that output files were created
        assert!(output_path.join("vectors.parquet").exists());
    }
    
    #[test]
    fn test_visualize_command() {
        // Create a temporary directory and sample parquet file
        let temp_dir = tempdir().unwrap();
        let parquet_path = temp_dir.path().join("test.parquet");
        let output_path = temp_dir.path().join("vis.png");
        
        // Create a sample parquet file with behavioral vectors
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
        
        cs2_ml::data::write_parquet(&vectors, &parquet_path).unwrap();
        
        // Test visualization command
        let mut cmd = Command::cargo_bin("cs2-demo-analyzer").unwrap();
        cmd.arg("visualize")
            .arg("--parquet").arg(&parquet_path)
            .arg("--output").arg(&output_path)
            .arg("--type").arg("both")
            .assert()
            .success();
            
        // The output files should be created with _movement and _aim suffixes
        let movement_path = output_path.with_file_name(
            format!("{}_movement.png", 
                    output_path.file_stem().unwrap().to_string_lossy())
        );
        let aim_path = output_path.with_file_name(
            format!("{}_aim.png", 
                    output_path.file_stem().unwrap().to_string_lossy())
        );
        
        assert!(movement_path.exists());
        assert!(aim_path.exists());
    }
}
