//! The spell catalogue (ticket 121, stage 1): every spell block on the saved guide pages, one file per page and level
//! heading (`spells/<page>/level-<n>.ron`), and every spell list table (`spells/lists.ron`), with the hand-fixes the
//! README lists.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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
    assert_eq!(hold.school, Some(SpellSchool::Enchantment));
    assert_eq!(
        hold.descriptors,
        descriptors(&[SpellDescriptor::MindAffecting, SpellDescriptor::Compulsion])
    );
    assert_eq!(hold.saving_throw, "Will negates");
    assert!(hold.spell_resistance);
    assert_eq!(hold.target, "One enemy creature within medium range");
    assert_eq!(hold.duration.as_deref(), Some("1 round / level"));
    assert_eq!(hold.casting_time, CastingTime::Standard);
    assert_eq!(hold.guide.0, "wizard-spells#hold-monster-enchantment");
    assert_eq!(hold.summary, "");
    assert_eq!(levels("wizard", "hold-monster"), [(5, None)]);
    assert!(
        list("wizard")
            .used_by
            .contains(&SpellCaster::Class(ClassKey::new("sorcerer")))
    );
    // Elsewhere: the bard's and the inquisitor's at 4, the witch's at 5, and not the cleric's, as their tables say.
    assert_eq!(levels("bard", "hold-monster"), [(4, None)]);
    assert_eq!(levels("inquisitor", "hold-monster"), [(4, None)]);
    assert_eq!(levels("witch", "hold-monster"), [(5, None)]);
    assert_eq!(levels("cleric", "hold-monster"), []);
}

