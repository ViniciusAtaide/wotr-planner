# X: Expected refusals, the detailed design (T8, with T0)

(Explorer, read-only, main at 790eff76, 2026-11-18. Fits `overall.md` §1 X, §4, §5 T0/T8. Findings: `6-expected-refusals.md`.)

## 1. X's interface

`libs/rules/src/refusals.rs`, `#[cfg(any(test, feature = "testing"))] pub mod refusals;` beside `agreement` (`lib.rs:3-4`). Engine-public only, as `agreement` is. `seeds` and `corpus` have the feature only as dev-dependencies (`libs/seeds/Cargo.toml:18`, `tools/corpus/Cargo.toml:14`), so X is called from their tests, never their libraries.

```rust
/// The rows of every rule file, parsed and checked against the data.
pub struct Rules { rules: Vec<Rule> }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(u32);

pub struct Rule {
    pub at: RowAt,              // file and line, for every message
    pub ticket: String,         // the file name's ticket, `308e` of `308e-alchemist-discoveries.txt`
    pub entry: Entry,
    pub place: Place,
    pub shape: Shape,
    pub when: Option<Fact>,     // the Build holds it: `Fact::Class`, `Fact::Archetype`
    pub then: Option<Then>,
    pub cite: Citation,
}
pub struct RowAt { pub file: String, pub line: usize }

pub enum Entry { Pick(Fact) /* Feat, Option, MythicAbility, MythicFeat */, Chosen(Fact) /* Archetype, Class */, Any }
pub enum Place { Slot(SlotKind), Archetype, Class, AnySlot }
pub enum Shape { NotAllowed, NotProficient, AlreadyHeld, MaxTimes, NoOptionLeft(FeatureSelectionKey), Unmet(Need) }
pub enum Need { Leaf(LeafShape), Not(Box<Need>), All(Vec<Need>), Any(Vec<Need>), AtLeast(u8, Vec<Need>) }
pub enum LeafShape { Fact(Fact, Option<u8>) /* `class alchemist 12` */, Other { kind: String, value: Option<String> } /* `None` = `*` */ }
pub enum Then { ItsSelection }  // the pick in the slot of the selection the entry grants, same level or rank
pub enum Citation { Blueprint { name: String, guid: BlueprintGuid }, Code { file: String, method: String, commit: String } }
pub struct RowError { pub at: RowAt, pub why: String }

impl Rules {
    /// `files`: (file name, text). Every error, not the first.
    pub fn parse(files: &[(String, String)], data: &GameData) -> Result<Rules, Vec<RowError>>;
    /// The Build with every excused pick taken out; never panics.
    pub fn excuse(&self, engine: &Engine, build: &Build) -> Excused;
    pub fn rule(&self, id: RuleId) -> &Rule;
    /// The rules `tally` never saw hit, in file and line order.
    pub fn unhit(&self, tally: &Tally) -> Vec<&Rule>;
}
pub struct Excused { pub build: Build, pub hits: Vec<Hit>, pub unexcused: Vec<Failure> }
impl Excused { pub fn is_expected(&self) -> bool /* unexcused.is_empty() */; }
pub struct Hit { pub rule: RuleId, pub choice: Choice, pub key: String, pub how: How }
pub enum How { Matched, Then, Follows }   // Follows: an Unmet that names only excused picks' entries
#[derive(Default)] pub struct Tally(BTreeMap<RuleId, usize>);
impl Tally { pub fn add(&mut self, excused: &Excused); }
```

**Row text.** One row per line, `#` comments, blank lines ignored. Each ticket has one file, `libs/game-data/refusals/<ticket>-<slug>.txt`. It sits beside `notes/`, outside the kinds' folders, which the loader ignores (`libs/seeds/src/files.rs:18-19`). `seeds::refusal_files() -> Vec<(String, String)>` reads the folder in name order (plain IO, no `rules::testing` types) and gives X the texts.

