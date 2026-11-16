# The progression header

Type: build (data form and engine)
Status: resolved
Blocked by: 303
Model: Opus to build: every class's option levels change source.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `option_levels` and its kin, the planner's way of saying which class levels a selection's options count.

## Change

- **The progression header fields replace them,** as the game's `BlueprintProgression` holds them (its classes, archetypes and level sources; cite the blueprint fields and the code that reads them).
- **A known limit:** `option_levels` can't move onto the options while the domains are shared ([294](294-the-domain-split-with-a-slot-aware-link-migration.md)). The domains keep the field until 294, and the Answer says so.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.


## Answer

**Built** (2026-11-16) on the main session's decisions (`<scratchpad>/game/304/decisions.md`, Q1–Q5 and the field
list), in two sessions: a first builder's uncommitted work, then a second builder who kept and finished it. Note:
`libs/game-data/notes/304-the-progression-header.md`.

**The header.** `SelectionOption::header: Option<ProgressionHeader>` (`libs/rules/src/game_data.rs`): `classes` and
`archetypes`, each with its offset, `half_other_classes` and `previous_levels`, the fields of the option's
`BlueprintProgression` that `BlueprintProgression.CalcLevel` and `LevelUpHelper.UpdateProgression` read
(WrathDecompiled 6ce4a6b). `class_levels_of` (`engine/walk/options.rs`) takes a header and adds as `CalcLevel` does:
no classes and no archetypes is the character level; a class's offset counts even untaken; a sum below 0 adds 0.
`Catalog::level_header` picks the header that makes a holding's level: the option's own where its selection names no
classes or sets `option_headers_first`, else the one the selection's `option_levels` and `option_level_archetypes` make
(`ProgressionHeader::of_selection`, every row up to the level), the planner's form until 293 and 294 (Q5). A holding
without `previous_levels` skips the rows of the levels it passes over (`Holding::previous_levels`, `reached`). Gone:
`FeatureSelection::half_other_classes`, `option_lists`, `OptionLists`, `OptionLevelRow::feat_list`,
`CharacterState::option_feat_lists`, `FeatureSelection::offers_feats` and `levels_by_class`. New: `GameData::offers_feats`
and `nests_feat_lists`, `Engine::nest_opened_by` (the web picker's "choose the … first"), `DataIssue::HeadersFirstWithoutClasses`.

**Nesting** (decisions, option_lists). A selection of feats whose every option is a list (`is_selection`, one level-1
row granting it) nests them, and its Feat slot offers the feats of each list whose option's prerequisite holds
(`Catalog::slot_lists`). `sorcerer/bloodline-feats` is `SorcererFeatSelection` 3a60f0c0442acfb419b0c03b584e1394 with its
nine `Bloodline*FeatSelection` options; the ranger's style feat is `ranger/combat-style-feats-6` (`RangerStyleSelection6`
61f82ba786fe05643beb3cd3910233a8), `-10` (`RangerStyleSelection10` 78177315fc63b474ea3cbb8df38fafcd) and the planner's
`-2`; the slayer's stage selections share the ranger's stage options; the bloodrager's and kineticist's stay planner
nests with a `why`. Every SlotId and pick stays (Q3).

**The data** (the first builder's Rust crate `<scratchpad>/game/304/rewrite/`, then by hand): headers on 143 options
(125 with rows, 18 without: the patrons, Order of the Nail, Form of Flame, Wolverine Aspect, Lore (Religion) II and
III, which the crate's line-based edit missed on one-line files); `option_levels` and its kin off the 23 selections
whose options all have one; `option_headers_first` on the three revelation selections; 38 nested options (19 with a
blueprint gated by a `Requires`, 19 planner ones by a `prerequisite`, as planner entries hold no `Requires`); `planner/selections/ranger/combat-style-feats.ron` replaced by the three stage selections; the 62
`feat_list` rows gone. `libs/game-data/README.md`'s option paragraph says the header and the nesting.

**Five levels toward the game** (Q1), each cited in `progression_headers.rs`: non-Draconic sorcerer bloodlines count an
Eldritch Scion's magus levels and no Dragon Disciple's (`BloodlineAbyssalProgression` d3a4cb7be97a6694290f0dcfbd147113);
Draconic ones count sorcerer, Dragon Disciple and Eldritch Scion levels from any selection
(`BloodlineDraconicRedProgression` 8c6e5b3cf12f71e43949f52c41ae70a8); a bloodrager's Draconic bloodline counts Dragon
Disciple levels (`BloodragerDragonRedBloodline` 34209f220733fc444a039df1b1076b0b); the Frost spirit counts Winter Witch
levels (`ShamanFrostSpiritProgression` 44c130a12d49ebf4a95945feffe5aae7); Weapon Mastery counts oracle and Magic
Deceiver levels (`OracleRevelationWeaponMastery` 0a4c3556355747241b2d3dcc0a88ec10). No corpus Build moved (the proof).

**The game check** (exits 0): a new area, "progression headers" (`tools/game-check/src/areas/headers.rs`), compares
every option's header with its blueprint: 796 options with a blueprint compared, 793 agree, 3 differ (the Lich's, known). The options area now compares the nests by their options (the
skip for option-list selections is gone); the rows area keeps "any pick" only for a blueprint-less nest. Q4's gating
quirks and the Lich's three upgrades (they count `LichMythicClass`, no planner class) are known under 293
(`known.txt`, "From 304"). Against the base's report: 24 ranger rows findings gone (RangerStyleSelection6 and 10 now
match), 3 Dragon Disciple rows findings new ("a None feat slot" at 2, 5 and 8: the Dragon Disciple's bloodline feat
is now compared as `SorcererFeatSelection`'s pool, under 287's line), the nests' prerequisites and the Lich headers new
and known.

