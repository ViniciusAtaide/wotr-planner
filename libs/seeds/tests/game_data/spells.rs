//! The spell catalogue (ticket 121, stage 1): every spell block on the saved guide pages, one file per page and level
//! heading (`spells/<page>/level-<n>.ron`), and every spell list table (`spells/lists.ron`), with the hand-fixes the
//! README lists.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, CastingTime, ClassKey, MythicPathKey, Spell, SpellCaster, SpellDescriptor,
    SpellList, SpellListOnly, SpellSchool, Story,
};

use crate::support::*;

fn spell(key: &str) -> &'static Spell {
    DATA.spells
        .iter()
        .find(|spell| spell.key.as_str() == key)
        .unwrap_or_else(|| panic!("no spell {key}"))
}

fn list(key: &str) -> &'static SpellList {
    DATA.spell_lists
        .iter()
        .find(|list| list.key.as_str() == key)
        .unwrap_or_else(|| panic!("no spell list {key}"))
}

/// The levels `spell` is at on the list `key`, with the footnote limiting each.
fn levels(key: &str, spell: &str) -> Vec<(u8, Option<SpellListOnly>)> {
    list(key)
        .entries
        .iter()
        .filter(|entry| entry.spell.as_str() == spell)
        .map(|entry| (entry.level, entry.only.clone()))
        .collect()
}

fn descriptors(names: &[SpellDescriptor]) -> BTreeSet<SpellDescriptor> {
    names.iter().copied().collect()
}

/// The ticket's example: Hold Monster, the Max DC run's recommendation.
#[test]
fn hold_monster_is_a_will_negates_compulsion_on_the_wizard_and_sorcerer_list_at_5() {
    let hold = spell("hold-monster");
    assert_eq!(hold.name, "Hold Monster");
    // Game: HoldMonster 41e8a952da7a5c247b3ec1c2dbb73018: Enchantment, spell resistance, a standard action; its
    // descriptors (Paralysis too) wait in hold_monster_and_hold_person_carry_the_paralysis_descriptor.
    assert_eq!(hold.school, Some(SpellSchool::Enchantment));
    assert!(hold.descriptors.is_superset(&descriptors(&[
        SpellDescriptor::MindAffecting,
        SpellDescriptor::Compulsion
    ])));
    assert_eq!(hold.saving_throw, "Will negates");
    assert!(hold.spell_resistance);
    assert_eq!(hold.target, "One enemy creature within medium range");
    assert_eq!(hold.duration.as_deref(), Some("1 round / level"));
    assert_eq!(hold.casting_time, CastingTime::Standard);
    assert_eq!(
        hold.guide.as_ref().unwrap().0,
        "wizard-spells#hold-monster-enchantment"
    );
    assert_eq!(
        hold.summary,
        "A living creature that fails is paralysed, with a new save each round to end it."
    );
    // Game: WizardSpellList ba0401fdeb4062f40a7aa95b6f07fe89 at 5, which SorcererSpellbook uses.
    assert_eq!(levels("wizard", "hold-monster"), [(5, None)]);
    assert!(
        list("wizard")
            .used_by
            .contains(&SpellCaster::Class(ClassKey::new("sorcerer")))
    );
    // Elsewhere: the bard's and the inquisitor's at 4, the witch's at 5, and not the cleric's, as their tables say.
    // Game: BardSpellList 25a5013493bdcf74bb2424532214d0c8 and InquisitorSpellList 57c894665b7895c499b3dce058c284b3
    // at 4, WitchSpellList e17df9977b879b64e8a8cbb4b3569f19 at 5, not ClericSpellList 8443ce803d2d31347897a3d85cc32f53.
    assert_eq!(levels("bard", "hold-monster"), [(4, None)]);
    assert_eq!(levels("inquisitor", "hold-monster"), [(4, None)]);
    assert_eq!(levels("witch", "hold-monster"), [(5, None)]);
    assert_eq!(levels("cleric", "hold-monster"), []);
}

/// The game's Hold Monster and Hold Person carry the Paralysis descriptor beside Mind-Affecting and Compulsion.
#[test]
#[ignore = "waits on ticket 290: the game gives Hold Monster and Hold Person the Paralysis descriptor"]
fn hold_monster_and_hold_person_carry_the_paralysis_descriptor() {
    // Game: HoldMonster 41e8a952da7a5c247b3ec1c2dbb73018 and HoldPerson c7104f7526c4c524f91474614054547e:
    // MindAffecting, Compulsion and Paralysis (the planner has no Paralysis descriptor yet, so it is read by name).
    for key in ["hold-monster", "hold-person"] {
        let names: BTreeSet<String> = spell(key)
            .descriptors
            .iter()
            .map(|descriptor| format!("{descriptor:?}"))
            .collect();
        assert_eq!(
            names,
            BTreeSet::from(["Compulsion", "MindAffecting", "Paralysis"].map(String::from)),
            "{key}"
        );
    }
}

