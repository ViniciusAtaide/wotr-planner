//! Deities (ticket 86): the guide's deity table, and what a deity does to a Build's alignment, domains, blessings,
//! archetypes and favoured weapons.

use std::collections::BTreeSet;

use rules::{
    Alignment, Build, Choice, DeityDomains, DeityKey, Faith, Issue, Pick, Prerequisite, SlotId,
};

use crate::support::*;

/// The guide's table (character-creation#deities), as `t86/deities.py` parsed the saved page: each deity's allowed
/// alignments, favoured weapons (Asmodeus's "Mace" is the heavy mace, question 9) and domains.
const TABLE: [(&str, &str, &[&str], &[&str]); 26] = [
    (
        "abadar",
        "LG LN TN LE",
        &["light-crossbow"],
        &["earth", "law", "nobility", "protection", "travel"],
    ),
    (
        "achaekek",
        "LN LE NE",
        &["sawtooth-saber"],
        &["death", "evil", "law", "murder", "trickery", "war"],
    ),
    (
        "apsu",
        "LG NG LN",
        &["quarterstaff"],
        &["artifice", "good", "law", "scalykind", "travel"],
    ),
    (
        "asmodeus",
        "LN LE NE",
        &["heavy-mace"],
        &["evil", "fire", "law", "magic", "trickery"],
    ),
    (
        "calistria",
        "CG TN CN CE",
        &["rapier"],
        &["chaos", "charm", "knowledge", "luck", "trickery"],
    ),
    (
        "cayden-cailean",
        "NG CG CN",
        &["rapier"],
        &["chaos", "charm", "good", "strength", "travel"],
    ),
    (
        "dahak",
        "CN NE CE",
        &["heavy-flail"],
        &["chaos", "destruction", "evil", "scalykind", "trickery"],
    ),
    (
        "desna",
        "NG CG CN",
        &["starknife"],
        &["chaos", "good", "liberation", "luck", "travel"],
    ),
    (
        "erastil",
        "LG NG LN",
        &["long-bow"],
        &["animal", "community", "good", "law", "plant"],
    ),
    (
        "godclaw",
        "LG LN LE",
        &["flail"],
        &["law", "protection", "strength", "war"],
    ),
    (
        "gorum",
        "CG TN CN CE",
        &["greatsword"],
        &["chaos", "destruction", "glory", "strength", "war"],
    ),
    (
        "gozreh",
        "NG LN TN CN NE",
        &["trident"],
        &["air", "animal", "plant", "water", "weather"],
    ),
    (
        "green-faith",
        "NG LN TN CN NE",
        &["sickle"],
        &["animal", "earth", "fire", "plant", "water"],
    ),
    (
        "gyronna",
        "CN NE CE",
        &["dagger"],
        &["chaos", "destruction", "evil", "madness"],
    ),
    (
        "iomedae",
        "LG NG LN",
        &["longsword"],
        &["glory", "good", "law", "sun", "war"],
    ),
    (
        "irori",
        "LG LN TN LE",
        &["unarmed-strike"],
        &["healing", "knowledge", "law", "rune", "strength"],
    ),
    (
        "lamashtu",
        "CN NE CE",
        &["falchion", "kukri"],
        &["chaos", "evil", "madness", "strength", "trickery"],
    ),
    (
        "nethys",
        "NG LN TN CN NE",
        &["quarterstaff"],
        &["destruction", "knowledge", "magic", "protection", "rune"],
    ),
    (
        "norgorber",
        "TN LE NE CE",
        &["short-sword"],
        &["charm", "death", "evil", "knowledge", "trickery"],
    ),
    (
        "pharasma",
        "NG LN TN CN NE",
        &["dagger"],
        &["death", "healing", "ice", "knowledge", "repose", "water"],
    ),
    (
        "rovagug",
        "CN NE CE",
        &["greataxe"],
        &["chaos", "destruction", "evil", "war", "weather"],
    ),
    (
        "sarenrae",
        "LG NG CG TN",
        &["scimitar"],
        &["fire", "glory", "good", "healing", "sun"],
    ),
    (
        "shelyn",
        "LG NG CG TN",
        &["glaive"],
        &["air", "charm", "good", "luck", "protection"],
    ),
    (
        "torag",
        "LG NG LN",
        &["warhammer"],
        &["artifice", "earth", "good", "law", "protection"],
    ),
    (
        "urgathoa",
        "TN LE NE CE",
        &["scythe"],
        &["death", "evil", "magic", "strength", "undead", "war"],
    ),
    (
        "zon-kuthon",
        "LN LE NE",
        &["flail"],
        &["darkness", "death", "destruction", "evil", "law"],
    ),
];