```
<entry> @ <place> : <shape> [when <fact>] [then its-selection] ; <citation>
entry  := feat K | option K | mythic-ability K | mythic-feat K | archetype K | class K | any
place  := feat SEL | selection SEL | mythic-ability | mythic-abilities SEL | mythic-feat | archetype | class | any
shape  := not-allowed | not-proficient | already-held | max-times | no-option-left SEL | unmet NEED
NEED   := LEAF | not(NEED) | all(NEED, …) | any(NEED, …) | at-least N(NEED, …)
LEAF   := feat K | feature K | option K | selection K | mythic-ability K | mythic-feat K | archetype K | race K
        | deity K | class K N | <leaf kind, kebab-case> (VALUE | *)
cite   := blueprint NAME GUID | code FILE METHOD COMMIT
```

`place` follows `SlotKind` (`engine.rs:437-448`). `entry` and the Fact leaves follow `Fact` (`component.rs:283`).

**Matching.** Each `Failure` (`Engine::failures`, `walk.rs:50`) is matched against the rows. Its place is the sheet's slot kind for `Choice::Slot`, or `Archetype`/`Class` for those choices. The `Engine::evaluate` sheet is read on the impossible Build, as `support.rs:411, 469` do today.

A row matches a Failure when all of these hold:
- the entry is the pick's Fact, or `any`;
- the place is equal;
- `when` is held;
- the shape is **structurally equal** to the issue. For `Unmet`, the comparison runs on P's normal form (T1): children compared as multisets, each leaf's `has` ignored, `*` matching any value of that leaf kind only. `MetLater` never occurs in `failures` (overall.md:3).

