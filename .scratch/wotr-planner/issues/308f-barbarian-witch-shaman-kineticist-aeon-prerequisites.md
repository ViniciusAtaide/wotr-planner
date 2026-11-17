# Importer S4, 308f: the barbarian's, witch's, shaman's, kineticist's, aeon's and prestige options' prerequisites

Type: build (the importer, stage S4)
Status: resolved
Blocked by: 308e
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

Built on base `c29e8aa8`. The note is `libs/game-data/notes/308f-barbarian-witch-shaman-kineticist-prerequisites.md`;
ticket 308 has its "308 left" section.

**What was built**

- **The run:** `game-check --write prerequisites --only 'selection option barbarian/*,selection option
  witch/*,selection option shaman/*,selection option kineticist/*,selection option aeon/*,selection option
  hellknight/*,selection option loremaster/*,selection option angel/*,selection option azata/*,selection option
  demon/*,selection option lich/*,selection option trickster/*,selection option gold-dragon/*' --log` wrote 75 options
  (308h's dry run said 75). 69 stay written: barbarian 24 (all rage powers but the four Lessers), shaman 18 (Iceplant,
  Intimidating Display, the 16 plague mutations), kineticist 16 (9 infusions, 7 wild talents), aeon 9, witch 1
  (Iceplant), Hellknight 1 (Pentamic Faith). The patterns are the note's.
- **Put back (mine): the Elemental Witch's three rays and the air, earth and fire elements.** Each refused characters
  the game allows, and no guard saw it:
  - The rays need WitchElementsPatronProgression 67e85f52b1f020847aaa738d8999d4cd. The game's Elemental Witch picks it in
    ElementalWitchPatronSelection 3172b6960c774e19ad029c5e4a96d3e4; the planner holds that selection as a feature and
    gives the archetype no patron slot, so `importer_lists::the_elemental_witch_picks_one_of_four_rays` failed (only
    cold open). An added requirement isn't a narrowing to the writer, so a run naming them writes them again: the
    second dry run of this `--only` writes these 3 and nothing else; with `selection option witch/hex/*` in place of
    `witch/*` it writes nothing. For the main session: the narrowing guard could count an added `Has` of a planner
    entry no row or slot gives (as 308d's Stalwart Defender).
  - The elements' PrerequisiteNoArchetype BloodKineticistArchetype 365b50dba54efb74fa24c07e9b7a838c (ElementalFocusAir
    2bd0d44953a536f489082534c48f8e31, …) also bound the planner's Expanded Element, which offers the same entries for
    the game's SecondaryElementAir 659c39542b728c04b83e969c834782a9 and the other second and third elements; those
    refuse no archetype. The compare refused three Blood Kineticist links at their expanded air (level 15). 308g's
    allowlist reason ("each maps one entry exactly") missed the shared use. Their six allowlist lines are gone, so the
    writer leaves them whole; `known.txt` names 306t.
- **The totems' verdict: the 72 `HasNot`s went.** `game-check show` of the eight plain and Greater totems
  (BeastTotemFeature c085888db293f2741b881cc989a2ab14, CelestialTotemFeature 5156331dc888e9347ae6fc81ad3f3cec,
  DaemonTotemFeature d673c30720e8e7c4bb0903dc3c9ab649, FiendTotemFeature ce449404eeb4a7c499fbe0248056174f and their
  Greater forms) shows no PrerequisiteNoFeature: each needs the totem before it and the level gate. The game does
  exclude lines another way: each Lesser totem refuses the other three Lessers and SpiritTotemLesserFeature
  bd63d3ea0e030cf4c930db9ba5eaf931 (BeastTotemLesserFeature d99dfc9238a8a6646b32be09057c1729 and the other three). That
  rule lives on the Lessers, which stay whole (SpiritTotemLesserFeature has no entry, 307) with the planner's refusal
  of the other lines' nine powers. Removing the 72 moves no character away from the game: a second line's plain or
  Greater power needs its Lesser, which the held Lesser refuses. **Inference:** no planner row grants a Lesser outside a
  pick (the Demon Dancer's row grants the Fiend Totem, not its Lesser, and the planner's refusals named the option, not
  that feature). The offers confirm it: no other line's power moves to `ok` (the census lists only moves to `Unmet` or
  `BreaksLater` for the totems), and `report_gaps::the_totem_rage_powers_are_exclusive_one_line_per_character` still
  finds every other line's power refused to a Beast Lesser holder.
