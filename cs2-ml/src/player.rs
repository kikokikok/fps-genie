use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlayerMeta {
    pub steamid: u64,
    pub props: HashMap<String, String>,
    pub active_weapon_name: Option<String>,
    pub ammo_clip: Option<u32>,
}

// Implement a conversion from a demo player to PlayerMeta
impl<T> From<&T> for PlayerMeta 
where
    T: PlayerLike,
{
    fn from(player: &T) -> Self {
        PlayerMeta {
            steamid: player.get_steamid(),
            props: player.get_props(),
            active_weapon_name: player.get_active_weapon_name(),
            ammo_clip: player.get_ammo_clip(),
        }
    }
}

// Trait to abstract over different player implementations
pub trait PlayerLike {
    fn get_steamid(&self) -> u64;
    fn get_props(&self) -> HashMap<String, String>;
    fn get_active_weapon_name(&self) -> Option<String>;
    fn get_ammo_clip(&self) -> Option<u32>;
}