/// The other spells the Max DC run recommended are all there, each with a descriptor it was chosen for.
#[test]
fn the_max_dc_runs_spells_are_on_the_wizard_list() {
    // Game: on WizardSpellList ba0401fdeb4062f40a7aa95b6f07fe89 at these levels, with these schools and descriptors:
    // HoldPerson c7104f7526c4c524f91474614054547e, HideousLaughter fd4d9fd7f87575d47aafe2a64a6e2d8d, StinkingCloud
    // 68a9e6d7256f1354289a39003a46d826, Boneshaker b7731c2b4fa1c9844a092329177be4c3, EuphoricTranquility
    // cbf3bafa8375340498b86a3313a11e2f, OverwhelmingPresence 41cf93453b027b94886901dbfc680cb9, PowerWordStun
    // f958ef62eea5050418fb92dfa944c631.
    for (key, level, school, descriptor) in [
        (
            "hold-person",
            3,
            SpellSchool::Enchantment,
            Some(SpellDescriptor::Compulsion),
        ),
        (
            "hideous-laughter",
            2,
            SpellSchool::Enchantment,
            Some(SpellDescriptor::Compulsion),
        ),
        ("stinking-cloud", 3, SpellSchool::Conjuration, None),
        ("boneshaker", 2, SpellSchool::Necromancy, None),
        (
            "euphoric-tranquility",
            8,
            SpellSchool::Enchantment,
            Some(SpellDescriptor::Emotion),
        ),
        (
            "overwhelming-presence",
            9,
            SpellSchool::Enchantment,
            Some(SpellDescriptor::Compulsion),
        ),
        (
            "power-word-stun",
            8,
            SpellSchool::Enchantment,
            Some(SpellDescriptor::Compulsion),
        ),
    ] {
        let found = spell(key);
        assert_eq!(found.school, Some(school), "{key}");
        if let Some(descriptor) = descriptor {
            assert!(found.descriptors.contains(&descriptor), "{key}");
        }
        assert_eq!(levels("wizard", key), [(level, None)], "{key}");
    }
}

/// Spell blocks per guide page, as the saved pages have them: 633 headings "Name (School)" with a table of facts,
/// less the bard's second Heaven's Wing Strike block (the wizard page's is kept).
#[test]
fn every_block_on_the_saved_pages_is_a_spell() {
    let mut per_page: BTreeMap<&str, usize> = BTreeMap::new();
    for spell in &DATA.spells {
        let page = spell
            .guide
            .as_ref()
            .unwrap()
            .0
            .split_once('#')
            .expect("an anchor")
            .0;
        *per_page.entry(page).or_default() += 1;
    }
    let expected = BTreeMap::from([
        ("aeon", 21),
        ("alchemist", 2),
        ("angel", 32),
        ("arcanist", 7),
        ("azata", 26),
        ("bard", 10),
        ("cleric-spells", 104),
        ("demon", 20),
        ("druid-spells", 53),
        ("gold-dragon", 11),
        ("inquisitor", 9),
        ("lich", 25),
        ("paladin", 9),
        ("ranger", 4),
        ("trickster", 17),
        ("witch", 2),
        ("wizard-spells", 280),
    ]);
    assert_eq!(per_page, expected);
    assert_eq!(DATA.spells.len(), 632);
}

