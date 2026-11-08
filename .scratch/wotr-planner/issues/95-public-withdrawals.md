# The engine says which choices a Build holds, and the withdrawal of each

Type: task
Status: resolved
Blocked by: [94](94-alignment-after-faith.md) (it changes the start choices this ticket lists)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 2) found the list of "which choices a Build holds, and the edit that takes each back" written four times:

- `Edit::choices` (`libs/rules/src/build.rs:474–520`);
- the engine's `withdrawals` and `first_level` (`libs/rules/src/engine/removable.rs:84–246`), deep but private;
- nine hand-built `(Choice, Edit)` remove pairs in `libs/ui/src/picker.rs` (around 399, 767, 811, 817, 940–949, 960–967, 1083–1086, 1129–1136, 1167–1174);
- the proof's `battery` and `every_choice` (`tools/proof/src/fingerprint.rs:196–312`).

The race withdrawal (race, subrace and racial +2 all cleared) is written three times. The proof's copy has fallen behind: `every_choice` has no `Choice::Deity`, `Choice::FirstAscension` or `Choice::CharacterLevel`, and `battery` has no deity starts and no `Edit::FirstAscension`. So the proof's `is_removable`-vs-`removable` and `allows` checks never covered tickets 86 and 91.

The user settled the design in a grilling round (2026-11-07, all recommendations taken).

## Build