/// The other spells the Max DC run recommended are all there, each with a descriptor it was chosen for.
#[test]
fn the_max_dc_runs_spells_are_on_the_wizard_list() {
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
        let page = spell.guide.0.split_once('#').expect("an anchor").0;
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
/// the levels its tables run.
#[test]
fn every_list_table_is_a_spell_list() {
    let lists: Vec<(&str, usize, u8, u8)> = DATA
        .spell_lists
        .iter()
        .map(|list| {
            let levels = list.entries.iter().map(|entry| entry.level);
            (
                list.key.as_str(),
                list.entries.len(),
                levels.clone().min().unwrap(),
                levels.max().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        lists,
        [
            ("aeon", 102, 1, 7),
            ("alchemist", 78, 1, 6),
            ("angel", 84, 1, 10),
            ("angel/merged", 32, 3, 10),
            ("arcanist/magic-deceiver", 60, 1, 6),
            ("azata", 81, 1, 7),
            ("bard", 121, 0, 6),
            ("bloodrager", 61, 1, 4),
            ("cleric", 191, 0, 9),
            ("demon", 63, 1, 5),
            ("druid", 148, 0, 9),
            ("gold-dragon", 60, 1, 10),
            ("hunter", 107, 0, 6),
            ("inquisitor", 118, 0, 6),
            ("lich", 126, 1, 10),
            ("lich/merged", 25, 3, 10),
            ("magus", 99, 0, 6),
            ("paladin", 63, 0, 4),
            ("ranger", 60, 1, 4),
            ("shaman", 154, 0, 9),
            ("trickster", 66, 1, 7),
            ("warpriest", 142, 0, 6),
            ("witch", 173, 0, 9),
            ("wizard", 280, 0, 9),
        ]
    );
}

/// The list a page folder's blocks sit under.
fn folder_list(folder: &str) -> &str {
    match folder {
        "angel" => "angel/merged",
        "lich" => "lich/merged",
        "arcanist" => "arcanist/magic-deceiver",
        other => other,
    }
}

/// A spell's file says where its block is: `spells/<page>/level-<n>.ron` holds the blocks under that page's "Level
/// <n>" heading, so each is on that page's list at that level, and its guide link is to that page.
#[test]
fn each_spell_is_in_the_file_of_its_blocks_page_and_level() {
    let dir = seeds::game_data_dir().join("spells");
    let mut files = 0;
    for folder in std::fs::read_dir(&dir).unwrap() {
        let folder = folder.unwrap().path();
        if !folder.is_dir() {
            continue;
        }
        let name = folder.file_name().unwrap().to_str().unwrap().to_owned();
        let page = match name.as_str() {
            "wizard" | "cleric" | "druid" => format!("{name}-spells"),
            other => other.to_owned(),
        };
        for file in std::fs::read_dir(&folder).unwrap() {
            let file = file.unwrap().path();
            let level: u8 = file
                .file_stem()
                .and_then(|stem| stem.to_str()?.strip_prefix("level-")?.parse().ok())
                .unwrap_or_else(|| panic!("{} isn't level-<n>.ron", file.display()));
            files += 1;
            for spell in read_spells(&file) {
                assert!(
                    spell.guide.0.starts_with(&format!("{page}#")),
                    "{}: {} links to {}",
                    file.display(),
                    spell.key,
                    spell.guide.0
                );
                assert!(
                    levels(folder_list(&name), spell.key.as_str())
                        .iter()
                        .any(|(at, _)| *at == level),
                    "{}: {} isn't on {} at {level}",
                    file.display(),
                    spell.key,
                    folder_list(&name)
                );
            }
        }
    }
    assert_eq!(files, 98);
}

fn read_spells(path: &Path) -> Vec<Spell> {
    let text = std::fs::read_to_string(path).unwrap();
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&text)
        .unwrap()
}

#[test]
fn classes_that_use_another_classs_list_are_its_users() {
    let used_by = |key: &str| list(key).used_by.clone();
    let class = |key: &str| SpellCaster::Class(ClassKey::new(key));
    let archetype = |key: &str| SpellCaster::Archetype(ArchetypeKey::new(key));
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
    assert_eq!(
        levels("bard", "burning-hands"),
        [(1, archetype("bard/flame-dancer"))]
    );
    // Fear is everyone's at 3 and the Dirge Bard's at 4 too.
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
    assert_eq!(
        levels("alchemist", "summon-natures-ally-v"),
        [(4, archetype("alchemist/preservationist"))]
    );
    assert_eq!(
        levels("alchemist", "breath-of-life"),
        [(4, archetype("alchemist/chirurgeon"))]
    );
    // The Gold Dragon's two footnotes sit under its level 7 table only, and number its entries at every level.
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
    let dismiss = spell("dismiss-spell");
    assert_eq!(dismiss.school, None);
    assert_eq!(dismiss.guide.0, "wizard-spells#dismiss-spell-universalist");
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
    assert_eq!(spell("soothing-mud").school, Some(SpellSchool::Conjuration));
    assert_eq!(spell("shades").school, Some(SpellSchool::Illusion));
    assert_eq!(
        spell("reviving-finale").guide.0,
        "bard#reviving-final-finale-conjuration"
    );
    assert_eq!(levels("bard", "reviving-finale"), [(3, None)]);
    assert_eq!(
        spell("embodiment-of-order").guide.0,
        "aeon#embodiment-of-law-abjuration"
    );
    assert_eq!(levels("aeon", "embodiment-of-order"), [(6, None)]);
    // Descriptors: a misspelling and a full stop for a comma.
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
    assert_eq!(
        spell("heavens-wing-strike").guide.0,
        "wizard-spells#heavens-wing-strike-evocation"
    );
    assert_eq!(levels("bard", "heavens-wing-strike"), [(5, None)]);
    assert_eq!(levels("wizard", "heavens-wing-strike"), [(7, None)]);
    // List names that misspell or rename the block's spell.
    assert_eq!(levels("bard", "neutralize-poison"), [(4, None)]);
    assert_eq!(levels("cleric", "flame-strike"), [(5, None)]);
    assert_eq!(levels("cleric", "unbreakable-heart"), [(1, None)]);
    assert_eq!(
        levels("arcanist/magic-deceiver", "dispel-magic"),
        [(3, None)]
    );
    assert_eq!(levels("paladin", "protection-from-alignment"), [(1, None)]);
    assert_eq!(
        levels("ranger", "summon-small-elemental"),
        [(
            1,
            Some(SpellListOnly::Archetype(ArchetypeKey::new(
                "ranger/flamewarden"
            )))
        )]
    );
    assert_eq!(levels("trickster", "summon-large-elemental"), [(4, None)]);
    // Blocks whose page's table omits them are on that list at their heading's level.
    assert_eq!(levels("wizard", "rage"), [(3, None)]);
    assert_eq!(levels("wizard", "winds-of-vengeance"), [(9, None)]);
    assert_eq!(levels("druid", "thirsting-entanglement"), [(4, None)]);
    // A link to the wrong block, read by the name: the bard's Sleep links to Hypnotism.
    assert_eq!(levels("bard", "sleep"), [(1, None)]);
}

/// A block's facts as printed: the Components row, a block with no Duration row or no Spell Descriptors row.
#[test]
fn a_blocks_optional_rows_are_optional_facts() {
    let stoneskin = spell("stoneskin");
    assert_eq!(stoneskin.components.as_deref(), Some("Diamond dust x 5"));
    assert_eq!(stoneskin.casting_time, CastingTime::Standard);
    let fireball = spell("fireball");
    assert_eq!(fireball.duration, None);
    assert_eq!(fireball.saving_throw, "Reflex half");
    assert_eq!(spell("transformation").descriptors, BTreeSet::new());
    assert_eq!(spell("changestaff").casting_time, CastingTime::FullRound);
}