**Excuse algorithm.**
1. `failures` and `evaluate` once. Classify each Failure as matched by 0, 1 or ≥2 rules. ≥2 is unexcused, with "matched by rows a and b".
2. Strip the matched picks. For `then its-selection`, also strip the pick in the slot of the selection the entry grants (`Feat::selection`, mythic ability's `selection`, ticket 302), at the same level or rank. This is C's `MythicSelection { rank, index: 0 }`.
3. Walk the stripped Build again. A Failure that is an `Unmet` whose every condition is `Fact::leaf()` (`component.rs:329`) of an excused pick's entry is `How::Follows`, recorded under that pick's rule. Cases: the Mythic Toughness resting on a refused Toughness (306s); `iron-will-mythic` on `iron-will` (306l). Strip those picks and repeat until nothing changes, at most once per pick.
4. Whatever still fails is `unexcused`.

The cost is about three walks per refused Build, and only for the roughly 460 refused links.

**Invariants.**
- I1: `is_expected()` ⇒ `engine.is_possible(&excused.build)`. The last walk's failures are, by construction, the unexcused ones.
- I2: no pick leaves the Build without a `Hit` naming it.
- I3: hits come in walk order, so the same Build always gives the same answer.

**Mechanical checks in `parse`** (each a `RowError`):
- Syntax.
- Every key exists in the data (entry, selections, Fact leaves, `when`).
- **Every row is cited.** The GUID is 32 hex characters; a code citation names a file, a method and a 7-hex WrathDecompiled commit.
- **The GUID is the refusal's.** It must be one of: the entry's own blueprint (`GameData::blueprints`, `check.rs:802`, by `DataAt`), the blueprint of the place's selection, or the blueprint of a Fact the shape or `when` names. 308f's "Increased Damage Reduction for an Inciter" cites ProvocateurArchetype, which the shape's `archetype skald/inciter` names.
- The file's ticket is a ticket file (`.scratch/wotr-planner/issues/<ticket>-*.md`; seeds gives the names).
- No two rows share an entry, place and shape. An `any` row may not overlap a keyed row of the same place and shape.
- **An `any` entry** needs either a shape that names its subject (`not-proficient`, whose issue names the weapon) or a `when`, and only a code citation or the `when` Fact's blueprint.

**Every row hit** is the corpus test of §5, run over all corpora at once.

**Shape is exact** comes from matching itself: a row never matches a subset of an issue. Today's D does (`support.rs:426` `all(alternatives.contains)`), and so would any key-alone list.

**Today's odd helpers as rows.**
- A (251): `any @ any : not-proficient ; code BlueprintParametrizedFeature.cs CanSelect 6ce4a6b`.
- A (293l): `any @ any : unmet base-attack-bonus * when archetype druid/feyspeaker ; blueprint <Feyspeaker archetype> <guid>`. The archetype's blueprint names BABLow 0538…; the builder looks up its GUID.
- C (308d):
  - `mythic-ability over-infused-blasts @ mythic-ability : unmet class kineticist 1 ; blueprint DoubleKineticistInfusions 7410cc35ae6aa9d4a82f1d23c081a1ac`
  - `mythic-ability impossible-domain @ mythic-ability : unmet any(selection cleric/domain, selection inquisitor/domain, selection inquisitor/sacred-huntsmaster/domain, selection arcanist/magic-deceiver/domain, selection hunter/divine-hunter/domain) then its-selection ; blueprint ExtraDomain 213a8480d22206b45acbfa0619ca5aaf`
- D (308e): `option alchemist/discovery/nauseating-flesh @ selection alchemist/discovery : unmet any(class alchemist 12, all(archetype fighter/mutation-warrior, class fighter 12), all(archetype rogue/underground-chemist, class rogue 12)) when class alchemist ; blueprint NauseatingFlesh de112af9600e0e446af0c7ef0707e1c8`. If the two links' failing trees differ, that is two rows.

## 2. Behind the seam: what goes

| Today | Goes to |
|---|---|
| `support.rs:342-388` `without_mythic_abilities_the_game_refuses_since_308d` (C) | `308d-*.txt`, 2 rows |
| `support.rs:390-433` `without_discoveries_the_game_refuses_since_308e` (D) | `308e-*.txt`, 1–2 rows; its returned slots become `Excused::hits` |
| `support.rs:435-463` `without_unproficient_picks` (A) | `251-*.txt` and `293l-*.txt`, 1 row each |
| `support.rs:465-500` `without_picks_the_fighters_list_lacks`, `THE_FIGHTERS_LIST_LACKS` (B) | `306s-*.txt`, one row per feat that is hit, citing FighterFeatSelection 41c8486641f7d6d4283ca9dae4147a9f (the place's own) |
| Ordered chains `first_ascension.rs:200-207`, `ranks_and_pools.rs:585-590` | one `rules.excuse(&engine, &build)`; the order no longer matters |
| `links.rs:252-268` `SKALD_BONUS_FEAT`/`_LACKS` | `306n-*.txt`: `feat <k> @ feat skald/bonus-feat : not-allowed ; blueprint SkaldFeatSelection 0a1999535b4f77b4d89f689a385e5ec9` |
| `links.rs:274-287` `BATTLE_PROWESS_LACKS` (key alone) | `306h-*.txt`: rows only for the 5 hit (Clear Mind, Come and Get Me, Daemon Totem, Celestial Totem Greater, Powerful Stance; 306h:30) `@ selection skald/battle-scion/rage-power-or-teamwork-feat`, citing BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6, which is that selection's own blueprint (`selections/skald/battle-scion/rage-power-or-teamwork-feat.ron:3`). The other 7 get no row (a row that is never hit fails). |
| `links.rs:322` `starts_with("rogue/rogue-talents/")` (prefix alone, uncited) | `306e-*.txt`: one row per talent hit. Slot kinds: `selection rogue/skald-talents` (d2a8fde8985691045b90e1ec57e3cc57), `selection bard/rogue-talents` (BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f), `selection skald/inciter/rogue-talents` (aa88cb5263a54791b8d1e471bc192ad4); each cites its own place's blueprint |
| `links.rs:289-335` `a_battle_scion_link_from_the_s2_base_still_opens` | deleted: the S2 test covers it with exact rows |
| `306-refused.txt` (466 names, 459 distinct, 22 blank lines, 7 duplicates) and `fixture-refused.txt` | deleted. `Stage.refused` goes; `problems` (`links.rs:119-162`) takes `&Rules` |
| `ranks_and_pools.rs:606` `(8, 5)` | kept, now counted as Builds where `!hits.is_empty()` and `is_expected()` |

**The 459 Builds.** A Build is refused-as-expected iff `excuse(...).is_expected()`. Each block of `306-refused.txt` has its blame (`git blame`) and its ticket Answer, and becomes rows in that ticket's file:

| Ticket | Lines | What its rows cover |
|---|---|---|
| 293l | 2 | Feyspeaker BAB |
| 306e | 30 | talents, Extra Rogue Talent's Unmet for a bard |
| 306g | 39 | slayer lists: Weapon Focus, Skill Focus, Intimidating Prowess |
| 306aa | 6 | Extra Rogue Talent, Extra Hex, Extra Discovery Unmets |
| 306h | 5 | Battle Prowess |
| 306i | 4 | hexes in Hex Arcana |
| 306j | 14 | |
| 306k | 7 | |
| 306l | 21 | 7 feats in 3 lists, plus `iron-will-mythic` as `Follows` |
| 306m | 10 | |
| 306n | 34 | |
| 306s | 73 | fighter, wizard, teamwork and Life-Bonding Friendship lists, plus Mythic Toughness as `Follows` |
| 308c | 77 | Spell Chain needs Spell Penetration; Greater Disarm Int 13 |
| 308d | 49 | |
| 308e | 48 | |
| 308f | 69 | |

Each row records the refusal as it is today. A Build listed by a 306 slice whose pick 308 later refused for another reason gets the row of the ticket whose change gives today's issue: `not-allowed` goes to the list ticket, `unmet` to the prerequisites ticket.

I expect about 150 rows in all; the builder generates the exact set.

**What the builder looks up** (`game-check show <guid>`, main checkout; I ran no cargo):
- the names of d2a8fde8… and aa88cb52…;
- the Feyspeaker archetype's GUID;
- each 306g, 306l, 306m, 306j, 306k and 306i list's blueprint name (the GUIDs are the `blueprint:` fields of their selection files);
- each 308 row's own blueprint.

The game sources don't settle the slot kind of a refused Battle Prowess or rogue-talent pick. The builder reads it from each Build's `failures` and sheet. The table above is the expected answer.

## 3. T0: the form proof's reasons record

- **Record.** `FORM_PROOF_REASONS=1` with `record` writes `reasons.bin` and `reasons-texts.txt`. It implies the offers pass and shares it (`offers_pass`, `form_proof.rs:939`), so it also writes `offers.bin`.
  - Per Build: `u32` name length, the name, `u32` count, then per non-ok option `u64 fnv(slot, pick)` (the same hash as `offers.bin`, `:590`) and a `u32` index into the interned text table.
  - Then a `u32` failure count and, per `Engine::failures` entry, `u64 fnv(choice Debug)` and a `u32` index.
  - The text is one function, `reason_text(engine, &Issue) -> String`: today `engine.issue_text(issue)`, with `"BreaksLater"` for its `None`. T5 swaps in R's Refusal text there, and that is the point: T5's compare lists exactly what R changed.
  - It is written and read a Build at a time (`OffersWriter`/`OffersReader` pattern, `:682-752`). The writer keeps only the distinct-text map.
- **Size and time.** About 12 bytes per non-ok option: an estimated 120–180 MB for the roughly 18M options. The text table should be under 30 MB.
  - Time: about 47 s for offers plus an estimated 50%. **To measure**, the median of alternating runs on a quiet machine, written in the file's header like the offers' "47 s against 6 s" (`:32`).
- **Compare.** When `FORM_PROOF_BEFORE` holds `reasons.bin`, it compares options that are non-ok on both sides; kind changes stay the offers' job.
  - Each changed pair is grouped by its **edit**: the before and now texts with their common prefix and suffix removed. `Needs 1 more of: Kinetic Healer` → `Needs Kinetic Healer` groups under `"1 more of: " -> ""` with every other such text.
  - It prints `reasons: N compared, M changed in K edits`. Then each edit, most frequent first, the first 40 (`FORM_PROOF_REASONS_SHOWN`): count, Builds, and one full example (Build, slot, key, both texts).
  - Failures are compared by choice the same way, plus `failures: N appeared, N gone`.
  - It fails on any change unless `FORM_PROOF_REASONS_ACCEPT=<file>` lists that edit (`before-mid<TAB>now-mid`, `#` comments). An accept line no change uses also fails, as a stale line does.
  - A record without `reasons.bin` says "reasons not recorded".
- **With `FORM_PROOF_CENSUS`.** Reason changes split as statuses do (`StatusSplit`, `:851`): on options the log names, in Builds whose links are refused, and the rest. Only the rest fails.
- **With `FORM_PROOF_KIND_MAP`.** No interaction: the slot id carries no kind.

## 4. Tickets

**T0. The form proof records reasons.**
- **Problem:** the form proof records status kinds, never texts (overall.md:3), so a form-only ticket cannot prove that no reason moved, and a wording ticket cannot list what it changed.
- **Done when:**
  - record and compare on one tree report `0 changed`;
  - the pieces tests pass;
  - the header documents the variable, its size and its measured time;
  - CLAUDE.md's form-proof bullet names `FORM_PROOF_REASONS=1` for a ticket that changes or must keep reason texts.
- **Files:** `tools/corpus/tests/form_proof.rs`, `CLAUDE.md`.
- **Checks:** `cargo test -p corpus`, `cargo clippy -p corpus --tests -- -D warnings`, fmt, and the same-tree run twice (release, `--ignored`).
- Model: Sonnet to build.

**T8. One list of expected refusals.**
- **Problem:** helpers A–D, three lists in `links.rs` and a Build-name file excuse refused picks. Two of the lists are by key alone (CODING_STANDARDS:32), and a listed Build hides any later refusal in it (`links.rs:149`). They cost fix rounds in 306s, 308e and 325.
- **Done when:**
  - Before the first edit, in the lane: each S2 base link's fate and each refused Build's failures are recorded.
  - After: every corpus, bench, S2 and old link's fate is unchanged: the same counts of same, heir and refused.
  - The Builds X calls expected are exactly the 459 that `306-refused.txt` listed.
  - `first_ascension`'s and `ranks_and_pools`' stripped Builds are unchanged.
  - Every rule is hit; every row parses and is cited.
  - Helpers A–D, the three lists, `306-refused.txt` and `fixture-refused.txt` are gone.
  - CODING_STANDARDS:32's model becomes a row of `libs/game-data/refusals/`.
  - The README Layout and CLAUDE.md's Game data section name the folder: a data ticket adds rows in its own file.
- **Blocked by:** T1 (shapes are written in normal form). T2 is not needed: `Fact::leaf` already exists.
- **Files:** `libs/rules/src/{lib.rs, refusals.rs}`, `libs/rules/tests/engine/{main.rs, refusals.rs}`, `libs/seeds/src/lib.rs`, `libs/game-data/refusals/*.txt` (about 17 files), `libs/game-data/README.md`, `libs/seeds/tests/game_data/{support.rs, first_ascension.rs, ranks_and_pools.rs, expected_refusals.rs, main.rs}`, `tools/corpus/tests/links.rs` and `links/`, `CODING_STANDARDS.md`, `CLAUDE.md`.
- **Checks:** `cargo test -p rules -p seeds -p corpus`, `check-data` (the new folder is ignored), the rules wasm build, fmt, `scripts/test-modules.sh`. No form proof is needed: no data or engine behaviour moves.
- Model: Opus to build.

## 5. Tests

**New, in rules** (`tests/engine/refusals.rs`, on the test world):
- `a_row_reads_its_entry_place_shape_when_then_and_citation`
- `a_row_without_a_citation_or_with_a_malformed_guid_is_refused`
- `a_row_naming_a_key_the_data_lacks_is_refused`
- `two_rows_with_one_entry_place_and_shape_are_refused`
- `an_any_entry_needs_a_when_or_a_shape_naming_its_subject`
- `a_pick_refused_in_another_slot_kind_is_unexcused` (325's key-alone case)
- `an_unmet_of_another_shape_is_unexcused` (a subset doesn't match)
- `a_wildcard_matches_any_value_of_its_leaf_kind_only`
- `then_its_selection_takes_the_granted_slots_pick`
- `a_pick_needing_only_an_excused_pick_follows_it`
- `an_expected_build_is_possible` (I1)
- `excuse_never_removes_a_pick_no_hit_names` (I2)

**New, in seeds** (`expected_refusals.rs`):
- `every_rule_file_parses_and_cites_a_blueprint_of_its_refusal` (the GUID check on real data)
- `every_rule_file_is_a_tickets`

**New, in corpus** (`links.rs`):
- `every_expected_refusal_rule_is_hit`: the union over the S2 base, `links-before-the-splits.tsv`, first_ascension's old links (moved to `old-mythic-links.tsv`) and bench.
- `a_planted_refusal_is_excused_only_by_its_rule`: real data with a feat taken out of a list. With its row, the Build is expected; with no row, it is unexcused; with a row in another slot kind, it is unexcused and the row is unhit.

**Surviving:**
- `every_corpus_and_bench_link_opens_with_the_same_picks`.
- `a_planted_rename_opens_through_its_alias_and_is_refused_without_one`.
- The fixture test keeps its heir half. A link that doesn't decode is never an expected refusal: the rename needs an alias.

**Moved:**
- `every_s2_base_link_…_is_listed` becomes `…_is_an_expected_refusal`.
- `first_ascension::every_older_link_…` runs one `excuse`, and its exact Nauseating Flesh pair is read from `hits` of the 308e rule.
- `ranks_and_pools::links_made_before_the_splits_…` keeps `(8, 5)` through X.

**Deleted:** `a_listed_refusal_that_opens_fails` (replaced by `unhit`) and `a_battle_scion_link_from_the_s2_base_still_opens`.

## 6. Risks and unknowns

1. **Does the walk keep a refused pick held?** If it does, Mythic Toughness fails only after the strip; if not, it fails in pass 1. `How::Follows` covers both. The builder runs `excuse` on the 306s and 306l Builds and asserts the `Follows` hits.
2. **Shape stability.** An `Unmet`'s failing children vary with the Build (D's alternatives, a held fighter level). The builder generates candidate rows by grouping today's failures by (entry, place, shape); more than one shape gives more than one row, each hit.
3. **Failures outside slots.** These include 308d's true-neutral Dark Elementalists and `targeted ranger10-slayer2` (308e). The builder lists the `Choice` kinds across the 459 Builds before writing. A kind other than Slot, Archetype or Class gets a `Place` of its own; it should not get `any`.
4. **Undecodable base links.** X needs a Build, so it can't excuse one. The builder counts them first and expects 0. Any found means its ticket owes an alias (`links.rs:1-4`).
5. **Ordering against T1.** Shapes are in normal form. If T1's flattening lands after T8, every `any(any(…))` row breaks. Keep T1 → T8.
6. **Rows track today's issue.** A later data ticket that changes an issue makes its row unhit, and the corpus test fails. That ticket edits or deletes the row in its own file, as `known.txt` lines are handled (287's Clear Mind is one).
7. **Guarding.** Not guarded (overall.md:94). If the user wants it, guard only rows whose hits include a gold Build or a bench link.
8. **Dev server.** `libs/game-data` is in `watch-additional-files` (`Cargo.toml:71`), so a row edit restarts `cargo leptos watch`, as a note edit does. This is harmless; a builder who minds can use `libs/seeds/refusals/` and keep `seeds::refusal_files`.
9. **Census link.** `StatusSplit`'s refused set could later come from X's `is_expected` rather than the link census. Left out of T0 and T8.