- **The word:** a **Withdrawal** is the edit that takes a held choice back (a race's also clears its subrace and racial +2). Add it to `GLOSSARY.md` beside **Removable**, and use it in the interface.
- **For a remove control:** one engine call per choice that answers both questions a picker's remove control asks: `None` when the Build doesn't hold that choice, otherwise its withdrawal and whether it's removable now. Its cost is no more than `is_removable`'s today.
- **For the proof:** a public list of every choice the Build holds, removable or not, each with its withdrawal. `removable`, `is_removable` and `removable_classes` stay as they are.
- **The UI:** every picker's remove control (slot Clear, class, archetype, path, first ascension, switch and the start choices) goes through the engine call; the picker header takes only a `Choice`. The nine hand-built pairs go.
- **The proof:** `every_choice` reads the engine's held-choices list, and `battery` adds each held choice's withdrawal plus deity starts and `Edit::FirstAscension`, so a new `Choice` is covered without editing the proof. (Ticket 96 later builds the battery from the engine's start options.)
- Test-first in `rules` (`tdd`): the held-choices list on the small world covers every `Choice` variant a Build can hold, including `Deity`, `FirstAscension` and `CharacterLevel`; the race withdrawal clears all three fields.

**Done when:**
- The rules tests above pass, and `Edit::choices`, `withdrawals` and the UI agree because the UI and proof have no copy left.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference, with the proof's added coverage reported as its own breakdown.
- The browser check (its own port): each remove control still removes, and is locked where it was.
- `scripts/check.sh` passes.

## Choices settled

1. **The interface:** `Engine::held_choices(&Build) -> Vec<Withdrawal>` (every held choice, removable or not, no walk) and `Engine::withdrawal(&Build, &Choice) -> Option<(Edit, bool)>` (`None` when not held, else the edit and whether it's removable now; one `Recheck`, as `is_removable`). `Withdrawal { pub choice, pub edit }` lives beside the other engine answers in `engine.rs`; the choices its edit excuses stay a private field, so only the engine builds one. `removable`, `is_removable` and `removable_classes` keep their signatures and now read `held_choices`.
2. **The character level is held as the last level only**, withdrawn by `Edit::RemoveLevel` (always removable: nothing after it depends on it). Earlier levels have no edit that takes back the level alone, and come off only after the ones above, leaves inward. So `removable` now also lists `CharacterLevel { level: last }` (proof below); no other answer changes.
3. **A level's skill points aren't held:** `Choice::SkillPoints` is what a level's ranks spend, which a walk fails, not a choice with a withdrawal. The covering test lists every other variant through an exhaustive `match`, so a new `Choice` fails to compile there until it says.
4. **`Edit::choices` and the withdrawals agree:** each withdrawal's edit names its choice, except Remove level, which can't know the Build's last level (a test).
5. **Remove controls:** the nine hand-built pairs went (slot, first ascension, path, switch, race, alignment, deity, background and the level's ability increase); `Head` takes `remove: Option<Choice>` and asks `Planner::withdrawal` (tracked) each time the Build changes, so the button applies the engine's current edit, not one built when the picker opened. Callers pass the choice whether held or not: the engine says. The class picker never had a remove control (a level's class changes through the picker, locked by `removable_classes` in the grid) and archetypes neither, so none was added; ticket text "class, archetype" read as the existing controls.
6. **Remove level** goes through the engine too: the grid asks `withdrawal(CharacterLevel { level: last })` for its edit and lock, in place of `allows(RemoveLevel)`. `Planner::allows` was then unused and went (`allows_many` stays for ticket 96 to replace); `Planner::is_removable` became `Planner::withdrawal`.
7. **The proof:** `every_choice` became the engine's held list plus `unheld_choices` (every start and Mythic choice, each level's increase, skill points, character level, archetype and advances, and the class and level past the end); held ones check `is_removable` and `withdrawal` against `removable`, unheld ones check `false` and `None`, and every removable choice must be held. `battery` gained the 27 faith starts (atheism and each deity), `FirstAscension(Some(path))` for each first-ascension option, and every held choice's withdrawal, which replace its hand-built race-withdrawal start, `Clear` per pick, `MythicPath(None)` and `PathSwitch(None)`. The record log counts the held choices checked, by kind.
8. **The proof harness was broken since ticket 94** (an alignment with no faith is no longer possible, so the groups that fill as they build found no open option and `export` panicked; the benchmark links with an alignment failed to open). The groups' start Builds (`character`, `neutral_human`, `selections::start`) are now atheists (through serde, as `generate::faithful`, so older crates still compile), and a benchmark link that opens as stale is decoded and made an atheist.
9. **Running the proof across this change:** the new harness can't compile against the parent's crates (no `held_choices`), so the rules comparison ran from a scratch clone holding the working tree's crates with the parent's `fingerprint.rs` (plus choice 8's fix) on both sides; the added coverage is the new harness's record compared with that side B (`tools/proof` crate docs say so).

## Progress log

- 2026-11-07: started at HEAD fe369c6. Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md`, this ticket and 96.
- 2026-11-07: tests first in `libs/rules/tests/engine/withdrawals.rs` (9 tests; `removable_builds` made `pub` to share). Red: the test crate didn't compile (no `Engine::held_choices` or `Engine::withdrawal`). Green after making the private `withdrawals` the public `held_choices`, adding the last level's withdrawal (choice 2) and `withdrawal`: `cargo test -p rules --test engine`: 436 passed.
- 2026-11-07: UI through the engine (choices 5, 6), proof harness (choices 7, 8, 9), `GLOSSARY.md` (**Withdrawal**), the map's Decisions line.
- 2026-11-07: rules proof, parent fe369c6 against the working tree, 2,162 Builds (scratch clone, choice 9): every difference is `removable` (2,160 Builds), and each only gains `CharacterLevel { level: <last> }` (the 2 others are the blank benchmark Builds, with no level); no other field differs (sheets, slots, class/path/switch pickers, `allows`).
- 2026-11-07: added coverage, new harness against the old one on the same crates and Builds: only `allows` differs; 1,110,199 edits became 1,276,499 (+166,300: 58,374 faith starts, 12,972 first ascensions, and 133,165 held-choice withdrawals in place of 38,211 hand-built ones). Held choices checked against `removable`, `is_removable` and `withdrawal`: Deity 2,161, FirstAscension 16, CharacterLevel 2,160 (all new), with Race 2,161, Alignment 2,161, Background 81, Subrace 4, AbilityScore 12,972, Class 28,392, Archetype 1,379, Advances 39, AbilityIncrease 4,002, SkillRanks 44,722, MythicPath 811, PathSwitch 379, Slot 31,725; no panic.
- 2026-11-07: browser check (127.0.0.1:3912, dev build, server stopped after), on ticket 91's `t91 fighter 20 trickster legend / first aeon` link: the faith had no remove control while Chaotic Neutral was set; alignment removed, then the faith's showed and removed it; a background chosen and removed; a level-1 combat feat removed; Power Attack's control hidden once Cleave needed it (and Fighter 1's class locked); the level-4 ability increase removed; the first ascension removed (MR 1 fell back to the path's) and its picker then showed none; the Path switch and the path removed; Remove level 20 removed it; the race (with its +2 on Strength) removed, Str back to 13; the class picker has no remove control, as before. No console errors.
- 2026-11-07: `scripts/check.sh`: ALL-OK; `scripts/check.sh --full`: ALL-OK.

## Answer

Done 2026-11-07. The engine owns the list of held choices and their **Withdrawal**s; the UI and the proof keep no copy.

- **Rules.** `Engine::held_choices(&Build) -> Vec<Withdrawal>` (`Withdrawal { choice, edit }`) lists every held choice, the last level's included (Remove level); `Engine::withdrawal(&Build, &Choice) -> Option<(Edit, bool)>` answers a remove control at `is_removable`'s cost. `removable`, `is_removable` and `removable_classes` read the list; `removable` now also lists the last level.
- **UI.** `Head` takes only a `Choice`; the nine hand-built pairs are gone, and the grid's Remove level asks the engine too.
- **Proof.** `every_choice` is the engine's list plus choices a Build may not hold; `battery` adds the withdrawals, the faith starts and the first ascensions. The harness runs again after ticket 94 (choice 8). Rules proof: no difference but `removable`'s last level; added coverage in the progress log.
- **Tests:** `libs/rules/tests/engine/withdrawals.rs`.
