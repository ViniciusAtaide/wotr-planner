//! Ticket 296: every blueprint entry's prerequisite is held as the vocabulary's `Requires` components (ticket 295), the
//! same values in the game's form. The checked data compiles them back into the `prerequisite` every reader reads
//! (`GameData::checked`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use rules::{
    Ability, ClassKey, Component, Condition, DataAt, Fact, FeatKey, GameData, Group, Prerequisite,
    StatRef,
};

use crate::support::*;

/// The nodes of a tree with no condition yet, each of which keeps a blueprint entry's `prerequisite` field: the
/// planner's own (`Story`), or a later stage's gap (the parameterised feats' header, the faith and its domains as facts,
/// the selections' own rules, a stat the planner doesn't count). An `AtLeast` over a feature's rank is one: the
/// vocabulary's `AtLeast` counts facts held.
fn without_condition(tree: &Prerequisite) -> BTreeSet<&'static str> {
    use Prerequisite as P;
    let leaves = tree.leaves().filter_map(|leaf| match leaf {
        P::Atheist => Some("Atheist"),
        P::DeityOffers(_) => Some("DeityOffers"),
        P::FeatForSameValue(_) => Some("FeatForSameValue"),
        P::FeatIfValue { .. } => Some("FeatIfValue"),
        P::NoOptionFrom(_) => Some("NoOptionFrom"),
        P::NoOtherOptionOfGroup(_) => Some("NoOtherOptionOfGroup"),
        P::PickedThrough(_) => Some("PickedThrough"),
        P::Story(_) => Some("Story"),
        P::Worships => Some("Worships"),
        _ => None,
    });
    let ranked = has_ranked_at_least(tree).then_some("AtLeast over a rank");
    leaves.chain(ranked).collect()
}

/// Whether an `AtLeast` anywhere in the tree counts a feature's rank above 1 among its children.
fn has_ranked_at_least(tree: &Prerequisite) -> bool {
    use Prerequisite as P;
    match tree {
        P::AtLeast(_, children)
            if children
                .iter()
                .any(|child| matches!(child, P::Feature { count, .. } if *count > 1)) =>
        {
            true
        }
        P::All(children) | P::Any(children) | P::AtLeast(_, children) => {
            children.iter().any(has_ranked_at_least)
        }
        P::Not(child) => has_ranked_at_least(child),
        _ => false,
    }
}

/// One picked blueprint entry in the files' form: where it is, its `prerequisite` field, and whether it holds a
/// `Requires`.
struct Picked<'a> {
    at: DataAt,
    field: Option<&'a Prerequisite>,
    requires: bool,
}

/// The picked entries with a blueprint (none of the planner's), in the files' form.
fn picked_blueprint_entries(files: &GameData) -> Vec<Picked<'_>> {
    macro_rules! picked {
        ($entries:expr, $at:path) => {
            ($entries)
                .filter(|entry| entry.blueprint.is_some())
                .map(|entry| Picked {
                    at: $at(entry.key.clone()),
                    field: entry.prerequisite.as_ref(),
                    requires: entry.components.iter().any(Component::is_requires),
                })
        };
    }
    picked!(files.feats.iter(), DataAt::Feat)
        .chain(picked!(files.all_classes(), DataAt::Class))
        .chain(picked!(files.archetypes.iter(), DataAt::Archetype))
        .chain(picked!(files.options.iter(), DataAt::SelectionOption))
        .chain(picked!(
            files.mythic_abilities.iter(),
            DataAt::MythicAbility
        ))
        .chain(picked!(files.mythic_feats.iter(), DataAt::MythicFeat))
        .collect()
}

/// The files as they are, before the check compiles their `Requires`: read once for the tests below.
static FILES: LazyLock<GameData> = LazyLock::new(|| {
    seeds::read_game_data_files(&seeds::game_data_dir()).expect("game-data reads")
});

