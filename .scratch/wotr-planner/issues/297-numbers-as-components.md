# Numbers as components

Type: build (data form)
Status: resolved
Blocked by: 296
Model: Opus to build: every number source changes form, and the proof must show no number moves.
Map: [WotR Build Planner](../map.md)

## Problem

The staged component plan (user, 2026-11-15; [295](295-the-component-vocabulary.md)) moves the hand-shaped number fields onto the vocabulary's stat, value and line kinds. Those fields are:
- `bonuses.ron`;
- `ability_bonuses`;
- the BAB and skill floors (`bab_at_least_mythic_rank_plus`, Perfect Mind's skill floor);
- `steps` (as `Scales` on the game's `ContextRankConfig`);
- gear bonuses;
- the attack and caster lines' named feats.

The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 kinds 10, 11, 13, 14 and 16, and §4.

## Change

- **A rewrite script, kept outside the repo,** moves each number source onto its component, from the current data, not the game.
- **The engine** reads the kinds instead of the named fields and feats. The attack and caster lines read kinds, not feat names.
- **The fields go** where nothing still needs them.

## Answer

**Built** (2026-11-16) as written, with ticket 258 built inside it (superseded into 297: the generic Spell Focus kind and
the Expanded Arsenal kind) and the main session's decision from 295's review (every source's stat bonuses stack together
by the game's rule). Note: `libs/game-data/notes/297-numbers-as-components.md`.

**The rewrite** is a script outside the repo, a small Rust crate (`<scratchpad>/game/297/rewrite/`, `cargo run --release
-- <game-data dir>`), run on the data at `f5409f88`, not on the game. It reads the old fields with its own copies of the
old types, appends each entry's new components after those it holds (296's `Requires`), takes the field out, reads the
file back, and deletes `bonuses.ron`; run again, it moves nothing. Run on a fresh copy of the base data it gives the tip's
files exactly, but the two hand edits (Spell Focus (Mythic)'s summary, the README). Moved: `bonuses.ron`'s 179 rows onto
123 entries; 12 features' number fields (2 `steps`); 92 gear items; 11 line feats given their kinds: 418 components in
238 files (feats 40, features 33, gear 92, mythic feats 5, options 15, races 11, subraces 35, backgrounds 8), plus the
seeds fixture's 7 files and its `bonuses.ron`.

**The kinds** (`libs/rules/src/component.rs`, each citing its game component and code at WrathDecompiled 6ce4a6b):
- caster line: `SpellDc(scope, value)`, `SpellFocus(bonus, mythic_focus)`, `ExpandedArsenal(focuses)`,
  `SpellPenetration(value, while)`, `Concentration(value)`, `Metamagic(Heighten)`;
- attack line: `WeaponFocus`, `WeaponSpecialization`, `ImprovedCritical`, `CriticalEdge`, `CriticalMultiplier`,
  `PowerAttack` (and 295's `Flag(WeaponFinesse)`);
- gear: `GearStat(stat, value, descriptor)`, the one kind a gear item holds;
- the Gold Dragon's: `HighestAbilityBonus`, `AbilityAtLeast`, `BaseAttackAtLeast`, `GoldenDragonSkills`;
- `Scales(RankConfig)` on a feature (254's steps: the step is the rank's value);
- `StatTarget::Parameter` and `StatRef::Parameter`: the stat a parameterised entry is held for (Skill Focus, Stat
  Focus), until 298 gives those the game's per-skill features.

**The engine reads the kinds.** The walk records the bonuses in force from the components of the entries held and the
gear's `GearStat`s (`engine/walk/bonuses.rs`), works the ability scores, the base attack bonus and its floor, the skill
floor and the Gold Dragon's floors out from them (`walk/components.rs`, `walk/hold.rs`), and `Engine::steps_reached`
reads `Scales`. Spell Focus and Expanded Arsenal are worked out in the walk, each recorded as a spell DC bonus to the
school it adds to. The attack line reads the kinds of the feats held for its weapon (part order: focus, then
specialization, a feat after those it requires) and the Weapon Finesse flag; the caster line reads the bonuses in force
and `Metamagic(Heighten)`. `counted` stacks every source's bonuses to one number by the game's rule (`game_stacking`).
An entry whose components the walk never acts on (the attack line's, `Scales`, Heighten) isn't activated.

**Fields removed:** `GameData::bonuses` (`ItemBonus`, `BonusAmount`, the `bonuses.ron` file), `Feature::ability_bonuses`
(`AbilityBonus`), `bab_at_least_mythic_rank_plus`, `skill_ranks_at_least_mythic_rank`, `steps` (`FeatureSteps`,
`StepLevel`), `GearItem::bonuses` (`GearBonus`), `BonusType` (gear and the bonuses in force carry a `Descriptor`), the
attack and caster lines' feat-key constants. **Kept, and why:** `BonusItem` and `interactions.ron` (the interactions
notes name an item by it: planner text, not a number); the state's `bab_at_least_mythic_rank_plus` and
`skill_ranks_at_least_mythic_rank` (the walk's record of the kinds, which the BAB and the totals read); a race's
`ability_modifiers` (not in this ticket's list).

**Decisions.**
- Main session: unify the stacking pools (above). **No corpus number changed by it**: the former `bonuses.ron` rows
  keep their type (untyped, as the planner held them), the former ability bonuses are `Racial` (both stack with
  everything), and the data's gear has no racial, dodge or natural-armour bonus and only untyped penalties, the cases
  where the planner's old gear rule and the game's differ. The game's own descriptors come with 293.
- Mine: descriptors are today's rule in the game's terms where they differ from the game's: the ability bonuses count
  toward ability prerequisites, which in the game is `Racial`; Legend's and the Swarm's are untyped in the game (the 252
  follow-up). The Gold Dragon's floor and skill floor keep the planner's reading (the note lists the game's). Both are
  numbers a later ticket changes, not this form change.
- Mine: Spell Penetration (Mythic)'s two stages are a `while` condition on `SpellPenetration` (`HasNot`/`Has` Greater
  Spell Penetration), the game's `AddFeatureIfHasFact` pair with `RecalculateOnFactsChange` in one entry; the Duelist's
  Improved Reaction a rank on duelist levels (the game grants it twice).
- Mine: the bonuses in force keep the order they always had (race, subrace, background, feats, mythic feats, mythic
  abilities, features, options, by key; gear last), so every breakdown reads as before.

**What changed in numbers: only ticket 258's rules**, each toward the game:
- Spell Focus and Greater Spell Focus give +2 while Spell Focus (Mythic) is held for their school, which adds nothing of
  its own (`SpellFocusParametrized.GetBonusDC`; `SpellFocus` 16fa59cc9a72a6043b566b49184f53fe, `SpellFocusGreater`
  5b04b45b228461c43bad768eb0f7c7bf, `SpellFocusMythicFeat` 41fa2470ab50ff441b4cfbb2fc725109).
- Each Expanded Arsenal held adds, to a spell of any school an Expanded Arsenal names, the most its focuses held for one
  other school add up to; two add it twice (`ExpandedArsenalMagicSchools.OnEventAboutToTrigger`; `ExpandedArsenalSchool`
  f137089c48364014aa3ec3b92ccaf2e2, m_Focuses SpellFocusGreater and SpellFocus).
- A Spell Focus gives nothing to a school an Expanded Arsenal names (`SpellFocusParametrized.OnEventAboutToTrigger`),
  where the planner gave it and added no arsenal bonus.
- In the corpus: 117 Builds, 2,270 caster lines, every one a DC part of these kinds; DC deltas −2 (2 lines), −1 (369),
  0 (707: Spell Focus 1 + Mythic 1 became Spell Focus 2), +1 (904), +2 (274), +3 (4), +4 (10). No total, score, BAB,
  save, skill, attack line, slot, sheet or link changed. The MCP tests on the Max DC run move with them (DC 34 to 35,
  36 with metamagic; MR 4's Spell Focus (Mythic) +2; Expanded Arsenal (Enchantment) −4 on its own school), each cited.

**Tests** (expected values from the game, cited):
- `libs/seeds/tests/game_data/numbers_as_components.rs`: `bonuses_ron_is_gone`;
  `the_attack_and_caster_lines_feats_hold_their_kinds` (WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e,
  WeaponFocusGreater 09c9e82965fb4334b984a1e9df3bd088, WeaponSpecialization 31470b17e8446ae4ea0dacd6c5817d86,
  WeaponSpecializationGreater 7cf5edc65e785a24f9cf93af987d66b3, ImprovedCritical f4201c85a991369408740c6888362e20,
  TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72, TricksterImprovedImprovedImprovedCritical
  006a966007802a0478c9e21007207aac, TricksterImprovedImprovedImprovedCriticalImproved
  319c882ab3cc51544ad2f3f43633d5b1, WeaponFinesse 90e54424d682d104ab36436bd527af09, PowerAttackFeature
  9972f33f977fc724c838e59641b2fca5, HeightenSpellFeat 2f5d1e705c7967546b72ad8218ccf99c);
  `perfect_minds_floors_are_its_components` (DragonLevel2BABOverride f4af14e9f4408d44c830d891fee8f4a5,
  GoldenDragonSharedSkillUMD 4e0b919e4fbd85142bce959fae129d1a, `AddGoldenDragonSkillBonus.OnTurnOn`);
  `the_dragon_disciples_constitution_is_a_racial_stat_bonus` (DragonDiscipleConstitution
  95de611796547be489e527fae84abbd6); `the_bloodrager_features_scale_on_bloodrager_levels`
  (BloodragerInfernalResistanceBuff 66c973ccad1293345ae6c5bfeced4a7b, BloodragerAbyssalBloodrageBonusProperty
  415f71e5a47f4cccb3dbd10bd7a0f8f8); `a_gear_items_bonuses_are_gear_stats` (BeltOfAlacrityItem
  4b0b9b95225d6034f84b75a6ad1e76c7: Dexterity4 15008553994dd7e48af2ad1a1426d733, BeltOfAlacrityFeature
  76d9b40f12694d34791a507e736d58c8 Reflex +4 Competence).
- `caster_line.rs`: 258's three tests no longer ignored (`mythic_spell_focus_doubles_the_max_dc_runs_spell_focus_feats`,
  `the_max_dc_runs_dcs_from_level_7_count_mythic_spell_focus_doubled`,
  `the_max_dc_runs_expanded_arsenal_gives_conjuration_enchantments_spell_focus_feats`, values as 242 found them), and
  `the_components_cover_the_dc_runs_sources` in place of the `bonuses.ron` one.
- Rules (hand-made data): the bonuses, totals, caster-line, steps, gear and check tests rewritten onto components with
  their values unchanged, but the test world's Spell Focus (Mythic) (doubling) and Expanded Arsenal (once per fact, own
  school's focus out), cited; `a_scales_names_known_classes_and_sits_on_a_feature`,
  `only_gear_holds_a_gear_stat_and_it_holds_nothing_else`,
  `the_data_check_refuses_a_value_of_the_held_skill_on_an_entry_taken_for_a_stat`.

**The browser JSON** changes in form: features lose the four number fields serde wrote on every one
(`ability_bonuses`, `bab_at_least_mythic_rank_plus`, `skill_ranks_at_least_mythic_rank`, `steps`), the top-level `bonuses`
array goes, a gear item's `bonuses` become its `components` (`GearStat`), and the entries that give a number gain
`components` with the new kinds. 2,097,210 bytes at `f5409f88` (240,826 gzip -9), 1,917,174 on the tip (237,951).

**The proof.** The form proof, extended by this ticket to record each Build's numbers at every level (`numbers.txt`:
the three score sets, BAB, base saves, caster level, the totals with their parts, the caster line for 16 spells, the
attack lines of the default weapon and the longsword, each held feature's steps), `record` at `f5409f88` in a temporary
worktree (removed after) with the same recorder, and `compare` on this tip: 2,162 Builds compared (2,001 open), 2,162
old links reopened, 117 differences, all in `numbers.txt`: the 117 Builds above, every differing line a caster line whose
DC parts differ by the 258 kinds alone (classified line by line: no other part, no other number). `builds.tsv`,
`slots.txt` and `sheets.txt` are byte-identical. The game check exits 0 at `f5409f88` and on this tip, the reports
differing only in the run time (`game-check: 1.0 s` / `0.7 s`). The gold builds and the self-checks' short runs pass.

**Checks**, after the last source edit:
- `cargo test -p rules -p seeds -p corpus -p mcp -p ui -p game-check`: every binary ok (rules engine 722 passed,
  1 ignored; seeds game_data 506 passed, 131 ignored; seeds files 34; corpus self_checks 1; mcp server 246; game-check
  fixture 30; ui 6).
- `cargo run -p seeds --bin check-data`: `checked …: 12 races, 11 skills, 73 weapons, 261 feats, …, 92 gear` (exit 0).
- `cargo build -p rules --target wasm32-unknown-unknown`, `cargo clippy -p frontend --target wasm32-unknown-unknown --
  -D warnings`, `cargo clippy -p rules -p seeds -p corpus -p mcp -p ui -p game-check --all-targets -- -D warnings`:
  finished; `scripts/css-classes.sh`: `163 classes, all used`; `cargo fmt --all --check`: clean.

**Simplify** (the skill, four reviewers): Spell Focus and Expanded Arsenal worked out in the walk as spell DC bonuses
(the caster line's own arsenal pass and two `InForceTarget` variants gone); one stat resolution and amount
(`StatTarget::resolve`, `stat_bonus_amount`) for the scores and the totals; the score pass reads only ability and BAB
bonuses; entries whose components the walk never acts on aren't activated; a rank on a level or a per-rank value no
longer refreshes the numbers at every hold; the gear's bonuses in force built once per walk; `holder_order` from
`HolderKind`; `feat_depth` through `collect_feats` with cached sort keys; `SpellScope::applies` without its dead value;
dead helpers removed. The tip's form-proof files are byte-identical before and after it; a release record takes 6.7 s
against 5.4 s at the base (7.4 s before it). Not taken: the gear as a holder in the walk's held components (one
pipeline; a larger change), a value-only stat reference instead of `StatRef::Parameter` (298 removes the
parameterised Skill Focus), compiling `SpellPenetration`'s `while` once at load (it is on one entry), Power Attack and
Heighten as state flags.

## Done when

- No number comes from a hand-shaped field the vocabulary covers.
- The proof is clean:
  - the corpus Builds' sheets, attack lines, caster lines and totals are identical before and after;
  - the hashes and links are identical;
  - the self-checks, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.
