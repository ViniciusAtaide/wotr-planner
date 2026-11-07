//! The first ascension (ticket 91): at MR 1 the character takes one path's path-linked ability, its MR 1 row (the
//! guide's `mythic-paths#first-ascension`: "an ability that is linked to one of the Mythic Paths", which doesn't
//! force the path), and at MR 2 that row's follow-ups and Hard to Kill, its MR 2 row. The path chosen at MR 3 no
//! longer gives its own MR 1 and 2 rows, and a link without the choice (every link made before it) takes its MR 3
//! path's.

use rules::{Build, Edit, MythicPathKey, decode_build, encode_build};

use crate::support::*;

/// The Enforcer Build of ticket 88 (its final link): a slayer whose write-up takes the Trickster's MR 1 and 2
/// abilities and goes Azata at MR 3.
const ENFORCER: &str = "AdVNPUhCYRT97veeqT1ToYiWSIogSKmkKOgPm7IgsBoqInHIoR-RighrkOBZ9DO4RAqtRVMUxOstIRlZg0sW4SCBk20NQTRI3ZdpSa-lTS7nfOfc795zt2C89sL_bNP5FntCex4xvd_0VCh6NmVBPTzDZexKzeD_Oqs6Ry4TddWSF2f8Z6d4a1vTmjSMvPcq8MthvI6Xc8qOnZPXqM0y4VI706PWgSOrRxvb3Gi40ZUkKiuWXWZTfXNkwSxWl3rb64LHTvdBKChcPvJ20enwJSddqsP1sKcrvIbrLQaBm766HzLx_Q_diVhbY-fYVFw4D_RFUgHj2657_oWfu4veUq2dAAEGQK_XFqs0ZQQlfBPk2z97VG5OIREQueczgwGSKSXhgBAUFJn-JirTwxPSCpfNoF-QislqyPdUDRTQINGcQmJZ9keCBAVIrGSkQyyiCKEGFf0A";