#[test]
fn a_blueprint_entry_holds_its_prerequisite_as_requires_components() {
    let files = &*FILES;
    let entries = picked_blueprint_entries(files);

    // A field only where a node has no condition yet.
    let mut kept: BTreeMap<&str, usize> = BTreeMap::new();
    let mut field_without_reason = Vec::new();
    for entry in &entries {
        let Some(field) = entry.field else { continue };
        let missing = without_condition(field);
        if missing.is_empty() {
            field_without_reason.push(&entry.at);
        }
        for leaf in missing {
            *kept.entry(leaf).or_default() += 1;
        }
    }
    assert_eq!(field_without_reason, Vec::<&DataAt>::new());
    assert_eq!(
        kept,
        BTreeMap::from([
            ("AtLeast over a rank", 1),
            ("Atheist", 1),
            ("DeityOffers", 6),
            ("FeatForSameValue", 11),
            ("FeatIfValue", 1),
            ("NoOptionFrom", 1),
            ("NoOtherOptionOfGroup", 28),
            ("PickedThrough", 8),
            ("Story", 4),
            ("Worships", 5),
        ])
    );
    let moved = entries.iter().filter(|entry| entry.requires).count();
    let fields = entries.iter().filter(|entry| entry.field.is_some()).count();
    // Ticket 303: three Loremaster secrets hold one now, their own `PrerequisiteNoFeature`.
    // Ticket 304: the 19 nested lists of the bloodline feat and the ranger's 6th and 10th-level style feats hold one,
    // their `PrerequisiteFeature` (a bloodline's requisite feature, a style's first selection) written as the bloodline
    // or style held.
    // Ticket 306g: the two slayer talent options that follow the game hold one each (the Vanguard's extra use, and
    // Painful Strike's five archetypes).
    // Ticket 306j: Extra Hex (Shaman) and the Hex Channeler's channel hold one each.
    // Ticket 306k: Extra Discovery (Vivisectionist) holds one.
    // Tickets 306q and 306r: the 13 feats and the mythic ability they add hold one each (Advanced Weapon Training 2 to 4,
    // the five Extra Channels, Extra Rage and the two Extra Performances, Bleeding Critical, Spell Bane, Abundant Smite).
    // Ticket 306aa: Evasion holds one, its own `PrerequisiteNoFeature`.
    // Ticket 308c: Light Armor Proficiency and Shield Proficiency hold one, their own `PrerequisiteNotProficient`
    // (LightArmorProficiency 6d3728d4e9c9898458fe5e9532951132, ShieldsProficiency cb8686e7357a68c42bdd9d4e65334633),
    // and Spell Chain its PrerequisiteFeature SpellPenetration ee7dc126939e4d9438357fbd5980d459; Rapid Shot holds none,
    // RapidShotFeature 9c928dc570bb9e54a9649b3ebfe47a41's one prerequisite being Intelligence 3, which no Build fails.
    // Ticket 308d: five entries that held none hold the game's: Over-Infused Blasts (DoubleKineticistInfusions
    // 7410cc35ae6aa9d4a82f1d23c081a1ac's kineticist 1), the oracle (OracleClass 20ce9bf8af32bee4c8557a045ab499b1's
    // NoFeature ApsuFeature), the Magic Deceiver (MagicDeceiver 5c77110cd0414e7eb4c2e485659c9a46's NoClassLevel
    // Loremaster and Mystic Theurge), and the alignments of DarkElementalistArchetype f12f18ae8842425489d29f302e69134c
    // and RageshaperShifterArchetype 41d5b44e421048d09d32623689173e11.
    // Ticket 308e: 31 options that held none hold the game's (Cognatogen, Feral Mutagen, Nauseating Flesh, Preserve
    // Organs, Potent Magic, Swift Consume, the two otherworldly companions, the ten magus arcana named by level, Mystery
    // Seeker, Survivalist, the ten nocturnal abilities, Clear Mind), and four hold none, as their blueprints
    // (MercyFatigued e5aa306af9b91974a9b2f2cbe702f562, MercyShaken c4732ca5b0af7214a98f9b4dafe87da9, MercySickened
    // 7ee2ef06226a4884f80b7647a2aa2dee, KiSuddenSpeedFeature 96d96f5b6f71004418e7a7e42a81c94d).
    // A count pin: when two branches each move it, a merge resolves it by sum, the base plus each side's change
    // (`.claude/skills/ticket-queue/SKILL.md`, "Hazards met before"), keeping both sides' comments above.
    assert_eq!((moved, fields), (684, 62));

    // The planner-made entries keep what they hold: their fields, and no `Requires`.
    let planner_requires = (files.feats.iter())
        .filter(|feat| feat.blueprint.is_none())
        .flat_map(|feat| &feat.components)
        .chain(
            (files.options.iter())
                .filter(|option| option.blueprint.is_none())
                .flat_map(|option| &option.components),
        )
        .filter(|c| c.is_requires())
        .count();
    assert_eq!(planner_requires, 0);
}

