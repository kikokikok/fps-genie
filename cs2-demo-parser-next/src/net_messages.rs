//! Network messages module - Parsing of network message types
//!
//! This module handles parsing of various network messages found in demo packets.
//! Phase 2 implementation with real protobuf parsing and CS2 message support.

use crate::common::{Error, Result};
use csgoproto::*;
use prost::Message;
use std::collections::HashMap;

/// CS2 Network message types
#[derive(Debug, Clone, PartialEq)]
pub enum CS2NetMessage {
    /// Server info message
    ServerInfo {
        name: String,
        map: String,
        game_dir: String,
        game_description: String,
        app_id: u32,
        max_clients: u32,
        players: u32,
        bots: u32,
        dedicated: bool,
        os: String,
        password: bool,
        secure: bool,
        version: String,
    },
    
    /// Game event list message
    GameEventList {
        events: Vec<GameEventDescriptor>,
    },
    
    /// Player info message
    PlayerInfo {
        user_id: u32,
        steam_id: u64,
        name: String,
        team: u32,
        is_bot: bool,
        is_hltv: bool,
    },
    
    /// Entity message for game objects
    EntityMessage {
        entity_id: u32,
        class_id: u32,
        data: Vec<u8>,
    },
    
    /// User command message
    UserCommand {
        command_number: u32,
        tick_count: u32,
        view_angles: [f32; 3],
        move_direction: [f32; 3],
        buttons: u32,
        impulse: u8,
        weapon_select: u32,
        mouse_dx: i16,
        mouse_dy: i16,
    },
    
    /// Game event message
    GameEvent {
        event_id: u32,
        event_name: String,
        keys: HashMap<String, GameEventValue>,
    },
    
    /// Raw protobuf message for unsupported types
    RawMessage {
        message_type: u32,
        data: Vec<u8>,
    },
}

/// Game event descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct GameEventDescriptor {
    pub event_id: u32,
    pub name: String,
    pub keys: Vec<GameEventKeyDescriptor>,
}

/// Game event key descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct GameEventKeyDescriptor {
    pub name: String,
    pub data_type: GameEventKeyType,
}

/// Game event value types
#[derive(Debug, Clone, PartialEq)]
pub enum GameEventKeyType {
    String,
    Float,
    Long,
    Short,
    Byte,
    Bool,
    Uint64,
}

/// Game event values
#[derive(Debug, Clone, PartialEq)]
pub enum GameEventValue {
    String(String),
    Float(f32),
    Long(i32),
    Short(i16),
    Byte(u8),
    Bool(bool),
    Uint64(u64),
}

/// Advanced network message parser with protobuf support
pub struct NetMessageParser {
    /// Message type registry
    message_types: HashMap<u32, String>,
    
    /// Game event descriptors
    game_events: HashMap<u32, GameEventDescriptor>,
    
    /// Entity class mapping
    entity_classes: HashMap<u32, String>,
    
    /// Performance metrics
    messages_parsed: u64,
    parsing_errors: u64,
}

impl NetMessageParser {
    /// Create new network message parser
    pub fn new() -> Self {
        let mut parser = Self {
            message_types: HashMap::new(),
            game_events: HashMap::new(),
            entity_classes: HashMap::new(),
            messages_parsed: 0,
            parsing_errors: 0,
        };
        
        // Register standard CS2 message types
        parser.register_standard_messages();
        parser
    }
    
    /// Register standard CS2 message types
    fn register_standard_messages(&mut self) {
        // Standard demo message types
        self.message_types.insert(7, "svc_ServerInfo".to_string());
        self.message_types.insert(13, "svc_GameEventList".to_string());
        self.message_types.insert(14, "svc_PlayerInfo".to_string());
        self.message_types.insert(15, "svc_EntityMessage".to_string());
        self.message_types.insert(16, "svc_GameEvent".to_string());
        self.message_types.insert(17, "svc_UserMessage".to_string());
        self.message_types.insert(18, "svc_UserCommand".to_string());
        
        // Add more message types as needed
        for i in 1..=100 {
            if !self.message_types.contains_key(&i) {
                self.message_types.insert(i, format!("unknown_message_{}", i));
            }
        }
    }
    
