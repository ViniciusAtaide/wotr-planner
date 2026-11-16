# Ticket 306, the importer's stage S2 (selections and the per-class splits): the slices (research, 2026-11-16)

Read-only research at `main` 69600a9f (S1's writer 293a–e merged; 293f–l and 305 open). Game: the pinned dump
(236,660 blueprints) through lane-game's `game-check` build; code WrathDecompiled 6ce4a6b. No game text below. Every
selection is identified by what grants it (`grants2.tsv`: the class progressions' `LevelEntries`, the archetypes'
`AddFeatures`/`RemoveFeatures`, the path progressions), never by name.

Evidence beside this file (all in this folder):
- `run.txt`: the check on main's data: 4,556 differences in 212 groups, exit 0.
- `planner.tsv` (GUID → kind, key: 4,085 entries), `grants2.tsv` (grantor, section, level, granted GUID),
  `all-lists-mapped.tsv` (each game list's members → planner entries), `ol-counts.tsv` (option-list findings per
  selection and ticket), `splits-by-sel-entry.txt` (the 255 split row findings by game selection and entry).
- Corpus: `picks-open.tsv` (every pick of the 2,000 open Builds of main's form-proof record
  `../305/proof-before/`, after 310, with the class, archetype and class level taking that level),
  `corpus-removed.tsv` (talents), `corpus-feat-removed.tsv` (feat lists), `corpus-extra-removed.tsv` (Extra feats),
  `corpus-refused-all.tsv` (all three: Build, game list, slot, key). The awk that made them: `fam.awk`, `featfam.awk`.

Slice names (306a…306ad) are placeholders. "Fixture" = `tools/game-check/tests/fixture/`. "Run" =
`game-check --write option-lists --only <keys> --log <scratchpad>/306x.tsv` (306a adds `--only`). Every slice runs
`cargo fmt --all --check` and the CLAUDE.md "Checks" rows of what it touches; every data slice also runs the game check
(exit 0) and writes `libs/game-data/notes/306x-<slug>.md`.

## 0. Decisions taken here (say if you disagree)

1. **The writer writes list membership only.** `selections/` owns `options`, `shared_options`, `feats`,
   `mythic_abilities`, `ignores_prerequisites` (design §1.2's table, less the header fields 303/304 already write by
   hand). Rows, new files, keys, names, aliases and `blueprint` are hand edits in the family slices (Q1).
2. **All or nothing per selection,** as the items target does per item: a selection is written only when every member
   of its game list resolves to one entry of its kind. A member with none (Evasion is a feature; the shaman's hexes
   have no option) leaves the whole selection unwritten and listed. That keeps 294's domains, G6's Thassilonian
   schools and 289's Finesse Training untouched with no special case, and it orders the work: a family slice makes
   the missing entries, then runs the writer.
3. **Resolution by the selection's kind:** `feats` non-empty → feats; `mythic_abilities` → mythic abilities; else
   options. A GUID several options hold → those of them the selection offers today; none offered and one holds it
   → that one; else ambiguous, unwritten. (rogue/kineticist Skill Focus c9629ef9…, Clear Mind, the arcane
   school powers' 8891303b… are the cases met.)
4. **Ownership** (`OptionNotDefinedOnce`, `libs/rules/src/check.rs:1380`): an option stays in its owner's `options`
   while that owner's game list holds it; else it moves to the first selection, in key order, whose game list holds
   it; else it stays and is listed. With `--only`, a move whose other end is outside the set is not made (listed).
5. **Selection keys are free to change.** A link carries picks by slot (positional `SlotId`, `libs/rules/src/build.rs:418`)
   and option/feat keys, never a selection key, so a split or rename of a selection needs no alias. Aliases are for
   option and feat keys only, and mythic abilities can't migrate (`LinkKeys::heirs` returns none for them,
   `codec.rs:375`): a mythic split keeps every old key's meaning.
6. **Copies copy the planner's option.** A per-class copy of an option (a mystery, curse, bloodline, hex, element) is
   the original's file with a new `key`, the copy's `blueprint` and `aliases: [<original key>]`; name, summary,
   rows, feats and components stay the original's (the planner's words, Q1 of the design). S4/S5 then write the
   copy's prerequisites and rows from its own blueprint.
7. **Extra feats:** 302's `needs_slot_of` goes (306aa) after every Extra selection holds its own list; 303's
   no-option-left reads the options' prerequisites as `FeatureSelectionExtensions.CanSelectAny` does (306d), so an Extra
   feat whose list has nothing the character can take is refused, as the game does.
8. **`once_per_option` is not generalised:** the game's `ThirdElement*` options each carry `PrerequisiteNoFeature` of the
   matching `SecondaryElement*` (`ThirdElementAir` 651570c873e22b84f893f146ce2de502 → `SecondaryElementAir`
   659c39542b728c04b83e969c834782a9), so the split's options carry that `Requires` and `expanded-element` loses
   `once_per_option` (its only user). The rules field is deleted in 306aa if no data names it.
