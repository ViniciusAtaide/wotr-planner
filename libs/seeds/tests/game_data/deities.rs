//! Deities (ticket 86): the guide's deity table, and what a deity does to a Build's alignment, domains, blessings,
//! archetypes and favoured weapons.

use std::collections::BTreeSet;

use rules::{
    Alignment, Build, Choice, DeityDomains, DeityKey, Faith, Issue, Pick, Prerequisite, SlotId,
};

use crate::support::*;

/// The guide's table (character-creation#deities), as `t86/deities.py` parsed the saved page: each deity's allowed
/// alignments, favoured weapons (Asmodeus's "Mace" is the heavy mace, question 9) and domains. Each row agrees with its
/// deity's feature (PrerequisiteAlignment, the sacred weapon's category, the `*DomainAllowed` facts) but the cells of
/// `WAITING`.
const TABLE: [(&str, &str, &[&str], &[&str]); 26] = [
    // Game: AbadarFeature 6122dacf418611540a3c91e67197ee4e
    (
        "abadar",
        "LG LN TN LE",
        &["light-crossbow"],
        &["earth", "law", "nobility", "protection", "travel"],
    ),
    // Game: AchaekekFeature a3189d5b7c4d4d91beaa8bfffac3e38e
    (
        "achaekek",
        "LN LE NE",
        &["sawtooth-saber"],
        &["death", "evil", "law", "murder", "trickery", "war"],
    ),
    // Game: ApsuFeature 772e2673945e4583a804ae01f67efea0
    (
        "apsu",
        "LG NG LN",
        &["quarterstaff"],
        &["artifice", "good", "law", "scalykind", "travel"],
    ),
    // Game: AsmodeusFeature a3a5ccc9c670e6f4ca4a686d23b89900 (weapons wait: see WAITING)
    (
        "asmodeus",
        "LN LE NE",
        &["heavy-mace", "light-mace"],
        &["evil", "fire", "law", "magic", "trickery"],
    ),
    // Game: CalistriaFeature c7531715a3f046d4da129619be63f44c
    (
        "calistria",
        "CG TN CN CE",
        &["rapier"],
        &["chaos", "charm", "knowledge", "luck", "trickery"],
    ),
    // Game: CaydenCaileanFeature 300e212868bca984687c92bcb66d381b
    (
        "cayden-cailean",
        "NG CG CN",
        &["rapier"],
        &["chaos", "charm", "good", "strength", "travel"],
    ),
    // Game: DahakFeature 8f7118d68f6e44dea94dddb51f38cbdd
    (
        "dahak",
        "CN NE CE",
        &["heavy-flail"],
        &["chaos", "destruction", "evil", "scalykind", "trickery"],
    ),
    // Game: DesnaFeature 2c0a3b9971327ba4d9d85354d16998c1
    (
        "desna",
        "NG CG CN",
        &["starknife"],
        &["chaos", "good", "liberation", "luck", "travel"],
    ),
    // Game: ErastilFeature afc775188deb7a44aa4cbde03512c671
    (
        "erastil",
        "LG NG LN",
        &["long-bow"],
        &["animal", "community", "good", "law", "plant"],
    ),
    // Game: GodclawFeature 583a26e88031d0a4a94c8180105692a5
    (
        "godclaw",
        "LG LN LE",
        &["flail"],
        &["law", "protection", "strength", "war"],
    ),
    // Game: GorumFeature 8f49a5d8528a82c44b8c117a89f6b68c
    (
        "gorum",
        "CG TN CN CE",
        &["greatsword"],
        &["chaos", "destruction", "glory", "strength", "war"],
    ),
    // Game: GozrehFeature 4af983eec2d821b40a3065eb5e8c3a72
    (
        "gozreh",
        "NG LN TN CN NE",
        &["trident"],
        &["air", "animal", "plant", "water", "weather"],
    ),
    // Game: GreenFaithFeature 99a7a8f13c1300c42878558fa9471e2f
    (
        "green-faith",
        "NG LN TN CN NE",
        &["sickle"],
        &["animal", "earth", "fire", "plant", "water"],
    ),
    // Game: GyronnaFeature 8b535b6842e063d48a571a042c3c6e8f
    (
        "gyronna",
        "CN NE CE",
        &["dagger"],
        &["chaos", "destruction", "evil", "madness"],
    ),
    // Game: IomedaeFeature 88d5da04361b16746bf5b65795e0c38c
    (
        "iomedae",
        "LG NG LN",
        &["longsword"],
        &["glory", "good", "law", "sun", "war"],
    ),
    // Game: IroriFeature 23a77a5985de08349820429ce1b5a234
    (
        "irori",
        "LG LN TN LE",
        &["unarmed-strike"],
        &["healing", "knowledge", "law", "rune", "strength"],
    ),
    // Game: LamashtuFeature f86bc8fbf13221f4f9041608a1fb8585
    (
        "lamashtu",
        "CN NE CE",
        &["falchion", "kukri"],
        &["chaos", "evil", "madness", "strength", "trickery"],
    ),
    // Game: NethysFeature 6262cfce7c31626458325ca0909de997
    (
        "nethys",
        "NG LN TN CN NE",
        &["quarterstaff"],
        &["destruction", "knowledge", "magic", "protection", "rune"],
    ),
    // Game: NorgorberFeature 805b6bdc8c96f4749afc687a003f9628 (domains wait: see WAITING)
    (
        "norgorber",
        "TN LE NE CE",
        &["short-sword"],
        &["charm", "death", "evil", "knowledge", "murder", "trickery"],
    ),
    // Game: PharasmaFeature 458750bc214ab2e44abdeae404ab22e9
    (
        "pharasma",
        "NG LN TN CN NE",
        &["dagger"],
        &["death", "healing", "ice", "knowledge", "repose", "water"],
    ),
    // Game: RovagugFeature 04bc2b62273ab744092d992ed72bff41
    (
        "rovagug",
        "CN NE CE",
        &["greataxe"],
        &["chaos", "destruction", "evil", "war", "weather"],
    ),
    // Game: SarenraeFeature c1c4f7f64842e7e48849e5e67be11a1b
    (
        "sarenrae",
        "LG NG CG TN",
        &["scimitar"],
        &["fire", "glory", "good", "healing", "sun"],
    ),
    // Game: ShelynFeature b382afa31e4287644b77a8b30ed4aa0b
    (
        "shelyn",
        "LG NG CG TN",
        &["glaive"],
        &["air", "charm", "good", "luck", "protection"],
    ),
    // Game: ToragFeature d2d5c5a58885a6b489727467e13c3337
    (
        "torag",
        "LG NG LN",
        &["warhammer"],
        &["artifice", "earth", "good", "law", "protection"],
    ),
    // Game: UrgathoaFeature 812f6c07148088e41a9ac94b56ac2fc8 (domains wait: see WAITING)
    (
        "urgathoa",
        "TN LE NE CE",
        &["scythe"],
        &[
            "death", "evil", "magic", "murder", "strength", "undead", "war",
        ],
    ),
    // Game: ZonKuthonFeature f7eed400baa66a744ad361d4df0e6f1b (domains wait: see WAITING)
    (
        "zon-kuthon",
        "LN LE NE",
        &["flail"],
        &[
            "darkness",
            "death",
            "destruction",
            "evil",
            "law",
            "murder",
            "undead",
        ],
    ),
];