fn deity(key: &str) -> Option<Faith> {
    Some(Faith::Deity(DeityKey::new(key)))
}

/// A Build of one class (with its archetype if any), `levels` levels, following `faith` at `alignment`.
fn follower(
    class: &str,
    archetype: Option<&str>,
    levels: usize,
    faith: Option<Faith>,
    alignment: Alignment,
) -> Build {
    let mut build = match archetype {
        Some(archetype) => archetype_build(class, archetype, levels),
        None => build(class, levels),
    };
    build.start.alignment = Some(alignment);
    build.start.deity = faith;
    build
}

fn short(alignments: &BTreeSet<Alignment>) -> String {
    alignments
        .iter()
        .map(|a| a.short_name())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The domains (by their last key part) a slot leaves open.
fn open_domains(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    offered_in(build, slot)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key.rsplit('/').next().unwrap().to_owned())
        .collect()
}

fn names(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

fn slot(level: u8, index: u8) -> SlotId {
    SlotId::Selection { level, index }
}

/// The deities (by key) with which the class (and archetype) can be taken at level 1, at any alignment.
fn open_to(class: &str, archetype: Option<&str>) -> BTreeSet<String> {
    let engine = engine();
    DATA.deities
        .iter()
        .filter(|d| {
            d.alignments.iter().any(|alignment| {
                engine.is_possible(&follower(
                    class,
                    archetype,
                    1,
                    deity(d.key.as_str()),
                    *alignment,
                ))
            })
        })
        .map(|d| d.key.to_string())
        .collect()
}

#[test]
fn the_26_deities_have_the_tables_alignments_weapons_and_domains() {
    assert_eq!(DATA.deities.len(), 26);
    for (key, alignments, weapons, domains) in TABLE {
        let d = DATA
            .deities
            .iter()
            .find(|d| d.key.as_str() == key)
            .unwrap_or_else(|| panic!("no deity {key}"));
        assert_eq!(short(&d.alignments), alignments, "{key}");
        let held: Vec<&str> = d.favoured_weapons.iter().map(|w| w.as_str()).collect();
        assert_eq!(held, weapons, "{key}");
        let held: Vec<String> = d
            .domains
            .iter()
            .map(|o| {
                o.as_str()
                    .strip_prefix("cleric/domain/")
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(held, domains, "{key}");
    }
}

#[test]
fn a_lawful_good_cleric_of_erastil_is_offered_his_five_domains() {
    let build = follower("cleric", None, 1, deity("erastil"), Alignment::LawfulGood);
    let five = names(&["animal", "community", "good", "law", "plant"]);
    assert_eq!(open_domains(&build, &slot(1, 0)), five);
    assert_eq!(open_domains(&build, &slot(1, 1)), five);
    assert_eq!(
        reason_in(&build, &slot(1, 0), "cleric/domain/fire"),
        "Not a domain of Erastil"
    );
}

#[test]
fn the_deity_and_the_alignment_must_fit() {
    let engine = engine();
    let neutral = follower("fighter", None, 1, deity("erastil"), Alignment::TrueNeutral);
    assert!(!engine.is_possible(&neutral));
    let misfit = engine.faith_fits(&neutral.start).unwrap_err();
    assert_eq!(
        engine.issue_text(&misfit).unwrap(),
        "Erastil allows Lawful Good, Neutral Good or Lawful Neutral"
    );
    let chaotic = follower(
        "fighter",
        None,
        1,
        deity("erastil"),
        Alignment::ChaoticNeutral,
    );
    assert!(engine.faith_fits(&chaotic.start).is_err());
    // Any character may follow a deity: a fighter of Erastil at an alignment he allows.
    assert!(engine.is_possible(&follower(
        "fighter",
        None,
        1,
        deity("erastil"),
        Alignment::NeutralGood
    )));
    // Atheism allows every alignment.
    for alignment in Alignment::ALL {
        assert!(engine.is_possible(&follower(
            "fighter",
            None,
            1,
            Some(Faith::Atheism),
            alignment
        )));
    }
}

#[test]
fn a_cleric_with_no_deity_is_locked() {
    let engine = engine();
    for faith in [None, Some(Faith::Atheism)] {
        let mut start = Build::default();
        start.start.deity = faith;
        let options = engine.class_options(&start, 1).unwrap();
        let cleric = options
            .iter()
            .find(|o| o.class.as_str() == "cleric" && o.archetype.is_none())
            .unwrap();
        assert_eq!(
            engine
                .issue_text(cleric.status.as_ref().unwrap_err())
                .unwrap(),
            "Needs a deity"
        );
    }
    // The other deity-bound classes and archetypes too.
    for (class, archetype) in [
        ("inquisitor", None),
        ("warpriest", None),
        ("hunter", Some("hunter/divine-hunter")),
        ("slayer", Some("slayer/deliverer")),
    ] {
        assert!(
            !possible(&follower(class, archetype, 1, None, Alignment::TrueNeutral)),
            "{class} {archetype:?}"
        );
        assert!(
            possible(&follower(
                class,
                archetype,
                1,
                deity("pharasma"),
                Alignment::TrueNeutral
            )),
            "{class} {archetype:?}"
        );
    }
    // A hunter or a slayer without those archetypes needs none: an atheist (the alignment needs a faith, ticket 94).
    assert!(possible(&follower(
        "hunter",
        None,
        1,
        Some(Faith::Atheism),
        Alignment::TrueNeutral
    )));
    assert!(possible(&follower(
        "slayer",
        None,
        1,
        Some(Faith::Atheism),
        Alignment::TrueNeutral
    )));
}

#[test]
fn a_separatist_of_erastils_second_domain_is_any_but_his() {
    let build = follower(
        "cleric",
        Some("cleric/separatist"),
        1,
        deity("erastil"),
        Alignment::LawfulGood,
    );
    let every: BTreeSet<String> = DATA
        .selections()
        .find(|s| s.key.as_str() == "cleric/domain")
        .unwrap()
        .options
        .iter()
        .map(|o| o.key.as_str().rsplit('/').next().unwrap().to_owned())
        .collect();
    // Erastil's five off; Chaos and Evil conflict with a lawful good Separatist, as before.
    let expected: BTreeSet<String> = every
        .difference(&names(&[
            "animal",
            "community",
            "good",
            "law",
            "plant",
            "chaos",
            "evil",
        ]))
        .cloned()
        .collect();
    assert_eq!(open_domains(&build, &slot(1, 1)), expected);
    assert_eq!(
        reason_in(&build, &slot(1, 1), "cleric/domain/animal"),
        "A domain of Erastil"
    );
}

#[test]
fn an_alignment_domain_needs_its_alignment() {
    // Calistria offers Chaos; a true neutral follower can't take it, a chaotic one can (question 7).
    let neutral = follower(
        "cleric",
        None,
        1,
        deity("calistria"),
        Alignment::TrueNeutral,
    );
    assert!(!open_domains(&neutral, &slot(1, 0)).contains("chaos"));
    let chaotic = follower(
        "cleric",
        None,
        1,
        deity("calistria"),
        Alignment::ChaoticNeutral,
    );
    assert!(open_domains(&chaotic, &slot(1, 0)).contains("chaos"));
    // The same for an inquisitor and a warpriest's blessing.
    let inquisitor = follower(
        "inquisitor",
        None,
        1,
        deity("calistria"),
        Alignment::TrueNeutral,
    );
    assert!(!open_domains(&inquisitor, &slot(1, 0)).contains("chaos"));
    let warpriest = follower(
        "warpriest",
        None,
        1,
        deity("calistria"),
        Alignment::TrueNeutral,
    );
    assert!(!open_domains(&warpriest, &slot(1, 0)).contains("chaos-domain"));
}

#[test]
fn a_warpriest_of_torag_is_offered_his_blessings() {
    let build = follower("warpriest", None, 1, deity("torag"), Alignment::LawfulGood);
    // Torag's Artifice has no blessing.
    assert_eq!(
        open_domains(&build, &slot(1, 0)),
        names(&["earth", "good-domain", "law-domain", "protection"])
    );
}

#[test]
fn a_divine_hunter_of_desna_is_offered_her_domains() {
    let build = follower(
        "hunter",
        Some("hunter/divine-hunter"),
        1,
        deity("desna"),
        Alignment::ChaoticGood,
    );
    assert_eq!(
        open_domains(&build, &slot(1, 0)),
        names(&["chaos", "good", "liberation", "luck", "travel"])
    );
}

#[test]
fn some_archetypes_need_one_deity_or_atheism() {
    assert_eq!(
        open_to("warpriest", Some("warpriest/feral-champion")),
        names(&["erastil", "gozreh", "green-faith"])
    );
    assert_eq!(
        open_to("warpriest", Some("warpriest/mantis-zealot")),
        names(&["achaekek"])
    );
    assert_eq!(
        open_to("witch", Some("witch/hag-of-gyronna")),
        names(&["gyronna"])
    );
    // The Living Deity's way needs atheism.
    let way = |faith| {
        let build = follower(
            "arcanist",
            Some("arcanist/magic-deceiver"),
            1,
            faith,
            Alignment::TrueNeutral,
        );
        offered_in(
            &build,
            &slot_of(&build, 1, "arcanist/magic-deceiver/ways-of-the-deceiver"),
        )["arcanist/magic-deceiver/ways-of-the-deceiver/living-deity"]
            .clone()
    };
    assert_eq!(way(Some(Faith::Atheism)), Ok(()));
    assert!(way(deity("nethys")).is_err());
    assert!(way(None).is_err());
}

#[test]
fn the_mantis_zealot_answers_as_before_at_every_alignment() {
    // Achaekek allows exactly the user's LN, LE and NE (question 1).
    for alignment in Alignment::ALL {
        let allowed = matches!(
            alignment,
            Alignment::LawfulNeutral | Alignment::LawfulEvil | Alignment::NeutralEvil
        );
        let zealot = follower(
            "warpriest",
            Some("warpriest/mantis-zealot"),
            1,
            deity("achaekek"),
            alignment,
        );
        assert_eq!(possible(&zealot), allowed, "{alignment:?}");
    }
}

#[test]
fn the_hag_of_gyronna_is_chaotic_neutral_or_evil() {
    for alignment in Alignment::ALL {
        let allowed = matches!(
            alignment,
            Alignment::ChaoticNeutral | Alignment::NeutralEvil | Alignment::ChaoticEvil
        );
        let hag = follower(
            "witch",
            Some("witch/hag-of-gyronna"),
            1,
            deity("gyronna"),
            alignment,
        );
        assert_eq!(possible(&hag), allowed, "{alignment:?}");
    }
}

#[test]
fn an_inquisitor_of_achaekek_is_proficient_with_the_sawtooth_saber() {
    let build = follower(
        "inquisitor",
        None,
        1,
        deity("achaekek"),
        Alignment::LawfulEvil,
    );
    assert!(at(&build, 1).has_feat_for(
        &rules::FeatKey::new("exotic-weapon-proficiency"),
        &weapon("sawtooth-saber")
    ));
    assert_eq!(
        engine().favoured_weapons_note(&at(&build, 1)).as_deref(),
        Some("Proficient with Sawtooth Saber (Achaekek)")
    );
}

#[test]
fn a_deliverer_of_irori_holds_improved_unarmed_strike_and_can_take_stunning_fist() {
    let build = follower(
        "slayer",
        Some("slayer/deliverer"),
        1,
        deity("irori"),
        Alignment::LawfulNeutral,
    );
    assert_eq!(
        at(&build, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        1
    );
    assert_eq!(
        feat_status(&build, &general_slot(1), "stunning-fist"),
        Ok(())
    );
    // A cleric of Irori too (question 10); a fighter of Irori, with no such feature, doesn't.
    let cleric = follower("cleric", None, 1, deity("irori"), Alignment::LawfulNeutral);
    assert_eq!(
        at(&cleric, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        1
    );
    let fighter = follower("fighter", None, 1, deity("irori"), Alignment::LawfulNeutral);
    assert_eq!(
        at(&fighter, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        0
    );
    // The Demonbane Priest's proficiencies don't name the deity's weapon (question 11).
    let demonbane = follower(
        "cleric",
        Some("cleric/demonbane-priest"),
        1,
        deity("irori"),
        Alignment::LawfulNeutral,
    );
    assert_eq!(
        at(&demonbane, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        0
    );
}

#[test]
fn the_druids_domain_impossible_domain_and_pentamic_faith_ignore_the_deity() {
    let engine = engine();
    // A druid's and a Blight Druid's domain slots offer the same with a deity and without.
    for archetype in [None, Some("druid/blight-druid")] {
        // Without a deity: an atheist (the alignment needs a faith, ticket 94).
        let without = with_selections_filled(follower(
            "druid",
            archetype,
            3,
            Some(Faith::Atheism),
            Alignment::TrueNeutral,
        ));
        let mut with = without.clone();
        with.start.deity = deity("abadar");
        assert!(engine.is_possible(&with), "{archetype:?}");
        for level in engine.evaluate(&without).levels {
            for slot in level.slots {
                if matches!(slot.kind, rules::SlotKind::Selection(_)) {
                    assert_eq!(
                        offered_in(&without, &slot.id),
                        offered_in(&with, &slot.id),
                        "{archetype:?} {:?}",
                        slot.id
                    );
                }
            }
        }
    }
    // Only these selections are deity-limited: not Impossible Domain, Pentamic Faith, a druid's, the Living
    // Deity's, the Champion's or the Feral Champion's.
    let limited: BTreeSet<&str> = DATA
        .selections()
        .filter(|s| s.deity_domains.is_some())
        .map(|s| s.key.as_str())
        .collect();
    assert_eq!(
        limited,
        BTreeSet::from([
            "cleric/domain",
            "cleric/separatist/domain",
            "hunter/divine-hunter/domain",
            "inquisitor/domain",
            "inquisitor/sacred-huntsmaster/domain",
            "warpriest/blessings",
        ])
    );
}

#[test]
fn the_angelfire_apostle_needs_a_good_deity_and_the_priest_of_balance_a_neutral_one() {
    let apostle = |key: &str, alignment| {
        possible(&follower(
            "cleric",
            Some("cleric/angelfire-apostle"),
            1,
            deity(key),
            alignment,
        ))
    };
    assert!(apostle("sarenrae", Alignment::NeutralGood));
    // Gozreh offers no Good domain.
    assert!(!apostle("gozreh", Alignment::NeutralGood));
    let balance = |key: &str| {
        possible(&follower(
            "cleric",
            Some("cleric/priest-of-balance"),
            1,
            deity(key),
            Alignment::TrueNeutral,
        ))
    };
    assert!(balance("gozreh"));
    assert!(balance("pharasma"));
    // Sarenrae offers Good, Urgathoa Evil.
    assert!(!balance("sarenrae"));
    assert!(!balance("urgathoa"));
}

#[test]
fn the_champion_of_the_faiths_alignment_is_one_his_deity_offers() {
    let champion = |key: &str, alignment| {
        let build = follower(
            "warpriest",
            Some("warpriest/champion-of-the-faith"),
            1,
            deity(key),
            alignment,
        );
        open_domains(&build, &slot(1, 1))
    };
    // Iomedae offers Good and Law; a lawful good Champion may choose either.
    assert_eq!(
        champion("iomedae", Alignment::LawfulGood),
        names(&["good-domain", "law-domain"])
    );
    // Gozreh offers no alignment domain.
    assert!(champion("gozreh", Alignment::TrueNeutral).is_empty());
}

#[test]
fn a_held_domain_locks_the_deities_that_do_not_offer_it() {
    let engine = engine();
    let mut build = follower("cleric", None, 1, deity("erastil"), Alignment::LawfulGood);
    build.picks.insert(
        slot(1, 0),
        Pick::option(rules::SelectionOptionKey::new("cleric/domain/animal")),
    );
    let to = |key: &str| {
        let mut start = build.start.clone();
        start.deity = deity(key);
        rules::Edit::Start(start)
    };
    // Gozreh offers Animal; Iomedae doesn't.
    assert!(engine.allows(&build, &to("gozreh")));
    assert!(!engine.allows(&build, &to("iomedae")));
    assert!(!engine.is_removable(&build, &Choice::Deity));
}

#[test]
fn a_deity_limit_is_reported_by_its_issue() {
    let build = follower("cleric", None, 1, deity("erastil"), Alignment::LawfulGood);
    let status = offered_in(&build, &slot(1, 0))
        .remove("cleric/domain/fire")
        .unwrap();
    assert_eq!(
        status,
        Err(Issue::DeityDomain {
            deity: DeityKey::new("erastil"),
            limit: DeityDomains::Offered
        })
    );
    let _ = Prerequisite::Worships;
}
