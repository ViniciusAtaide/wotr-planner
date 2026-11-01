# Option levels for domains, mysteries, blessings and elements

Type: task
Status: resolved
Blocked by: 30
Map: [WotR Build Planner](../map.md)

## Question

Fill the option-level model from [ticket 30](30-build-option-levels.md) for the second group of families in [ticket 29](29-design-option-level-grants.md):
- **Domains** (37, shared by cleric, druid via Nature Bond, inquisitor, Divine Hunter and Hellknight's Pentamic Faith): powers at option levels 1, 6 or 8, and sometimes 12/20; War's bonus combat feat at 8; class skills. A cleric holds two. Animal and Scalykind keep their class skill; their companion is out of scope.
- **Oracle mysteries:** class skill, Final Revelation at 20, and the revelation picks at 1/3/7/11/15/19 drawn from the held mystery's list. Revelations that grant later (Weapon Mastery's feats at 8/12) are option rows too. Curses stay a plain pick.
- **Warpriest blessings:** two held; minor power at 1, major at 10, replacing today's second pick at 10.
- **Kineticist elements:** class skills, the element's wild-talent feat list, the defensive talent at 2, and Expanded Element at 7/15 as a second pick.
- The domain part of Druid Nature Bond ([ticket 28](28-class-rules-follow-ups.md)) and Domain Zealot's domain.

Data from the saved guide pages only (offline). Model changes only if a family doesn't fit; if so, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a cleric 8 holds both domains' second powers, an oracle's revelation slot offers only its mystery's revelations, a warpriest 10 holds both major blessings.

## Progress log

