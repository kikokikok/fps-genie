//! Entities module - Entity parsing and management
//!
//! This module handles parsing and tracking of game entities from demo data.
//! Phase 2 implementation with comprehensive entity management and CS2 entity support.

use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CS2 Entity types and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity ID
    pub id: u32,
    
    /// Entity class ID
    pub class_id: u32,
    
    /// Entity class name
    pub class_name: String,
    
    /// Entity properties as key-value pairs
    pub properties: HashMap<String, EntityProperty>,
    
    /// Entity position in world coordinates
    pub position: Option<[f32; 3]>,
    
    /// Entity rotation (pitch, yaw, roll)
    pub rotation: Option<[f32; 3]>,
    
    /// Entity velocity
    pub velocity: Option<[f32; 3]>,
    
    /// Entity health (for living entities)
    pub health: Option<u32>,
    
    /// Entity armor (for player entities)
    pub armor: Option<u32>,
    
    /// Entity team (for team-based entities)
    pub team: Option<u32>,
    
    /// Entity flags
    pub flags: u32,
    
    /// Last update tick
    pub last_update_tick: u32,
}

/// Entity property value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityProperty {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Vector([f32; 3]),
    Entity(u32), // Reference to another entity
    Array(Vec<EntityProperty>),
}

/// CS2 Entity classes
#[derive(Debug, Clone, PartialEq)]
pub enum CS2EntityClass {
    // Player entities
    CCSPlayerPawn,
    CCSPlayerController,
    CCSPlayerResource,
    
    // Weapon entities
    CWeaponAK47,
    CWeaponM4A1,
    CWeaponAWP,
    CWeaponGlock,
    CWeaponUSPSilencer,
    CWeaponDeagle,
    
    // Grenade entities
    CHEGrenade,
    CFlashbang,
    CSmokeGrenade,
    CDecoyGrenade,
    CMolotovGrenade,
    
    // Game entities
    CCSGameRulesProxy,
    CCSTeam,
    CBombTarget,
    CHostage,
    
    // World entities
    CWorld,
    CFunc_Door,
    CFunc_Button,
    CFunc_Buyzone,
    
    // Effects
    CParticleSystem,
    CEnvExplosion,
    
    // Unknown entity
    Unknown(String),
}

impl CS2EntityClass {
    /// Get entity class from class name string
    pub fn from_name(name: &str) -> Self {
        match name {
            "CCSPlayerPawn" => Self::CCSPlayerPawn,
            "CCSPlayerController" => Self::CCSPlayerController,
            "CCSPlayerResource" => Self::CCSPlayerResource,
            "CWeaponAK47" => Self::CWeaponAK47,
            "CWeaponM4A1" => Self::CWeaponM4A1,
            "CWeaponAWP" => Self::CWeaponAWP,
            "CWeaponGlock" => Self::CWeaponGlock,
            "CWeaponUSPSilencer" => Self::CWeaponUSPSilencer,
            "CWeaponDeagle" => Self::CWeaponDeagle,
            "CHEGrenade" => Self::CHEGrenade,
            "CFlashbang" => Self::CFlashbang,
            "CSmokeGrenade" => Self::CSmokeGrenade,
            "CDecoyGrenade" => Self::CDecoyGrenade,
            "CMolotovGrenade" => Self::CMolotovGrenade,
            "CCSGameRulesProxy" => Self::CCSGameRulesProxy,
            "CCSTeam" => Self::CCSTeam,
            "CBombTarget" => Self::CBombTarget,
            "CHostage" => Self::CHostage,
            "CWorld" => Self::CWorld,
            "CFunc_Door" => Self::CFunc_Door,
            "CFunc_Button" => Self::CFunc_Button,
            "CFunc_Buyzone" => Self::CFunc_Buyzone,
            "CParticleSystem" => Self::CParticleSystem,
            "CEnvExplosion" => Self::CEnvExplosion,
            _ => Self::Unknown(name.to_string()),
        }
    }
    