**Decisions.**
- Main session: Q1–Q5 and the field list, as in `decisions.md`.
- First builder: `option_headers_first` on the revelation selections, so Weapon Mastery's header goes before the
  revelations' `option_levels` (the alternative, a header always first, would break Q5's domains).
- First builder: a nest is recognised from the data (every option a list), not marked by a field.
- Mine: headers on the 18 rowless progression options too ("each option's blueprint"); their level is read by
  nothing today. The Lich's three get none and are known under 293.
- Mine: `option_levels` stays on 50 selections (the 48 of the research's list that keep it, plus the 302 copies
  `kineticist/extra-wild-talent` and `slayer/extra-slayer-talent`); `ranger/combat-style-feat` and
  `slayer/combat-style` keep theirs though their options have no rows now (inert; 293's).

**Form proof** (ticket 300; base record `f68e6954` before any edit, compare on the tip): `builds.tsv`, `sheets.txt` and `numbers.txt` byte-identical (hashes 64f733a24439cf0a, b365ca4eeea5a35f, 98e2338907381909, as the base record); `compare` prints 2,162 Builds compared (2,001 open), 2,162 old links reopened, 64 differences: `slots` and `reopened slots` each in the same 32 Builds (ranger, slayer and multiclass corpus Builds), all the kind column. A Rust classifier over a kind map (`<scratchpad>/slot-kinds/`, std only, streaming) over the two `slots.txt`: 49,648 lines, 132 renamed kinds in 32 Builds (`ranger/combat-style-feats` to `-2` at level 2: 32, `-6` at 6: 26, `-10` from 10: 74), 0 other differences. Every SlotId and pick is the same; `cargo test -p corpus --test links` passes. So the five level changes and `previous_levels` moved no corpus Build, as the research found.

**Tests.** Rules (hand-made): `ranks_and_pools::a_header_with_no_classes_counts_the_character_level_and_a_class_its_offset`, `a_header_without_previous_levels_skips_the_rows_it_passes_over`, `a_selections_own_classes_go_before_an_options_header_unless_it_lets_headers_first` (`CalcLevel`, `UpdateProgression`); `check::a_selection_of_feats_lists_them_or_nests_lists_not_both` (renamed); the test world's ranger, slayer and sorcerer nest as the data does, and the curse world levels by a header. Real data (`libs/seeds/tests/game_data/progression_headers.rs`, each value cited by blueprint name and GUID): `a_non_draconic_bloodline_counts_an_eldritch_scions_magus_levels_and_no_dragon_disciples`, `a_draconic_bloodline_counts_sorcerer_dragon_disciple_and_eldritch_scion_levels`, `a_bloodragers_draconic_bloodline_counts_dragon_disciple_levels`, `the_frost_spirit_counts_winter_witch_levels`, `weapon_mastery_counts_oracle_and_magic_deceiver_levels`, `a_school_and_an_element_get_no_rows_of_the_levels_before`; `feat_slots_as_selections::the_bloodline_feat_slot_is_sorcerer_feat_selection` (the nine nested lists, `SorcererFeatSelection`'s `m_AllFeatures`). Fixture: `files.rs` reads a header and a nest (`reads_subrace_slots_a_nesting_feat_slot_and_options_levelled_alongside_and_defaults_them`, renamed). Moved by form only: the entry counts (`blueprints.rs` 58 blueprint-less options, `prerequisites_as_components.rs` 630 `Requires`), the ranger stage keys (`archetype_losses.ron`, the engine tests' slot kinds), the kineticist's list read from the nest (`option_families.rs`). No gold build or self-check changed.

**Checks** (after the last edit): `cargo test -p rules` (750 passed, 1 ignored), `-p game-check` (31), `-p mcp` (246), `-p ui` (6), `-p frontend`, `-p seeds -p corpus` (game_data 531 passed, 130 ignored; files 34; links 2; self_checks 1) all ok; `cargo run -p seeds --bin check-data` checks the data; `cargo build -p rules --target wasm32-unknown-unknown` and `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings` finish; `scripts/css-classes.sh`: 163 classes, all used; `cargo fmt --all --check` clean; the game check exits 0 (4,556 differences, all known).

**Not done or not verified.** The self-checks' long runs (`--ignored`) are the main session's. The five level changes are pinned by header and by two engine walks (the sorcerer's with an Eldritch Scion); the bloodrager-with-Dragon-Disciple, Frost-with-Winter-Witch and Weapon Mastery walks are pinned by header only (the planner refuses or makes long those Builds). `simplify` was skipped (a form ticket, by the brief).

**From the review** (main session, 2026-11-16):
- **The bloodline level changes reach the main selection only.** The non-Draconic bloodlines dropping Dragon Disciple levels holds for `sorcerer/bloodline` alone. The per-slot copies keep their own `option_levels` (for example `["dragon-disciple", "sorcerer"]`), which still override the header, as before. Those copies are `sorcerer/crossblooded/second-bloodline`, the Sage, Empyreal, Seeker and Sylvan sorcerers', `fighter/dragonheir-scion/bloodline`, `oracle/second-curse`, `living-deity/curse`, `kineticist/expanded-element` and the kinetic knight's element. 293 writes their headers when it splits them.
- **50 selections keep `option_levels`, not the decided 49.** The two extra are 302's `kineticist/extra-wild-talent` and `slayer/extra-slayer-talent`. Both behave as before.
- **Dragon Disciple's three new row findings** (a bonus feat slot at levels 2, 5 and 8) come from this ticket's nesting. The game grants `BloodlineDraconicFeatSelection` f4b011d0… there, and the planner names `sorcerer/bloodline-feats`. The offer is unchanged. They sit under 287's line, and 287 removes them.