- **Allowlist** (`prerequisite-narrowing.txt`): 32 lines removed, every entry's reason and narrowing: the 9 written wild
  talents' and infusions' (18), Increased Damage Reduction's, the Iceplants' and Intimidating Display's (8), and the
  elements' (6), which the run used and which went with them. The file now holds only its header.
- **`known.txt`:** 65 stale lines deleted (9 aeon gazes under 266, 24 rage powers under 269, 6 infusions under 270, 7
  wild talents under 271, 19 hexes and mutations under 278); re-homed: the four Lesser totems 269 → 307, Life-Bonding
  Friendship 266 → 307, Fire's Fury 271 → 270, 11 wild talents 271 → 306t, 17 shaman hexes 278 → 321 (307's decision
  3); the three elements stay, now 306t. Kept as they were (mine): the Hexcrafter's 13 hexes (283) and Battle Master
  (278), whose writer owner is 307 but whose gap is those tickets' rows (308a's fix-round comment says so), the
  Loremaster's combat feat (287), the Lich's upgrades and the Gold Dragon's breaths (266), the infusions naming a
  blast (270). Section comments name what 308f wrote.
- **`carriers-known.txt`:** regenerated with its header's command: unchanged (176 hits, all listed, no stale line).
- **Generators** (the Build count held at 2,157, each keeping what it tests): `talent fire 20` swaps Flame Shield (now
  14) and Skill Focus (now 6); `t65 celestial barbarian 6` takes Celestial Totem at 6, not 4; `t66 each power` and `t66
  deflect arrows only, extra rage power` have Dex 13 (RageDeflectArrowsFeature d971bbb15f354621ac37fd51d0ab9044).
- **Gold:** no change; the gold tests pass.

**Proof** (`record` with offers on the base in this lane; `compare` with `FORM_PROOF_CENSUS` = the run's log less the
six put back, `FORM_PROOF_OFFER_BUILDS_OUT`, `FORM_PROOF_LINKS_OUT`)

- **Builds:** 2,157 compared (1,992 open), 2,157 old links reopened, 0 differences: no lost, new or closed Build.
- **Census:** `census prerequisite: 1322 lines in 86 Builds (builds.tsv 86, numbers.txt 912, sheets.txt 169, slots.txt
  155)`, 0 unattributed.
- **Links:** 1,909 the same, 83 refused.
- **Offers:** 164,860 statuses changed in 539 groups and 286 Builds. The split: **7,114 on options the log names,
  157,705 in Builds whose links are refused, 41 else.** Narrowings (open kind → `Unmet`/`BreaksLater`, in Builds whose
  links open): the six plain and Greater totems 207–235 each (in 70–75 Builds; their level and the totem before), Come and
  Get Me 192 (85), Flame Shield 108 (41), Extended Range and Spindle 27 each (7), Wall 20 (5), Increased Damage
  Reduction 9 (7), Fearless Rage, Internal Fortitude, Protect Vitals and Reflexive Dodge 2 each; 53 entries narrowed
  none.