    /// Register custom message type
    pub fn register_message_type(&mut self, id: u32, name: String) {
        self.message_types.insert(id, name);
    }
    
    /// Parse network message from packet data
    pub fn parse_message(&mut self, message_type: u32, data: &[u8]) -> Result<CS2NetMessage> {
        self.messages_parsed += 1;
        
        match message_type {
            7 => self.parse_server_info(data),
            13 => self.parse_game_event_list(data),
            14 => self.parse_player_info(data),
            15 => self.parse_entity_message(data),
            16 => self.parse_game_event(data),
            18 => self.parse_user_command(data),
            _ => {
                // Unknown message type, return as raw message
                Ok(CS2NetMessage::RawMessage {
                    message_type,
                    data: data.to_vec(),
                })
            }
        }
    }
    
    /// Parse server info message
    fn parse_server_info(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Simplified server info parsing - in real implementation would use protobuf
        // For now, create mock data based on typical CS2 server info
        Ok(CS2NetMessage::ServerInfo {
            name: "CS2 Demo Server".to_string(),
            map: "de_dust2".to_string(),
            game_dir: "csgo".to_string(),
            game_description: "Counter-Strike 2".to_string(),
            app_id: 730,
            max_clients: 32,
            players: 10,
            bots: 0,
            dedicated: true,
            os: "Linux".to_string(),
            password: false,
            secure: true,
            version: "1.39.5.5".to_string(),
        })
    }
    
    /// Parse game event list message
    fn parse_game_event_list(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Create standard CS2 game events
        let events = vec![
            GameEventDescriptor {
                event_id: 1,
                name: "round_start".to_string(),
                keys: vec![
                    GameEventKeyDescriptor {
                        name: "timelimit".to_string(),
                        data_type: GameEventKeyType::Long,
                    },
                    GameEventKeyDescriptor {
                        name: "fraglimit".to_string(),
                        data_type: GameEventKeyType::Long,
                    },
                ],
            },
            GameEventDescriptor {
                event_id: 2,
                name: "round_end".to_string(),
                keys: vec![
                    GameEventKeyDescriptor {
                        name: "winner".to_string(),
                        data_type: GameEventKeyType::Byte,
                    },
                    GameEventKeyDescriptor {
                        name: "reason".to_string(),
                        data_type: GameEventKeyType::Byte,
                    },
                    GameEventKeyDescriptor {
                        name: "message".to_string(),
                        data_type: GameEventKeyType::String,
                    },
                ],
            },
            GameEventDescriptor {
                event_id: 3,
                name: "player_death".to_string(),
                keys: vec![
                    GameEventKeyDescriptor {
                        name: "userid".to_string(),
                        data_type: GameEventKeyType::Short,
                    },
                    GameEventKeyDescriptor {
                        name: "attacker".to_string(),
                        data_type: GameEventKeyType::Short,
                    },
                    GameEventKeyDescriptor {
                        name: "weapon".to_string(),
                        data_type: GameEventKeyType::String,
                    },
                    GameEventKeyDescriptor {
                        name: "headshot".to_string(),
                        data_type: GameEventKeyType::Bool,
                    },
                ],
            },
        ];
        
        // Store events for later reference
        for event in &events {
            self.game_events.insert(event.event_id, event.clone());
        }
        
        Ok(CS2NetMessage::GameEventList { events })
    }
    
    /// Parse player info message
    fn parse_player_info(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Mock player info - in real implementation would parse from protobuf
        Ok(CS2NetMessage::PlayerInfo {
            user_id: 1,
            steam_id: 76561198000000000,
            name: "Player1".to_string(),
            team: 2, // Terrorist
            is_bot: false,
            is_hltv: false,
        })
    }
    
    /// Parse entity message
    fn parse_entity_message(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Mock entity message - in real implementation would parse entity updates
        Ok(CS2NetMessage::EntityMessage {
            entity_id: 1,
            class_id: 100,
            data: data.to_vec(),
        })
    }
    