/// Links the planner made before the first ascension existed (`proof export` at the commit before ticket 91), each
/// with its Build's MR 3 path.
const OLD_LINKS: [(&str, &str); 7] = [
    // `bench fighter20` (the proof harness's Build of that name): aeon.
    (
        "aeon",
        "AdWPPUhCURSAz33P93hcCjMqKAhqzanh0dIQ0tDvUBANvQaHoILCWcFBQZxEwUlwUHER_EEUfxB8iII_CLqL8BAEcRBcHBTxPlFxchAXudzvfOdwOIfzrQ885_abJqVWI7pro-52CetRjHtwapfub3yEu0KGMPyXlmoXrPCSf0wdvtr-Ofa9whta2YbXHx1U3WWj5Cg-nPdwsC_wcTtTj2g_js24oLRovj75s0RXlypFOcLL9t2wo_5JnjhFUMfMIduTeGv9dV1d77w1VBIgQAwCFd7DGB8AhdD8oyVfN0dA5tNwRCLICgwAJZuspGOu1EJhM0Zhsp6GBWZE04KCo-WzV7wJ",
    ),
    // `full arcanist arcanist/brown-fur-transmuter` (the proof harness's Build of that name): angel.
    (
        "angel",
        "AXVOPUgCYRh-v--8RM2torIkaLBaStKWzKFJkn6IXCoEaYmICqlFhTLShBAxKIhoihxqkWsIoaGhIKmhoZ-loSIMvWipraLe87PjPOju3uee93ne9_m-ycB-v63u_dM_Gsh8I-k92Tmc8jCCIpJQ8zDiQDiMiJZZ0CEZmTYwggobwwGWcPUlsuGUNa_UcUWlyASjXowmdqgcIl-DWagw64a7LSw458nzj4tHq7LTHYxZBGv7o-0JebJ6KxVLRNctyB0H9rfcpdaxed1Y32b0woPHnxxs6IpsFMy7Qz1L3S2z8RnvXo1vnIhzGXvTUd9rx0Q8n02Lju2PbNBpzbki7ruVqOlYOF-0rbauhZbPLu5d9DRRm_aNAQHgAfTFpwooIX9FCAX8Sh2vcFCuULZYGlVPiwEYzoMBTwAOVcA9BFoOIIOqBaKTAjQgrbI_EMbLSk8o4TheQpChRDVaae2_V0f09Bc",
    ),
    // `full alchemist alchemist/grenadier` (the proof harness's Build of that name): azata.
    (
        "azata",
        "AY2OP0hCYRTFv-97r6hnJoFGYn8016CspSWNtiyDoiCkqSWQCJGWtMWCloSCBqciCiOaQumPEBiCQwUhNAU1VEKEg0M0WdDxXXiWOPjgnfd7h3PuvSvBE08x6h-53p0Ih4sBLzj7ne-MN77rLWCYCd8MwfSi7sjxAYajBWAGkz_4paI2BAMpRhkUV-1T5CNGXQQqfDKRpwBMmk8HECCmXUIt2oIiRmlF7XIEqNgSPH21zi7z3Gb2OPSW0s9dgeOO3i3bHaBpYOy-y9mmW4NjTNmh29H2xFOksLB3NmlqtqZbTbHAgXvnYt_GLe7-JddXzPX8cBNyjs93QzOGjbw_2ZPLmNeVwqUHTsf55-jw4WDf0Ish4vU9mm_TjDNWx5iiPkYmcc5JhOBy-U-T-mqe4OJvk4tqKQ4fyLFOh6VMkpnAR_wTiZdUqMg0LJPgisCIml4F-yRJVlUqM1ORS6IBg0uHyLJcUf0F",
    ),
    // `full alchemist` (the proof harness's Build of that name): demon.
    (
        "demon",
        "AY2OPUhCURiGzzl6K3-iBm0oCPqZaghpK5d-oCKKjFwUWoMoUGmSBh0KqSChohArKSqMhtIirEVqCBLBppaGWjIxilokNOy999BVwqF7Oc95v_e-3_ddF3UeDmXXbYlcOuuwzDaNQo9NafYNKYhhtzs8aYYP3R3dhOA-hDPynays4xp53lUfUvEuPofHZBMBuYWbmCnPl4fzpXIAJQ9wwXchgBV8GhxsKe7lGqac8Xz58v0CHBBOKBOfb7R6hbuq8mTI0LbcENO2D-pn7kHfp87UNX4Zv4IeeT0zbYfc14nww1K1cXflPLC62JO98dbo994ntk48T_ZYsNVqtL9Np22RHZUPdDAHaAlugP6WDv_RcbS39qPT4bJoLzSBxwV16mUg48k35w6eb32uub5T8xqhhAiEqKVHRxSUUsIok4RSqmSLATikyIXzKwo2LQRkCNIHgm0C0eCGYAADSAmwUqBqJrX946gpowoiQiFIZBLLJZYptUqExJeI_8QooUX1Xy3eFVSA-gE",
    ),
    // `full alchemist alchemist/chirurgeon` (the proof harness's Build of that name): lich.
    (
        "lich",
        "AY2PTyiDYRzHn-fZFtuQZA5IyUFZmUWKduGysqgJxWurXRykpfdALWEHFlsSxUmp4bBardeEi9QOksvKykktF7M5SDntXfm-75NXamnP0-_b5_k-vz_Ps04DsdHi4eJYMHhmf2uTjOC0XOAwsWBe6RgHDN8enc9Pvta2gAHweQJ8FOIILoozSOY56ABTY60WHLgu8VqYGqMPv9KYJ2sNMQW3_DFgzefJvAl8APcxXWsOwNe4nw9LX0Jmc6_TFP-s6XX5jXPQZH9O0akTaG5NlOy2xwPJPeRdashCvfIyVE7VhaOZ_cvjYCp94bbUO6LFu11PfLXJcpoTBj8iIfElENnxP0eqEz5hw_Vk02-NvPdsl6bvEwWRiVbBIbd2Z-NXMWtz-0DfTejB6Uk6u_K-WUooIQZCTOpqJDpKKWEMQZlePfwVWsYzaEX_punUC2WggZgxFYspwZSoTAg1MdRXFCbKqI6UE32VTvm1-pTfgp9tpHr2DQ",
    ),
    // `full alchemist alchemist/reanimator` (the proof harness's Build of that name): trickster.
    (
        "trickster",
        "AY2NO0hCYRTHv3OvRWlWg1RgBBnhkEHZakUFDb3A0qUXJEURIiJNaoOEFnEhIUKKtCGoaAm0By4RtRQ02GBTDUkRJZRIutjr5EfXBxV9l-9____z_c45DjBvdydcxh6bTaMXVXjz0bceudGb_W_oA69ha1XvvViaxWAdIz6hT5j6Keyb1CJAYRxCI_Ub9Q8URs_zOCG9jpH24vD0RTyMhgewkQK4BesYsxi6hdbpQOq5WMjIlLhP7UPKp1KPo6ChcyUqQV2tjk3BrbpleP9Fj9EyJ0d1uspRyx6lWF-Onu9ql3xXXISbLVatP497bCeBDqVBveatBKkpZB4ZhLDR3za2c2ZpCvbVXnN5Bs1EzYCqSyfDSrPuXX5piSjEB9JFaD8Oxq2jzro7heHicL5RFC_asy9syjwzW9M3HwQIySFEmDwSwgJASiAVGYaa3EziN0zwA5ZsRQ9EQES4Fg_zdb-F4YVlMnOagJDBAf-6QmCAZXNTSnhhWUEhuuRHAAhk-b_-nw",
    ),
    // `bench legend40` (the proof harness's Build of that name): aeon.
    (
        "aeon",
        "Ae2RS0hCQRSGz7l5VUbCipQUAttFbdq4aRXSxqzAQIS0hWBQQSItCwoVwpUZFK0sStxET1PKCLz0fhBY6whuEURCQkEulGi0kl5EQkKLZjj_fP8_szicecD-2ZbNsnOn1KLhfLF0_M9Cq8Ox0mP43qYm7O9sX_t_UqCETl44LbCmyx8TUOnltQ1-qsXmDaqLvet8rEZo1m_rIrI2j00sNB6pBy6jZzOBYPLYdzjIj-43VSXu0WpWh9NKa4XE2QVXp9JdMn83FK7zX4x42RNFUmTTRo0SnWMrFtbLUnveyZu5kGH82lQvlwX0U8Fb4Jbd_JLFpHCRHemwprNDrVqN2yMHQXH3mnKMg9qQa8HTzDW6AQFZhFJSQgipBgbxtfAN5-UFlF8Un5nFzwd-FRb-jgG6aF-gQsgiAAvAZEPIPM0hk0P4RWLyESRy2iDzkxLR8RdBTjIqYIsyv_thPwE",
    ),
];