9. **Magical Tail is a remap, not an 8-way split** (the design's §2.3 is wrong here): the pool's member is
   `MagicalTail` febb8fe9a2d142fb80c1be6b0b539d9d, one feat of `Ranks` 8; `MagicalTail1`… are what each rank grants.
   `feats/magical-tail.ron` changes its `blueprint` to it and keeps `ranks: 8` (now the game's).
10. **What S2 leaves** (each re-homed to its gap ticket by the slice that meets it; Q3): 294's domains (114), 289's
    Finesse Training (31), G3's parametrised lists (255: 48; the mythic Armor Focus pool 10), G5's choices
    (13: Greater Wyrmshifter, Imitation, Know Your Enemy, Geomancy, Judgment, Hell's Decree/Seal, Feral Champion's
    blessing, Cruromancer ×2, Greater Exploits (260), and the Empyreal/Sage/Sylvan "bloodline" copies' 60), G6's
    Thassilonian schools (14), the Lich's skeletal upgrades (6), the Elemental Specialist's schools (8), the pets'
    Order of the Paw mount (1 row, 174).

## 1. The slices, in order

Lanes (CLAUDE.md: two builders at most). All S2 work waits on S1 (306 is blocked by 293): the writer lane on 293h
(shares `src/write/mod.rs`, `main.rs`), the data lane on 293l (shares `known.txt`, `user-facts.txt`).
- **Lane W** (`tools/game-check`): a → b, later ab → ac.
- **Lane C** (`tools/corpus/tests`): c, beside a/b.
- **Lane R** (`libs/rules`): d beside a/b; aa after the Extra lists (f, g, h, i, j, k).
- **Lane D** (`libs/game-data`, `known.txt`, `user-facts.txt`, `libs/seeds/tests/game_data/`): e → z in order, one
  at a time (every one edits `known.txt`). Starts when a and b are merged.
- **Then** ad (main session; heavy runs alone).

Parallel group per slice: **P1** = {a→b} ∥ {c, d}; **P2** = Lane D ∥ {ab, ac, aa when ready}.

Shared test file for Lane D: `libs/seeds/tests/game_data/importer_lists.rs` (new in 306e, `mod importer_lists;` in
`game_data/main.rs`), one section per slice. "Un-ignore" = run `cargo test -p seeds --test game_data <name> -- --ignored`;
pass → drop `#[ignore]`; fail → rewrite its reason to name the gap ticket.

---

### 306a. Writer: the selections target, feat and mythic lists, `--only` (Lane W, P1)

**Goal:** `--write option-lists` brings a selection of feats or mythic abilities, and its `ignores_prerequisites`, to its
blueprint's `m_AllFeatures`/`IgnorePrerequisites`.

**Edits:**
- `src/areas/options.rs`: lift `references(json["m_AllFeatures"])` (line 145) into
  `pub(crate) fn game_list(json: &Value) -> Option<BTreeSet<BlueprintGuid>>` (`None`: no `m_AllFeatures`, "a choice
  the game doesn't offer"); `compare` calls it, output unchanged.
- `src/write/selections.rs` (new): `pub(crate) fn changes(ctx, only: Option<&BTreeSet<String>>) -> (Vec<Change>,
  Vec<Unexpressed>)`: selections with a blueprint, `has_list()`, in key order, filtered by `only`; kind by decision 3;
  each member → `ctx.planner` entries of that kind holding the GUID (add `Planner::entries_of(guid) -> Vec<&DataAt>`
  in `src/planner.rs`); all or nothing (decision 2); `feats`/`mythic_abilities` as `Value::List` of quoted keys,
  sorted; `ignores_prerequisites` (`Value::One("true"|"false")`) only where `data.offers_feats` and it differs.
  `kind: "list"`, `old`/`new`: the keys removed / added.
- `src/write/span.rs`: `pub const SELECTION_FIELDS` in `FeatureSelection`'s declared order
  (`libs/rules/src/game_data.rs:1143-1264`: key, blueprint, name, summary, guide, options, feats, mythic_abilities,
  needs_slot_of, ignores_prerequisites, skips_proficiency, shared_options, option_levels, …, components).
- `src/write/mod.rs`: `pub mod selections;`; `WriteArea::OptionLists` (`"option-lists"`, area `Area::OptionLists`,
  kind `"list"`); `declared`/`owned` for `selections`: `options`, `shared_options`, `feats`, `mythic_abilities`,
  `ignores_prerequisites`; `run(…, only: Option<&BTreeSet<String>>, …)`.
- `src/main.rs`: `--only <key>[,<key>…]` (only with `--write`), USAGE and doc header (`items, option-lists`).
- Fixture: `blueprints.json` gains `TestBonusFeatSelection` (two feats, `IgnorePrerequisites: true`) and
  `TestPathAbilitySelection`; `planner.ron`/`data/selections/` the two selections (one with a feat the game lacks,
  one lacking a game feat); `data/feats/` as needed.

**Tests** (`tests/fixture/write.rs`): `writing_a_feat_list_brings_it_to_the_game`,
`ignores_prerequisites_is_written_where_it_differs`, `a_member_with_no_feat_entry_leaves_the_selection_unwritten`,
`only_writes_only_the_selections_named`, `a_second_option_lists_run_writes_nothing`.

**Proof:** none of §4.2 (no data). `game-check --write option-lists --dry-run` on main's data: lists the feat selections
it would write (expect the 267/268/275/279/285 feat lists, `magic-feats` and `teamwork-feats`; `feats`, `combat-feats`
and the cavalier's list unwritten until 306q/r make the AWT, Extra Channel, … feats). The data slices write each list
only through their own `--only`.
**Blockers:** 293h merged. **Shares:** `src/write/mod.rs`, `span.rs`, `main.rs`, fixture with b. **Estimate:** 20 min.

### 306b. Writer: option lists and ownership (Lane W, P1)

**Goal:** `options`/`shared_options` from the game's list, each option defined once (decision 4).

**Edits:** `src/write/selections.rs`: options kind; the two-phase plan (every selection's resolved game list first,
then the owners: decision 4); `options` = owned members, `shared_options` = the rest; a removed own option moves or
stays listed; `--only` limits writes, not resolution. Fixture: an IW-shaped pair (a base list owning an option only
the copy's game list holds), a shared option, two options on one GUID.

**Tests:** `an_option_moves_to_the_selection_whose_game_list_holds_it`, `an_option_no_game_list_holds_stays_and_is_listed`,
`a_guid_two_options_hold_resolves_to_the_offered_one`, `a_move_outside_only_is_not_made`,
`the_written_lists_pass_the_data_check` (each option defined once).

**Proof:** dry run on main's data: the selections written, the unexpressed list (expect the shaman's hexes, the copies,
294's domains). **Blockers:** a. **Shares:** as a. **Estimate:** 20 min.

### 306c. The link census in the form proof (Lane C, P1)

**Goal:** `compare` counts every old link three ways (§4.2 point 4) and a links test pins a stage's refusals.

**Edits:** `tools/corpus/tests/form_proof.rs` `compare` (line 834): for each recorded Build that opened, `engine.open_link`
→ `same` (picks equal), `heir` (equal but for picks carried through `aliases`: lift `carried`/`assert_same_picks` from
`tests/links.rs` into `tools/corpus/src/links.rs`, `pub`), `refused`; print the three counts and, with
`FORM_PROOF_LINKS_OUT=<file>`, write `name\tslot\tkey` of each refused pick and each heir. `tests/links.rs`: a test
reading `tests/links/<stage>-base.tsv` (name, link) and `tests/links/<stage>-refused.txt` (names): each opens with the
same picks or an heir, or is listed and refused, and no listed one opens. Fixture: three links in
`tests/links/fixture-base.tsv`.

**Tests:** `a_base_link_opens_the_same_through_an_heir_or_is_a_listed_refusal` (fixture),
`a_listed_refusal_that_opens_fails`. **Proof:** `cargo test -p corpus`. **Blockers:** none. **Shares:** nothing with
W/D. **Estimate:** 15 min.

### 306d. No option left reads the options' prerequisites (Lane R, P1)

**Goal:** a pick granting a selection is refused when no option of it is takeable, prerequisites included, as
`FeatureSelectionExtensions.CanSelectAny` → `CanSelect` (WrathDecompiled 6ce4a6b,
`Kingmaker/Blueprints/Classes/Selection/FeatureSelectionExtensions.cs` line 10, called by
`NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`).

**Edits:** `libs/rules/src/engine/slots.rs:103` `has_option_left`: an option, feat or mythic ability counts when it is
under its repeat count **and** its prerequisite holds in `state`; `libs/rules/src/choice.rs:85` `NoOptionLeft` doc drops
"not the options' prerequisites". Test world (`libs/rules/src/test_world/world/nested.rs`): an Extra-feat-shaped feat
whose list's one option needs a class level.

**Tests:** `libs/rules/tests/engine/nested_selections.rs`: `a_selection_whose_options_all_fail_their_prerequisites_is_no_option_left`,
`one_takeable_option_keeps_the_pick_open`. **Proof:** the agreement sweeps' short runs (`cargo test -p seeds -p corpus`);
note any corpus Build that stops opening (expect none: Extra feats still stand for the first had until 306aa).
**Blockers:** 305 merged. **Shares:** none with W/D. **Estimate:** 15 min.

---

### 306e. Rogue-talent copies: bard, skald, Inciter (Lane D)

**Goal:** the bard's, skald's and Inciter's talents are their own lists.
**Edits:**
- New `selections/bard/rogue-talents.ron` (`BardTalentSelection` **94e2cd84bf3a8e04f8609fe502892f4f**, granted by
  `BardProgression` 8127f5ff40f5b484b8be98609358b9d2 at 2, 6, 10, 14, 18; the other GUID of that name,
  40f85fbe…, only units reference), `selections/skald/rogue-talents.ron` (`SkaldTalentSelection`
  d2a8fde8985691045b90e1ec57e3cc57: `SkaldProgression` 2, 7, 12, 17), `selections/skald/inciter/rogue-talents.ron`
  (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4: `ProvocateurArchetype` a1f5ab7eb78942f6889304acbd92e945
  `AddFeatures` 4, 9, 14, 19). Each copies `selections/rogue/rogue-talents.ron` (name, summary, guide,
  `option_levels`) with `options: []` and `shared_options` its current offer.
- Rows: `classes/bard.ron` 2, 6, 10, 14, 18 and the bard archetype rows repeating them (archaeologist, beast-tamer 14,
  chelaxian-diva, dirge-bard, flame-dancer, thundercaller, tranquil-whisperer) → `bard/rogue-talents`;
  `classes/skald.ron` 2, 7, 12, 17 and court-poet, battle-scion, demon-dancer, herald-of-the-horn, hunt-caller →
  `skald/rogue-talents`; `archetypes/skald/inciter.ron` 4, 9, 14, 19 → `skald/inciter/rogue-talents`.
- Run `--only bard/rogue-talents,skald/rogue-talents,skald/inciter/rogue-talents`: 14, 17 and 14 offers removed.
- `known.txt`: nothing to delete (the kind line 293:20 stays until the last split).
**Tests** (`importer_lists.rs`): `the_bard_skald_and_inciter_talents_are_their_own_lists` (each count and removed key,
cited by GUID); un-ignore `a_bard_and_a_skald_pick_from_their_own_talent_lists` (276).
**Proof:** rows "a selection the game keeps separate" −70 (Bard 38, Skald 28, Provocateur 4); **refused: 28 open
Builds** (bard 14, skald 12, Inciter 2; `corpus-removed.tsv`). **Estimate:** 15 min.

### 306f. Dark Lurker, Evasion, and the rogue-family memberships (Lane D)

**Edits:** new option `options/rogue/rogue-talents/evasion.ron` (`Evasion` 576933720c440aa4d8d42b0c54b77e80; today only
`features/evasion`; rows grant that feature; name and one-line summary in our own words), owned by the new
`selections/rogue/dark-lurker/rogue-talents.ron` (`DarkLurkerRogueTalentSelection` 1edbb68422c046f98a86f4726f6721b2:
`DarkLurkerArchetype` `AddFeatures` 4, 6, 10, 12, 16, 18, 20); `archetypes/rogue/dark-lurker.ron` gains rows 4, 6, 10,
12, 16, 18 naming it (its talents come from inherited rows today) and 20's. Run `--only rogue/dark-lurker/rogue-talents,
rogue/rogue-talents,rogue/sylvan-trickster/fey-tricks,rogue/underground-chemist/discovery,shifter/feyform-shifter/rogue-talent,
rogue/master-of-all/skill-focus`. `known.txt`: delete the option-list lines of 276 (sylvan-trickster, underground-chemist,
master-of-all) and 277 (feyform). `archetype_losses.ron`'s dark-lurker and underground-chemist entries per 276.
**Tests:** `the_dark_lurker_offers_its_own_list_with_evasion`; un-ignore `a_feyform_shifters_rogue_talent_is_the_rogues_selection`,
`a_sylvan_tricksters_fey_tricks_are_the_games_64`, `an_underground_chemist_takes_a_discovery_at_10_only`.
**Proof:** rows −6, option lists −40 (fey tricks 4, underground chemist 17, feyform 18, master of all 1); refused 0;
heirs 0. **Estimate:** 20 min.

### 306g. Slayer talents at 2, 6 and 10 (Lane D; user fact 276)

**Edits:** `selections/slayer/slayer-talents.ron` keeps `SlayerTalentSelection2` 04430ad24988baa4daa0bcd4f1c7d118; new
`slayer/slayer-talents-6` (`SlayerTalentSelection6` 43d1b15873e926848be2abf0ea3ad9a8) and `slayer/slayer-talents-10`
(913b9cf25c9536949b43a2651b7ffb66), copies with `option_levels: ["inquisitor", "slayer"]`. Rows: `classes/slayer.ron` 6, 8
→ -6; 10–20 → -10; `archetypes/slayer/imitator.ron` 6 → -6, 12, 18, 20 → -10; arcane-enforcer 14, spawn-slayer 10, 20,
executioner 10, 14 → -10; stygian-slayer 8 → -6, 12, 16, 20 → -10; `archetypes/inquisitor/sanctified-slayer.ron` 8 → -6,
16, 17, 20 → -10 (`SanctifiedSlayerArchetype` `AddFeatures`). New options (own words): `slayer/slayer-talents/vanguard-tactician`
(`VanguardTacticianExtraUseTalent` c67db0b6ede802b43a6cf00136b1a5e6) and `slayer/slayer-talents/painful-strike`
(`ExecutionerPainfulStrike` 052c82521039f7b4e978207eb533ead5, rows granting `slayer/executioner/painful-strike`).
`user-facts.txt`: `option lists / * / selection slayer/slayer-talents -> applied 276` and the same for -6, -10 and
`slayer/extra-slayer-talent`. Run `--only` those four. `known.txt`: delete `option lists / * / selection slayer/slayer-talents
-> 276` and `… slayer/extra-slayer-talent -> 293`.
**Tests:** `the_slayers_three_lists_are_the_games` (37/38/39 offers, Uncanny Dodge and Improved in each);
un-ignore `a_slayers_talent_list_at_10_is_the_games_39`, `a_slayer_may_take_the_first_combat_style_feat_at_6`,
`a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style`, `a_sanctified_slayer_8_may_take_the_first_combat_style_feat`,
`a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style`, `the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents`
(may stay on S4: Uncanny Dodge still needs Eldritch Scoundrel until 308 writes its `Any`).
**Proof:** rows −62 (Selection6 17, Selection10 45); option lists −20; **refused: 38 open Builds** (Selection2 25,
6 26, 10 26, Extra 6; weapon-focus 136 picks, skill-focus 20, intimidating-prowess 5). Gold: none (the Enforcer's
picks are in the lists; its gap 173 stays until 308). **Estimate:** 20 min.

### 306h. Rage powers (Lane D)

**Edits:** new `skald/rage-power` (`SkaldRagePowerSelection` 2476514e31791394fa140f1a07941c96: `SkaldProgression` 3, 6, 9,
12, 15, 18) and `barbarian/instinctual-warrior/rage-power` (`InstinctualWarriorRagePowerSelection`
609f0e5336084442a0dafa3abd4d31c5: `AddFeatures` 4, 8, 10, 12, 16, 18, 20). Rows: `classes/skald.ron` 3–18, demon-dancer 12,
inciter 6, 12, 18 → skald copy; `archetypes/barbarian/instinctual-warrior.ron` 4–20 → IW copy. Run `--only
barbarian/rage-power,skald/rage-power,barbarian/instinctual-warrior/rage-power,barbarian/extra-rage-power,
barbarian/pack-rager/bonus-feats,skald/battle-scion/rage-power-or-teamwork-feat`: the five `*-instinctual-warrior`
options move from `barbarian/rage-power` to the IW copy (keys unchanged); Extra Rage Power gets
`ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a's 38. `known.txt`: delete 269's `barbarian/rage-power`,
`pack-rager/bonus-feats` and 286's battle-scion option-list lines.
**Tests:** `the_skald_and_the_instinctual_warrior_have_their_own_rage_powers`; un-ignore
`the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38` (264),
`the_battle_scions_selection_offers_21_rage_powers` (286).
**Proof:** rows −33 (Skald 26, IW 7); option lists −35; refused 0 (no open Build holds an IW power outside an IW).
**Estimate:** 20 min.

### 306i. Exploits, arcana, mercies (Lane D)

**Edits:** new `wizard/exploiter-wizard/arcanist-exploits` (`ExploiterExploitSelection` 2ba8a0040e0149e9ae9bfcb01a8ff01d:
`ExploiterWizardArchetype` 1, 5, 9, 13, 17); rows of `archetypes/wizard/exploiter-wizard.ron` → it. Run `--only
arcanist/arcanist-exploits,wizard/exploiter-wizard/arcanist-exploits,arcanist/extra-arcanist-exploit,magus/extra-arcana,
paladin/extra-mercy`: the six greater exploits join as shared options (they stay `arcanist/greater-exploits`'s own: its
blueprint has no list, 260); Extra Arcana gets `ExtraArcanaSelection` 00727883edf145e2a6bce9ad176ecfd8's 15.
`known.txt`: 282's `arcanist/arcanist-exploits`, 293's `arcanist/extra-arcanist-exploit`.
**Tests:** `the_exploiters_and_extra_arcanist_exploits_lists_are_the_games`; un-ignore
`an_exploiter_1_takes_extra_arcanist_exploit_with_no_exploit_picked` (293), `a_hexcrafters_extra_arcana_gives_a_magus_arcana` (264).
**Proof:** rows −5; option lists −12; **refused: 4 open Builds** (a hexcrafter's Extra Arcana holding a witch hex: 8
picks). **Estimate:** 15 min.

### 306j. Hexes (Lane D)

**Edits:** 11 copies `options/shaman/hex/<x>.ron` of `witch/hex/{ameliorating, aura-of-purity, evil-eye, fortune,
healing-hex, iceplant, misfortune, nails, protective-luck, slumber, ward}` with `ShamanHex*Feature` GUIDs (from
`all-lists-mapped.tsv`, `ShamanHexSelection` 4223fe18c75d4d14787af196a04e14e7) and `aliases: ["witch/hex/<x>"]`, owned by
`shaman/hex`. New option `witch/hex/channel` (`HexChannelerChannelFeature` b40316f05d4772e4894688e6743602bd, own
words). New `magus/hexcrafter/hex` (`HexcrafterMagusHexMagusSelection` a18b8c3d6251d8641a8094e5c2a7bc78: Hexcrafter 4),
row 4 of `archetypes/magus/hexcrafter.ron`. New feat `feats/extra-hex-shaman.ron` (`ExtraShamanHexSelection`
d0b4c8245d504b8c9c6d3fccc1f8c5b6) with `selection: "shaman/extra-hex"` and that selection (a copy of `witch/extra-hex`,
its blueprint); no alias (an old `extra-hex` pick stays valid as a feat). Run `--only shaman/hex,witch/hex,witch/extra-hex,
shaman/extra-hex,magus/hexcrafter/hex,magus/hexcrafter/hex-arcana,rogue/sylvan-trickster/fey-tricks,shaman/wildland-shaman/spirit`.
`known.txt`: 293's `shaman/hex`, the pool line `ExtraShamanHexSelection`; 278's `witch/hex`, `wildland-shaman/spirit`.
**Tests:** `the_shamans_hexes_are_their_own_and_open_old_links_through_aliases`; un-ignore
`a_battle_master_hex_names_no_weapon_and_8_and_16_each_choose_one` if it now passes.
**Proof:** rows −1; option lists −32; **heirs: 22 open Builds** (78 picks of the 11 keys at `shaman/hex`); **refused: 3**
(a shaman's Extra Hex holding a shaman hex). **Estimate:** 20 min.

### 306k. The small splits: Aeon gaze, Elemental Witch, Nine-Tailed Heir, discoveries, angel (Lane D)

**Edits:** `aeon/enforcing-gaze` keeps `AeonGazeFirstSelection` 7e468a5266235454e9dcefee315ee6d5 (ranks 3–4); new
`aeon/enforcing-gaze-2` (7d498e034d18ca94baea19d7edee7403, ranks 5–7) and `-3` (0bec49f67ecb49a5826fcfefb9408a35,
ranks 8–10) in `mythic/paths/aeon.ron` (`AeonProgression` levels 1–2, 3–5, 6–8 = ranks +2); options keep their keys.
`archetypes/witch/elemental-witch.ron` row 1: the feature `witch/elemental-witch/elemental-ray` becomes the new
selection `witch/elemental-witch/energy` (`ElementaWitchEnergySelection` 9ec1bf2087874390a9d9be62884a0027) with 4 new
options (`ElementalWitch{Acid,Cold,Electricity,Fire}RayFeature`, own words), each option's row 1 granting that feature
so the sheet keeps it; row 11's grant stays (S5). `sorcerer/nine-tailed-heir/bloodline`
(`NineTailedHeirBloodlineSelection` 7c813fb495d74246918a690ba86f9c86) on its row 1. New feat `extra-discovery-vivisectionist`
(`ExtraVivsectionistDiscoverySelection` 10287e7b8cee479e82ea88bd6d2d4dae) with its selection. New option
`angel/greater-sword-of-heaven/anti-demon` (`AngelSwordAntiDemonFeature`). Run `--only` those plus
`alchemist/extra-discovery`. `known.txt`: 293's `aeon/enforcing-gaze`, 266's `angel/greater-sword-of-heaven`, the mythic
ranks kind line 293:21, pool `ExtraVivsectionistDiscoverySelection` (263).
**Tests:** `the_aeons_three_gaze_lists_are_the_games`, `the_elemental_witch_picks_one_of_four_rays`.
**Proof:** rows −8 (Aeon 6, Elemental Witch 1, Nine-Tailed 1); option lists −19; refused 3 (Extra Discovery: 2
Vivisectionist, 1 Underground Chemist); Aeon 0 refused (every pick of 146 Builds sits in its tier's list).
**Estimate:** 20 min.

### 306l. Bonus feat lists I: magus, Arcane Rider, warpriest (Lane D)

**Edits:** new `magus/bonus-feats` (`MagusFeatSelection` 66befe7b24c42dd458952e3c47c93563: `MagusProgression` 5, 11, 17),
`magus/arcane-rider/bonus-feats` (`ArcaneRiderFeatSelection` 8e627812dc034b9db12fa396fdc9ec75: 5, 11, 17),
`warpriest/bonus-feats` (`WarpriestFeatSelection` 303fd456ddb14437946e344bad9a893b: 3–18), each with
`ignores_prerequisites: false` and `feats` copied from `combat-feats`. Rows: `classes/magus.ron` 5, 11, 17 and the rows
naming `feats` in armored-battlemage, bladebound, eldritch-scion, hexcrafter, spell-dancer, sword-saint; arcane-rider 5,
11, 17; `classes/warpriest.ron` and champion-of-the-faith 12, cult-leader 6, 18, feral-champion 9, 15, mantis-zealot 18,
shieldbearer 12. Slot ids stay (`FeatSource::Class`). Run `--only` the three. `known.txt`: 283's
`progression rows / a bonus pick of another group / archetype magus/hexcrafter`.
**Tests:** `the_magus_arcane_rider_and_warpriest_bonus_feats_are_their_own`.
**Proof:** rows −25 (283); **refused: 20 open Builds** (magus 12, Arcane Rider 2, warpriest 6). **Estimate:** 15 min.

### 306m. Bonus feat lists II: cavalier (Lane D)

**Edits:** rename `mounted-combat-feats` → `cavalier/bonus-feats` (file and every row naming it; `CavalierBonusFeatSelection`
dd17090d14958ef48ba601688b611970) and name it at `classes/cavalier.ron` 6, 12, 18, disciple-of-the-pike 6, 12,
fearsome-leader 6, 12, 18; new `cavalier/tactician-feats` (`CavalierTacticianFeatSelection` 7bc55b5e381358c45b42153b8b2603a6:
1, 9, 17) at the cavalier's `teamwork-feats` rows and beast-rider, disciple-of-the-pike, standard-bearer,
cavalier-of-the-paw 1; new `cavalier/gendarme/bonus-feats` (`GendarmeFeatSelection` ef1cd58e0b7fc7f45baedb09407a1cd1: 1, 3, 6,
9, 12, 15, 18, 20). Run `--only` the three. `known.txt`: 293's `mounted-combat-feats`; 273's cavalier-of-the-paw
bonus-pick line becomes 309's (its row 8 is `OrderOfThePawCanineFerocitySelection`, a row question).
**Tests:** `the_cavaliers_three_feat_lists_are_the_games`; un-ignore `the_standard_bearer_holds_the_cavaliers_own_banner_from_1` if it passes.
**Proof:** rows −31 (273); option lists −141; **refused: 9 open Builds** (bonus 4, tactician 5, Gendarme 2).
**Estimate:** 15 min.

### 306n. Bonus feat lists III: sorcerer, Seekers, skald (Lane D; user fact 281)

**Edits:** new `sorcerer/bonus-feat` (`SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3: `SorcererProgression` 1; Overwhelming
Mage removes it) at `classes/sorcerer.ron` 1 and the sorcerer archetype rows naming `magic-feats` at 1 (crossblooded,
empyreal, geomancer, nine-tailed-heir, sage, sylvan); `sorcerer/seeker-sorcerer/bonus-feats` and
`oracle/seeker/bonus-feats` (`SeekerFeatSelection` c6b609279cc3174478624182ac1ad812: both `SeekerArchetype`s' `AddFeatures`
5, 10, 15) at their 5, 10, 15 rows; `skald/bonus-feat` (`SkaldFeatSelection` 0a1999535b4f77b4d89f689a385e5ec9:
`SkaldProgression` 1) at skald 1 and battle-scion, court-poet, herald-of-the-horn, inciter 1. `user-facts.txt`:
`option lists / * / selection sorcerer/bonus-feat -> applied 281` and both seekers'. Run `--only` the four.
**Tests:** un-ignore `every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21`, `the_seekers_feats_at_5_10_and_15_are_its_own_26` (281).
**Proof:** no finding today (the rows pair feat pools: 306ab shows them); **refused: 33 open Builds** (sorcerer 21,
skald 10, Seeker 2). Gold: Jack's level-1 Spell Penetration is in the 21: no change. **Estimate:** 15 min.

### 306o. Bonus feat lists IV: prestige and the rest (Lane D)

**Edits:** new `eldritch-knight/bonus-feats` (`EldritchKnightFeatSelection` da03141df23f3fe45b0c7c323a8e5a0e: 1, 5, 9),
`student-of-war/bonus-feats` (`StudentOfWarCombatFeatSelection` 78fffe8e5d5bc574a9fd5efbbb364a03: 2, 5, 8),
`paladin/tortured-crusader/bonus-feats` (`SelfSuffisientBonusFeats` db3c26e791cb4b70a82203b28563da02: 2, 8, 12, 16),
`slayer/vanguard/teamwork-feat` (`TeamworkFeat` **90b882830b3988446ae681c6596460cc**, `VanguardArchetype` 2; not 301's
d87e2f6a…), `shifter/griffonheart-shifter/bonus-feats` (`GriffonheartShifterFeatSelection` f1add10c87fa4563ad5f71779eecde19,
granted by `ShifterAspectGriffonProgression`: its rows 4, 9, 14 in the planner), `cleric/domain/war/greater-feats`
(`WarDomainGreaterFeatSelection` 79c6421dbdb028c4fa0c31b8eea95f16, `WarDomainProgression`: the War domain option's row).
Run `--only` the six.
**Tests:** `the_prestige_and_archetype_bonus_feats_are_their_own`. **Proof:** no finding today; refused 0.
**Estimate:** 15 min.

### 306p. The remaining lists of existing selections (Lane D)

**Edits:** new options (own words): `stalwart-defender/defensive-powers/smash` (`SmashDefensivePower`),
`monk/zen-archer/way-of-the-bow/shortbow` (`ZenArcherWayOfTheBowShortbowFocus`), `loremaster/secret/trickster-combat-feat`
(`LoremasterTricksterCombatFeatSelection`). Run `--only` monk scaled-fist 1/6/10, student-of-stone 6, zen-archer 1/6/10 and
way-of-the-bow; hunter/bonus-feats-3; ranger/bonus-feats-2b/6b/10b; fighter/titan-fighter/bonus-feats-2/6/10;
cleric/demonbane-priest/bonus-feats; cleric/impossible-domain; magus/sword-saint/chosen-weapon;
magus/hexcrafter/hex-arcana; warpriest/disenchanter/bonus-feats-3/9/12 (`ignores_prerequisites` only: the lists are G3's);
half-elf/adaptability (`ignores_prerequisites` only); ranger/combat-style-feats-10; gold-dragon/dragon-feats (after 306q/r);
stalwart-defender/defensive-powers; loremaster/secret. `user-facts.txt`: `option lists / * / selection
cleric/crusader/bonus-feats-{1,10,20} -> held` (ticket 21 R5; they agree today). `known.txt`: the option-list lines of
279, 275 (hunter, ranger), 285 (titan), 267 (demonbane, impossible-domain), 268 (disenchanter ignores), 283
(sword-saint, hex-arcana), 291 (adaptability ignores), 287 (stalwart, loremaster), 293's `ranger/combat-style-feats-10`
and `gold-dragon/dragon-feats`.
**Tests:** un-ignore `a_hunters_teamwork_slot_at_3_offers_no_precise_shot`, `the_stormwalkers_lists_are_the_rangers_archery_lists`
(275), `the_titans_massive_attacks_lists_hold_the_style_extras` (285), `a_sword_saint_chooses_a_melee_weapon` (283),
`a_gold_dragons_dragon_feats_leave_out_stat_focus`, `the_slots_ignore_prerequisites`,
`the_slots_offer_every_teamwork_feat_but_spell_chain` (266), `a_drunken_master_is_offered_neither_diamond_body_nor_diamond_soul` (279).
**Proof:** option lists −78; refused: count from the census (feat lists of archetypes; none found in the corpus scan).
**Blockers:** 306q, 306r (the Dragon Feats list holds their new feats). **Estimate:** 20 min.

### 306q. Feats holding several blueprints I (Lane D)

**Edits:** `feats/advanced-weapon-training.ron` keeps `AdvancedWeaponTraining1` 3aa4cbdd4af5ba54888b0dc7f07f80c4; new
`advanced-weapon-training-2/3/4` (70a139f0a4c6c534eaa34feea0d08622, ee9ab0117ca06b84f9c66469f4428c61,
0b55d725ded1ae549bb858fba1d84114), each `aliases: ["advanced-weapon-training"]` (a second pick of the old key migrates
to -2, the only heir that holds). `feats/magical-tail.ron` → `MagicalTail` febb8fe9a2d142fb80c1be6b0b539d9d (decision 9).
`mythic/abilities/abundant-smite-smite-chaos.ron` → `AbundantSmiteChaos` 4cdc155e26204491ba4d193646cb4443 (its key and
summary say Chaos) and new `abundant-smite` (`AbundantSmite` 7e5b63faeca24474db0bfd019167dda4), no alias (decision 5).
New feats `extra-rage-instinctual-warrior` (`ExtraRageInstinctualWarrior` dcc338110bab48ddbdc22e82cef7a6db),
`extra-performance-skald` (15c78493a0ce471cb279e6363d702cc8), `extra-performance-martyr` (639ce2b85a13404caa618f36748d7fa5),
each `aliases: ["extra-rage"]` / `["extra-performance"]`. Names and summaries in our own words; `feats` lists that name
the old keys gain the new ones through the 306s run. `known.txt`: the pool lines `AdvancedWeaponTraining2–4`,
`MagicalTail`, `magical-tail (MagicalTail1)`, `AbundantSmiteChaos`; `mapping / ranks differ / feat magical-tail`; 263's
pool line keeps only its other members until 306r.
**Tests:** `advanced_weapon_training_is_four_feats_each_after_the_last`; un-ignore
`advanced_weapon_training_needs_the_fighters_weapon_training` (263) if it passes.
**Proof:** option lists −8 (pool), mapping −1; heirs: corpus picks of a repeated `advanced-weapon-training` (count in the
census). **Estimate:** 20 min.

### 306r. Feats holding several blueprints II (Lane D)

**Edits:** new `extra-channel-oracle`, `-shaman`, `-hospitaler`, `-hex-channeler`, `-empyreal-sorcerer` (`ExtraChannelOracle`
53da6bf487997e947960bd6c02be9adf, `…Shaman` 4cc73e43ab814bc48ade2e291729b359, `…Hospitaler` 8d4f82fdb4d09b247ae8cd1ae7ce02de,
`…HexChanneler` 88f426f13de236642b9585efcb82f1d8, `…EmpyrealSorcerer` 347e8d794c9598e4abed70adda868ccd), each
`aliases: ["extra-channel"]`, `ranks` from the blueprint; new `bleeding-critical` (`BleedingCriticalFeature`), `spell-bane`
(`SpellBane`). Own words. `known.txt`: the pool's ExtraChannel ×5 lines; 263's pool line.
**Tests:** `extra_channel_is_six_feats_one_per_channel`. **Proof:** option lists −7. **Estimate:** 15 min.

### 306s. The general and group lists (Lane D)

**Edits:** run `--only feats,combat-feats,magic-feats,teamwork-feats,mythic/extra-mythic-ability`. `known.txt`: 293's
lines for the five. **Tests:** un-ignore `extra_feat_offers_the_general_slots_feats_but_the_tricksters_five`,
`extra_mythic_ability_leaves_out_emergency_help_and_any_means_necessary` (265).
**Proof:** option lists −66 (feats 17, combat-feats 33, magic-feats 11, teamwork 1, Extra Mythic Ability 4); **refused:
70 open Builds** (fighter's list 26: iron-will, throw-anything, great-fortitude, diehard, toughness…; wizard's 27:
dispel-synergy, brew-potions, arcane-strike…; the inquisitor's teamwork 17: spell-chain). **Blockers:** l–o, q, r.
**Estimate:** 10 min.

### 306t. Kineticist elements (Lane D)

**Edits:** `kineticist/expanded-element` (`SecondatyElementalFocusSelection` 4204bc10b3d5db440b1f52f0c375848b, level 7)
gets 4 own options `kineticist/expanded-element/{air,earth,fire,water}` (`SecondaryElement*`: 659c39542b728c04b83e969c834782a9
…), copies of `kineticist/element/<x>` with `aliases: ["kineticist/element/<x>"]`; drops `once_per_option`. New
`kineticist/expanded-element-15` (`ThirdElementalFocusSelection` e2c1718828fc843479f18ab4d75ded86, `KineticistProgression`
15) with `kineticist/expanded-element-15/{air,…}` (`ThirdElementAir` 651570c873e22b84f893f146ce2de502,
`ThirdElementEarth` c43d9c2d23e56fb428a4eb60da9ba1cb, `ThirdElementFire` 56e2fc3abed8f2247a621ac37e75f303,
`ThirdElementWater` 86eff374d040404438ad97fedd7218bc), same aliases, each `Requires(condition: Not(Has(Option(
"kineticist/expanded-element/<x>"))))`. Rows 15 of `classes/kineticist.ron`, kinetic-knight, overwhelming-soul → the
-15 copy. `kineticist/kinetic-knight/element` gets 4 copies (`KineticKnightElementalFocus*`). The four
`kineticist/element/<x>` lose `ranks: 2`. Run `--only` the selections and `kineticist/blood-kineticist/element`.
`known.txt`: 270's three element lines; 293's four `mapping / ranks differ / … kineticist/element/*`.
**Tests:** `the_second_and_third_elements_are_their_own_options`; un-ignore
`a_kinetic_knight_is_refused_the_infusions_that_need_the_elemental_focus_selection` (270) if it passes.
**Proof:** rows −8; option lists −19; mapping −4; **heirs: 99 open Builds** (expanded element 88 with 154 picks,
Kinetic Knight 11). **Estimate:** 20 min.

### 306u. Kineticist wild-talent bonus feats (Lane D)

**Edits:** the game's 14 `WildTalentBonusFeat<Element><n>` selections (`grep '^WildTalentBonusFeat'`), offered by
`WildTalentSelection` 5c883ae0cd6d7d5448b7a420f51f8459 and `ExtraWildTalentSelection` 0f73730cf1b44ee882671e55d5f6e471,
against the planner's 4 `kineticist/element/<x>/wild-talent-feats` and the planner options
`planner/options/kineticist/wild-talent-feats/<x>.ron`: 14 option entries `kineticist/wild-talents/bonus-feat-<element>-<n>`
(own words), each nesting its game selection as a feats selection `kineticist/element/<x>/wild-talent-feats-<n>`; the
planner-made 4 options get `aliases` heirs (each old key aliased by its element's first). Run `--only` the 14, the 4, 
`kineticist/wild-talents`, `kineticist/extra-wild-talent`. `known.txt`: 293's four wild-talent-feats and extra-wild-talent
lines, 271's wild-talents.
**Tests:** un-ignore `the_first_bonus_feat_talent_offers_its_elements_entry_feats_and_the_talent_repeats_down_the_chain` (271).
**Proof:** option lists −46; heirs: picks of the 4 planner options (census). **Estimate:** 20 min (the biggest; split
into the lists and the options if it runs over).

### 306v. Oracle mysteries (Lane D)

**Edits:** 9 copies each for `oracle/divine-herbalist/mystery` (`DivineHerbalistMysterySelection`
c11ff5dbd8518c941849b3112d4d6b68: `DivineHerbalist<X>MysteryFeature`) and `oracle/enlightened-philosopher/mystery`
(9d5fdd3b4a6cd4f40beddbc72b2c07a0), keys `oracle/divine-herbalist/mystery/<x>`, aliases `oracle/mystery/<x>`; new
mythic abilities `second-mystery-herbalist` and `second-mystery-enlightened-philosopher` (`SecondMysteryHerbalist`,
`SecondMysteryEnlightenedPhilosopher`; own words; no alias). The Hermit (`HermitArchetype` 8f205adfab73b3c4c8c93d251a3cef02)
has no planner archetype: `SecondMysteryHermit`'s pool line and `HermitMysterySelection` go to a gap ticket. Run `--only` those and `oracle/lone-strider/mystery`, `oracle/wind-whisperer/mystery`.
`known.txt`: 293's two mystery lines; 272's lone-strider, wind-whisperer; 265's pool `SecondMystery*` line; the
"293 (from 244 by way of 298)" revelation prerequisite lines move to 308 (their blueprints are now mapped: S4 writes them).
**Tests:** `the_herbalist_and_philosopher_mysteries_are_their_own_and_open_old_links`.
**Proof:** option lists −50 (the Hermit's 1 re-homed); **heirs: 10 open Builds** (Herbalist 7, Philosopher 3). **Estimate:** 20 min.

### 306w. Curses (Lane D)

**Edits:** 9 options `oracle/second-curse/<x>` (`<X>CurseNoProgression`, offered by `SecondCurseSelection`
cc6fda79e8c340b88c84689414a9abbe and `MysteryGiftFeatureCurseSelection` 4e7265c0ae1345db90d3375f4ced94cc), owned by
`oracle/second-curse`, shared by `arcanist/magic-deceiver/ways-of-the-deceiver/living-deity/curse`; 9 options
`oracle/beneficial-curse/<x>` (`<X>CurseNoPenaltyProgression`, `BeneficialCurse` 2dda67424ee8e0b4d83ef01a73ca6bff);
aliases the curse keys they copy. Run `--only` the three. `known.txt`: 272's second-curse, 282's living-deity curse,
293's beneficial-curse lines and `prerequisites / not in game / mythic ability beneficial-curse`.
**Proof:** option lists −54, prerequisites −1; **heirs: 52 open Builds** (second curse 35, Beneficial Curse 14, Living
Deity 3; Beneficial Curse is a mythic ability's selection: its option picks migrate). **Estimate:** 15 min.

### 306x. Second bloodlines (Lane D)

**Edits:** 21 options `sorcerer/crossblooded/second-bloodline/<x>` (`CrossbloodedSecondaryBloodline*Progression`,
`CrossbloodedSecondaryBloodlineSelection` 60c99d78a70e0b44f87ba01d02d909a6) and 21 `bloodrager/second-bloodline/<x>`
(`Bloodrager*SecondBloodline`, `SecondBloodragerBloodline` b7f62628915bdb14d8888c25da3fac56), copies with aliases.
`feat_lists_only` stays on the selections. Run `--only` the two. `known.txt`: 281's crossblooded, 293's second-bloodline.
**Proof:** option lists −85; heirs: 5 open Builds (Crossblooded 4, Bloodrager 1). **Estimate:** 20 min (42 files: a
small Rust copy tool in the scratchpad, CLAUDE.md "One-off tools").

### 306y. Other bloodlines (Lane D)

**Edits:** 10 copies `fighter/dragonheir-scion/bloodline/<x>` (`Dragonheir*Progression`, `DragonheirDragonSelection`
729411185291d704696e58316420fe38); run `--only` it, `bloodrager/hag-riven/bloodline`, `bloodrager/bloodline/arcane/feats`,
`bloodrager/bloodline/elemental/feats`. `known.txt`: 285's dragonheir, 281's hag-riven and the two feat lines; the
Empyreal/Sage/Sylvan bloodline lines move to G5 (decision 10).
**Proof:** option lists −37; heirs: 3 open Builds. **Estimate:** 15 min.

### 306z. Blessings and alignments (Lane D)

**Edits:** 9 new blessing options (`ArtificeBlessingFeature` 04e5d412a7b94f809a28607618b71619, Charm 516bc13e0e76a834bb3a4c3e3d01c0cf,
Community 36fcd6ca7e279874d9197f38501f0e93, Glory d082055b649b44c1880442a98f384556, Knowledge 1585a2eec1984d1496ce03021cfa9738,
Plant 4cd28bbb761f490fa418d471383e38c7, Rune f76713cc1768432cab0d6352f030519d, War 2097edd687ff4cdeb33872c048599fc1, Scalykind
606ca13281f4404a9c224aa7ce8a802e) shaped as `options/warpriest/blessings/fire.ron` (its `domain` field set where the
planner has the domain), own words, no rows (S3/S5 bring the minor and major powers). New `warpriest/second-blessing`
(`SecondBlessingSelection` b7ce4a67287cda746a59b31c042305cf) as the second `warpriest/blessings` of row 1 in
`classes/warpriest.ron`, shieldbearer, cult-leader (Champion of the Faith and Feral Champion remove it). 4 options
`warpriest/champion-of-the-faith/chosen-alignment/<x>` (`ChampionOfTheFaith<X>Feature`), aliases the old keys. Run
`--only` the three. `known.txt`: 293's blessings, 267's chosen-alignment, the kind line 293:20 (re-homing the Order of the
Paw row to 174).
**Tests:** un-ignore `a_warpriest_of_torag_is_offered_five_blessings`, `a_true_neutral_champion_of_the_faith_takes_chaos` if they pass.
**Proof:** rows −5 (+1 re-homed); option lists −17; heirs: 5 open Builds (alignment). **Estimate:** 20 min.

---

### 306aa. `needs_slot_of` goes (Lane R, P2)

**Goal:** an Extra feat opens its own list's slot whenever it is taken (the game); 302's stand-in goes.
**Edits:** `libs/rules/src/game_data.rs`: delete `FeatureSelection::needs_slot_of` (1173) and the stand-in branch of
`has_list` docs; `libs/rules/src/engine/walk.rs:254`, `walk/level.rs:580-589` (the gate); `check.rs:1325`; test world
(`world/mod.rs:136-205`, `nested.rs`); the 9 `selections/*/extra-*.ron` files lose the field; `tools/game-check/src/areas/options.rs:21`
(the "standing for the first selection had" skip goes); `once_per_option` field if 306t left no user. Tests that name it:
`libs/rules/tests/engine/nested_selections.rs`, `libs/seeds/tests/game_data/{nested_selections,extra_picks,support}.rs`.
**Tests:** `an_extra_feat_opens_its_own_list_for_any_class`, and un-ignore 264's
`the_extra_feats_that_give_a_pick_name_the_games_selections`,
`the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter`,
`extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection`, `extra_rogue_talent_offers_the_rogue_talents_and_evasion`,
293's `a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot`.
**Proof:** the census attributes newly open Extra slots (offers only; no refusal). **Blockers:** d, f, g, h, i, j, k merged.
**Shares:** `extra-*.ron` (after Lane D's slices that run on them). **Estimate:** 20 min.

### 306ab. The rows compare feat selections by GUID (Lane W, P2)

**Edits:** `tools/game-check/src/areas/rows.rs`: a grant of a selection of feats is compared by GUID, not by group;
`areas/mod.rs` `FEAT_POOLS` (line 63) is deleted with its doc. Fixture `areas.rs` expectation updated.
**Proof:** 0 new findings on the data after 306l–o (any new one is a missed split: fix, never list). **Blockers:** l, m,
n, o merged. **Estimate:** 15 min.

### 306ac. `is_selection` against the blueprint's type (Lane W, P2)

**Edits:** `src/areas/mapping.rs`: an option whose blueprint is `BlueprintFeatureSelection`/`BlueprintParametrizedFeature`
is a selection on the planner side (`is_selection`, a nested selection, a chosen feat or a parameter); a new kind
"selection or not" otherwise. Today 33 options with such a blueprint carry no hand `is_selection` (the slice must read
them through the derived rule, not the hand field: expect 0 findings; any left gets a line under G5). Witch Wandering
Hex's `plain_feature` stays 259's. **Estimate:** 15 min.

### 306ad. S2's proof (main session; heavy runs alone)

- Record `FORM_PROOF_OUT` on the base before 306e merges (with `FORM_PROOF_OFFERS=1`); after 306z/aa, `compare` with
  `FORM_PROOF_LINKS_OUT`; commit `tools/corpus/tests/links/306-base.tsv` and `306-refused.txt` (306c's test).
- The census classifier (a Rust crate in the scratchpad) ties every line to a change-log row; 0 unattributed.
- `--write option-lists` writes 0 files; two runs on two copies byte-identical; the 100-field hand check; the long
  self-checks; gold builds (expected unchanged); the ignored tests above; `known.txt` holds no `-> 293` option-list,
  split or S2 line; the remaining `-> 293` lines (304's bloodline-feat and ranger-style prerequisites → 308; the Lich's
  headers → its gap) re-homed.

## 2. Facts

**Today's findings** (`run.txt`, main 69600a9f): 4,556 in 212 groups. S2's: option lists 1,080 (293's 418, 294's 114,
281's 120, 255's 48, 282's 48, 276 35, 285 35, 289 31, 272 30, 286 29, 277 28, 270 19, 271 18, 266 17, 275 16, 267 14,
265 13, 279 10, 278 9, 263 6, 269 6, others 1–4); rows "a selection the game keeps separate" 193 (293), "a bonus pick of
another group" 56 (273's 31, 283's 25), mythic 6 (293); mapping ranks 6 (293). **S2 removes ≈1,029:** option lists 767,
rows 255 less 1 (pets → 174), mapping 5, prerequisites 1. Left: 294 114, 289 31, G3 58, G5 73, G6 14, Lich 6, Elemental
Specialist 8, Primalist's ranks 1 (S5).

**Per split** (rows findings; entries that inherit the class row need no row edit): SlayerTalentSelection10 45,
BardTalentSelection 38, SkaldTalentSelection 28, SkaldRagePowerSelection 26, CavalierBonusFeatSelection 23,
MagusFeatSelection 22, SlayerTalentSelection6 17, ThirdElementalFocusSelection 8, GendarmeFeatSelection 8, IW 7,
DarkLurker 6, SecondBlessing 5, Exploiter 5, Provocateur 4, ArcaneRider 3, AeonGaze 2nd/3rd 3+3, NineTailedHeir,
Hexcrafter, ElementalWitch, OrderOfThePaw 1 each.

**Removed offers per list** (game list resolved to keys, against main's): Bard 14, Skald 17, Inciter 14, Dark Lurker 0
(+Evasion), rogue 0; slayer 2/6/10 8/7/6 and Extra Slayer 4 (each +Uncanny Dodge, Improved, the Vanguard talent and
Painful Strike); rage 5 (RagePowerSelection, SkaldRagePowerSelection; IW and Extra 0); fighter's 11, wizard's 10,
inquisitor's teamwork 1, general 1 (Magical Tail, a remap); the class feat lists (MagusFeatSelection 181,
WarpriestFeatSelection 155, SorcererBonusFeat 21, SeekerFeatSelection 26, SkaldFeatSelection 26, …) against the general
or group list each row named.

**Open corpus Builds** (2,000 open of 2,161 in main's record after 310): **refused 194** (talents 66: bard 14, skald 12,
Inciter 2, slayer 38; feat lists 132: fighter 26, wizard 27, inquisitor teamwork 17, sorcerer 21, magus 12, skald 10,
cavalier 9, warpriest 6, Arcane Rider 2, Seeker 2; Extra feats 10 more: Extra Arcana 4, Extra Discovery 3, Extra Hex 3);
**moved to an alias heir ≈196** (expanded element 88, second curse 35, shaman hexes 22, Beneficial Curse 14, Kinetic
Knight 11, Herbalist 7, alignment 5, Crossblooded 4, Philosopher 3, Living Deity 3, Dragonheir 3, Bloodrager 1; AWT and
the wild-talent options counted by the census). The design's "62 open" was talents only, before 310.

**The user's facts S2 touches:** 276 (Uncanny Dodge and Improved in the slayer lists: 306g, `applied 276`; their
prerequisites are 308's), 281 (SorcererBonusFeat 21, SeekerFeatSelection 26: 306n, `applied 281`), ticket 21 R5 (the
Crusader's bonus feats ignore prerequisites: `held`, agree today), 249 (Weapon Focus per weapon: kept; the slayer lists'
removal of it is the game's list, not 249's fact). **Gold builds:** no change expected (every Enforcer and Jack pick is
in its game list; the Enforcer's Uncanny Dodge gap 173 stays until 308).

## 3. Open questions

**Q1. Who makes the splits: the writer or the slice?** The ticket says `--write` writes the splits and their rows
(design §1.2: new files from a template). Evidence: every new pickable entry needs our own words (user, Q1), each split
needs aliases chosen by slot, and rows aren't the writer's until S5. Options: A (recommended) hand edits per family plus
the writer's membership run (`--only`), proven per slice by the check and the census; B a split table in the writer
(selection, grantor, levels, copies) that makes files and rows, about four more writer slices, names still by hand.

**Q2. Commit the base links?** 306c's test needs S2's base links (`306-base.tsv`, ~2,000 lines, ~160 KB) and refused
list in the repo. A (recommended): commit them; B: keep them in the scratchpad and run the three-way count only in the
census (the test then covers the 5 bench links).

**Q3. Gap tickets before S2's data slices.** Re-homed lines need ticket names (G3, G5, G6, Lich, Elemental Specialist).
A (recommended): file G1–G9 (313+) before 306e; B: name them by slug and rename at the queue's close.

**Gold builds:** none expected to change; the census confirms before the merge.