/// `PowerAttackFeature` 9972f33f977fc724c838e59641b2fca5: one `PrerequisiteStatValue`, Strength 13, group All. Its
/// entry holds it as one `Requires` of the stat, which the checked data holds as the prerequisite Strength 13.
#[test]
fn power_attacks_strength_13_is_a_requires_of_the_stat() {
    let power_attack = (FILES.feats.iter())
        .find(|entry| entry.key == FeatKey::new("power-attack"))
        .unwrap();
    assert_eq!(power_attack.prerequisite, None);
    // Its `Requires` (its Power Attack trade is a component of its own since ticket 297).
    let requires: Vec<&Component> = (power_attack.components.iter())
        .filter(|component| component.is_requires())
        .collect();
    assert_eq!(
        requires,
        [&Component::Requires {
            condition: Condition::Stat {
                stat: StatRef::Ability(Ability::Strength),
                value: 13,
                full: false,
            },
            group: Group::All,
        }]
    );
    assert_eq!(
        feat("power-attack").prerequisite,
        Some(Prerequisite::AbilityScore {
            ability: Ability::Strength,
            score: 13,
        })
    );
}

/// `CraneStyleFeat` 0c17102f650d9044290922b0fad9132f: `PrerequisiteFeature` `Dodge` 97e216dbb46ae3c4faef90cf6bbe6fd5 and
/// `ImprovedUnarmedStrike` 7812ad3672a4b9a4fb894ea402095167 in group All; `PrerequisiteClassLevel` `MonkClass`
/// e8f21e5b58e0569468e420ebea456124 1 and `PrerequisiteStatValue` `BaseAttackBonus` 2 in group Any
/// (`BlueprintFeature.MeetsPrerequisites`: every All, one Any). Its `PrerequisiteStatValue` Intelligence 3 is not in the
/// data: a score of 3 always holds (the point-buy's lowest is 7, a race's penalty 2).
#[test]
fn crane_styles_groups_are_the_games() {
    let crane = (FILES.feats.iter())
        .find(|entry| entry.key == FeatKey::new("crane-style"))
        .unwrap();
    let requires = |condition, group| Component::Requires { condition, group };
    let has = |key: &str| Condition::Has(Fact::Feat(FeatKey::new(key)));
    assert_eq!(
        crane.components,
        [
            requires(Condition::ClassLevel(ClassKey::new("monk"), 1), Group::Any),
            requires(
                Condition::Stat {
                    stat: StatRef::BaseAttack,
                    value: 2,
                    full: false,
                },
                Group::Any
            ),
            requires(has("dodge"), Group::All),
            requires(has("improved-unarmed-strike"), Group::All),
        ]
    );
    let mut expected = Prerequisite::All(vec![
        Prerequisite::Feat(FeatKey::new("dodge")),
        Prerequisite::Feat(FeatKey::new("improved-unarmed-strike")),
        Prerequisite::Any(vec![
            Prerequisite::ClassLevel {
                class: ClassKey::new("monk"),
                level: 1,
            },
            Prerequisite::BaseAttackBonus(2),
        ]),
    ]);
    expected.canonicalize();
    assert_eq!(feat("crane-style").prerequisite, Some(expected));
}
