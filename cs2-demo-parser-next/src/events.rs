//! Events module - Game event parsing and processing  
//!
//! This module handles parsing of game events from demo data.

use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};

/// Game event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerConnect { 
        user_id: u32, 
        name: String,
        steam_id: u64,
    },
    PlayerDisconnect {
        user_id: u32,
        reason: String,
    },
    PlayerDeath {
        user_id: u32,
        attacker: u32,
        weapon: String,
        headshot: bool,
    },
    RoundStart {
        time_limit: u32,
        frag_limit: u32,
    },
    RoundEnd {
        winner: u8,
        reason: u8,
        message: String,
    },
}

/// Event manager for processing game events
pub struct EventManager {
    events: Vec<(u32, GameEvent)>, // (tick, event)
}

impl EventManager {
    /// Create new event manager
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    /// Add event at specific tick
    pub fn add_event(&mut self, tick: u32, event: GameEvent) {
        self.events.push((tick, event));
    }
    
    /// Get events for tick range
    pub fn get_events_in_range(&self, start_tick: u32, end_tick: u32) -> Vec<&(u32, GameEvent)> {
        self.events
            .iter()
            .filter(|(tick, _)| *tick >= start_tick && *tick <= end_tick)
            .collect()
    }
    
    /// Get all events
    pub fn events(&self) -> &Vec<(u32, GameEvent)> {
        &self.events
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_manager() {
        let mut manager = EventManager::new();
        
        let event = GameEvent::PlayerConnect {
            user_id: 1,
            name: "test_player".to_string(),
            steam_id: 76561198000000000,
        };
        
        manager.add_event(100, event);
        assert_eq!(manager.events().len(), 1);
        
        let events_in_range = manager.get_events_in_range(50, 150);
        assert_eq!(events_in_range.len(), 1);
    }
}