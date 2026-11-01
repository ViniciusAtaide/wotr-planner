# Mythic path rules

Type: task
Status: resolved
Blocked by: 25
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) R1, R3 and S3).** Background: [Hand-fix the extracted mythic data](20-handfix-mythic.md) follow-ups and ticket 21's follow-up 5 and 7.

- **MR 1–2 on choosing the path:** choosing a path at MR 3 grants its MR 1–2 rows (12 rows, 30 features today). They show at their ranks and are held from the MR 3 ascension on; a pick made before MR 3 can't rely on them.
- **Kept ranks:** `PathSwitch.kept_through_rank: u8` replaces `kept_ranks`; drop `path_switch_kept_rank` for a column on `path_switch`.
- **Partial keeps:** from the guide's prose, Gold Dragon, Swarm That Walks and Devil keep the general picks (mythic abilities and feats) of MR 1–7 but drop the old path's MR 3–7 path powers. Model what the guide states; decide the shape with `kept_through_rank`.
- **Previous path:** `PathSwitch.from: BTreeSet<MythicPathKey>` (empty = any). Devil only from Aeon or Azata.
- **Ability bonuses from features:** a new fact, a feature that grants ability-score bonuses (Legend's +4 to every score; Gold Dragon's Perfect Body, Mind and Soul; check the guide for others). They count toward ability prerequisites from the point they're held: they're the character's own scores, and only gear stays excluded.
- **Legend keeps Trickster feats:** a Trickster who switches to Legend keeps the feats Perception II unlocked, and picks made with them stay valid.
- **`MythicRank(n)` prerequisite:** a `prerequisite_node` kind `mythic_rank` (`amount` = rank). Use it for the Gold Dragon breaths and replace the Aeon gazes' `CharacterLevel` stand-ins.
- **Azata's All-skilled** (user, 2026-11-01): a selection option may grant several feats; All-skilled grants every weapon, armour and shield proficiency feat.
- **Gold Dragon's Dragon Feats** ignore prerequisites: allow a bonus list on that path rank's slot (narrow `BonusListOutsideAClass` to exactly what the guide says).

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): data by one-off script, docs and `GLOSSARY.md` (**Path switch**, **Mythic track**) updated, the map's Out of scope line on gear stays true. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests for each bullet.

