# The sorcerer's level-1 bonus feat: which list?

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). Since [ticket 44](44-sorcerer-feat-slayer-deities.md), every sorcerer's level-1 "Bonus Feat" is a slot of the held bloodline's bonus-feat list (`OptionList(selection: "sorcerer/bloodline")`). [Ticket 43](43-extraction-report-gaps.md)'s question 1 chose that reading because the guide never says what the level-1 feat is, and said "if you can check in the game, go by it". The Jack-in-the-Green write-up (a guide-style sample build of a Sylvan Sorcerer) takes **Spell Penetration** as its level-1 "Bonus Feat". Spell Penetration isn't on the Fey list, so the app can't build it.

**Repro:** the Jack-in-the-Green Build at level 1 (`#b=AQFHALj_C3B5t9x-n6bdRmDXQFRsC6Yx6ecNG2DXQCrWmNBTLAktDgABAQADAQEMDg4MCBEBAAMBAQEBAQAAAAEAAgEBAAAAAAIAAAA`), level 1, the "Bloodline feat" slot. It offers Dodge, Improved Initiative, Lightning Reflexes, Metamagic (Quicken Spell), Point-Blank Shot and Skill Focus; locked are Agile Maneuvers ("Needs Bull Rush or Dirty Trick or Sunder Armor or Trip") and Precise Shot ("Needs Point-Blank Shot"). Spell Penetration isn't listed. (In ticket 88 the QA build took Spell Penetration in the level-1 general feat slot instead, which drops the write-up's Mounted Combat, and put Dodge in the bloodline slot.)

**The guide:** every sorcerer table but the Overwhelming Mage's has "Bonus Feat" at level 1 in the column that holds "Bloodline Feat" at 7, 13 and 19 (`sorcerer#sylvan-sorcerer`, `sorcerer#fey`, …); `character-creation#bloodlines` names only 7, 13 and 19 as bloodline-feat levels. The Fey bonus-feat list (`character-creation#fey-bloodline`) is Agile Maneuvers, Dodge, Improved Initiative, Lightning Reflexes, Quicken Spell, Point-Blank Shot, Precise Shot and Skill Focus. The write-up's level 1 reads "Bonus Feat - Spell Penetration; Mounted Combat": two feats, one of them off the bloodline list.

**The options:**
1. Keep the bloodline list (today); the write-up is wrong.
2. Any feat (a General slot), as the Geomancer has: the level-1 feat isn't tied to the bloodline, only 7/13/19 are. This matches the write-up and `character-creation#bloodlines`.
3. A Magic feat (Spell Penetration is one; Mounted Combat is the write-up's general feat).

**Recommendation:** 2, if you can confirm in the game that a sorcerer's level-1 bonus feat isn't limited; it fits both the write-up and the character-creation sentence. Otherwise keep 1 and close this ticket.

## Build

Once the user decides: a data change in `libs/game-data/classes/sorcerer.ron` (each row-1 `OptionList` slot that becomes General or Magic), a `libs/game-data/README.md` note, and real-data tests in `libs/seeds/tests/game_data/` replacing ticket 44's level-1 tests; the behaviour proof shows only the row-1 slots move.

**Done when:** the decision is recorded here, and, if it changes the slot, a Sylvan Sorcerer 1 can take Spell Penetration in the level-1 bonus slot and Mounted Combat in its general slot.

**User, 2026-11-07:** the sorcerer's level-1 bonus feat is a **Magic feat** (option 3): a `Category(Magic)` slot on row 1, in place of ticket 44's bloodline-list slot, for the base sorcerer and every archetype that got the bloodline-list slot (Crossblooded, Empyreal, Sage, Sylvan, Nine Tailed Heir, Seeker). The bloodline feats stay at 7, 13 and 19. A Sylvan Sorcerer 1 can then take Spell Penetration in that slot and Mounted Combat in its general slot. The Geomancer's level-1 General slot and the Overwhelming Mage (Overwhelming Penetration at 1) are as the lead's follow-up question to the user settles.

**User, 2026-11-07, on the follow-up:** the Geomancer's level-1 slot becomes a Magic feat too, in place of its General slot. The Overwhelming Mage keeps Overwhelming Penetration at 1 (the guide's table entry, not a feat slot).