/// A column of `TABLE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Weapons,
    Domains,
}

/// The cells of `TABLE` where the game gives more than the data: they hold the game's value and wait on a ticket.
const WAITING: [(&str, Column); 4] = [
    ("asmodeus", Column::Weapons), // Game: AsmodeusSacredWeaponFeature 2eacfd16f6bc0b445813f28473d3e6ba (heavy and light mace)
    ("norgorber", Column::Domains), // Game: NorgorberFeature 805b6bdc8c96f4749afc687a003f9628 (MurderSubdomainAllowed)
    ("urgathoa", Column::Domains), // Game: UrgathoaFeature 812f6c07148088e41a9ac94b56ac2fc8 (MurderSubdomainAllowed)
    ("zon-kuthon", Column::Domains), // Game: ZonKuthonFeature f7eed400baa66a744ad361d4df0e6f1b (MurderSubdomainAllowed, UndeadDomainAllowed)
];

/// Whether a cell of `TABLE` waits on a ticket.
fn waits(key: &str, column: Column) -> bool {
    WAITING.contains(&(key, column))
}

/// A deity's favoured weapons (by key).
fn weapons_of(key: &str) -> Vec<String> {
    deity_entry(key)
        .favoured_weapons
        .iter()
        .map(|w| w.as_str().to_owned())
        .collect()
}