    /// Get entity class name as string
    pub fn name(&self) -> &str {
        match self {
            Self::CCSPlayerPawn => "CCSPlayerPawn",
            Self::CCSPlayerController => "CCSPlayerController",
            Self::CCSPlayerResource => "CCSPlayerResource",
            Self::CWeaponAK47 => "CWeaponAK47",
            Self::CWeaponM4A1 => "CWeaponM4A1",
            Self::CWeaponAWP => "CWeaponAWP",
            Self::CWeaponGlock => "CWeaponGlock",
            Self::CWeaponUSPSilencer => "CWeaponUSPSilencer",
            Self::CWeaponDeagle => "CWeaponDeagle",
            Self::CHEGrenade => "CHEGrenade",
            Self::CFlashbang => "CFlashbang",
            Self::CSmokeGrenade => "CSmokeGrenade",
            Self::CDecoyGrenade => "CDecoyGrenade",
            Self::CMolotovGrenade => "CMolotovGrenade",
            Self::CCSGameRulesProxy => "CCSGameRulesProxy",
            Self::CCSTeam => "CCSTeam",
            Self::CBombTarget => "CBombTarget",
            Self::CHostage => "CHostage",
            Self::CWorld => "CWorld",
            Self::CFunc_Door => "CFunc_Door",
            Self::CFunc_Button => "CFunc_Button",
            Self::CFunc_Buyzone => "CFunc_Buyzone",
            Self::CParticleSystem => "CParticleSystem",
            Self::CEnvExplosion => "CEnvExplosion",
            Self::Unknown(name) => name,
        }
    }
    
    /// Check if entity is a player
    pub fn is_player(&self) -> bool {
        matches!(self, Self::CCSPlayerPawn | Self::CCSPlayerController)
    }
    
    /// Check if entity is a weapon
    pub fn is_weapon(&self) -> bool {
        matches!(self, 
            Self::CWeaponAK47 | Self::CWeaponM4A1 | Self::CWeaponAWP | 
            Self::CWeaponGlock | Self::CWeaponUSPSilencer | Self::CWeaponDeagle
        )
    }
    
    /// Check if entity is a grenade
    pub fn is_grenade(&self) -> bool {
        matches!(self, 
            Self::CHEGrenade | Self::CFlashbang | Self::CSmokeGrenade | 
            Self::CDecoyGrenade | Self::CMolotovGrenade
        )
    }
}

/// Entity update information
#[derive(Debug, Clone)]
pub struct EntityUpdate {
    pub entity_id: u32,
    pub class_id: u32,
    pub tick: u32,
    pub update_type: EntityUpdateType,
    pub properties: HashMap<String, EntityProperty>,
}

/// Types of entity updates
#[derive(Debug, Clone, PartialEq)]
pub enum EntityUpdateType {
    Create,
    Update,
    Delete,
    EnterPVS, // Enter potentially visible set
    LeavePVS, // Leave potentially visible set
}

/// Advanced entity manager for tracking entity state
pub struct EntityManager {
    /// Active entities
    entities: HashMap<u32, Entity>,
    
    /// Entity class mapping
    entity_classes: HashMap<u32, CS2EntityClass>,
    
    /// Entity updates history (for analysis)
    update_history: Vec<EntityUpdate>,
    
    /// Player entity IDs for quick lookup
    player_entities: HashMap<u32, u32>, // user_id -> entity_id
    
    /// Weapon entity tracking
    weapon_entities: HashMap<u32, Vec<u32>>, // owner_entity_id -> weapon_entity_ids
    
    /// Performance tracking
    total_updates: u64,
    active_entity_count: usize,
    max_entities: usize,
}