/// Every list table on the saved pages, its entries (one per table row, and the three blocks their tables omit) and
/// the levels its tables run, as the game's lists have them once the entries `the_spell_lists_hold_the_games_spells`
/// waits on are counted.
#[test]
fn every_list_table_is_a_spell_list() {
    let lists: Vec<(&str, usize, u8, u8)> = DATA
        .spell_lists
        .iter()
        .map(|list| {
            let key = list.key.as_str();
            // The game's list: these entries, less those the game lacks, with those it adds, both still waiting in
            // `the_spell_lists_hold_the_games_spells`.
            let mut entry_levels: Vec<u8> = list.entries.iter().map(|entry| entry.level).collect();
            for (_, _, level) in GAME_DROPS.iter().filter(|(list, spell, level)| {
                *list == key && levels(key, spell).contains(&(*level, None))
            }) {
                let at = entry_levels.iter().position(|l| l == level).unwrap();
                entry_levels.swap_remove(at);
            }
            entry_levels.extend(
                GAME_ADDS
                    .iter()
                    .filter(|(list, guid, level)| *list == key && !on_list_at(key, guid, *level))
                    .map(|(_, _, level)| *level),
            );
            (
                key,
                entry_levels.len(),
                *entry_levels.iter().min().unwrap(),
                *entry_levels.iter().max().unwrap(),
            )
        })
        .collect();
    // Game: each list's SpellsByLevel (the blueprints `GAME_ADDS` names); the footnoted rows of the alchemist's,
    // the bard's and others are archetype features that add a known spell (equivalent model). The Gold Dragon's list
    // holds its spells at level 0 in the game, so its levels were not compared.
    assert_eq!(
        lists,
        [
            ("aeon", 103, 0, 7),
            ("alchemist", 78, 1, 6),
            ("angel", 85, 1, 10),
            ("angel/merged", 32, 3, 10),
            ("arcanist/magic-deceiver", 66, 0, 6),
            ("azata", 85, 1, 9),
            ("bard", 121, 0, 6),
            ("bloodrager", 75, 0, 4),
            ("cleric", 193, 0, 9),
            ("demon", 63, 1, 5),
            ("druid", 150, 0, 9),
            ("gold-dragon", 60, 1, 10),
            ("hunter", 124, 0, 6),
            ("inquisitor", 122, 0, 6),
            ("lich", 131, 1, 10),
            ("lich/merged", 28, 3, 10),
            ("magus", 101, 0, 6),
            ("paladin", 62, 1, 4),
            ("ranger", 63, 1, 4),
            ("shaman", 150, 0, 9),
            ("trickster", 67, 1, 7),
            ("warpriest", 145, 0, 6),
            ("witch", 175, 0, 9),
            ("wizard", 280, 0, 9),
        ]
    );
}

/// Whether the list `key` holds, for no archetype or story branch, the spell whose blueprint is `guid` at `level`.
fn on_list_at(key: &str, guid: &str, level: u8) -> bool {
    list(key).entries.iter().any(|entry| {
        entry.only.is_none()
            && entry.level == level
            && spell(entry.spell.as_str())
                .blueprint
                .as_ref()
                .is_some_and(|blueprint| blueprint.as_str() == guid)
    })
}