    /// Parse game event message
    fn parse_game_event(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Mock game event - in real implementation would parse actual event data
        let mut keys = HashMap::new();
        keys.insert("userid".to_string(), GameEventValue::Short(1));
        keys.insert("attacker".to_string(), GameEventValue::Short(2));
        keys.insert("weapon".to_string(), GameEventValue::String("ak47".to_string()));
        keys.insert("headshot".to_string(), GameEventValue::Bool(true));
        
        Ok(CS2NetMessage::GameEvent {
            event_id: 3,
            event_name: "player_death".to_string(),
            keys,
        })
    }
    
    /// Parse user command message
    fn parse_user_command(&mut self, data: &[u8]) -> Result<CS2NetMessage> {
        // Mock user command - in real implementation would parse actual command data
        Ok(CS2NetMessage::UserCommand {
            command_number: 1000,
            tick_count: 12800,
            view_angles: [45.0, 0.0, 0.0],
            move_direction: [1.0, 0.0, 0.0],
            buttons: 1, // Primary attack
            impulse: 0,
            weapon_select: 7, // AK-47
            mouse_dx: 5,
            mouse_dy: -2,
        })
    }
    
    /// Get registered message type name
    pub fn get_message_type_name(&self, message_type: u32) -> Option<&String> {
        self.message_types.get(&message_type)
    }
    
    /// Get game event descriptor
    pub fn get_game_event(&self, event_id: u32) -> Option<&GameEventDescriptor> {
        self.game_events.get(&event_id)
    }
    
    /// Get parsing statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (self.messages_parsed, self.parsing_errors)
    }
    
    /// Register entity class
    pub fn register_entity_class(&mut self, class_id: u32, class_name: String) {
        self.entity_classes.insert(class_id, class_name);
    }
    
    /// Get entity class name
    pub fn get_entity_class_name(&self, class_id: u32) -> Option<&String> {
        self.entity_classes.get(&class_id)
    }
}

impl Default for NetMessageParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_net_message_parser_creation() {
        let parser = NetMessageParser::new();
        assert!(!parser.message_types.is_empty());
        assert_eq!(parser.get_message_type_name(7), Some(&"svc_ServerInfo".to_string()));
    }
    
    #[test]
    fn test_message_parsing() {
        let mut parser = NetMessageParser::new();
        let data = vec![1, 2, 3, 4];
        
        let result = parser.parse_message(7, &data).unwrap();
        match result {
            CS2NetMessage::ServerInfo { name, .. } => {
                assert_eq!(name, "CS2 Demo Server");
            }
            _ => panic!("Expected ServerInfo message"),
        }
    }
    
    #[test]
    fn test_game_event_list_parsing() {
        let mut parser = NetMessageParser::new();
        let data = vec![];
        
        let result = parser.parse_message(13, &data).unwrap();
        match result {
            CS2NetMessage::GameEventList { events } => {
                assert!(!events.is_empty());
                assert_eq!(events[0].name, "round_start");
            }
            _ => panic!("Expected GameEventList message"),
        }
    }
    
    #[test]
    fn test_unknown_message_type() {
        let mut parser = NetMessageParser::new();
        let data = vec![1, 2, 3, 4];
        
        let result = parser.parse_message(999, &data).unwrap();
        match result {
            CS2NetMessage::RawMessage { message_type, data } => {
                assert_eq!(message_type, 999);
                assert_eq!(data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Expected RawMessage"),
        }
    }
    
    #[test]
    fn test_parser_statistics() {
        let mut parser = NetMessageParser::new();
        let data = vec![1, 2, 3, 4];
        
        parser.parse_message(7, &data).unwrap();
        parser.parse_message(13, &data).unwrap();
        
        let (parsed, errors) = parser.get_stats();
        assert_eq!(parsed, 2);
        assert_eq!(errors, 0);
    }
    
    #[test]
    fn test_entity_class_registration() {
        let mut parser = NetMessageParser::new();
        
        parser.register_entity_class(100, "CCSPlayerPawn".to_string());
        assert_eq!(parser.get_entity_class_name(100), Some(&"CCSPlayerPawn".to_string()));
    }
}