impl EntityManager {
    /// Create new entity manager
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            entity_classes: HashMap::new(),
            update_history: Vec::new(),
            player_entities: HashMap::new(),
            weapon_entities: HashMap::new(),
            total_updates: 0,
            active_entity_count: 0,
            max_entities: 2048, // CS2 default max entities
        }
    }
    
    /// Register entity class
    pub fn register_entity_class(&mut self, class_id: u32, class_name: String) {
        let entity_class = CS2EntityClass::from_name(&class_name);
        self.entity_classes.insert(class_id, entity_class);
    }
    
    /// Process entity update
    pub fn process_entity_update(&mut self, update: EntityUpdate) -> Result<()> {
        self.total_updates += 1;
        self.update_history.push(update.clone());
        
        match update.update_type {
            EntityUpdateType::Create => {
                self.create_entity(update)?;
            }
            EntityUpdateType::Update => {
                self.update_entity(update)?;
            }
            EntityUpdateType::Delete => {
                self.delete_entity(update.entity_id)?;
            }
            EntityUpdateType::EnterPVS => {
                self.entity_enter_pvs(update)?;
            }
            EntityUpdateType::LeavePVS => {
                self.entity_leave_pvs(update.entity_id)?;
            }
        }
        
        self.active_entity_count = self.entities.len();
        Ok(())
    }
    
    /// Create new entity
    fn create_entity(&mut self, update: EntityUpdate) -> Result<()> {
        if self.entities.len() >= self.max_entities {
            return Err(Error::InvalidFormat("Maximum entities reached".to_string()));
        }
        
        let class_name = self.entity_classes
            .get(&update.class_id)
            .map(|c| c.name().to_string())
            .unwrap_or_else(|| format!("UnknownClass_{}", update.class_id));
        
        let mut entity = Entity {
            id: update.entity_id,
            class_id: update.class_id,
            class_name: class_name.clone(),
            properties: update.properties.clone(),
            position: None,
            rotation: None,
            velocity: None,
            health: None,
            armor: None,
            team: None,
            flags: 0,
            last_update_tick: update.tick,
        };
        
        // Extract derived fields from properties during creation
        if let Some(EntityProperty::Vector(pos)) = update.properties.get("m_vecOrigin") {
            entity.position = Some(*pos);
        }
        if let Some(EntityProperty::Vector(rot)) = update.properties.get("m_angRotation") {
            entity.rotation = Some(*rot);
        }
        if let Some(EntityProperty::Vector(vel)) = update.properties.get("m_vecVelocity") {
            entity.velocity = Some(*vel);
        }
        if let Some(EntityProperty::Int(health)) = update.properties.get("m_iHealth") {
            entity.health = Some(*health as u32);
        }
        if let Some(EntityProperty::Int(armor)) = update.properties.get("m_ArmorValue") {
            entity.armor = Some(*armor as u32);
        }
        if let Some(EntityProperty::Int(team)) = update.properties.get("m_iTeamNum") {
            entity.team = Some(*team as u32);
        }
        if let Some(EntityProperty::Int(flags)) = update.properties.get("m_fFlags") {
            entity.flags = *flags as u32;
        }
        
        // Special handling for player entities
        if let Some(entity_class) = self.entity_classes.get(&update.class_id) {
            if entity_class.is_player() {
                // Extract player ID if available
                if let Some(EntityProperty::Int(user_id)) = entity.properties.get("m_iUserID") {
                    self.player_entities.insert(*user_id as u32, update.entity_id);
                }
            }
        }
        
        self.entities.insert(update.entity_id, entity);
        Ok(())
    }
    
    /// Update existing entity
    fn update_entity(&mut self, update: EntityUpdate) -> Result<()> {
        if let Some(entity) = self.entities.get_mut(&update.entity_id) {
            // Update properties
            for (key, value) in update.properties {
                entity.properties.insert(key, value);
            }
            
            // Update derived fields from properties
            entity.last_update_tick = update.tick;
            
            // Extract common properties
            if let Some(EntityProperty::Vector(pos)) = entity.properties.get("m_vecOrigin") {
                entity.position = Some(*pos);
            }
            if let Some(EntityProperty::Vector(rot)) = entity.properties.get("m_angRotation") {
                entity.rotation = Some(*rot);
            }
            if let Some(EntityProperty::Vector(vel)) = entity.properties.get("m_vecVelocity") {
                entity.velocity = Some(*vel);
            }
            if let Some(EntityProperty::Int(health)) = entity.properties.get("m_iHealth") {
                entity.health = Some(*health as u32);
            }
            if let Some(EntityProperty::Int(armor)) = entity.properties.get("m_ArmorValue") {
                entity.armor = Some(*armor as u32);
            }
            if let Some(EntityProperty::Int(team)) = entity.properties.get("m_iTeamNum") {
                entity.team = Some(*team as u32);
            }
            if let Some(EntityProperty::Int(flags)) = entity.properties.get("m_fFlags") {
                entity.flags = *flags as u32;
            }
        }
        
        Ok(())
    }
    
    /// Delete entity
    fn delete_entity(&mut self, entity_id: u32) -> Result<()> {
        if let Some(entity) = self.entities.remove(&entity_id) {
            // Clean up player mapping
            if let Some(entity_class) = self.entity_classes.get(&entity.class_id) {
                if entity_class.is_player() {
                    if let Some(EntityProperty::Int(user_id)) = entity.properties.get("m_iUserID") {
                        self.player_entities.remove(&(*user_id as u32));
                    }
                }
            }
            
            // Clean up weapon ownership
            self.weapon_entities.remove(&entity_id);
            for weapon_list in self.weapon_entities.values_mut() {
                weapon_list.retain(|&id| id != entity_id);
            }
        }
        
        Ok(())
    }
    
    /// Handle entity entering PVS
    fn entity_enter_pvs(&mut self, update: EntityUpdate) -> Result<()> {
        // For entities entering PVS, treat as create if not exists, update if exists
        if self.entities.contains_key(&update.entity_id) {
            self.update_entity(update)
        } else {
            self.create_entity(update)
        }
    }
    
    /// Handle entity leaving PVS
    fn entity_leave_pvs(&mut self, entity_id: u32) -> Result<()> {
        // Entity leaves PVS but might still exist, just mark as not visible
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.properties.insert("m_bInPVS".to_string(), EntityProperty::Bool(false));
        }
        Ok(())
    }
    
    /// Get entity by ID
    pub fn get_entity(&self, id: u32) -> Option<&Entity> {
        self.entities.get(&id)
    }
    
    /// Get entity by ID (mutable)
    pub fn get_entity_mut(&mut self, id: u32) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }
    
    /// Get all entities
    pub fn entities(&self) -> &HashMap<u32, Entity> {
        &self.entities
    }
    
    /// Get entities by class
    pub fn get_entities_by_class(&self, class_name: &str) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|entity| entity.class_name == class_name)
            .collect()
    }
    
    /// Get player entities
    pub fn get_player_entities(&self) -> Vec<&Entity> {
        self.player_entities
            .values()
            .filter_map(|entity_id| self.entities.get(entity_id))
            .collect()
    }
    
    /// Get entity by player user ID
    pub fn get_entity_by_user_id(&self, user_id: u32) -> Option<&Entity> {
        self.player_entities
            .get(&user_id)
            .and_then(|entity_id| self.entities.get(entity_id))
    }
    
    /// Get entities within radius of position
    pub fn get_entities_in_radius(&self, center: [f32; 3], radius: f32) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|entity| {
                if let Some(pos) = entity.position {
                    let dx = pos[0] - center[0];
                    let dy = pos[1] - center[1];
                    let dz = pos[2] - center[2];
                    (dx * dx + dy * dy + dz * dz).sqrt() <= radius
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// Get management statistics
    pub fn get_stats(&self) -> EntityManagerStats {
        EntityManagerStats {
            total_updates: self.total_updates,
            active_entities: self.active_entity_count,
            max_entities: self.max_entities,
            player_count: self.player_entities.len(),
            update_history_size: self.update_history.len(),
        }
    }
    
    /// Clear update history (for memory management)
    pub fn clear_update_history(&mut self) {
        self.update_history.clear();
    }
    
    /// Get recent updates
    pub fn get_recent_updates(&self, count: usize) -> &[EntityUpdate] {
        let start = if self.update_history.len() > count {
            self.update_history.len() - count
        } else {
            0
        };
        &self.update_history[start..]
    }
}

/// Entity manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityManagerStats {
    pub total_updates: u64,
    pub active_entities: usize,
    pub max_entities: usize,
    pub player_count: usize,
    pub update_history_size: usize,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_manager_creation() {
        let manager = EntityManager::new();
        assert_eq!(manager.entities().len(), 0);
        assert_eq!(manager.get_stats().total_updates, 0);
    }
    
    #[test]
    fn test_entity_class_from_name() {
        assert_eq!(CS2EntityClass::from_name("CCSPlayerPawn"), CS2EntityClass::CCSPlayerPawn);
        assert_eq!(CS2EntityClass::from_name("CWeaponAK47"), CS2EntityClass::CWeaponAK47);
        assert!(matches!(CS2EntityClass::from_name("UnknownClass"), CS2EntityClass::Unknown(_)));
    }
    
    #[test]
    fn test_entity_class_properties() {
        assert!(CS2EntityClass::CCSPlayerPawn.is_player());
        assert!(CS2EntityClass::CWeaponAK47.is_weapon());
        assert!(CS2EntityClass::CHEGrenade.is_grenade());
        assert!(!CS2EntityClass::CWorld.is_player());
    }
    
    #[test]
    fn test_entity_creation() {
        let mut manager = EntityManager::new();
        manager.register_entity_class(1, "CCSPlayerPawn".to_string());
        
        let mut properties = HashMap::new();
        properties.insert("m_iUserID".to_string(), EntityProperty::Int(1));
        properties.insert("m_iHealth".to_string(), EntityProperty::Int(100));
        
        let update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1000,
            update_type: EntityUpdateType::Create,
            properties,
        };
        
        manager.process_entity_update(update).unwrap();
        
        assert_eq!(manager.entities().len(), 1);
        let entity = manager.get_entity(10).unwrap();
        assert_eq!(entity.class_name, "CCSPlayerPawn");
        assert_eq!(entity.health, Some(100));
    }
    
    #[test]
    fn test_entity_update() {
        let mut manager = EntityManager::new();
        manager.register_entity_class(1, "CCSPlayerPawn".to_string());
        
        // Create entity
        let mut properties = HashMap::new();
        properties.insert("m_iHealth".to_string(), EntityProperty::Int(100));
        
        let create_update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1000,
            update_type: EntityUpdateType::Create,
            properties,
        };
        
        manager.process_entity_update(create_update).unwrap();
        
        // Update entity
        let mut update_properties = HashMap::new();
        update_properties.insert("m_iHealth".to_string(), EntityProperty::Int(50));
        update_properties.insert("m_vecOrigin".to_string(), EntityProperty::Vector([100.0, 200.0, 300.0]));
        
        let update_update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1100,
            update_type: EntityUpdateType::Update,
            properties: update_properties,
        };
        
        manager.process_entity_update(update_update).unwrap();
        
        let entity = manager.get_entity(10).unwrap();
        assert_eq!(entity.health, Some(50));
        assert_eq!(entity.position, Some([100.0, 200.0, 300.0]));
        assert_eq!(entity.last_update_tick, 1100);
    }
    
    #[test]
    fn test_entity_deletion() {
        let mut manager = EntityManager::new();
        manager.register_entity_class(1, "CCSPlayerPawn".to_string());
        
        let properties = HashMap::new();
        let create_update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1000,
            update_type: EntityUpdateType::Create,
            properties,
        };
        
        manager.process_entity_update(create_update).unwrap();
        assert_eq!(manager.entities().len(), 1);
        
        let delete_update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1100,
            update_type: EntityUpdateType::Delete,
            properties: HashMap::new(),
        };
        
        manager.process_entity_update(delete_update).unwrap();
        assert_eq!(manager.entities().len(), 0);
    }
    
    #[test]
    fn test_get_entities_in_radius() {
        let mut manager = EntityManager::new();
        manager.register_entity_class(1, "CCSPlayerPawn".to_string());
        
        // Create entities at different positions
        for i in 0..3 {
            let mut properties = HashMap::new();
            properties.insert("m_vecOrigin".to_string(), EntityProperty::Vector([i as f32 * 100.0, 0.0, 0.0]));
            
            let update = EntityUpdate {
                entity_id: i,
                class_id: 1,
                tick: 1000,
                update_type: EntityUpdateType::Create,
                properties,
            };
            
            manager.process_entity_update(update).unwrap();
        }
        
        let entities_near_origin = manager.get_entities_in_radius([0.0, 0.0, 0.0], 150.0);
        assert_eq!(entities_near_origin.len(), 2); // Entities at 0.0 and 100.0
    }
    
    #[test]
    fn test_player_entity_tracking() {
        let mut manager = EntityManager::new();
        manager.register_entity_class(1, "CCSPlayerPawn".to_string());
        
        let mut properties = HashMap::new();
        properties.insert("m_iUserID".to_string(), EntityProperty::Int(5));
        
        let update = EntityUpdate {
            entity_id: 10,
            class_id: 1,
            tick: 1000,
            update_type: EntityUpdateType::Create,
            properties,
        };
        
        manager.process_entity_update(update).unwrap();
        
        let player_entity = manager.get_entity_by_user_id(5).unwrap();
        assert_eq!(player_entity.id, 10);
        
        let player_entities = manager.get_player_entities();
        assert_eq!(player_entities.len(), 1);
    }
}