/// The game's spells on each list that the planner's lists lack, as (list, the spell's blueprint GUID, level). Game: each
/// list's SpellsByLevel: InquisitorSpellList 57c894665b7895c499b3dce058c284b3, ClericSpellList
/// 8443ce803d2d31347897a3d85cc32f53, WitchSpellList e17df9977b879b64e8a8cbb4b3569f19, DruidSpellList
/// bad8638d40639d04fa2f80a1cac67d6b, BloodragerSpellList 98c05aeff6e3d384f8aec6d584973642, HunterSpelllist
/// d090b791bfe381740b98ed4ff909b1cf, MagusSpellList 4d72e1e7bd6bc4f4caaea7aa43a14639, PaladinSpellList
/// 9f5be2f7ea64fe04eb40878347b147bc, RangerSpellList 29f3c338532390546bc5347826a655c4, ShamanSpelllist
/// c0c40e42f07ff104fa85492da464ac69, WarpriestSpelllist c5a1b8df32914d74c9b44052ba3e686a, MagicDeceiverSpellList
/// cf20c10b131344c1a61b75e9cc4a4c13, AeonSpellList 24b0c796f723a144e9891b6c4794c595 with AeonSpellMythicList
/// ca8c6024bd2519f4b97162a3ad286920, AngelClericSpelllist c074062863fbc1e4bab02f9e6e4eb78d with AngelMythicSpelllist
/// deaffb4218ccf2f419ffd6e41603131a, AzataMythicSpelllist db216faa0ff2b984399e7495755c7111 with
/// AzataMythicSpellsSpelllist 10c634d2b386d8d41b18a889adb8cd49, LichWizardSpelllist 7d5987082120bb943ac96cde7b2257ad
/// with LichMythicSpelllist a06bcb035c214ad4db605491be9da13d (`lich/merged` the latter alone), TricksterSpelllist
/// 40f8cfe98ca4ebd43894267dbd3fc3ae with TricksterSpelllistMythic 7a5ea54564c7d494794f34d0f5a9abb3.
const GAME_ADDS: [(&str, &str, u8); 86] = [
    ("inquisitor", "0557ccee0a86dc44cb3d3f6a3b235329", 0), // Game: Stabilize
    ("inquisitor", "564c2ac83c7844beb1921e69ab159ac6", 0), // Game: Ignition
    ("inquisitor", "64923c9c808143c7ab7adba5a27f0ca4", 3), // Game: KeenEdge
    ("inquisitor", "faabd2cc67efa4646ac58c7bb3e40fcc", 3), // Game: Prayer
    ("cleric", "0557ccee0a86dc44cb3d3f6a3b235329", 0),     // Game: Stabilize
    ("cleric", "c0c2ef5955cb415ca482597f1bfc4eca", 5),     // Game: CurseOfMagicNegation
    ("witch", "0557ccee0a86dc44cb3d3f6a3b235329", 0),      // Game: Stabilize
    ("witch", "40ec382849b60504d88946df46a10f2d", 1),      // Game: HazeOfDreams
    ("druid", "0557ccee0a86dc44cb3d3f6a3b235329", 0),      // Game: Stabilize
    ("druid", "0da2046b4517427bb9b2e304ea6342bf", 2),      // Game: BoneFists
    ("druid", "4fe69e00f43203e4fb4c5d128af18309", 3),      // Game: NaturesExile
    ("bloodrager", "97a23111df7547fd8f6417f9ba9b9775", 0), // Game: DismissAreaEffect
    ("bloodrager", "4f8181e7a7f1d904fbaea64220e83379", 1), // Game: ExpeditiousRetreat
    ("bloodrager", "85067a04a97416949b5d1dbf986d93f3", 1), // Game: StoneFist
    ("bloodrager", "ad10bfec6d7ae8b47870e3a545cc8900", 1), // Game: TouchOfGracelessnessCast
    ("bloodrager", "b6010dda6333bcf4093ce20f0063cd41", 2), // Game: FrigidTouchCast
    ("bloodrager", "a2b05555c704458aaadc34be52a63633", 2), // Game: GhoulTouchCast
    ("bloodrager", "ce7dad2b25acf85429b6c9550787b2d9", 2), // Game: Glitterdust
    ("bloodrager", "5181c2ed0190fc34b8a1162783af5bf4", 2), // Game: StoneCall
    ("bloodrager", "448aa456600749d18c14ad979cc88a67", 3), // Game: ExcruciatingDeformation
    ("bloodrager", "fc58ddcff6ab1394eb6c18e9126bb990", 3), // Game: ForcePunchCast
    ("bloodrager", "b0ffc8eaff404f2e9e0a3ee9f4c35486", 3), // Game: SilverDarts
    ("bloodrager", "cf6c901fb7acc904e85c63b342e9c949", 4), // Game: ConfusionSpell
    ("bloodrager", "2427f2e3ca22ae54ea7337bbab555b16", 4), // Game: ReducePersonMass
    ("bloodrager", "16ce660837fb2544e96c3b7eaad73c63", 4), // Game: VolcanicStorm
    ("hunter", "0557ccee0a86dc44cb3d3f6a3b235329", 0),     // Game: Stabilize
    ("hunter", "8a1992f59e06dd64ab9ba52337bf8cb5", 0),     // Game: DivineZap
    ("hunter", "f6f95242abdfac346befd6f4f6222140", 1),     // Game: RemoveSickness
    ("hunter", "d4b28341acdfa9443a3a779acb58be51", 2),     // Game: AnimalAspectBase
    ("hunter", "0da2046b4517427bb9b2e304ea6342bf", 2),     // Game: BoneFists
    ("hunter", "406c6e4a631b43ce8f7a77844b75bf75", 2),     // Game: WinterGrasp
    ("hunter", "c9c56af3b25be3942aa0ffd12f11cf35", 3),     // Game: AnimalAspectGreaterBase
    ("hunter", "8a76293f5ab8485da95ef6293a11358c", 3),     // Game: BurningEntangle
    ("hunter", "2a9ef0e0b5822a24d88b16673a267456", 3),     // Game: CallLightning
    ("hunter", "42c78009dd5cb8e429b27c13d92152b7", 3),     // Game: InstantEnemy
    ("hunter", "265582bc494c4b12b5860b508a2f89a2", 3),     // Game: LifeBubble
    ("hunter", "4fe69e00f43203e4fb4c5d128af18309", 3),     // Game: NaturesExile
    ("hunter", "0e1272506f9f4480b7c3e7e1e53b6439", 3),     // Game: VengefulComets
    ("hunter", "98544596a01d4f7bbf7cc3ff98a1fb69", 4),     // Game: ExplosionOfRot
    ("hunter", "d1afa8bc28c99104da7d784115552de5", 4),     // Game: SpikeStones
    ("hunter", "b844a29343974aeab1d37d38db2f530e", 4),     // Game: ThirstingEntangle
    ("hunter", "50bdf209dede4ce1acc722d5fdfcf53c", 5),     // Game: TidalSurge
    ("hunter", "07d577a74441a3a44890e3006efcf604", 6),     // Game: PrimalRegression
    ("magus", "97a23111df7547fd8f6417f9ba9b9775", 0),      // Game: DismissAreaEffect
    ("magus", "2daf19a1e59545fba89b3d2a822c20b4", 3),      // Game: SongOfDiscordGreater
    ("paladin", "c28f7234f5fb8c943a77621ad96ad8f9", 1),    // Game: ProtectionFromChaosEvil
    ("paladin", "3026de673d4d8fe45baf40e0b5edd718", 2),    // Game: ProtectionFromChaosEvilCommunal
    ("ranger", "5590652e1c2225c4ca30c4a699ab3649", 2),     // Game: CureLightWoundsCast
    ("ranger", "8a76293f5ab8485da95ef6293a11358c", 3),     // Game: BurningEntangle
    ("ranger", "265582bc494c4b12b5860b508a2f89a2", 3),     // Game: LifeBubble
    ("ranger", "b844a29343974aeab1d37d38db2f530e", 4),     // Game: ThirstingEntangle
    ("shaman", "0557ccee0a86dc44cb3d3f6a3b235329", 0),     // Game: Stabilize
    ("warpriest", "0557ccee0a86dc44cb3d3f6a3b235329", 0),  // Game: Stabilize
    ("warpriest", "97a23111df7547fd8f6417f9ba9b9775", 0),  // Game: DismissAreaEffect
    ("warpriest", "8a1992f59e06dd64ab9ba52337bf8cb5", 0),  // Game: DivineZap
    (
        "arcanist/magic-deceiver",
        "55f14bc84d7c85446b07a1b5dd6b2b4c",
        0,
    ), // Game: Daze
    (
        "arcanist/magic-deceiver",
        "97a23111df7547fd8f6417f9ba9b9775",
        0,
    ), // Game: DismissAreaEffect
    (
        "arcanist/magic-deceiver",
        "f0f8e5b9808f44e4eadd22b138131d52",
        0,
    ), // Game: Flare
    (
        "arcanist/magic-deceiver",
        "95f206566c5261c42aa5b3e7e0d1e36c",
        0,
    ), // Game: MageLight
    (
        "arcanist/magic-deceiver",
        "877297946a6b70744a188eb15dd2daab",
        0,
    ), // Game: Lullaby
    (
        "arcanist/magic-deceiver",
        "7bc8e27cba24f0e43ae64ed201ad5785",
        0,
    ), // Game: Resistance
    (
        "arcanist/magic-deceiver",
        "143775c49ae6b7446b805d3b2e702298",
        3,
    ), // Game: DispelMagicTarget
    (
        "arcanist/magic-deceiver",
        "6d490c80598f1d34bb277735b52d52c1",
        6,
    ), // Game: DispelMagicGreaterTarget
    ("aeon", "97a23111df7547fd8f6417f9ba9b9775", 0),       // Game: DismissAreaEffect
    ("angel", "33e53b74891b4c34ba6ee3baa322beeb", 4),      // Game: AngelPureForm
    ("azata", "2daf19a1e59545fba89b3d2a822c20b4", 6),      // Game: SongOfDiscordGreater
    ("azata", "f958ef62eea5050418fb92dfa944c631", 8),      // Game: PowerWordStun
    ("azata", "0883ea8000404b28a6d5f2b8f8bbed15", 9),      // Game: EvilSuccubusDominate
    ("azata", "3c17035ec4717674cae2e841a190e757", 9),      // Game: DominateMonster
    ("azata", "43740dab07286fe4aa00a6ee104ce7c1", 9),      // Game: HeroicInvocation
    ("azata", "7f4b66a2b1fdab142904a263c7866d46", 9),      // Game: HoldMonsterMass
    ("azata", "2f8a67c483dfa0f439b293e094ca9e3c", 9),      // Game: PowerWordKill
    ("lich", "64923c9c808143c7ab7adba5a27f0ca4", 3),       // Game: KeenEdge
    ("lich", "0e1272506f9f4480b7c3e7e1e53b6439", 3),       // Game: VengefulComets
    ("lich", "ca8bc7d438e6b004a87222f3c32572f2", 7),       // Game: BoneSpearConjure
    ("lich", "8ba9b6e4df4c46a597154e2b8e7e6e4a", 8),       // Game: CreateUndeadGreaterBase
    ("lich", "16c3fe873538010459a8a3fd61304825", 10),      // Game: DeathRealm
    ("lich/merged", "03944622fbe04824684ec29ff2cec6a7", 6), // Game: InflictModerateWoundsMass
    ("lich/merged", "ca8bc7d438e6b004a87222f3c32572f2", 7), // Game: BoneSpearConjure
    ("lich/merged", "16c3fe873538010459a8a3fd61304825", 10), // Game: DeathRealm
    ("trickster", "df75c9faf5829da4aabb47995d5a6f8a", 2), // Game: TricksterSummonBeerElementalSmall
    ("trickster", "8d1c8d7d4fa82884ebf99fc280d49978", 3), // Game: TricksterSummonBeerElementalMedium
    ("trickster", "112d1c1b34bd66b43b68a131959b4e85", 4), // Game: TricksterSummonBeerElementalLarge
    ("trickster", "caac251ca7601324bbe000372a0a1005", 4), // Game: ShadowConjuration
    ("trickster", "31456292261253d4f996ab2e82b328a4", 5), // Game: TricksterSummonBeerElementalGreater
    ("trickster", "844c042deeae2c5408f120f1b3b45c7c", 6), // Game: TricksterSummonBeerElementalElder
];
/// The planner's list entries, for no archetype or story branch, that those lists don't hold at that level, as
/// (list, spell, level).
const GAME_DROPS: [(&str, &str, u8); 21] = [
    ("druid", "burst-of-nettles", 3),
    ("hunter", "call-lightning", 4),
    ("paladin", "dismiss-spell", 0),
    ("paladin", "protection-from-alignment", 1),
    ("paladin", "protection-from-alignment-communal", 2),
    ("ranger", "cure-light-wounds", 1),
    ("shaman", "cape-of-wasps", 4),
    ("shaman", "confusion", 4),
    ("shaman", "crushing-despair", 4),
    ("shaman", "protection-from-energy-communal", 4),
    ("shaman", "stoneskin", 4),
    ("arcanist/magic-deceiver", "dispel-magic", 3),
    ("arcanist/magic-deceiver", "dispel-magic-greater", 6),
    ("azata", "believe-in-yourself", 3),
    ("azata", "elusive-nature", 4),
    ("azata", "moral-support", 5),
    ("trickster", "summon-small-elemental", 2),
    ("trickster", "summon-medium-elemental", 3),
    ("trickster", "summon-large-elemental", 4),
    ("trickster", "summon-greater-elemental", 5),
    ("trickster", "summon-elder-elemental", 6),
];