## Progress log

- 2026-11-07: started. Baseline HEAD 24bcf67, clean tree; scratch in `t92/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t92/`). `sorcerer.ron` has eight archetypes; every row 1 checked (the table in the Answer).
- 2026-11-07: real-data tests first, in `libs/seeds/tests/game_data/report_gaps.rs` in place of ticket 44's five level-1 tests (5 new; Extra Slayer Talent's stays). Red on HEAD's data (`t92/real-data-red.txt`): 3 fail, each at the level-1 slot (the base sorcerer's kind is the bloodline list; Spell Penetration "isn't offered" to a Sylvan 1; the label is "Bloodline feat"); the Overwhelming Mage and 7/13/19 tests pass from the start, since they pin data that stays.
- 2026-11-07: data by hand: the eight row-1 slots in `classes/sorcerer.ron` are `Category(Magic)`. `cargo test -p seeds` green (game_data 251). README: ticket 92's paragraph; ticket 44's says the level-1 slot was replaced. Reseeded `wotr-planner.db` (no schema change): `feat_slot` `category` 53 → 61, `option_list` 54 → 47, `general` 23 → 22. `scripts/check.sh`: ALL-OK.
- 2026-11-07: proof. `scripts/proof.sh data t92/proof` (A = HEAD 24bcf67) differs, as expected: `compare.txt` has 266 `differs` and 2 `only in` lines. Each side generates its own full fills, so for the breakdown `t92/analyze/` (a scratch crate on `rules`, `seeds` and `proof`) answers side A's 2146 Builds on both data sides and itemises every difference (`t92/diffs.jsonl`, `t92/generated.jsonl`, `t92/diffs-minus.jsonl`). Every difference comes from a sorcerer's level-1 slot (the Answer has the breakdown). Map: a Decisions-so-far line.

## Answer

Done 2026-11-07, uncommitted. Every sorcerer's level-1 bonus feat is a Magic feat (`feat_slots: [Category(Magic)]` on row 1), as the user decided. The sheet labels it "Magic feat". No schema, `rules` or `Build` change, so the link layout stays. A Sylvan Sorcerer 1 can take Spell Penetration in the bonus slot and Mounted Combat (with 1 rank of Mobility) in its general slot, so the "Done when" holds.

### Row 1 of every sorcerer in `classes/sorcerer.ron`

| Sorcerer | Had (ticket 44) | Has |
| --- | --- | --- |
| base | `OptionList(selection: "sorcerer/bloodline")` | `Category(Magic)` |
| Crossblooded | `OptionList(selection: "sorcerer/bloodline")` (either bloodline's list) | `Category(Magic)` |
| Empyreal Sorcerer | `OptionList(selection: "sorcerer/bloodline")` (Celestial) | `Category(Magic)` |
| Geomancer | `General` | `Category(Magic)` |
| Nine Tailed Heir | `OptionList(selection: "sorcerer/bloodline")` (chosen bloodline) | `Category(Magic)` |
| Overwhelming Mage | no slot; Overwhelming Penetration | unchanged |
| Sage Sorcerer | `OptionList(selection: "sorcerer/bloodline")` (Arcane) | `Category(Magic)` |
| Seeker | `OptionList(selection: "sorcerer/bloodline")` (chosen bloodline) | `Category(Magic)` |
| Sylvan Sorcerer | `OptionList(selection: "sorcerer/bloodline")` (Fey) | `Category(Magic)` |

The bloodline feats at 7, 13 and 19 (`OptionList(selection: "sorcerer/bloodline")`; none for the Geomancer or the Overwhelming Mage) and the Seeker's `Category(Magic)` at 5, 10 and 15 are unchanged. A pick off the Magic feats in an old link (a sorcerer's Toughness or Dodge at 1) is now an impossible Build, which the sheet treats as a stale link.

