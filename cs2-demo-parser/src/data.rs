#![allow(non_snake_case)]

use crate::first_pass::parser_settings::ParserInputs;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::prop_controller::*;
use crate::parse_demo::DemoOutput;
use crate::parse_demo::Parser;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::parser_settings::create_huffman_lookup_table;
use ahash::AHashMap;

use memmap2::MmapOptions;
use std::collections::BTreeMap;
use std::fs::File;

/// Parse a demo file with comprehensive settings to extract all data
/// This mimics the e2e_test.rs approach for complete demo parsing
pub fn parse_demo_comprehensive(demo_path: &str) -> Result<(DemoOutput, PropController, BTreeMap<String, Vec<GameEvent>>), Box<dyn std::error::Error>> {
    let huf = create_huffman_lookup_table();
    
    // Comprehensive list of wanted properties - similar to e2e_test.rs
    let wanted_props = vec![
        "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_flLastTeleportTime".to_string(),
        "CCSPlayerPawn.CCSPlayer_BulletServices.m_totalHitsOnServer".to_string(),
        "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasDefuser".to_string(),
        "CCSPlayerPawn.CCSPlayer_ItemServices.m_bHasHelmet".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_arrForceSubtickMoveWhen".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDesiresDuck".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDuckOverride".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_bOldJumpPressed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_fStashGrenadeParameterWhen".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flOffsetTickCompleteTime".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flOffsetTickStashedSpeed".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_flStamina".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_nButtonDownMaskPrev".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_nLadderSurfacePropIndex".to_string(),
        "CCSPlayerPawn.CCSPlayer_MovementServices.m_vecLadderNormal".to_string(),
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_iAmmo".to_string(),
        "CCSPlayerPawn.m_ArmorValue".to_string(),
        "CCSPlayerPawn.m_MoveType".to_string(),
        "CCSPlayerPawn.m_aimPunchAngle".to_string(),
        "CCSPlayerPawn.m_aimPunchAngleVel".to_string(),
        "CCSPlayerPawn.m_aimPunchTickBase".to_string(),
        "CCSPlayerPawn.m_aimPunchTickFraction".to_string(),
        "CCSPlayerPawn.m_angEyeAngles".to_string(),
        "CCSPlayerPawn.m_bClientRagdoll".to_string(),
        "CCSPlayerPawn.m_bClientSideRagdoll".to_string(),
        "CCSPlayerPawn.m_bHasMovedSinceSpawn".to_string(),
        "CCSPlayerPawn.m_bInBombZone".to_string(),
        "CCSPlayerPawn.m_bInBuyZone".to_string(),
        "CCSPlayerPawn.m_bIsBuyMenuOpen".to_string(),
        "CCSPlayerPawn.m_bIsDefusing".to_string(),
        "CCSPlayerPawn.m_bIsScoped".to_string(),
        "CCSPlayerPawn.m_bIsWalking".to_string(),
        "CCSPlayerPawn.m_bKilledByHeadshot".to_string(),
        "CCSPlayerPawn.m_bRagdollDamageHeadshot".to_string(),
        "CCSPlayerPawn.m_bResumeZoom".to_string(),
        "CCSPlayerPawn.m_bSpotted".to_string(),
        "CCSPlayerPawn.m_bSpottedByMask".to_string(),
        "CCSPlayerPawn.m_bWaitForNoAttack".to_string(),
        "CCSPlayerPawn.m_fFlags".to_string(),
        "CCSPlayerPawn.m_fMolotovDamageTime".to_string(),
        "CCSPlayerPawn.m_flCreateTime".to_string(),
        "CCSPlayerPawn.m_flDeathTime".to_string(),
        "CCSPlayerPawn.m_flEmitSoundTime".to_string(),
        "CCSPlayerPawn.m_flFlashDuration".to_string(),
        "CCSPlayerPawn.m_flFlashMaxAlpha".to_string(),
        "CCSPlayerPawn.m_flHitHeading".to_string(),
        "CCSPlayerPawn.m_flProgressBarStartTime".to_string(),
        "CCSPlayerPawn.m_flSlopeDropHeight".to_string(),
        "CCSPlayerPawn.m_flSlopeDropOffset".to_string(),
        "CCSPlayerPawn.m_flTimeOfLastInjury".to_string(),
        "CCSPlayerPawn.m_flVelocityModifier".to_string(),
        "CCSPlayerPawn.m_iHealth".to_string(),
        "CCSPlayerPawn.m_iMoveState".to_string(),
        "CCSPlayerPawn.m_iPlayerState".to_string(),
        "CCSPlayerPawn.m_iProgressBarDuration".to_string(),
        "CCSPlayerPawn.m_iShotsFired".to_string(),
        "CCSPlayerPawn.m_iTeamNum".to_string(),
        "CCSPlayerPawn.m_lifeState".to_string(),
        "CCSPlayerPawn.m_nCollisionFunctionMask".to_string(),
        "CCSPlayerPawn.m_nEnablePhysics".to_string(),
        "CCSPlayerPawn.m_nEntityId".to_string(),
        "CCSPlayerPawn.m_nForceBone".to_string(),
        "CCSPlayerPawn.m_nHierarchyId".to_string(),
        "CCSPlayerPawn.m_nHitBodyPart".to_string(),
        "CCSPlayerPawn.m_nInteractsAs".to_string(),
        "CCSPlayerPawn.m_nInteractsExclude".to_string(),
        "CCSPlayerPawn.m_nInteractsWith".to_string(),
        "CCSPlayerPawn.m_nLastConcurrentKilled".to_string(),
        "CCSPlayerPawn.m_nLastKillerIndex".to_string(),
        "CCSPlayerPawn.m_nRagdollDamageBone".to_string(),
        "CCSPlayerPawn.m_nWhichBombZone".to_string(),
        "CCSPlayerPawn.m_qDeathEyeAngles".to_string(),
        "CCSPlayerPawn.m_szLastPlaceName".to_string(),
        "CCSPlayerPawn.m_szRagdollDamageWeaponName".to_string(),
        "CCSPlayerPawn.m_thirdPersonHeading".to_string(),
        "CCSPlayerPawn.m_ubInterpolationFrame".to_string(),
        "CCSPlayerPawn.m_unCurrentEquipmentValue".to_string(),
        "CCSPlayerPawn.m_unFreezetimeEndEquipmentValue".to_string(),
        "CCSPlayerPawn.m_unRoundStartEquipmentValue".to_string(),
        "CCSPlayerPawn.m_vDecalForwardAxis".to_string(),
        "CCSPlayerPawn.m_vDecalPosition".to_string(),
        "CCSPlayerPawn.m_vHeadConstraintOffset".to_string(),
        "CCSPlayerPawn.m_vRagdollDamageForce".to_string(),
        "CCSPlayerPawn.m_vRagdollDamagePosition".to_string(),
        "CCSPlayerPawn.m_vRagdollServerOrigin".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bBombDropped".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bBombPlanted".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bGameRestart".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bHasMatchStarted".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bTeamIntroPeriod".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_bWarmupPeriod".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fMatchStartTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodEnd".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodStart".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_flGameStartTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_flRestartRoundTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_gamePhase".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_CT".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_T".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iMatchStats_RoundResults".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveCTLoses".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveTerroristLoses".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iRoundTime".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_iRoundWinStatus".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_nRoundsPlayedThisPhase".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_timeUntilNextPhaseStarts".to_string(),
        "CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iAssists".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iCashEarned".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDamage".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDeaths".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEnemiesFlashed".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEquipmentValue".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iHeadShotKills".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKillReward".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKills".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iLiveTime".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iMoneySaved".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iObjective".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iUtilityDamage".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iAssists".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iCashEarned".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDamage".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDeaths".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemiesFlashed".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy3Ks".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEquipmentValue".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iHeadShotKills".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKillReward".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKills".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iLiveTime".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iMoneySaved".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iNumRoundKills".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iNumRoundKillsHeadshots".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iObjective".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iUtilityDamage".to_string(),
        "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_unTotalRoundDamageDealt".to_string(),
        "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount".to_string(),
        "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iCashSpentThisRound".to_string(),
        "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iStartAccount".to_string(),
        "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iTotalCashSpent".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsFriendly".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsLeader".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsTeacher".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicLevel".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_rank".to_string(),
        "CCSPlayerController.CCSPlayerController_InventoryServices.m_unMusicID".to_string(),
        "CCSPlayerController.m_bCanControlObservedBot".to_string(),
        "CCSPlayerController.m_bEverPlayedOnTeam".to_string(),
        "CCSPlayerController.m_bPawnHasDefuser".to_string(),
        "CCSPlayerController.m_bPawnHasHelmet".to_string(),
        "CCSPlayerController.m_bPawnIsAlive".to_string(),
        "CCSPlayerController.m_fFlags".to_string(),
        "CCSPlayerController.m_flCreateTime".to_string(),
        "CCSPlayerController.m_hOriginalControllerOfCurrentPawn".to_string(),
        "CCSPlayerController.m_hPawn".to_string(),
        "CCSPlayerController.m_hPlayerPawn".to_string(),
        "CCSPlayerController.m_iCompTeammateColor".to_string(),
        "CCSPlayerController.m_iCompetitiveRankType".to_string(),
        "CCSPlayerController.m_iCompetitiveRanking".to_string(),
        "CCSPlayerController.m_iCompetitiveRankingPredicted_Loss".to_string(),
        "CCSPlayerController.m_iCompetitiveRankingPredicted_Tie".to_string(),
        "CCSPlayerController.m_iCompetitiveRankingPredicted_Win".to_string(),
        "CCSPlayerController.m_iCompetitiveWins".to_string(),
        "CCSPlayerController.m_iConnected".to_string(),
        "CCSPlayerController.m_iMVPs".to_string(),
        "CCSPlayerController.m_iPawnArmor".to_string(),
        "CCSPlayerController.m_iPawnHealth".to_string(),
        "CCSPlayerController.m_iPawnLifetimeEnd".to_string(),
        "CCSPlayerController.m_iPawnLifetimeStart".to_string(),
        "CCSPlayerController.m_iPendingTeamNum".to_string(),
        "CCSPlayerController.m_iPing".to_string(),
        "CCSPlayerController.m_iScore".to_string(),
        "CCSPlayerController.m_iTeamNum".to_string(),
        "CCSPlayerController.m_iszPlayerName".to_string(),
        "CCSPlayerController.m_nDisconnectionTick".to_string(),
        "CCSPlayerController.m_nPawnCharacterDefIndex".to_string(),
        "CCSPlayerController.m_nQuestProgressReason".to_string(),
        "CCSPlayerController.m_nTickBase".to_string(),
        "CCSPlayerController.m_steamID".to_string(),
        "CCSPlayerController.m_szCrosshairCodes".to_string(),
        "CCSPlayerController.m_unActiveQuestId".to_string(),
        "CCSPlayerController.m_unPlayerTvControlFlags".to_string(),
        "CBodyComponentBaseAnimGraph.m_MeshGroupMask".to_string(),
        "CBodyComponentBaseAnimGraph.m_angRotation".to_string(),
        "CBodyComponentBaseAnimGraph.m_cellX".to_string(),
        "CBodyComponentBaseAnimGraph.m_cellY".to_string(),
        "CBodyComponentBaseAnimGraph.m_cellZ".to_string(),
        "CBodyComponentBaseAnimGraph.m_hParent".to_string(),
        "CBodyComponentBaseAnimGraph.m_hSequence".to_string(),
        "CBodyComponentBaseAnimGraph.m_nAnimLoopMode".to_string(),
        "CBodyComponentBaseAnimGraph.m_nIdealMotionType".to_string(),
        "CBodyComponentBaseAnimGraph.m_nNewSequenceParity".to_string(),
        "CBodyComponentBaseAnimGraph.m_nRandomSeedOffset".to_string(),
        "CBodyComponentBaseAnimGraph.m_nResetEventsParity".to_string(),
        "CBodyComponentBaseAnimGraph.m_vecX".to_string(),
        "CBodyComponentBaseAnimGraph.m_vecY".to_string(),
        "CBodyComponentBaseAnimGraph.m_vecZ".to_string(),
        "CEconItemAttribute.m_bSetBonus".to_string(),
        "CEconItemAttribute.m_flInitialValue".to_string(),
        "CEconItemAttribute.m_iAttributeDefinitionIndex".to_string(),
        "CEconItemAttribute.m_iRawValue32".to_string(),
        "CEconItemAttribute.m_nRefundableCurrency".to_string(),
        "m_MoveType".to_string(),
        "m_OriginalOwnerXuidHigh".to_string(),
        "m_OriginalOwnerXuidLow".to_string(),
        "m_bBurstMode".to_string(),
        "m_bInReload".to_string(),
        "m_bReloadVisuallyComplete".to_string(),
        "m_fAccuracyPenalty".to_string(),
        "m_fEffects".to_string(),
        "m_fLastShotTime".to_string(),
        "m_flCreateTime".to_string(),
        "m_flDroppedAtTime".to_string(),
        "m_flFireSequenceStartTime".to_string(),
        "m_flNextPrimaryAttackTickRatio".to_string(),
        "m_flNextSecondaryAttackTickRatio".to_string(),
        "m_flRecoilIndex".to_string(),
        "m_flSimulationTime".to_string(),
        "m_flTimeSilencerSwitchComplete".to_string(),
        "m_hOuter".to_string(),
        "m_hOwnerEntity".to_string(),
        "m_hPrevOwner".to_string(),
        "m_iAccountID".to_string(),
        "m_iClip1".to_string(),
        "m_iClip2".to_string(),
        "m_iEntityQuality".to_string(),
        "m_iInventoryPosition".to_string(),
        "m_iIronSightMode".to_string(),
        "m_iItemIDHigh".to_string(),
        "m_iItemIDLow".to_string(),
        "m_iState".to_string(),
        "m_nAddDecal".to_string(),
        "m_nCollisionFunctionMask".to_string(),
        "m_nDropTick".to_string(),
        "m_nEnablePhysics".to_string(),
        "m_nEntityId".to_string(),
        "m_nFireSequenceStartTimeChange".to_string(),
        "m_nHierarchyId".to_string(),
        "m_nInteractsAs".to_string(),
        "m_nNextPrimaryAttackTick".to_string(),
        "m_nNextSecondaryAttackTick".to_string(),
        "m_nNextThinkTick".to_string(),
        "m_nOwnerId".to_string(),
        "m_nSubclassID".to_string(),
        "m_nViewModelIndex".to_string(),
        "m_pReserveAmmo".to_string(),
        "m_ubInterpolationFrame".to_string(),
        "m_usSolidFlags".to_string(),
        "m_vDecalForwardAxis".to_string(),
        "m_vDecalPosition".to_string(),
        "m_weaponMode".to_string(),
        "X".to_string(),
        "Y".to_string(),
        "Z".to_string(),
        "velocity".to_string(),
        "velocity_X".to_string(),
        "velocity_Y".to_string(),
        "velocity_Z".to_string(),
        "pitch".to_string(),
        "yaw".to_string(),
        "weapon_name".to_string(),
        "weapon_skin".to_string(),
        "weapon_skin_id".to_string(),
        "active_weapon_original_owner".to_string(),
        "inventory".to_string(),
        "inventory_as_ids".to_string(),
        "entity_id".to_string(),
        "is_alive".to_string(),
        "user_id".to_string(),
        "agent_skin".to_string(),
        "weapon_stickers".to_string(),
        "weapon_float".to_string(),
        "weapon_paint_seed".to_string(),
        "is_airborne".to_string(),
    ];

    // Parse all events
    let wanted_events = vec!["all".to_string()];

    let settings = ParserInputs {
        wanted_player_props: wanted_props,
        wanted_events: wanted_events,
        real_name_to_og_name: AHashMap::default(),
        wanted_other_props: vec![],
        parse_ents: true,
        wanted_players: vec![], // Empty to capture all players
        wanted_ticks: vec![], // Empty to capture all ticks
        parse_projectiles: true,
        parse_grenades: true,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &huf,
        order_by_steamid: false,
        wanted_prop_states: AHashMap::default(),
        fallback_bytes: None,
    };
    
    let mut parser = Parser::new(settings, crate::parse_demo::ParsingMode::ForceSingleThreaded);
    
    let file = File::open(demo_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let output = parser.parse_demo(&mmap)?;

    // Extract the prop_controller from the output
    let prop_controller = output.prop_controller.clone();

    // Extract events grouped by type (similar to e2e_test.rs)
    let mut events_by_type = BTreeMap::new();
    for event in &output.game_events {
        events_by_type
            .entry(event.name.clone())
            .or_insert_with(Vec::new)
            .push(event.clone());
    }

    Ok((output, prop_controller, events_by_type))
}

/// Parse the Vitality vs Spirit demo with all events, props, and players
/// This is the main function that mimics e2e_test.rs for the real demo
pub fn parse_vitality_vs_spirit_demo() -> Result<(DemoOutput, PropController, BTreeMap<String, Vec<GameEvent>>), Box<dyn std::error::Error>> {
    parse_demo_comprehensive("../test_data/vitality-vs-spirit-m1-dust2.dem")
}

/// Custom property mapping similar to e2e_test.rs
pub fn create_custom_property_mapping() -> AHashMap<u32, &'static str> {
    let mut custom = AHashMap::default();

    custom.insert(WEAPON_ORIGINGAL_OWNER_ID, "active_weapon_original_owner");
    custom.insert(INVENTORY_ID, "inventory");
    custom.insert(INVENTORY_AS_IDS_ID, "inventory_as_ids");
    custom.insert(USERID_ID, "user_id");
    custom.insert(VELOCITY_X_ID, "velocity_X");
    custom.insert(VELOCITY_Y_ID, "velocity_Y");
    custom.insert(VELOCITY_Z_ID, "velocity_Z");
    custom.insert(VELOCITY_ID, "velocity");
    custom.insert(IS_ALIVE_ID, "is_alive");
    custom.insert(ENTITY_ID_ID, "entity_id");
    custom.insert(GAME_TIME_ID, "game_time");
    custom.insert(WEAPON_SKIN_NAME, "weapon_skin");
    custom.insert(WEAPON_SKIN_ID, "weapon_skin_id");
    custom.insert(WEAPON_NAME_ID, "weapon_name");
    custom.insert(PITCH_ID, "pitch");
    custom.insert(YAW_ID, "yaw");
    custom.insert(PLAYER_X_ID, "X");
    custom.insert(PLAYER_Y_ID, "Y");
    custom.insert(PLAYER_Z_ID, "Z");
    custom.insert(TICK_ID, "tick");
    custom.insert(STEAMID_ID, "steamid");
    custom.insert(NAME_ID, "name");
    custom.insert(WEAPON_STICKERS_ID, "weapon_stickers");
    custom.insert(IS_AIRBORNE_ID, "is_airborne");

    custom
}