/// The game's spell lists: each spell the game has on a list is there at its level, and no other.
#[test]
#[ignore = "waits on ticket 290: the game gives each list the spells of its SpellsByLevel"]
fn the_spell_lists_hold_the_games_spells() {
    let missing: Vec<_> = GAME_ADDS
        .iter()
        .filter(|(key, guid, level)| !on_list_at(key, guid, *level))
        .collect();
    let extra: Vec<_> = GAME_DROPS
        .iter()
        .filter(|(key, spell, level)| levels(key, spell).contains(&(*level, None)))
        .collect();
    assert_eq!((missing, extra), (vec![], vec![]));
}

#[test]
fn classes_that_use_another_classs_list_are_its_users() {
    let used_by = |key: &str| list(key).used_by.clone();
    let class = |key: &str| SpellCaster::Class(ClassKey::new(key));
    let archetype = |key: &str| SpellCaster::Archetype(ArchetypeKey::new(key));
    // Game: the spellbooks' m_SpellList: SorcererSpellbook, ArcanistSpellbook and EldritchScoundrelSpellbook use
    // WizardSpellList; OracleSpellbook ClericSpellList; SkaldSpellbook BardSpellList; NatureMageSpellbook
    // DruidSpellList; UnletteredArcanistSpellbook WitchSpellList (equivalent model: the game's other users are
    // archetypes of these classes, which keep their class's list).
    assert_eq!(
        used_by("wizard"),
        BTreeSet::from([
            class("arcanist"),
            class("sorcerer"),
            archetype("rogue/eldritch-scoundrel")
        ])
    );
    assert_eq!(used_by("cleric"), BTreeSet::from([class("oracle")]));
    assert_eq!(used_by("bard"), BTreeSet::from([class("skald")]));
    assert_eq!(
        used_by("druid"),
        BTreeSet::from([archetype("arcanist/nature-mage")])
    );
    assert_eq!(
        used_by("witch"),
        BTreeSet::from([archetype("arcanist/unlettered-arcanist")])
    );
    assert_eq!(
        list("arcanist/magic-deceiver").owner,
        archetype("arcanist/magic-deceiver")
    );
    assert_eq!(
        list("angel/merged").owner,
        SpellCaster::Path(MythicPathKey::new("angel"))
    );
}