/// A deity's domains (by their last key part).
fn domains_of(key: &str) -> Vec<String> {
    deity_entry(key)
        .domains
        .iter()
        .map(|o| {
            o.as_str()
                .strip_prefix("cleric/domain/")
                .unwrap()
                .to_owned()
        })
        .collect()
}

fn deity_entry(key: &str) -> &'static rules::Deity {
    DATA.deities
        .iter()
        .find(|d| d.key.as_str() == key)
        .unwrap_or_else(|| panic!("no deity {key}"))
}

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
    // Game: DeitySelection 59e7a76987fe3b547b9cce045f4db3e4 lists 32 deities and Atheism; six need not
    // ForbiddenDeityFeature c4403567ea9e5fe478ca38e3052fadc2, which the player character holds: 26
    assert_eq!(DATA.deities.len(), 26);
    for (key, alignments, weapons, domains) in TABLE {
        assert_eq!(short(&deity_entry(key).alignments), alignments, "{key}");
        if !waits(key, Column::Weapons) {
            assert_eq!(weapons_of(key), weapons, "{key}");
        }
        if !waits(key, Column::Domains) {
            assert_eq!(domains_of(key), domains, "{key}");
        }
    }
}

/// Asmodeus's favoured weapons are the heavy and the light mace; Norgorber, Urgathoa and Zon-Kuthon also offer Murder,
/// Zon-Kuthon Undead too.
// Game: the rows of WAITING and TABLE
#[test]
#[ignore = "waits on ticket 267: the game gives Asmodeus the light mace too, and Murder to Norgorber, Urgathoa and Zon-Kuthon, Undead to Zon-Kuthon"]
fn four_deities_have_the_games_weapons_and_domains() {
    for (key, _, weapons, domains) in TABLE {
        for (column, held, expected) in [
            (Column::Weapons, weapons_of(key), weapons),
            (Column::Domains, domains_of(key), domains),
        ] {
            if waits(key, column) {
                let held: BTreeSet<String> = held.into_iter().collect();
                assert_eq!(held, names(expected), "{key} {column:?}");
            }
        }
    }
}