- **The 157,705 are the refused links' Builds, measured by `compare`'s own split:** a picker's option is `BreaksLater`
  when the walk after taking it fails at a later pick, so in a Build whose later pick the game now refuses, every
  earlier option turns `ok → BreaksLater` (308e's finding). `FORM_PROOF_LINKS_OUT` names each refused pick; every one
  is a written option refused for its blueprint's condition (below).
- **The 41 else, each read from the record's slots:**
  - `t66 deflect arrows bare` and its filled copy (6): Improved Unarmed Strike `ok → BreaksLater`. The Build (closed
    on both sides, by design) holds Deflect Arrows at 4 with Dex 10; taking the feat no longer opens it, since the
    power also needs Dex 13.
  - `full gap barbarian fiend` and its legend copy (20): five rage powers `ok → BreaksLater`. The Build holds Fiend
    Totem, Lesser at 2, Fiend Totem at 12 and the Greater at 18; replacing the Lesser or the plain totem now breaks the
    next one (FiendTotemFeature needs FiendTotemLesserFeature 76437492f801f054ba536473ad2fde79, the Greater
    FiendTotemFeature).
  - `full gap shaman None` and `full t62 shaman 20 / empty` with its legend copy (12): Dazzling Display `ok →
    BreaksLater`. They take Intimidating Display at 8 or 16, which refuses Dazzling Display
    (ShamanHexIntimidatingDisplayFeature 2eacbef4be7c6434e9cfaa9632fdc429).
  - `full t91 fighter 20 trickster legend / first aeon / gold-dragon` (3): Extra Wild Talent `NoOptionLeft → ok`. The
    Gold Dragon's dragon feats ignore prerequisites, and the feat's own list now holds Skill Focus and Skilled
    Kineticist with no kineticist level (SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a has no prerequisite).
- **`306-refused.txt`:** 63 links of 306's base in six new blocks, each a written option's game condition:
  - 7: Come and Get Me in an Extra Rage Power slot at 3 or 7 (ComeAndGetMeFeature 032402bde3fb47e39e6ebb6724ae9c22:
    12);
  - 32: a plain or Greater totem below 6 or 10, or before the totem it needs (barbarian, skald, Primalist and
    Instinctual Warrior Builds; t65 celestial barbarian 6's base link at 4);
  - 10: Deflect Arrows with Dex 10 (the t66 base links);
  - 2: Increased Damage Reduction for an Inciter (ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945);
  - 4: Spindle for a Kinetic Knight (SpindleInfusion c4f4a62a325f7c14dbcace3ce34782b5 needs ElementalFocusSelection
    1f3a15a3ae8a5524ab8b97f469bf4e3d, which KineticKnightArchetype 7d61d9b2250260a45b18c5634524a8fb removes at 1);
  - 8: Flame Shield below kineticist 10 (FlameShieldFeature 733bbf836ec575b44b3c88b61a26282e).

**Tests**

- New, `barbarian_witch_shaman_prerequisites.rs` (9, each citing its blueprints):
  `the_totems_need_the_one_before_them_and_refuse_no_line`,
  `increased_damage_reduction_refuses_the_inciter_and_names_no_shifter`,
  `a_shifter_4_is_offered_renewed_vigor_through_its_level_gate`,
  `the_infusions_take_the_levels_and_the_element_selection_their_blueprints_name`,
  `the_wild_talents_name_their_features_and_levels_not_an_element`, `the_hexes_refuse_their_twins`,
  `the_plague_mutations_need_the_prophet_of_pestilences_shaman_level`, `the_aeons_gazes_name_no_rank`,
  `pentamic_faith_refuses_itself_and_its_selection`.
- Un-ignored (4): `instinctual_rage_powers::the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name`
  (its Deflect Arrows also takes the game's barbarian 4), `skald_rage_powers::a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels`,
  `wild_talents::flame_shield_needs_kineticist_10_and_searing_flesh` (no fire element, as FlameShieldFeature),
  `infusions::a_kinetic_knight_is_refused_the_infusions_that_need_the_elemental_focus_selection`.
- Left ignored: 269's two row tests and 278's Battle Master (rows); 271's two (306t); 270's
  `each_infusion_needs_the_kineticist_level_its_blueprint_names` is re-pointed to 307 (Burning, Dazzling, Fan of Flames
  and Pushing name blasts), `infusions_need_a_blast_not_an_element` and the two infusion-at-1 tests stay 270's.
- Updated toward the game: `report_gaps::the_totem_rage_powers_are_exclusive_one_line_per_character` (the Lessers
  rule out the other lines, the plain and Greater totems need the one before; every other line's power is still
  refused, now for its own Lesser), `negated_prerequisites::every_negated_…` (Celestial Totem at 6; the Lesser's
  refusal and the plain totem's "Needs Beast Totem, Lesser"), `repeats_from_the_header::the_selections_with_their_own_no_feature_are_taken_once`
  (Pentamic Faith's shared NoFeature), `wild_talents::a_talent_needing_a_defensive_talent_waits_for_it` (the reasons
  name kineticist 10, and Healing Burst's list of one reads "Needs 1 more of: Kinetic Healer"), the infusion and wild
  talent tables (the written entries are held exactly in the new file), `prerequisites_as_components` (684 → 677).

**Checks** (each once after the last source edit)

- `cargo test -p seeds -p corpus`: game_data 645 passed, 90 ignored; files 34; links 6; form_proof 23; self_checks 1.
- `cargo test -p game-check`: 170 passed.
- `check-data`: checked (12 races … ).
- The game check: exit 0, "Differences: 3280 in 185 groups; 0 not known in 0 groups, 3280 known in 185 groups", no
  stale `known.txt` or allowlist line, "Carrier hits (ticket 322): 176, each listed in carriers-known.txt";
  prerequisites 1,447 compared, 1,094 agree.
- Second dry run: the same `--only` writes the 3 rays (above); with `witch/hex/*` for `witch/*`, "0 fields to write".
- `cargo fmt --all --check` clean; `scripts/test-modules.sh`: test-modules OK; `scripts/guarded-files.sh c29e8aa8`
  after the commit (the report gives its result).
- No `libs/rules` change. The self-checks' long runs are the main session's.