/// The names of the features a rank's path row gives, as the Mythic track lists them.
fn names(build: &Build, rank: u8) -> Vec<String> {
    let engine = engine();
    let sheet = engine.evaluate(build);
    engine
        .rank_features(&sheet.ranks[usize::from(rank) - 1])
        .into_iter()
        .map(|key| engine.feature_name(key).to_owned())
        .collect()
}

fn names_has(names: &[String], name: &str) -> bool {
    names.iter().any(|held| held == name)
}

#[test]
fn the_first_ascension_offers_the_six_paths_path_linked_abilities() {
    let engine = engine();
    let build = build("fighter", 6);
    let options = engine.first_ascension_options(&build);
    let named: Vec<(&str, String)> = options
        .iter()
        .map(|option| {
            (
                option.path.as_str(),
                engine.first_ascension_name(&option.path),
            )
        })
        .collect();
    assert_eq!(
        named,
        [
            ("aeon", "Force Reality".to_owned()),
            ("angel", "Close to the Heavens".to_owned()),
            ("azata", "Instrument of Freedom".to_owned()),
            ("demon", "Close to the Abyss".to_owned()),
            ("lich", "Danse Macabre".to_owned()),
            ("trickster", "Bit of Fun".to_owned()),
        ]
    );
    assert!(options.iter().all(|option| option.status.is_ok()));
}

#[test]
fn every_first_ascension_gives_bypass_epic_damage_reduction_at_mr_1_and_hard_to_kill_at_mr_2() {
    // The guide's two fixed abilities: Bypass Epic Damage Reduction "At 1st mythic rank", Hard to Kill "At 2nd".
    for path in ["aeon", "angel", "azata", "demon", "lich", "trickster"] {
        let mut build = build("fighter", 7);
        build.first_ascension = Some(MythicPathKey::new(path));
        assert!(possible(&build), "{path}");
        assert!(
            names_has(&names(&build, 1), "Bypass Epic Damage Reduction"),
            "{path}"
        );
        assert!(names_has(&names(&build, 2), "Hard to Kill"), "{path}");
        assert!(holds(
            &at(&build, 6),
            &format!("{path}/bypass-epic-damage-reduction")
        ));
        assert!(
            !holds(&at(&build, 6), &format!("{path}/hard-to-kill")),
            "{path}"
        );
        assert!(
            holds(&at(&build, 7), &format!("{path}/hard-to-kill")),
            "{path}"
        );
    }
}