/// The tables' footnotes are list facts: an entry for one archetype, or one Gold Dragon story branch.
#[test]
fn footnoted_entries_are_only_for_their_archetype_or_story_branch() {
    let archetype = |key: &str| Some(SpellListOnly::Archetype(ArchetypeKey::new(key)));
    // Game: FlameDancerFanTheFlames a317c030f0b3f304f84f2a52e7258e6a adds Burning Hands as a 1st-level known spell
    // (equivalent model: an archetype feature, not a list entry).
    assert_eq!(
        levels("bard", "burning-hands"),
        [(1, archetype("bard/flame-dancer"))]
    );
    // Fear is everyone's at 3 and the Dirge Bard's at 4 too.
    // Game: BardSpellList 25a5013493bdcf74bb2424532214d0c8 at 3; DirgeBardSecretsOfTheGraveFear
    // 8557fd70d47149c47b8162117604a776 at 4 (equivalent model).
    assert_eq!(
        levels("bard", "fear"),
        [(3, None), (4, archetype("bard/dirge-bard"))]
    );
    assert_eq!(
        levels("witch", "acid-splash"),
        [(0, archetype("witch/keen-eyed-adventurer"))]
    );
    assert_eq!(
        levels("druid", "aspect-of-the-stag"),
        [(4, archetype("druid/drovier"))]
    );
    // Game: PreservationistArchetype d218568445cf0524e889a2dfce97d76f's BottledAllyIV f31df3b8f4cad80438d5e57a968729ff
    // adds it at 4 (equivalent model).
    assert_eq!(
        levels("alchemist", "summon-natures-ally-v"),
        [(4, archetype("alchemist/preservationist"))]
    );
    assert_eq!(
        levels("alchemist", "breath-of-life"),
        [(4, archetype("alchemist/chirurgeon"))]
    );
    // The Gold Dragon's two footnotes sit under its level 7 table only, and number its entries at every level.
    // Game: Bless is on GoldDragonSpellList 6b8d215a54cd49d88a6de62352e9cac2 only, Bane on CorruptedGoldDragonSpellList
    // only (their levels not traced: the lists hold their spells at level 0).
    assert_eq!(
        levels("gold-dragon", "bless"),
        [(1, Some(SpellListOnly::Story(Story::Uncorrupted)))]
    );
    assert_eq!(
        levels("gold-dragon", "bane"),
        [(1, Some(SpellListOnly::Story(Story::VowOfDahak)))]
    );
}