### Tests

`libs/seeds/tests/game_data/report_gaps.rs`: 5 tests in place of ticket 44's five level-1 tests. Red on HEAD's data first (`t92/real-data-red.txt`); the last two passed from the start, since they pin data that stays:

- every sorcerer with a level-1 bonus feat (base, Crossblooded, Empyreal, Sage, Sylvan, a kitsune Nine Tailed Heir, Seeker, Geomancer) has a Magic slot at 1. It offers exactly the Magic feats, with Spell Penetration open, and not Dodge, Power Attack, Toughness or Improved Initiative. A Dodge pick there is an impossible Build;
- a Sylvan Sorcerer 1 with Mobility 1 takes Spell Penetration in the bonus slot and Mounted Combat in the general slot, and that Build is possible;
- the sheet labels the slot "Magic feat" and the 7th "Bloodline feat", and every slot of a sorcerer 20 has a label;
- an Overwhelming Mage 20 has Overwhelming Penetration at 1 and no class feat slot at any level;
- 7, 13 and 19 still offer exactly the held bloodline's list (Crossblooded: both). The Geomancer has no slot there, and the Seeker has Magic slots at 5, 10 and 15.

### Proof

`scripts/proof.sh data` against HEAD (`t92/proof/`; 266 Builds differ, 2 are only on one side). The breakdown answers the same Builds (side A's 2146) on both data sides (`t92/analyze/`), and every difference comes from a sorcerer's level-1 slot:

- **The generated Builds:** 66 full fills of sorcerer Builds draw a different level-1 class pick, because the slot offers other feats, so the random picks after it differ. Their levels and starts are the same. `full decision new sorcerer toughness` is only in A: its base, Toughness at 1, is impossible now, so it gets no fill. `full decision stale seeker extend` is only in B: Extend Spell is a Magic feat, so it gets a fill.
- **The same Builds answered on both sides:** 267 differ.
  - 61 differ in the level-1 slot only: its kind changes to `Category(Magic)` from the bloodline list (base 25, Seeker 23, and Crossblooded, Empyreal, Nine Tailed Heir, Sage and Sylvan 2 each) or from `General` (Geomancer 3).
  - 137 Builds without sorcerer levels differ in the class picker only. The entries that change are always a sorcerer one, never the Overwhelming Mage's, at a level that holds a class-feat pick:
    - the Geomancer goes from open to `BreaksLater` 119 times (its slot no longer takes a non-Magic pick);
    - the base, Crossblooded, Empyreal, Seeker and Sylvan open 20 times each, and the Sage 10 times (a Magic pick now fits);
    - the Sylvan closes 7 times, the Empyreal 3 and the Sage once (a pick on its fixed list that isn't a Magic feat).
  - 62 change whether they're possible. 61 full fills whose level-1 pick is a non-Magic bloodline feat become impossible (stale links), and the Seeker's Extend Spell link becomes possible.
  - In 6 full sorcerer Builds the level-1 pick is a Magic feat that was also on the bloodline's list. The bloodline no longer constrains that pick, so clearing the bloodline is now allowed, its options open, and `removable` changes.
  - 1 Build, `decision stale empyreal toughness`, is impossible on both sides; its slot and its sorcerer class entries differ.
  - With the level-1 pick taken out of those 69 Builds, each differs only in the level-1 slot itself.

### Docs, DB, checks

- `libs/game-data/README.md`: ticket 92's paragraph; ticket 44's now says ticket 92 replaced its level-1 slot.
- The map: a Decisions-so-far line. `GLOSSARY.md` is unchanged (no new term).
- Dev DB reseeded (no schema change): `feat_slot` `category` 53 → 61, `option_list` 54 → 47, `general` 23 → 22.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm clippy, workspace tests, rules wasm, seed check).
- The proof harness's `decision new sorcerer toughness` and `decision stale seeker extend` Builds keep their names (the records compare by name), though their outcomes are now the other way round.

### Open questions

None.
