pub mod parsing_features;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Error types for CS2-related operations
#[derive(Debug, thiserror::Error)]
pub enum CS2Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Model error: {0}")]
    ModelError(String),
}

/// A behavioral vector representing player state and actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralVector {
    pub tick: u32,
    pub steamid: u64,
    pub health: f32,
    pub armor: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub weapon_id: u16,
    pub ammo: f32,
    pub is_airborne: f32,
    pub delta_yaw: f32,
    pub delta_pitch: f32,
}

/// Network input vector (C-compatible, for fast binary serialization)
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct InputVector {
    pub health: f32,
    pub armor: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub weapon_id_f32: f32, // Encoded as float for uniformity
    pub ammo: f32,
    pub is_airborne: f32,
    pub padding: f32, // Ensure alignment
}

/// Network output vector (C-compatible, for fast binary serialization)
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct OutputVector {
    pub delta_yaw: f32,
    pub delta_pitch: f32,
}

impl InputVector {
    /// Convert from a behavioral vector to network input format
    pub fn from_behavioral(bv: &BehavioralVector) -> Self {
        Self {
            health: bv.health,
            armor: bv.armor,
            pos_x: bv.pos_x,
            pos_y: bv.pos_y,
            pos_z: bv.pos_z,
            vel_x: bv.vel_x,
            vel_y: bv.vel_y,
            vel_z: bv.vel_z,
            yaw: bv.yaw,
            pitch: bv.pitch,
            weapon_id_f32: bv.weapon_id as f32,
            ammo: bv.ammo,
            is_airborne: bv.is_airborne,
            padding: 0.0,
        }
    }
}

impl BehavioralVector {
    /// Create a new behavioral vector with default values
    pub fn new(tick: u32, steamid: u64) -> Self {
        Self {
            tick,
            steamid,
            health: 100.0,
            armor: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            weapon_id: 0,
            ammo: 0.0,
            is_airborne: 0.0,
            delta_yaw: 0.0,
            delta_pitch: 0.0,
        }
    }

    /// Calculate the player's 2D speed (horizontal)
    pub fn speed_2d(&self) -> f32 {
        (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt()
    }

    /// Calculate the player's 3D speed
    pub fn speed_3d(&self) -> f32 {
        (self.vel_x * self.vel_x + self.vel_y * self.vel_y + self.vel_z * self.vel_z).sqrt()
    }

    /// Calculate distance to another player
    pub fn distance_to(&self, other: &BehavioralVector) -> f32 {
        let dx = self.pos_x - other.pos_x;
        let dy = self.pos_y - other.pos_y;
        let dz = self.pos_z - other.pos_z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_input_vector_size() {
        assert_eq!(std::mem::size_of::<InputVector>(), 14 * 4); // 14 f32 values = 56 bytes
    }

    #[test]
    fn test_output_vector_size() {
        assert_eq!(std::mem::size_of::<OutputVector>(), 2 * 4); // 2 f32 values = 8 bytes
    }

    #[test]
    fn test_behavioral_vector_new() {
        let bv = BehavioralVector::new(42, 76561198123456789);
        assert_eq!(bv.tick, 42);
        assert_eq!(bv.steamid, 76561198123456789);
        assert_eq!(bv.health, 100.0);
        assert_eq!(bv.armor, 0.0);
    }

    #[rstest]
    #[case(0.0, 0.0, 0.0, 0.0)]
    #[case(3.0, 4.0, 0.0, 5.0)]
    #[case(3.0, 4.0, 12.0, 13.0)]
    fn test_speed_calculations(
        #[case] vel_x: f32,
        #[case] vel_y: f32,
        #[case] vel_z: f32,
        #[case] expected_speed_3d: f32,
    ) {
        let mut bv = BehavioralVector::new(1, 1);
        bv.vel_x = vel_x;
        bv.vel_y = vel_y;
        bv.vel_z = vel_z;

        let expected_speed_2d = (vel_x * vel_x + vel_y * vel_y).sqrt();

        assert!((bv.speed_2d() - expected_speed_2d).abs() < 0.001);
        assert!((bv.speed_3d() - expected_speed_3d).abs() < 0.001);
    }

    #[test]
    fn test_distance_calculation() {
        let mut bv1 = BehavioralVector::new(1, 1);
        bv1.pos_x = 0.0;
        bv1.pos_y = 0.0;
        bv1.pos_z = 0.0;

        let mut bv2 = BehavioralVector::new(1, 2);
        bv2.pos_x = 3.0;
        bv2.pos_y = 4.0;
        bv2.pos_z = 0.0;

        assert!((bv1.distance_to(&bv2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_input_vector_from_behavioral() {
        let mut bv = BehavioralVector::new(1, 1);
        bv.health = 75.0;
        bv.armor = 50.0;
        bv.pos_x = 100.0;
        bv.weapon_id = 42;
        bv.is_airborne = 1.0;

        let input = InputVector::from_behavioral(&bv);

        assert_eq!(input.health, 75.0);
        assert_eq!(input.armor, 50.0);
        assert_eq!(input.pos_x, 100.0);
        assert_eq!(input.weapon_id_f32, 42.0);
        assert_eq!(input.is_airborne, 1.0);
        assert_eq!(input.padding, 0.0);
    }

    #[test]
    fn test_serde_behavioral_vector() {
        let bv = BehavioralVector::new(42, 76561198123456789);

        // Test serialization/deserialization with JSON
        let json = serde_json::to_string(&bv).unwrap();
        let bv_deserialized: BehavioralVector = serde_json::from_str(&json).unwrap();

        assert_eq!(bv.tick, bv_deserialized.tick);
        assert_eq!(bv.steamid, bv_deserialized.steamid);
    }

    #[test]
    fn test_binary_serialization() {
        let input = InputVector {
            health: 100.0,
            armor: 50.0,
            pos_x: 1.0,
            pos_y: 2.0,
            pos_z: 3.0,
            vel_x: 0.1,
            vel_y: 0.2,
            vel_z: 0.3,
            yaw: 90.0,
            pitch: 45.0,
            weapon_id_f32: 42.0,
            ammo: 30.0,
            is_airborne: 0.0,
            padding: 0.0,
        };

        let bytes = bytemuck::bytes_of(&input);
        assert_eq!(bytes.len(), std::mem::size_of::<InputVector>());

        let input_deserialized: InputVector = *bytemuck::from_bytes(bytes);
        assert_eq!(input.health, input_deserialized.health);
        assert_eq!(input.pos_x, input_deserialized.pos_x);
        assert_eq!(input.weapon_id_f32, input_deserialized.weapon_id_f32);
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let cs2_error: CS2Error = io_error.into();
        match cs2_error {
            CS2Error::IoError(_) => {} // Success!
            _ => panic!("Error conversion failed"),
        }
    }
}