- 2026-11-01: baseline in the session scratchpad `t31/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t31/`): `old-src/` is `git archive HEAD` (e136fff). Guide read offline (`t24/pages/*.txt`, made from `guide-cache/`). Milestones: (1) model gaps in `rules`, test world first; (2) schema, seed, loader, fixture; (3) data family by family through a new `extract::option_families` module run by a one-off `t31/rewrite/` program; (4) extractor wiring and a fresh extraction; (5) proof, docs, DB reset, Verification. Shape decided:
  - Two model gaps, both inside ticket 29's design ("options grant features, feat slots and selections"): `OptionLevelRow.feat_slots` (War's bonus combat feat at 8; slot `SlotId::OptionFeat { level, index }`) and `OptionLevelRow.chosen_feats` (Weapon Mastery's Improved Critical at 8 and Greater Weapon Focus at 12 for the weapon its pick named; a holding keeps its pick's value).
  - Class skills follow ticket 30: a "Class Skill: …" feature on the option's first row (the engine has no class skill state; ticket 30's open item 8 stays the user's).
  - Domains are one selection `cleric/domain` (37 options, rows by cleric levels) that the others share through their own selections (druid via Nature Bond's domain option, inquisitor, Divine Hunter, Blight Druid, Winter Child, Hellknight's Pentamic Faith, the Living Deity's Hollow Spark). A cleric's two domains are the selection granted twice at 1.
  - Mysteries `oracle/mystery` (pool of each: its revelations); the oracle's revelation slots are a pool selection `oracle/revelation` (rows by oracle level); Final Revelation is each mystery's row 20. Blessings: rows at 1 and 10 on `warpriest/blessings`; the level-10 picks go. Elements: new `kineticist/element` (class skills, wild-talent bonus feat list, defensive talent at 2) and `kineticist/expanded-element` at 7 and 15 (a plain pick of an element, which may double the first).
  - A scaling number stays in the summary (ticket 29 round 1, 2): domains' resistances growing to immunity, uses per day; so no domain has a row at 12 or 20.
- 2026-11-01: milestone 1 (`rules`) done. `OptionLevelRow` gains `chosen_feats` and `feat_slots` (and `Default`); `SlotId::OptionFeat { level, index }` (its own counter per level; strays reported like `OptionSelection`); the walker's `pending` holds slot kinds, a `Holding` keeps its pick's value for `grant_row`'s chosen feats (Bloodline Ascendance's `grant_option_row` too). Check: `RowChosenFeatOfOtherKind(feat)` (a row's chosen feat must take the kind of value the option's pick names), option rows' slots name lists; `OptionRowsWithoutLevels` skips a selection that is only some option's pool and checks a pool selection against the options its pools offer. Tests first: the test cleric gains a War domain (row 8: a Combat feat slot) and a Weapon Mastery fire revelation (chosen Weapon Focus; row 8: Weapon Specialization for the same weapon), `cleric/revelation` gets cleric option levels; 2 engine tests and 2 check tests (red: the engine's 2 and the pool test's offer, saved as `t31/engine-red.rs`; the check variant didn't exist). v1 link vector regenerated with an `OptionFeat` pick. `cargo test -p rules` 217 + 1 pass, clippy clean.
- 2026-11-01: milestone 2 (`data`) done. Migration: `feat_slot.option_level_id` (now one of four owners), new `option_level_chosen_feat (option_level_id, feat_key)`; 39 tables. Seed and loader. Tests first: the fixture's Draconic row 3 has a Combat and a `OneOf` slot, a new `sorcerer/abyssal-powers/mastery` (chosen Weapon Focus; row 3: Weapon Specialization chosen, Toughness granted); files test extended, postgres CHECK/FK lines for the two. Red run with a loader that defaulted the fields: 10 postgres failures. Dev DB reset with `t31/reset-db.sh --no-seed` (only `wotr-planner-postgres`, 5434). `ron_out` writes the two fields. `cargo test -p data` and `-p extract` pass. Next: the data, family by family.
- 2026-11-01: milestone 3 (data) done. One implementation: new `extract::option_families::apply(guide, data)` (prose tables with the phrase each power's level comes from, asserted against the saved pages; `option_levels`' helpers made `pub(crate)`, its `drop_unused` takes the touched classes), run over `game-data/` by the one-off `t31/rewrite/` (Rust, from `t31/old-src/game-data/`; dry run in `t31/dry/game-data/` byte-identical; log `t31/game-data-log.json`, 97 changes in 11 files). Families: 37 domains (`cleric/domain`; inquisitor, Sacred Huntsmaster (no Animal or Scalykind), druid via Nature Bond's domain option (8), Blight Druid (12, skips the companion at 4), Winter Child (Ice, no pick), Divine Hunter (skips 4/6/8: its table has no cells), Pentamic Faith (11), the Living Deity's Hollow Spark (two at 20)), the classes' "Domain Abilit…" cells and the "Animal domain only" companions at 4 gone; 9 mysteries with 9 revelation pools (`oracle/revelations/<mystery>`, "Level N" prerequisites as oracle or arcanist level), Weapon Mastery's chosen feats at 8 and 12, Maneuver Mastery's nested selection (feat at 7, Greater at 11), War Sight's Uncanny Dodge at 7, Stone Stability's Trip/Greater Trip at 5/10, Fluid Nature's Dodge at 5, Skill at Arms' two feats; oracle archetypes (Divine Herbalist skips the class skills, Enlightened Philosopher skips 20, Lone Strider's five, Wind Whisperer's Wind with no pick), the revelation cells are `oracle/revelation` slots; the Living Deity's mystery, curse, revelations (by arcanist level); 25 blessings' minor and major powers, level-10 picks gone, Feral Champion's Animal blessing with no pick, Champion of the Faith's alignment blessing; 4 elements (class skills, wild-talent bonus feat list, defensive talent at 2, `Times(2)`), Expanded Element at 7/15 (lists only), Blood Kineticist's Water with no pick, Kinetic Knight's talent at 4 through `option_row`; the Hellknight's order (only without one held) and Pentamic Faith's domain; Domain Zealot and Second Mystery's prerequisites (any domain, any mystery), Second Mystery picks one; Boon Companion and Mythical Beast accept the domains' companion and the Nature revelation's. Real-data tests first (`game_data.rs`, 16 new; red on HEAD's data: 17 failures with the updated Domain Zealot test); 3 older tests updated (Domain Zealot needs a domain picked; list count 69 → 73; the elements repeat twice). Next: extractor wiring, a fresh extraction, proof, docs.
- 2026-11-01: milestone 4 (extractor) done: `extract` runs `option_families::apply` after `option_levels::apply` (it settles no item of an extraction's report). A fresh extraction (`t31/extract-new/`) is **byte-identical** (RON and report) to `rewrite` applied to HEAD's extraction (`t30/extract-new/` → `t31/dry/extract-old/`, 96 changes: the hand-fixed data has one more hunter feature and one more oracle feature to drop). Tests: new `extract/tests/extract/option_families.rs` (4: domain names, the "Level N" line in both spellings, the elements' feat spellings, the files writing an option row's slots and chosen feats), written with the code in one pass. `cargo test -p extract` 80 pass. Next: proof, docs, DB reset, Verification.
- 2026-11-01: milestone 5 done. Proof (`t31/proof-{old,new}` on one shared `t31/proof-common.rs`, `t31/compare.py`) passes. Docs: `game-data/README.md`, `GLOSSARY.md` (new Pool selection; Option level, Feature selection), `game-data/EXTRACTION_REPORT.md` (2 items narrowed, still 14). Final DB reset and reseed (`t31/reset-db.sh`), `cargo sqlx prepare` (5 query files new, 3 gone); every Verification check passes. Done.

## Answer

Done 2026-11-01, uncommitted. Domains, oracle mysteries with their revelations, warpriest blessings and kineticist elements now have rows by option level on ticket 30's model. A domain gives its power at 1 and its second at 6 or 8 (the Animal and Scalykind companion at 4, the War domain's bonus Combat feat at 8), levelled by whichever class took it: the cleric (two), the inquisitor, the druid through Nature Bond, the Divine Hunter, the Blight Druid, the Winter Child (Ice, no pick), the Hellknight's Pentamic Faith and the Living Deity's Hollow Spark. A mystery gives its class skills and, at 20, its Final Revelation, and the oracle's revelation slots offer only the held mysteries' revelations (Weapon Mastery's Improved Critical at 8 and Greater Weapon Focus at 12 in its weapon). A blessing gives its minor power at 1 and its major at 10, so the level-10 picks are gone. An element gives its class skills, its wild-talent bonus feat list and its defensive talent at 2; Expanded Element picks a second (or the first again) at 7 and 15. On the real data a cleric 8 holds both domains' second powers, an oracle's revelation slot offers only its mystery's revelations, and a warpriest 10 holds both major blessings. The model needed two fields inside ticket 29's design (an option row's feat slots and its feats for the pick's value); the `Build` gained a slot id, so the v1 link vector is regenerated (no v2).

### Rust (`rules`)

- **Option rows.** `OptionLevelRow.feat_slots` (the War domain's `CombatFeat` at 8) and `OptionLevelRow.chosen_feats` (held for the value the option's pick named for its `chosen_feat`: Weapon Mastery). `OptionLevelRow` derives `Default`.
- **Slots.** `SlotId::OptionFeat { level, index }`: a feat slot an option row grants, at the level that reaches it (its own counter per level; a pick for one the Build lacks is `NoSuchSlot`, like `OptionSelection`).
- **Walker.** `pending` holds slot kinds (selections and feat slots); a `Holding` keeps its pick's `parameter`, which `grant_row` gives the row's chosen feats (Bloodline Ascendance's `grant_option_row` too).
- **Check.** `RowChosenFeatOfOtherKind(feat)` (a row's chosen feat must take the kind of value the option's pick names); option rows' slots name their lists; `OptionRowsWithoutLevels` skips a selection that is only some option's pool, and checks a pool selection against the options its pools offer.
- Tests first: the test cleric gains a War domain (row 8: a Combat feat slot) and a Weapon Mastery fire revelation (chosen Weapon Focus; row 8: Weapon Specialization for the same weapon), its revelation pool gets cleric option levels; 2 engine tests, 2 check tests; the pool test's offer gains the revelation. Red run: 3 engine failures (HEAD's walker, saved as `t31/engine-red.rs`). The v1 link vector holds an `OptionFeat` pick now.

### Schema (the one migration)

- `feat_slot.option_level_id`: an option level owns feat slots (now one of four owners, CHECKed).
- New `option_level_chosen_feat (option_level_id, feat_key)`, both FKs, primary key the pair.
- 39 tables (38 + `option_level_chosen_feat`). Seed, loader, columns. Tests: the fixture's Draconic row 3 has a Combat and a `OneOf` slot, a new abyssal power has a chosen Weapon Focus and a row 3 with Weapon Specialization chosen and Toughness granted (files test extended); postgres CHECK/FK lines for both. Red run with a loader that defaulted the fields: 10 postgres failures.

### Data rewrite

One implementation: `extract::option_families::apply(guide, data)`. The guide gives these families in prose, so each power has a table entry with its level and the phrase in the option's section that says so, asserted against the saved pages (the elements' lines and the blessings' "At 1st level"/"At 10th level" are read directly). Then each class and archetype takes the family: the placeholder cells ("Domain Abilities", "Domain Ability (2)", the "Animal domain only" Animal Companion at 4, the druid's Lightning Lord, "Mystery", "Revelation", "Final Revelation", "Feral Blessing", "Class Skills", "Defensive Wild Talent", "Expanded Element", the Hellknight's "Order") go, and the choices become grants. The one-off `t31/rewrite/` (Rust, in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t31/`) ran it from `t31/old-src/game-data/` and wrote only the 11 files whose data changed (`t31/game-data-log.json`, 97 changes; byte-identical to the dry run in `t31/dry/game-data/`). The extractor was never run over `game-data/`.

| Family | Options | Notes |
| --- | --- | --- |
| Domains (`cleric/domain`, cleric levels) | 37 (character-creation) | power at 1; second at 6 (Air, Earth, Fire, Healing, Luck, Plant, Water) or 8; Animal and Scalykind: companion at 4; War: `CombatFeat` slot at 8; class skills (Animal: Lore (Nature); Knowledge: both Knowledges and Lores; Trickery: Stealth, Trickery); 0 rows at 12 or 20 (resistances growing to immunity are summaries) |
| Cleric | two picks at 1 | Angelfire Apostle, Ecclesitheurge two; Crusader, Demonbane Priest, Herald Caller, Priest of Balance, Separatist one (the Separatist's delayed second domain stays as cells: question 1) |
| Inquisitor | `inquisitor/domain` (inquisitor levels) | Sacred Huntsmaster: all but Animal and Scalykind; Faith Hunter: none |
| Druid | `druid/domain` (8: Air, Animal, Earth, Fire, Ice, Plant, Water, Weather) | granted by Nature Bond's domain option row 1 (`druid/nature-bond` levelled by druid); the druid's, Defender's and Feyspeaker's cells at 4, 6, 8 go (ticket 28's question 2); Blight Druid: its 12, skips the companion at 4; Winter Child: Ice with no pick |
| Divine Hunter | `hunter/divine-hunter/domain` (hunter levels) | its table has no cells after "Domain", so it skips 4, 6 and 8 (question 2) |
| Hellknight | `hellknight/order`, `hellknight/pentamic-faith` | the order (the Armiger's seven) only for a character holding none; Pentamic Faith's row 1 grants one of its 11 domains (Hellknight levels; `hellknight/disciplines` levelled by Hellknight) |
| Mysteries (`oracle/mystery`, oracle levels) | 9 | class skills at 1, "X Final Revelation" at 20, `pool: "oracle/revelations/<mystery>"` (61 revelations); "Prerequisites: Level N" is oracle or arcanist level N |
| Revelations with rows | 6 | Weapon Mastery (chosen Weapon Focus; Improved Critical at 8, Greater Weapon Focus at 12, same weapon), Maneuver Mastery (a nested `oracle/maneuver-mastery`: Bull Rush, Dirty Trick, Disarm, Sunder Armor or Trip at 7, its Greater at 11), War Sight (Uncanny Dodge at 7), Stone Stability (Trip at 5, Greater Trip at 10), Fluid Nature (Dodge at 5), Skill at Arms (Martial Weapons and Heavy Armor Proficiency) |
| Oracle | `oracle/revelation` slots (pool of `oracle/mystery`) | Divine Herbalist skips the class skills at 1; Enlightened Philosopher skips 20 (its own stays); Lone Strider: Ancestor, Life, Nature, Stone, Waves; Wind Whisperer: Wind with no pick |
| Living Deity (a Magic Deceiver way) | `arcanist/magic-deceiver/{mystery,revelation,domain}` (arcanist levels) | row 1: mystery, curse (`oracle/oracles-curse`), revelation; 4, 12: revelation; 20: revelation and two domains (Hollow Spark); its Final Revelation is the mystery's row 20 (ticket 30's open item 8) |
| Blessings (`warpriest/blessings`, warpriest levels) | 25 (existing) | "Minor Blessing (X)" at 1, "Major Blessing (X)" at 10; the 12 level-10 picks (base, Proclaimer, Shieldbearer) go; Feral Champion: Animal with no pick, no second; Champion of the Faith: `chosen-alignment` (Chaos, Evil, Good or Law) beside its one blessing |
| Elements (`kineticist/element`, kineticist levels) | 4 | class skills and "Wild Talent Bonus Feats (X)" (ignores prerequisites; 4 new lists) at 1, the defensive talent at 2; `repeat: Times(2)`; Blood Kineticist: Water with no pick; Kinetic Knight: skips 2, its "Defensive Wild Talent" at 4 has `option_row` level 2 |
| Expanded Element | `kineticist/expanded-element` at 7 and 15 | lists only (the guide gives the new element's blasts only) |
| Prerequisites | | Domain Zealot: any of the 37 domains; Second Mystery: any mystery, and it picks another (`selections: ["oracle/mystery"]`); Boon Companion and Mythical Beast: the 16 "Animal domain only" companions become the two domains' companions and the Nature revelation's Animal Companion |

Three older real-data tests changed: Domain Zealot needs a domain picked (a druid's Nature Bond domain option alone no longer counts, nor a Blight Druid without its pick), the list count (69 → 73), the repeating options (the four elements).

### Proof

`t31/proof-old/` (against `git archive HEAD`, e136fff) and `t31/proof-new/` build one shared source, `t31/proof-common.rs`; `t31/compare.py` checks:

- **Data:** outside the 9 touched classes everything is identical but Boon Companion, Mythical Beast, Domain Zealot and Second Mystery, each checked exactly; the 30 untouched classes are identical. In the touched classes, every level of every class and archetype (1310 rows), as its effective row, is the same before and after once the placeholders removed (176 cells over the effective rows, each named in the list above), the new grants added (90 rows) and the warpriest's 12 level-10 picks are normalised; no row gains a feature, feat, slot or spell level; every feature that went (91) was a placeholder, every new one (159) a family's; only the elements' lists are new; the existing selections change only by levels and rows (25 blessings, Nature Bond's domain, Pentamic Faith, the Living Deity's 5 rows); the Kinetic Knight's talent gains its `option_row`.
- **Walk:** every base class and archetype through 20 levels and each prestige class after wizard, cleric or witch 10 (the new side fills the new selections with their first open option): every Build of an untouched class is identical; 4423 of 5481 lines are identical, the 1058 others are Builds of the touched classes.

### Extractor

- `extract` runs `option_families::apply` after `option_levels::apply`; it settles no item of an extraction's report. A fresh extraction (`t31/extract-new/`) is **byte-identical** (RON and report) to `rewrite` applied to HEAD's extraction (`t30/extract-new/` → `t31/dry/extract-old/`, 96 changes; the report stays at 34 items).
- `ron_out` writes an option row's `chosen_feats` and `feat_slots`.
- Tests: `option_families.rs` (4: domain names, the revelation "Level N" line in both spellings, the elements' feat spellings, the files writing an option row's slots and chosen feats), written with the code in one pass.

### Docs

- `game-data/README.md`: the four families, `feat_slots`/`OptionFeat`, `chosen_feats`, the domain selections per class, Nature Bond's domain row.
- `GLOSSARY.md`: new Pool selection; Option level (the new families, rows' slots and chosen feats) and Feature selection (a cleric's two domains) updated.
- `game-data/EXTRACTION_REPORT.md`: still 14 items; the kineticist bonus-feat item narrowed (Elemental Focus is data, wild talents aren't), the Hellknight order item narrowed to the Signifer's.

### Verification

- `cargo test --workspace`: pass (rules 217 + unit 1, extract 80, data files 21 + game_data 60 + postgres 18).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (5 query files new, 3 gone).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434 (`t31/reset-db.sh`): drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 63 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths. Rows: 39 tables; `option_level` 357 → 524 (165 options with rows), 50 levelled selections, 20 skipped levels, 1 option-level feat slot, 2 option-level chosen feats, 2 pool selections, `feat_list` 69 → 73, `feature` 1901 → 1969, `feature_selection` 109, `selection_option` 727.
- The v1 link vector is regenerated (the `Build` type gained `SlotId::OptionFeat`).
- Proof: passes (above).

### Open questions (for the user)

1. **The Separatist's second domain** ("Higher level abilities are delayed by two class levels", Forbidden Rites) needs an option level of cleric levels minus 2, which the model can't say. It picks its first domain; the second domain's cells (companion at 6, a Domain Ability at 8 and 10) stay as features, and it has no second pick. A selection field "levels behind" (2) would fit; or keep it as is?
2. **The Divine Hunter's domain** gets only the level-1 power: its table shows "Domain" at 0 and no domain cells after (ticket 30's rule: an archetype skips the option levels its table has no cell for). The guide's "The domain spells are added to her spellbook" doesn't settle the 6th/8th-level powers. Should it get them by hunter level?
3. **The domains' companion.** The design kept the Animal and Scalykind companion out of scope; I read that as the companion itself (no creature choice), while the cells "Animal Companion¹ (Animal domain only)" are now the Animal domain's row 4 (Scalykind's too, from its section), so only those clerics, inquisitors and druids hold a companion and Boon Companion opens for them. Confirm, or drop the companion rows?
4. **The guide's footnotes disagree on two domains.** The cleric table puts Nobility's second power at 6 and Air's at 8; the domains' own sections (and the druid's footnote for Air) say Air at 6 and Nobility at 8. The sections won.
5. **Impossible Domain** ("You may select any domain") is unchanged: whether it grants a further domain, and whose levels drive it, isn't settled by the guide. Its prerequisite still excludes the druid ("you can't take the Impossible Domain mythic ability").
6. **Expanded Element** gives the element's list only (the guide names only the blasts), and the 15th-level rule "a third element or the one she chose at 1st level, but only if she didn't choose it at 7th" is only partly enforced: `Times(2)` stops a third pick of one element, but a kineticist can double the 7th-level element at 15.
7. **Revelations:** "Level N" prerequisites accept oracle or arcanist level N (for the Living Deity), so a non-Living-Deity arcanist's levels count too for an oracle; Cinder Dance's "You cannot select this revelation if you have the Lame curse" isn't enforced (no "not" leaf); the Dual-Cursed Oracle's two unique revelations (Misfortune, Fortune) aren't offered (a pool selection has no options of its own); Maneuver Mastery's feats come by oracle level, so a Living Deity's never come; Second Mystery picks through `oracle/mystery` (oracle levels), so a Living Deity's second mystery gets no rows.
8. **Champion of the Faith:** the alignment blessing (Chaos, Evil, Good or Law) doesn't check the character's alignment ("must correspond to his own alignment").
9. **Class skills** stay "Class Skill: …" features, as ticket 30 did for bloodlines (its open item 8).
10. **The Signifer's order** is still a feature (report item); only the Hellknight's is a selection.

## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. The Separatist's second domain arrives two class levels late: a selection's second pick can lag by a set number of levels. [Ticket 33](33-option-follow-ups.md).
2. The Divine Hunter gets every domain power, by hunter level. Ticket 33.
3. The Animal and Scalykind domains' level-4 row counts as a companion for Boon Companion, as built.
4. The cleric table's footnotes lose to the domain sections (Air at 6, Nobility at 8), as built.
5. Impossible Domain (user fact): it grants one domain from the full list, levelled by the class that gives the character's other domains (cleric, inquisitor or hunter). Ticket 33.
6. Expanded Element can't take the 7th-level element again at 15, using the option-group check of ticket 30's decision 1. Ticket 33.
7. Revelations: Cinder Dance needs no Lame curse; the Dual-Cursed Oracle is offered Misfortune and Fortune; Maneuver Mastery and Second Mystery count arcanist levels for a Living Deity. "Level N" prerequisites keep accepting oracle or arcanist levels. Ticket 33.
8. Champion of the Faith's alignment blessing checks the character's alignment. Ticket 33.
9. Options' class skills stay features.
10. The Signifer's order is a selection sharing the Hellknight's orders. Ticket 33.
