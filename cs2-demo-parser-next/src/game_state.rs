//! Game state module - Tracking overall game state
//!
//! This module manages the overall game state derived from demo parsing.

use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};

/// Overall game state tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current tick
    pub current_tick: u32,
    
    /// Current round number
    pub round_number: u32,
    
    /// Game phase (warmup, live, etc.)
    pub game_phase: GamePhase,
    
    /// Score information
    pub score: Score,
}

/// Game phase enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    Warmup,
    Live,
    Halftime,
    Overtime,
    Finished,
}

/// Score tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    /// Team 1 score
    pub team1: u32,
    
    /// Team 2 score
    pub team2: u32,
}

/// Game state manager
pub struct GameStateManager {
    state: GameState,
}

impl GameStateManager {
    /// Create new game state manager
    pub fn new() -> Self {
        Self {
            state: GameState {
                current_tick: 0,
                round_number: 0,
                game_phase: GamePhase::Warmup,
                score: Score { team1: 0, team2: 0 },
            },
        }
    }
    
    /// Update current tick
    pub fn update_tick(&mut self, tick: u32) {
        self.state.current_tick = tick;
    }
    
    /// Update game phase
    pub fn update_phase(&mut self, phase: GamePhase) {
        self.state.game_phase = phase;
    }
    
    /// Get current state
    pub fn state(&self) -> &GameState {
        &self.state
    }
}

impl Default for GameStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_state_manager() {
        let mut manager = GameStateManager::new();
        
        manager.update_tick(1000);
        assert_eq!(manager.state().current_tick, 1000);
        
        manager.update_phase(GamePhase::Live);
        matches!(manager.state().game_phase, GamePhase::Live);
    }
}