#[test]
fn a_lawful_good_cleric_of_erastil_is_offered_his_five_domains() {
    // Game: ErastilFeature afc775188deb7a44aa4cbde03512c671 (the five DomainAllowed facts); DomainsSelection
    // 48525e5da45c9c243a343fc6545dbdb9 and SecondDomainsSelection 43281c3d7fe18cc4d91928395837cd1e (each option needs
    // its DomainAllowed)
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
    // Game: ErastilFeature afc775188deb7a44aa4cbde03512c671 (LG, NG, LN); PrerequisiteAlignment.RestrictInternal,
    // WrathDecompiled 6ce4a6b; BasicFeatsProgression 5b72dd2ca2cb73b49903806ee8986325 (every character picks from
    // DeitySelection 59e7a76987fe3b547b9cce045f4db3e4); AtheismFeature 92c0d2da0a836ce418a267093c09ca54 (no alignment
    // rule)
    let engine = engine();
    let neutral = follower("fighter", None, 1, deity("erastil"), Alignment::TrueNeutral);
    assert!(!engine.is_possible(&neutral));
    // The alignment picker of a fighter of Erastil locks the alignments he doesn't allow, with his.
    let good = follower("fighter", None, 1, deity("erastil"), Alignment::NeutralGood);
    let wheel = engine.alignment_options(&good);
    let status = |alignment: Alignment| {
        wheel
            .iter()
            .find(|option| option.value == alignment)
            .unwrap()
            .status
            .clone()
    };
    let misfit = status(Alignment::TrueNeutral).unwrap_err();
    assert_eq!(
        engine.issue_text(&misfit).unwrap(),
        "Erastil allows Lawful Good, Neutral Good or Lawful Neutral"
    );
    assert!(status(Alignment::ChaoticNeutral).is_err());
    assert_eq!(status(Alignment::LawfulGood), Ok(()));
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
    // Game: ClericClass 67819271767a9dd4fbfd4ae700befea0 needs not AtheismFeature 92c0d2da0a836ce418a267093c09ca54 (no
    // faith at all is no game state)
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
    // The other deity-bound classes and archetypes too. The Deliverer's: `an_atheist_deliverer_is_possible`.
    // Game: InquisitorClass f1a70d9e1b0b41e49874e1fa9052a1ce and WarpriestClass 30b5e47d47a0e37438cc5a80c96cfb99 need
    // not AtheismFeature; AtheismFeature 92c0d2da0a836ce418a267093c09ca54 excludes DivineHunterArchetype
    // f996f0a18e5d945459e710ee3a6dd485
    for (class, archetype) in [
        ("inquisitor", None),
        ("warpriest", None),
        ("hunter", Some("hunter/divine-hunter")),
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
    // Game: HunterClass 34ecd1b5e1b90b9498795791b0855239 and SlayerClass c75e0971973957d4dbad24bc7957e4fb have no faith
    // rule
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

/// The Deliverer has no faith rule in the game: an atheist one is possible.
// Game: DelivererArchetype 738c5666f8ef46c45942df9100a0d254 (no prerequisite); AtheismFeature
// 92c0d2da0a836ce418a267093c09ca54 doesn't exclude it
#[test]
#[ignore = "waits on ticket 267: the game gives the Deliverer no faith rule, so an atheist Deliverer is possible"]
fn an_atheist_deliverer_is_possible() {
    assert!(possible(&follower(
        "slayer",
        Some("slayer/deliverer"),
        1,
        Some(Faith::Atheism),
        Alignment::TrueNeutral
    )));
}

#[test]
fn a_separatist_of_erastils_second_domain_is_any_but_his() {
    // Game: SecondDomainsSeparatistSelection 42b781e4375d499383b2602d90661283 (each option needs its AllowedSeparatist
    // and not its DomainAllowed; Chaos needs not LawDomainAllowed and Chaotic, Evil not GoodDomainAllowed and Evil);
    // SeparatistArchetype 67401c74ee094e82a2afd645a847f5cc
    let build = follower(
        "cleric",
        Some("cleric/separatist"),
        1,
        deity("erastil"),
        Alignment::LawfulGood,
    );
    let every: BTreeSet<String> = DATA
        .selection(&rules::FeatureSelectionKey::new("cleric/domain"))
        .unwrap()
        .options
        .iter()
        .map(|o| o.as_str().rsplit('/').next().unwrap().to_owned())
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
    // Game: ChaosDomainProgression 5a5d19c246961484a97e1e5dded98ab2 needs Chaotic; CalistriaFeature
    // c7531715a3f046d4da129619be63f44c; the inquisitor's slot is DomainsSelection 48525e5da45c9c243a343fc6545dbdb9
    // (InquisitorProgression 4e945c2fe5e252f4ea61eee7fb560017). The warpriest's blessing:
    // `a_true_neutral_warpriest_of_calistria_is_offered_the_chaos_blessing`.
    let inquisitor = follower(
        "inquisitor",
        None,
        1,
        deity("calistria"),
        Alignment::TrueNeutral,
    );
    assert!(!open_domains(&inquisitor, &slot(1, 0)).contains("chaos"));
}

/// The alignment blessings need only their domain from the deity in the game, not the alignment.
// Game: ChaosBlessingFeature 528e316f9f092954b9e38d3a82b1634a (needs ChaosDomainAllowed only), in BlessingSelection
// 6d9dcc2a59210a14891aeedb09d406aa; CalistriaFeature c7531715a3f046d4da129619be63f44c
#[test]
#[ignore = "waits on ticket 267: the game offers the Chaos blessing to a True Neutral warpriest of Calistria"]
fn a_true_neutral_warpriest_of_calistria_is_offered_the_chaos_blessing() {
    let warpriest = follower(
        "warpriest",
        None,
        1,
        deity("calistria"),
        Alignment::TrueNeutral,
    );
    assert!(open_domains(&warpriest, &slot(1, 0)).contains("chaos-domain"));
}

#[test]
fn a_warpriest_of_torag_is_offered_his_blessings() {
    let build = follower("warpriest", None, 1, deity("torag"), Alignment::LawfulGood);
    // Game: ToragFeature d2d5c5a58885a6b489727467e13c3337; BlessingSelection 6d9dcc2a59210a14891aeedb09d406aa (each
    // blessing needs its DomainAllowed). Artifice's blessing: `a_warpriest_of_torag_is_offered_five_blessings`.
    assert!(
        names(&["earth", "good-domain", "law-domain", "protection"])
            .is_subset(&open_domains(&build, &slot(1, 0)))
    );
}

/// The game has an Artifice blessing (and eight more the data lacks): a warpriest of Torag is offered five.
// Game: ArtificeBlessingFeature 04e5d412a7b94f809a28607618b71619 in BlessingSelection 6d9dcc2a59210a14891aeedb09d406aa
// (34 blessings); ToragFeature d2d5c5a58885a6b489727467e13c3337
#[test]
#[ignore = "waits on ticket 267: the game gives the warpriest 34 blessings, Artifice among them, so Torag offers five"]
fn a_warpriest_of_torag_is_offered_five_blessings() {
    let build = follower("warpriest", None, 1, deity("torag"), Alignment::LawfulGood);
    assert_eq!(open_domains(&build, &slot(1, 0)).len(), 5);
}

#[test]
fn a_divine_hunter_of_desna_is_offered_her_domains() {
    // Game: DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b; DesnaFeature 2c0a3b9971327ba4d9d85354d16998c1
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
    // Game: FeralChampionArchetype f68ca492c9c15e241ab73735fbd0fb9f, MantisZealotArchetype
    // fc403240fdca4a52a413578169ea7117, DarkSisterArchetype 411fa458481e44f0855d47a19358874b (the archetype's or every
    // other deity's exclusions); MagicDeceiverWayOfLivingGodProgression 464ffaf88140474a949618bec7955e78 (none of the
    // deities: PrerequisiteFeaturesFromList with m_RestrictIfNot, WrathDecompiled 6ce4a6b)
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
    // Game: AchaekekFeature a3189d5b7c4d4d91beaa8bfffac3e38e
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
    // Game: DarkSisterArchetype 411fa458481e44f0855d47a19358874b (CN, NE, CE, ArchetypeAlignment); GyronnaFeature
    // 8b535b6842e063d48a571a042c3c6e8f
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
    // Game: AchaekekFeature a3189d5b7c4d4d91beaa8bfffac3e38e (SawtoothSabreProficiency
    // 5829f3b193704b8096820875f2eb2368 at cleric, inquisitor or warpriest 1); the note's text is the planner's
    assert!(at(&build, 1).has_feat_for(
        &rules::FeatKey::new("exotic-weapon-proficiency"),
        &weapon("sawtooth-saber")
    ));
    assert_eq!(
        engine().favoured_weapons_note(&at(&build, 1)).as_deref(),
        Some("Proficient with Sawtooth Saber (Achaekek)")
    );
}

/// A cleric of Irori holds Improved Unarmed Strike (question 10); a fighter of Irori, with no such feature, doesn't. The
/// Deliverer's and the Demonbane Priest's: `a_deliverer_of_irori_gets_no_improved_unarmed_strike`,
/// `a_demonbane_priest_of_irori_holds_improved_unarmed_strike`.
#[test]
fn a_cleric_of_irori_holds_improved_unarmed_strike_and_a_fighter_doesnt() {
    // Game: IroriFeature 23a77a5985de08349820429ce1b5a234 (AddFeatureOnClassLevel: ImprovedUnarmedStrike
    // 7812ad3672a4b9a4fb894ea402095167 at cleric, inquisitor or warpriest 1)
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
}

/// Irori's grant names the Deliverer but counts only cleric, inquisitor and warpriest levels: a Deliverer of Irori
/// gets no Improved Unarmed Strike, so Stunning Fist stays locked.
// Game: IroriFeature 23a77a5985de08349820429ce1b5a234; AddFeatureOnClassLevel.IsFeatureShouldBeApplied and
// ReplaceCasterLevelOfAbility.CheckArchetype, WrathDecompiled 6ce4a6b; DelivererArchetype
// 738c5666f8ef46c45942df9100a0d254; StunningFist a29a582c3daa4c24bb0e991c596ccb28 needs ImprovedUnarmedStrike
#[test]
#[ignore = "waits on ticket 267: the game gives a Deliverer of Irori no Improved Unarmed Strike, so Stunning Fist stays locked"]
fn a_deliverer_of_irori_gets_no_improved_unarmed_strike() {
    let build = follower(
        "slayer",
        Some("slayer/deliverer"),
        1,
        deity("irori"),
        Alignment::LawfulNeutral,
    );
    assert_eq!(
        at(&build, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        0
    );
    assert!(feat_status(&build, &general_slot(1), "stunning-fist").is_err());
}

/// The deity's grant counts every cleric level, whatever the archetype: a Demonbane Priest of Irori holds Improved
/// Unarmed Strike.
// Game: IroriFeature 23a77a5985de08349820429ce1b5a234 (AddFeatureOnClassLevel, ClericClass
// 67819271767a9dd4fbfd4ae700befea0); DevilbanePriestArchetype 2537d1312a462834aadbd1153a2a0d54 (removes only the
// second domain)
#[test]
#[ignore = "waits on ticket 267: the game gives a Demonbane Priest of Irori Improved Unarmed Strike"]
fn a_demonbane_priest_of_irori_holds_improved_unarmed_strike() {
    let demonbane = follower(
        "cleric",
        Some("cleric/demonbane-priest"),
        1,
        deity("irori"),
        Alignment::LawfulNeutral,
    );
    assert_eq!(
        at(&demonbane, 1).feat_count(&rules::FeatKey::new("improved-unarmed-strike")),
        1
    );
}

#[test]
fn the_druids_domain_impossible_domain_and_pentamic_faith_ignore_the_deity() {
    let engine = engine();
    // A druid's and a Blight Druid's domain slots offer the same with a deity and without.
    // Game: DruidDomainSelection 5edfe84c93823d04f8c40ca2b4e0f039 and BlightDruidBond 096fc02f6cc817a43991c4b437e12b8e
    // (their options have no deity prerequisite)
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
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf and HellknightDisciplinePentamicFaith
    // b9750875e9d7454e85347d739a1bc894 ignore prerequisites; ChampionOfTheFaithAlignmentSelection
    // 4146f0ff2c5e461488b0ace718020670 goes by alignment; the Sacred Huntsmaster uses the inquisitor's DomainsSelection
    // 48525e5da45c9c243a343fc6545dbdb9 (AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0 bars it)
    let limited: BTreeSet<&str> = DATA
        .selections
        .iter()
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
    // Game: AngelfireApostleArchetype 857bc9fadf70f294795a9cba974a48b8; every deity without a Good domain excludes
    // it (GozrehFeature 4af983eec2d821b40a3065eb5e8c3a72 does, SarenraeFeature c1c4f7f64842e7e48849e5e67be11a1b
    // doesn't)
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
    // Game: PriestOfBalanceArchetype a4560e3fb5d247d68fb1a2738fcc0855 (one of Abadar, Calistria, Gorum, Gozreh,
    // Irori, Nethys, Pharasma); the rule itself: `a_priest_of_balance_follows_one_of_seven_deities`
    assert!(balance("gozreh"));
    assert!(balance("pharasma"));
    // Sarenrae offers Good, Urgathoa Evil.
    assert!(!balance("sarenrae"));
    assert!(!balance("urgathoa"));
}

/// The Priest of Balance follows one of seven deities in the game: the Green Faith and Godclaw, who offer neither Good
/// nor Evil, aren't among them.
// Game: PriestOfBalanceArchetype a4560e3fb5d247d68fb1a2738fcc0855 (PrerequisiteFeaturesFromList: AbadarFeature,
// CalistriaFeature, GorumFeature, GozrehFeature, IroriFeature, NethysFeature, PharasmaFeature); GreenFaithFeature
// 99a7a8f13c1300c42878558fa9471e2f; GodclawFeature 583a26e88031d0a4a94c8180105692a5
#[test]
#[ignore = "waits on ticket 267: the game gives the Priest of Balance seven deities, not the Green Faith or Godclaw"]
fn a_priest_of_balance_follows_one_of_seven_deities() {
    for (key, alignment) in [
        ("green-faith", Alignment::TrueNeutral),
        ("godclaw", Alignment::LawfulNeutral),
    ] {
        assert!(
            !possible(&follower(
                "cleric",
                Some("cleric/priest-of-balance"),
                1,
                deity(key),
                alignment,
            )),
            "{key}"
        );
    }
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
    // Game: ChampionOfTheFaithGoodFeature 92b34412524dda24fa544c54189792d7 needs not Evil, its Law sibling not Chaotic,
    // its Chaos and Evil siblings not Lawful and not Good (ChampionOfTheFaithAlignmentSelection
    // 4146f0ff2c5e461488b0ace718020670)
    assert_eq!(
        champion("iomedae", Alignment::LawfulGood),
        names(&["good-domain", "law-domain"])
    );
}

/// The Champion of the Faith's choice goes by alignment alone in the game: a True Neutral one of Gozreh may choose any
/// of the four.
// Game: ChampionOfTheFaithAlignmentSelection 4146f0ff2c5e461488b0ace718020670 (no deity check; True Neutral meets each
// option's alignment mask, ChampionOfTheFaithGoodFeature 92b34412524dda24fa544c54189792d7 and its siblings)
#[test]
#[ignore = "waits on ticket 267: the game lets a True Neutral Champion of the Faith of Gozreh choose any of the four alignments"]
fn a_true_neutral_champion_of_the_faith_chooses_any_alignment() {
    let build = follower(
        "warpriest",
        Some("warpriest/champion-of-the-faith"),
        1,
        deity("gozreh"),
        Alignment::TrueNeutral,
    );
    assert_eq!(
        open_domains(&build, &slot(1, 1)),
        names(&["chaos-domain", "evil-domain", "good-domain", "law-domain"])
    );
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
    // Game: GozrehFeature 4af983eec2d821b40a3065eb5e8c3a72 (AnimalDomainAllowed); IomedaeFeature
    // 88d5da04361b16746bf5b65795e0c38c (none)
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