#[test]
fn the_universalist_spell_has_no_school() {
    // Game: DismissAreaEffect 97a23111df7547fd8f6417f9ba9b9775, school Universalist (equivalent model: no school).
    let dismiss = spell("dismiss-spell");
    assert_eq!(dismiss.school, None);
    assert_eq!(
        dismiss.guide.as_ref().unwrap().0,
        "wizard-spells#dismiss-spell-universalist"
    );
    let universalist: Vec<&str> = DATA
        .spells
        .iter()
        .filter(|spell| spell.school.is_none())
        .map(|spell| spell.key.as_str())
        .collect();
    assert_eq!(universalist, ["dismiss-spell"]);
}

/// The hand-fixes the README lists, each still in the data.
#[test]
fn the_hand_fixes_hold() {
    // The witch's level 6 table is headed by an h5, not an h4: its 19 spells are level 6.
    // Game: WitchSpellList e17df9977b879b64e8a8cbb4b3569f19 has 19 spells at 6 and levels 0 to 9.
    let witch_levels: BTreeMap<u8, usize> =
        list("witch")
            .entries
            .iter()
            .fold(BTreeMap::new(), |mut counts, entry| {
                *counts.entry(entry.level).or_default() += 1;
                counts
            });
    assert_eq!(witch_levels.get(&6), Some(&19));
    assert_eq!(witch_levels.len(), 10);
    // Block headings: a missing parenthesis, a lowercase school, a doubled word, and a name the list says otherwise.
    // Game: SoothingMud 1a36c8b9ed655c249a9f9e8d4731f001 Conjuration; Shades 70e12191790f69a439ea0132c75f83aa Illusion.
    assert_eq!(spell("soothing-mud").school, Some(SpellSchool::Conjuration));
    assert_eq!(spell("shades").school, Some(SpellSchool::Illusion));
    assert_eq!(
        spell("reviving-finale").guide.as_ref().unwrap().0,
        "bard#reviving-final-finale-conjuration"
    );
    // Game: on BardSpellList 25a5013493bdcf74bb2424532214d0c8 at 3.
    assert_eq!(levels("bard", "reviving-finale"), [(3, None)]);
    assert_eq!(
        spell("embodiment-of-order").guide.as_ref().unwrap().0,
        "aeon#embodiment-of-law-abjuration"
    );
    // Game: on AeonSpellMythicList ca8c6024bd2519f4b97162a3ad286920 at 6.
    assert_eq!(levels("aeon", "embodiment-of-order"), [(6, None)]);
    // Descriptors: a misspelling and a full stop for a comma.
    // Game: AnimateDeadLesser 57fcf8016cf04da4a8b33d2add14de7e carries Summoning (and Evil); DeathClutch
    // c3d2294a6740bc147870fff652f3ced5 Evil and Death.
    assert!(
        spell("animate-dead-lesser")
            .descriptors
            .contains(&SpellDescriptor::Summoning)
    );
    assert_eq!(
        spell("death-clutch").descriptors,
        descriptors(&[SpellDescriptor::Evil, SpellDescriptor::Death])
    );
    // One spell for the two Heaven's Wing Strike blocks.
    // Game: ArchonsTrumpet c6c516e59ca34f34aefb82a850287b97, on BardSpellList 25a5013493bdcf74bb2424532214d0c8 at 5
    // and WizardSpellList ba0401fdeb4062f40a7aa95b6f07fe89 at 7.
    assert_eq!(
        spell("heavens-wing-strike").guide.as_ref().unwrap().0,
        "wizard-spells#heavens-wing-strike-evocation"
    );
    assert_eq!(levels("bard", "heavens-wing-strike"), [(5, None)]);
    assert_eq!(levels("wizard", "heavens-wing-strike"), [(7, None)]);
    // List names that misspell or rename the block's spell. The deceiver's Dispel Magic, the paladin's Protection from
    // Alignment and the trickster's Summon Large Elemental are other spells in the game (Dispel Magic (Target),
    // Protection from Chaos/Evil, the trickster's own elemental): they wait in the_spell_lists_hold_the_games_spells.
    // Game: on BardSpellList 25a5013493bdcf74bb2424532214d0c8 at 4 and ClericSpellList
    // 8443ce803d2d31347897a3d85cc32f53 at 5 and 1.
    assert_eq!(levels("bard", "neutralize-poison"), [(4, None)]);
    assert_eq!(levels("cleric", "flame-strike"), [(5, None)]);
    assert_eq!(levels("cleric", "unbreakable-heart"), [(1, None)]);
    assert_eq!(
        levels("ranger", "summon-small-elemental"),
        [(
            1,
            Some(SpellListOnly::Archetype(ArchetypeKey::new(
                "ranger/flamewarden"
            )))
        )]
    );
    // Blocks whose page's table omits them are on that list at their heading's level.
    // Game: on WizardSpellList ba0401fdeb4062f40a7aa95b6f07fe89 at 3 and 9, DruidSpellList
    // bad8638d40639d04fa2f80a1cac67d6b at 4.
    assert_eq!(levels("wizard", "rage"), [(3, None)]);
    assert_eq!(levels("wizard", "winds-of-vengeance"), [(9, None)]);
    assert_eq!(levels("druid", "thirsting-entanglement"), [(4, None)]);
    // A link to the wrong block, read by the name: the bard's Sleep links to Hypnotism.
    // Game: on BardSpellList 25a5013493bdcf74bb2424532214d0c8 at 1.
    assert_eq!(levels("bard", "sleep"), [(1, None)]);
}