#[test]
fn the_enforcer_takes_bit_of_fun_at_mr_1_and_goes_azata_at_mr_3() {
    let engine = engine();
    let old = engine
        .open_link(ENFORCER)
        .expect("the Enforcer's link opens");
    assert_eq!(old.mythic_path, Some(MythicPathKey::new("azata")));
    assert_eq!(old.first_ascension, None);
    // Its link reads as having taken the Azata's: Instrument of Freedom at MR 1.
    assert!(names_has(&names(&old, 1), "Instrument of Freedom"));

    let trickster = Edit::FirstAscension(Some(MythicPathKey::new("trickster")));
    assert!(engine.allows(&old, &trickster));
    let mut build = old.clone();
    engine.apply(&mut build, &trickster);
    assert!(possible(&build));
    assert_eq!(build.mythic_path, Some(MythicPathKey::new("azata")));

    // The Mythic track: Bit of Fun at MR 1; Creative Approach, Summon Will-o'-Wisp and Hard to Kill at MR 2.
    assert!(names_has(&names(&build, 1), "Bit of Fun"));
    let mr_2 = names(&build, 2);
    for name in ["Creative Approach", "Summon Will-o'-Wisp", "Hard to Kill"] {
        assert!(names_has(&mr_2, name), "{name} at MR 2: {mr_2:?}");
    }
    let sheet = engine.evaluate(&build);
    assert_eq!(sheet.ranks[0].path, Some(MythicPathKey::new("trickster")));
    assert_eq!(sheet.ranks[1].path, Some(MythicPathKey::new("trickster")));
    assert_eq!(sheet.ranks[2].path, Some(MythicPathKey::new("azata")));

    // Held from their ranks; the Azata gives none of its MR 1 and 2 rows, and its own from MR 3 on.
    assert!(holds(&at(&build, 6), "trickster/bit-of-fun"));
    assert!(holds(&at(&build, 7), "trickster/creative-approach"));
    let last = at(&build, u8::try_from(build.levels.len()).unwrap());
    for key in [
        "trickster/bit-of-fun",
        "trickster/creative-approach",
        "trickster/summon-will-o-wisp",
        "trickster/hard-to-kill",
    ] {
        assert!(holds(&last, key), "{key}");
    }
    for key in [
        "azata/instrument-of-freedom",
        "azata/freedoms-child",
        "azata/summon-azata-bralani",
    ] {
        assert!(!holds(&last, key), "{key}");
    }
    assert!(holds(&last, "azata/dragon-companion"));

    // The choice is in the link.
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
}

#[test]
fn every_older_link_decodes_to_the_same_build_and_takes_its_mr_3_paths_first_ascension() {
    let engine = engine();
    let keys = rules::LinkKeys::new(&DATA);
    for (path, link) in OLD_LINKS.into_iter().chain([("azata", ENFORCER)]) {
        let build = decode_build(link, &keys).expect("an old link decodes");
        assert_eq!(build.first_ascension, None, "{path}");
        assert_eq!(build.mythic_path, Some(MythicPathKey::new(path)));
        // The same Build writes the same link.
        assert_eq!(encode_build(&build), link, "{path}");
        assert!(possible(&build), "{path}");
        // MR 1 and 2 give the MR 3 path's rows, from their own ranks.
        let sheet = engine.evaluate(&build);
        assert_eq!(sheet.ranks[0].path, Some(MythicPathKey::new(path)));
        assert_eq!(sheet.ranks[1].path, Some(MythicPathKey::new(path)));
        assert!(
            holds(
                &at(&build, 6),
                &format!("{path}/bypass-epic-damage-reduction")
            ),
            "{path}"
        );
        assert!(
            holds(&at(&build, 7), &format!("{path}/hard-to-kill")),
            "{path}"
        );
        // Taking that path's first ascension changes no answer.
        let mut chosen = build.clone();
        chosen.first_ascension = Some(MythicPathKey::new(path));
        assert_eq!(engine.evaluate(&chosen), sheet, "{path}");
    }
}