**Scope addition (user, 2026-11-01, answering ticket 24's open question 3):** a selection option may grant several feats, not just one. Azata's All-skilled ("proficient with all types of weapons and armour") grants every weapon, armour and shield proficiency feat.

## Progress log

- 2026-11-01: baseline in the session scratchpad `t26/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t26/`): `old-src/` is `git archive HEAD` (dfdfc7a); `extract-old/` is the HEAD extractor's output (identical to `t25/extract-new/`). Guide read offline (`t24/pages/*.txt`). Shape decided: `PathSwitch { from, kept_through_rank, path_kept_through_rank, dropped_features, dropped_selections, features, further_ascensions, level_cap }`; `Feature.ability_bonuses: Vec<AbilityBonus>` (`Fixed`, `Every`, `Highest`, `AtLeast`); `Prerequisite::MythicRank`; `FeatList.every_feat` for Dragon Feats (a list of every feat would be a derived value); `SelectionOption.feats: BTreeSet<FeatKey>` replaces `feat`. Next: rules types, tests first.
- 2026-11-01: `rules` done. Types as above, plus `Issue::NotFromPath`, `RankStatus::PartlyDropped`, `RankSheet.dropped_slots`, `CharacterState.base_ability_scores` (ability_scores now include feature bonuses), `with_bonuses`, `FeatList::contains`, `PathSwitch::open_from`; check: `MisplacedBonusList` (was `BonusListOutsideAClass`: a path rank may name its own bonus list), `EveryFeatListWithFeats`, `SlotBeforePathChoice`, `SwitchFromASwitchPath`. Engine: grants carry their source (rank, path or general, selection, slot); feats are held per source (a dropped pick keeps a feat a class also grants). Tests first: new `path_rules.rs` (20); the world gains Aeon, Devil (from Aeon), Swarm, a fuller Gold Dragon and Legend, Trickster MR 1-2 rows, an All-armour trick and Stat Focus. Red run: 33 failures; three passed at once as expected (both Perception II tests: picks are checked in Build order, so the engine already kept them; the multi-feat option once `feats` was a set). `cargo test -p rules` 155 + 1 pass, clippy clean. Next: `data` crate and migration. (Correction: the `MythicRank` tests also passed at once: its one-line check went in with the type.)
- 2026-11-01: `data` done and the data rewritten. Migration: `prerequisite_node` kind `mythic_rank`; `path_switch.kept_through_rank`/`path_kept_through_rank` (FK to `mythic_ascension`) replace `path_switch_kept_rank`; new `path_switch_from`, `path_switch_drop` (feature or selection, exactly one), `path_switch_feature`, `feature_ability_bonus`, `selection_option_feat` (replaces `selection_option.feat_key`); `feat_list.every_feat`. 33 tables. Seed and loader for each; tests: files (new shapes read and defaulted), postgres (round trip incl. `MythicRank`, the new CHECKs/FKs/unique), `game_data.rs` on the real files (7 new). `t26/rewrite.py` ran over `game-data/` (same as the dry run in `t26/dry/`; log `t26/game-data-log.json`) and the fixture (switch shape, `feats`); fixture extended by hand (Legend's bonus feature and switch grant; Gold Dragon from Trickster with drops, every ability-bonus kind, Dragon Feats list, a `MythicRank(9)` breath; Combat Trick grants two feats). Dev DB reset (only `wotr-planner-postgres`, 5434), migrated, seeded (same counts), `cargo sqlx prepare`. Next: extractor, proof.
- 2026-11-01: extractor done: `prose::path_switch` (the new shape, quoting the guide), `prose::path_rules` (Legend's features, the ability-bonus table, All-skilled's feats, the Dragon Feats list), `prose::dragon_feats_slots`, `Cond::MythicRank` resolves to `Prerequisite::MythicRank` (`Catalog::rank_levels` gone), `ron_out` writes the switch, `ability_bonuses`, `every_feat` and option `feats`; the five report items it now encodes are no longer reported. Tests first: new `mythic_paths.rs` (4; red run 4 failures). `cargo test -p extract` 61 pass. A fresh extraction (`t26/extract-new/`) is byte-identical to `rewrite.py` applied to the HEAD extraction (`t26/dry/extract-old/`); its report went 82 → 77 items, exactly those five.
- 2026-11-01: proof (`t26/proof-{old,new}`, `t26/compare.py`) passes; docs (`game-data/README.md`, `GLOSSARY.md`, `game-data/EXTRACTION_REPORT.md` 77 → 70); final DB reset and reseed, `cargo sqlx prepare`; every Verification check passes. Done.

## Answer

Done 2026-11-01, uncommitted. The mythic path rules are data and engine now: choosing a path at MR 3 grants its MR 1–2 rows; a Path switch keeps the general picks and the old path's own grants up to separate ranks, can be limited to previous paths, drops named grants and grants features; features carry ability bonuses that count toward prerequisites; `MythicRank(n)` is a prerequisite; Gold Dragon's Dragon Feats are a bonus list of every feat on its own rank; and (scope addition) a selection option grants a set of feats, so All-skilled grants every weapon, armour and shield proficiency.

### Rust (`rules`)

- **MR 1–2 on choosing the path.** At the MR 3 ascension the chosen path's rank rows below MR 3 are held, each as its own rank's grant (so a switch that keeps MR 1–2 keeps them), and `RankSheet.path` of MR 1 and 2 shows the path. A pick before the MR 3 ascension (level 9's own slots included) can't rely on them. `GameData::check` adds `SlotBeforePathChoice(rank)`: those rows may grant features only (the real data's 12 rows hold 30 features and no slot).
- **`PathSwitch`** is `{ from, kept_through_rank, path_kept_through_rank, dropped_features, dropped_selections, features, further_ascensions, level_cap }`. `rank_after_switch()` is `kept_through_rank + 1`; `open_from(path)`.
  - Every ascension grant now records its source (rank, general pick or the path's own, the selection and slot picked). A switch releases a general pick past `kept_through_rank`, a path grant past `path_kept_through_rank`, and a path grant named in `dropped_features`/`dropped_selections` whatever its rank.
  - `RankStatus::PartlyDropped` (one cut-off past the rank, or a named drop at it) joins `Dropped` (both); `RankSheet.dropped_slots` lists the slots whose picks a switch took.
  - A switch not open from the Build's path is `Issue::NotFromPath { from }` at `Choice::PathSwitch` and doesn't happen; `switch_options(build)` gives the same status. `SwitchFromASwitchPath` is a data check.
  - `features` are held from the switch on (Legend's).
  - Feats are held per source, so a switch that drops an option's feats keeps one a class also grants.
- **Ability bonuses.** `Feature.ability_bonuses: Vec<AbilityBonus>` with `Fixed { ability, amount }`, `Every { amount }`, `Highest { amount }`, `AtLeast { score }`. `CharacterState.base_ability_scores` is point buy, race and increases; `ability_scores` is those plus the bonuses of every feature held (`with_bonuses`), recomputed when a bonus feature is held or released and after an increase. Prerequisites and skill points use `ability_scores` (the user's "the character's own scores"); see open question 2.
- **Legend keeps the Trickster's Perception II feats**: no engine change was needed. Picks are checked in Build order against the state before their slot, so a feat taken by level 17 (whose slots come before its ascensions) stays valid after the switch drops Perception II, and one taken after it fails, as the guide says ("if you subsequently respec, you will not be able to take them again"). Two tests pin it.
- **`Prerequisite::MythicRank(n)`**: the character's Mythic rank at least `n` (3 after a Legend switch).
- **Dragon Feats.** `FeatList.every_feat` (with `feats` empty) is a list of every feat in the data, `FeatList::contains` answers for both kinds; a list holding all 261 feats would have been a derived value in the database. `BonusListOutsideAClass` is narrowed and renamed `MisplacedBonusList`: a race may never name a bonus list, a path's rank only its own path's, class and archetype rows any. `EveryFeatListWithFeats` refuses members on an every-feat list.
- **Option feats** (scope addition): `SelectionOption.feats: BTreeSet<FeatKey>` replaces `feat`; picking the option holds them all.
- Tests first, in the test world (new `path_rules.rs`, 20 tests, plus check tests): the world gains Aeon (gazes, one needing MR 5), Devil (from Aeon only, dropping the gaze), Swarm (a fixed bonus), a fuller Gold Dragon (Perfect Body/Mind/Soul, Dragon Feats, an MR 9 breath) and Legend (+4), Trickster MR 1–2 rows, an All-armour trick granting two feats, and Stat Focus behind Perception II. Red run: 33 failures; five tests passed at once (both Perception II tests, as explained above; the two `MythicRank` tests, whose one-line check went in with the type; the multi-feat pick, once `feats` was a set).

### Schema (the one migration)

- `prerequisite_node` kind `mythic_rank` (`amount` = rank).
- `path_switch.kept_through_rank` and `path_kept_through_rank` (both FK to `mythic_ascension (rank)`) replace the `path_switch_kept_rank` table. New `path_switch_from (path_key, from_path_key, CHECK not itself)`, `path_switch_drop (path_key, feature_key | selection_key, exactly one)`, `path_switch_feature (path_key, feature_key)`.
- `feature_ability_bonus (feature_key, kind fixed|every|highest|at_least, ability set iff fixed, amount, UNIQUE (feature, kind, ability))`.
- `feat_list.every_feat`.
- `selection_option_feat (option_key, feat_key)` replaces `selection_option.feat_key`.
- 33 tables (29 − 1 + 5). The seed and loader cover each; tests: the fixture round trip carries every new shape (the fixture gained Legend's bonus feature and switch grant, a Gold Dragon open from the Trickster with drops, every bonus kind, an every-feat list and a `MythicRank(9)` breath, and Combat Trick granting two feats), the new CHECKs, FKs and unique key reject what they should, `files.rs` reads and defaults the new fields, and `data/tests/game_data.rs` checks the real files (7 new: the 12 rows/30 features, a gaze at MR 5, Legend's +4 and kept rows, Devil's previous paths and drops, Dragon Feats and the MR 9 breath, All-skilled).

### Data rewrite

Script: `t26/rewrite.py` in the session scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t26/`), line based, quoting the guide for each rule; it changes only the lines it logs (`t26/game-data-log.json`). The extractor was never run over `game-data/`.

| Change | Real data | Fixture |
| --- | --- | --- |
| switch rules in the new shape | 4 | 2 (then extended by hand) |
| option `feat` → `feats` | 6 rogue talents | 1 (then two feats by hand) |
| All-skilled's feats | 8 | 0 |
| Legend's features (+4 on `legend/legend`) | 2 | 1 by hand |
| features with ability bonuses | 6 (Legend, Perfect Body/Mind/Soul, Eye of the Swarm, The Swarm) | 3 by hand |
| Dragon Feats: 3 `General` slots → the Gold Dragon's every-feat list | 1 row, 1 list | by hand |
| `MythicRank` | 9 gazes (`CharacterLevel(14)` → 5, `(17)` → 8), 4 breaths gain `MythicRank(9)` | 1 by hand |

The switch rules, from the guide's prose:

| Path | `from` | general picks kept | old path's grants kept | dropped by name | grants | ascends | cap |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Legend | any | MR 1–2 | MR 1–2 | | Legend (+4 every score), Corruption Resistance | no (stays MR 3) | 40 |
| Gold Dragon | any | MR 1–7 | MR 1–2 | | | yes | |
| Swarm That Walks | any | MR 1–7 | MR 1–2 | | | yes | |
| Devil | Aeon, Azata | MR 1–7 | MR 1–7 | Extra Gaze, the Enforcing Gaze picks, Dragon Companion | | yes | |

- Legend: "you lose ... All rank 3 - 8 mythic abilities and feats ... your rank is stuck at 3". The old path's MR 3–7 powers go too, as ticket 20 encoded and the Perception II note implies.
- Gold Dragon and Swarm: "You lose any powers you gained at ranks 3 - 7, retaining only your mythic abilities and feats."
- Devil: "If you were following the Azata path, you lose your dragon companion. If you were following the Aeon path, you will lose your gaze abilities." The guide states no other loss, so the rest of MR 1–7 is kept (its "Playing a Devil" weighs Hell's Decree against "continued use of Enforcing Gaze").
- Ability bonuses: Legend `Every(4)`; Perfect Body `AtLeast(14)` + `Highest(4)`; Perfect Mind `AtLeast(18)` + `Highest(2)`; Perfect Soul `Highest(2)` ("for a total of +8"); Eye of the Swarm Str/Dex +4 and The Swarm Int/Wis/Cha +4 ("4 + your swarm size", which starts at 0 and grows in play, so +4 is what's certain). Left out as not the character's own scores: Gold Dragon Form's size bonuses (an assumed form), the Demon's major aspects ("when raging"), Master Shapeshifter ("whenever you are the target of a polymorph effect"), and bonuses to summons and companions.
- All-skilled: simple, martial and exotic weapons, light, medium and heavy armour, shield and tower shield. The three barding feats are left out: "Your companion becomes proficient".

### Proof

`t26/proof-old/` (against `git archive HEAD`) and `t26/proof-new/` build the same source, which walks a 20-level fighter on each of the 6 paths, with no switch and with each of the 4, filling every general and path pick with the first open option, and dumps each level's scores, feats, features, options, mythic picks, rank and path, plus the picks and problems (630 lines each). `t26/compare.py`: **all 504 lines without a switch or before it match**, 240 identically and 264 except for exactly the chosen path's MR 1–2 row features from level 9 (and All-skilled's eight feats on the Azata, whose first superpower is All-skilled); the no-switch builds make **the same picks with the same problems**, so the gazes' `MythicRank(5)`/`(8)` open exactly where `CharacterLevel(14)`/`(17)` did. From the switch on, the differences are the new rules: MR 3–7 path grants gone under Gold Dragon, Swarm and Legend (MR 1–2 rows kept), Legend at 14 in every score from 10, Gold Dragon at 24/18/18/18/18/18 at 17 and 26 at 19, Swarm's +4s at 17 and 19, the breath picks shifted so the MR 9 ones come at MR 9, and Devil refused (`NotFromPath`) from Angel, Demon, Lich and Trickster.

### Extractor

- `prose::path_switch` builds the new shape (same quotes as the script); `prose::path_rules` adds Legend's features, the ability bonuses, All-skilled's feats and the Dragon Feats list; `prose::dragon_feats_slots` is the MR 9 cell; "Mythic Rank: N" resolves to `MythicRank(N)`; `ron_out` writes the new fields, omitting the empty ones.
- Tests first: `mythic_paths.rs` (4, including a write-and-read-back of the new shapes).
- A fresh extraction (`t26/extract-new/`) is **byte-identical** to `rewrite.py` applied to the HEAD extractor's output (`t26/dry/extract-old/`). Its report lost exactly the five items now encoded (Dragon Feats, the Gold Dragon, Devil and Swarm switch notes, Legend's "no rank table"): 82 → 77, 0 check problems.

### Docs

- `game-data/README.md`: switch rules, ability bonuses, every-feat lists and a path's own bonus list, option `feats`, `MythicRank`, MR 1–2 on choosing the path.
- `GLOSSARY.md`: Path switch and Mythic track updated; new Mythic path and Ability bonus; Mythic rank, Feat list, Bonus-list slot and Selection option updated.
- `game-data/EXTRACTION_REPORT.md`: 7 Mythic items fixed (Dragon Feats, the six prose items), the breaths item narrowed to the story conditions: 77 → 70.

### Verification

- `cargo test --workspace`: pass (rules 155 + unit 1, extract 61, data files 17 + game_data 17 + postgres 15).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (9 query files replaced, 17 new).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths (unchanged). Rows: `feature` 1747 → 1749, `feat_list` 43 → 44, `path_switch` 4 (kept 2/2, 7/2, 7/2, 7/7), `path_switch_from` 2, `path_switch_drop` 3, `path_switch_feature` 2, `feature_ability_bonus` 11, `selection_option_feat` 14, `mythic_rank` nodes 13.
- The v1 link vector needed no regeneration: the `Build` type didn't change.
- Proof: matches (above).

### Open questions (for the user)

1. **Which score is "highest" when two tie?** Perfect Body, Mind and Soul raise "your highest ability score". The engine takes the highest before the floors ("at least 14/18") are applied, and gives a tie to the first in Str, Dex, Con, Int, Wis, Cha order. The guide doesn't settle ties; a test pins the provisional rule.
2. **Skill points.** Feature bonuses count in `ability_scores`, which skill points also read, so a Legend's +4 Intelligence adds 2 skill points a level from 18. The ticket names ability prerequisites only; say if skill points should use the base scores instead.
3. **Perfect Mind's other floors**: "Your base attack bonus is at least 15" and "your rank in any skill is at least your mythic rank" also feed prerequisites (BAB, skill ranks) and aren't modelled.
4. **Destiny Beyond Birth** (a mythic feat: "You no longer suffer your racial [penalty] to an ability score") changes the character's own scores too, but it isn't a feature; modelling it needs a bonus on mythic feats (or a "no racial penalty" kind).
5. **Exotic Weapon Proficiency** is one parameterised feat ("one of the following weapons"); All-skilled grants the key, which ticket 27 (parameterised feats) may need to read as every weapon.
6. Still out: the Azata's dragon and the Swarm's companions are dropped as the guide says, but other companions aren't modelled; the Gold Dragon breaths' story conditions stay out (decision R5).