/// A block's facts as printed: the Components row, a block with no Duration row or no Spell Descriptors row.
#[test]
fn a_blocks_optional_rows_are_optional_facts() {
    // Game: Stoneskin c66e86905f7606c4eaa5c774f0357b2b, a material component (5), a standard action; Fireball
    // 2d81362af43aeac4387a3d4fced489c3 (its texts not compared); Transformation 27203d62eb3d4184c9aced94f22e1806 no
    // descriptor; Changestaff 26be70c4664d07446bdfe83504c1d757 a full round.
    let stoneskin = spell("stoneskin");
    assert_eq!(stoneskin.components.as_deref(), Some("Diamond dust x 5"));
    assert_eq!(stoneskin.casting_time, CastingTime::Standard);
    let fireball = spell("fireball");
    assert_eq!(fireball.duration, None);
    assert_eq!(fireball.saving_throw, "Reflex half");
    assert_eq!(spell("transformation").descriptors, BTreeSet::new());
    assert_eq!(spell("changestaff").casting_time, CastingTime::FullRound);
}

/// Stage 2: every spell has a one-sentence summary of at most 160 characters, written from its block and reviewed.
#[test]
fn every_spell_has_a_summary_of_at_most_160_characters() {
    let missing: Vec<&str> = DATA
        .spells
        .iter()
        .filter(|spell| spell.summary.is_empty())
        .map(|spell| spell.key.as_str())
        .collect();
    assert!(missing.is_empty(), "spells without a summary: {missing:?}");
    let long: Vec<(&str, usize)> = DATA
        .spells
        .iter()
        .map(|spell| (spell.key.as_str(), spell.summary.chars().count()))
        .filter(|&(_, length)| length > 160)
        .collect();
    assert!(long.is_empty(), "summaries over 160 characters: {long:?}");
}
