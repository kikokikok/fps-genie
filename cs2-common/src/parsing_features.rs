use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingPreset {
    Minimal,
    Standard,
    Rich,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ParsingFeatures: u32 {
        const AIM        = 1 << 0;
        const MOVEMENT   = 1 << 1;
        const INFO       = 1 << 2;
        const UTILITY    = 1 << 3;
        const OBJECTIVE  = 1 << 4;
        const ECONOMY    = 1 << 5;
        const RULES      = 1 << 6;
        const VALIDATION = 1 << 7;
    }
}

impl ParsingPreset {
    pub fn to_features(self) -> ParsingFeatures {
        match self {
            ParsingPreset::Minimal => {
                ParsingFeatures::AIM | ParsingFeatures::UTILITY | ParsingFeatures::OBJECTIVE
            }
            ParsingPreset::Standard => {
                ParsingFeatures::AIM
                    | ParsingFeatures::MOVEMENT
                    | ParsingFeatures::UTILITY
                    | ParsingFeatures::OBJECTIVE
                    | ParsingFeatures::INFO
                    | ParsingFeatures::RULES
            }
            ParsingPreset::Rich => {
                ParsingFeatures::AIM
                    | ParsingFeatures::MOVEMENT
                    | ParsingFeatures::UTILITY
                    | ParsingFeatures::OBJECTIVE
                    | ParsingFeatures::ECONOMY
                    | ParsingFeatures::RULES
                    | ParsingFeatures::INFO
                    | ParsingFeatures::VALIDATION
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Wanted {
    pub player_props: Vec<String>,
    pub other_props: Vec<String>,
    pub events: Vec<String>,
}

pub fn build_wanted(features: ParsingFeatures) -> Wanted {
    let mut player_props: Vec<String> = Vec::new();
    let mut other_props: Vec<String> = Vec::new();
    let mut events: Vec<String> = Vec::new();

    if features.contains(ParsingFeatures::AIM) {
        player_props.extend(
            [
                "X",
                "Y",
                "Z",
                "velocity[0]",
                "velocity[1]",
                "velocity[2]",
                "m_angEyeAngles",
                "m_aimPunchAngle",
                "m_aimPunchAngleVel",
                "m_aimPunchTickBase",
                "m_aimPunchTickFraction",
                "m_iHealth",
                "m_ArmorValue",
                "m_lifeState",
                "m_bIsScoped",
                "m_hActiveWeapon",
                "m_iClip1",
                "CCSPlayerPawn.CCSPlayer_WeaponServices.m_iAmmo",
                "m_fAccuracyPenalty",
                "m_flRecoilIndex",
                "m_fLastShotTime",
                "m_bInReload",
                "m_flNextPrimaryAttackTickRatio",
                "m_flNextSecondaryAttackTickRatio",
                "m_weaponMode",
                "m_bBurstMode",
                "CCSPlayerPawn.m_iShotsFired",
                "CCSPlayerPawn.CCSPlayer_BulletServices.m_totalHitsOnServer",
            ]
            .into_iter()
            .map(String::from),
        );
        events.extend(
            [
                "weapon_fire",
                "player_hurt",
                "player_death",
                "weapon_reload",
                "weapon_zoom",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::MOVEMENT) {
        player_props.extend(
            [
                "m_bIsWalking",
                "is_airborne",
                "m_fFlags",
                "m_iMoveState",
                "m_MoveType",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDesiresDuck",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDuckOverride",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_bOldJumpPressed",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_nLadderSurfacePropIndex",
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_vecLadderNormal",
                "CCSPlayerPawn.m_flVelocityModifier",
            ]
            .into_iter()
            .map(String::from),
        );
        events.extend(
            ["player_jump", "player_footstep"]
                .into_iter()
                .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::INFO) {
        player_props.extend(
            [
                "CCSPlayerPawn.m_bSpotted",
                "CCSPlayerPawn.m_bSpottedByMask",
                "CCSPlayerPawn.m_szLastPlaceName",
                "CCSPlayerPawn.m_flFlashDuration",
                "CCSPlayerPawn.m_flFlashMaxAlpha",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::UTILITY) {
        events.extend(
            [
                "flashbang_detonate",
                "player_blind",
                "smokegrenade_detonate",
                "smokegrenade_expired",
                "inferno_startburn",
                "inferno_expire",
                "hegrenade_detonate",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::OBJECTIVE) {
        events.extend(
            [
                "bomb_beginplant",
                "bomb_planted",
                "bomb_exploded",
                "bomb_begindefuse",
                "bomb_defused",
                "bomb_dropped",
                "bomb_pickup",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::ECONOMY) {
        player_props.extend(
            [
                "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount",
                "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iStartAccount",
                "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iCashSpentThisRound",
                "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iTotalCashSpent",
                "CCSPlayerPawn.m_unRoundStartEquipmentValue",
                "CCSPlayerPawn.m_unFreezetimeEndEquipmentValue",
                "CCSPlayerPawn.m_unCurrentEquipmentValue",
                "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasDefuser",
                "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasHelmet",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::RULES) {
        other_props.extend(
            [
                "CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod",
                "CCSGameRulesProxy.CCSGameRules.m_bWarmupPeriod",
                "CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime",
                "CCSGameRulesProxy.CCSGameRules.m_iRoundTime",
                "CCSGameRulesProxy.CCSGameRules.m_flRestartRoundTime",
                "CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed",
                "CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveCTLoses",
                "CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveTerroristLoses",
                "CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_CT",
                "CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_T",
                "CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason",
            ]
            .into_iter()
            .map(String::from),
        );
        events.extend(
            [
                "round_prestart",
                "round_freeze_end",
                "buytime_ended",
                "round_start",
                "round_end",
                "round_officially_ended",
                "cs_pre_restart",
            ]
            .into_iter()
            .map(String::from),
        );
    }

    if features.contains(ParsingFeatures::VALIDATION) {
        // Reserved for extended cross-check props
    }

    Wanted {
        player_props,
        other_props,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_mapping() {
        let rich = ParsingPreset::Rich.to_features();
        assert!(rich.contains(ParsingFeatures::AIM));
        assert!(rich.contains(ParsingFeatures::ECONOMY));
        assert!(rich.contains(ParsingFeatures::RULES));
    }

    #[test]
    fn build_wanted_nonempty() {
        let w = build_wanted(ParsingPreset::Standard.to_features());
        assert!(!w.player_props.is_empty());
        assert!(!w.events.is_empty());
    }
}
