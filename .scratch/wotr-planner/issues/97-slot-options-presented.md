# Slot options come back presented

Type: task
Status: resolved
Blocked by: none (runs alongside [98](98-check-validates-references.md), in its own worktree)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 3) found the slot picker's presentation split between the UI and the engine's labels:

- The UI groups a slot's options by item (`groups`, `libs/ui/src/picker.rs` ~349), builds the "(Story: …)" note, picks a group's reason (`group_reason` ~522: anything over "Already taken"), and makes about five engine label calls per option.
- `value_noun` (~536) repeats `ParameterKind::noun` with articles added.
- The deity picker strips " Domain"/" Subdomain" from option names itself (~1017).
- Inside `rules`, five matches over `Pick`'s four variants in `libs/rules/src/engine/labels.rs` and `pickers.rs` (plus one in `slots.rs`, for checking) only reach the picked item's name, category, summary or prerequisite.

Tickets 95–96 moved every other control's rules into the engine; this is the slot picker's turn. The user chose to build it (2026-11-08) without a design round: settle small choices yourself, record them, and bring a real open question back.

## Build

- **One lookup for "the item a pick takes":** the catalog (or engine) resolves a `Pick` to its item once, and the label and picker code read the item instead of matching `Pick` each time. The checker's match in `slots.rs` may stay if it checks something the item can't say.
- **Slot options come back presented:** the engine returns a slot's options already grouped by item, each group with its name, category chip, summary, story note and the one reason it shows, plus each member's own status and edit (from ticket 96). The UI renders them. Pick the shape (a new type beside `SlotOption`, or `options` itself changing) and record why; `SlotOption` keeps its status and edit.
- **The reason rule** (`group_reason`'s priority) and the story note move into `rules` and get tests.
- **`value_noun` goes:** the UI uses the engine's noun (add the article form to `ParameterKind` or labels if the UI needs it).
- **Domain names:** the engine gives the name the deity picker shows, so the UI strips nothing.
- Test-first in `rules` (`tdd`), on the small world: grouping, the reason priority, the story note, the noun, and the domain name.

**Done when:**
- The rules tests above pass; `libs/ui` has no `groups`, `group_reason` or `value_noun`, and makes no per-option label calls to build a slot list.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- The browser check (a release build on its own port): slot pickers show the same groups, chips, notes and reasons as before, no slower by the `timed(...)` logs, and no console errors.
- `scripts/check.sh` passes, and `--full` once at the end.

## Choices settled

1. **The shape: a new type beside `SlotOption`.** `Engine::slot_entries(&Build, &SlotId) -> Option<Vec<SlotEntry>>` (in `pickers.rs`) groups `Engine::options` by what the pick takes, in the options' order. `SlotEntry { name, category, summary, story_note, noun, open, reason, options: Vec<EntryOption> }`, `EntryOption { option: SlotOption, value, reason }`, and `SlotEntry::detail()` joins "Choose a weapon.", the summary and the story note as the item's line. `options` itself stays: it is the flat, checkable list the proof, the edits and about 80 tests read, and `SlotOption` keeps its status and edit unchanged inside each entry. The name "Option group" was taken (the glossary's family of options held once), so the type is a slot *entry*; no glossary term changes.
2. **One lookup for the item a pick takes:** `Catalog::pick_item(&Pick) -> Option<PickItem>` (name, chip, summary, prerequisite, the kind of value it names, the feat holding it holds) is the one match over `Pick`'s variants outside the walk and the checker. `pick_name`, the slot entries and `named_feats` read it; `pick_base_name` is private, and `pick_category`, `pick_summary` and `story_conditions` left the public interface (the UI was their only caller). The checker's match in `slots.rs` stays: it checks per-variant rules (slot kind, repeats, value lists, deity domains) the item can't say.
3. **The reason rule** moved as it was: a locked entry shows the first option's reason whose issue isn't `AlreadyHeld` or `MaxTimes`, else the first option's; none when that one is `BreaksLater`, and none for an open entry. Each option carries its own reason too (the value list shows them), and an option failing with the same issue as the one before it reuses its text, so a 73-weapon Weapon Focus formats one "Needs BAB +1".
4. **The noun:** `ParameterKind::with_article()` beside `noun()`, read off the item's kind. Exotic Weapon Proficiency now says "Choose an exotic weapon." where the UI said "Choose a weapon." (its `value_noun` saw a `Parameter::Weapon`); the only visible change, and the browser check compared everything else.
5. **Domain names:** `Engine::domain_name(&SelectionOptionKey)` strips " Domain" or " Subdomain"; the deity picker calls it per domain (that's the deity list, not a slot list).
6. **The UI renders:** `SlotPicker` keeps its after-paint fill and its one `timed("options", …)`, now around `slot_entries`, so that timer covers the grouping and labels the parent did outside it. It still compares an option's pick with the slot's for `current` (not a status). `groups`, `group_reason`, `value_noun` and the per-option `pick_*`, `story_conditions`, `parameter_name` and `issue_text` calls are gone from the slot picker.
7. **Batching kept:** one `options` call (one walk to the slot, its rechecks) per picker, as before; the grouping only reads its result.
8. **Tests moved:** `labels.rs`'s category test became `slot_entries.rs`' shared-option chip test; `class_rules.rs`' story-condition note test became the slot entries' story tests; the real-data Gold Dragon breath test (`libs/seeds/tests/game_data/class_rules.rs`) reads the breaths' story notes off `slot_entries`.
9. **The proof ran directly:** `tools/proof` calls none of the removed methods, so the working tree's harness compiled against the parent's crates and no scratch clone was needed.

## Progress log

- 2026-11-08: started at HEAD aede2ac, clean tree. Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md`, this ticket and 96.
- 2026-11-08: test-first on the small world, one slice at a time (`libs/rules/tests/engine/slot_entries.rs`, `deities.rs`): grouping, chips and values (3 red against a stub returning no entries, then green); the reason priority (3 red, green); the story note and detail (2 red, green); the noun with its article (3 red against a stub without the article, green); the domain name (1 red, green). `cargo test -p rules --test engine`: 469 passed.
- 2026-11-08: UI onto `slot_entries` and `domain_name`. `scripts/check.sh`: ALL-OK.
- 2026-11-08: rules proof against aede2ac (`scripts/proof.sh rules … aede2ac`), 2,162 Builds: `A: 2162 Builds, B: 2162; only in A: 0, only in B: 0, differing: 0`.
- 2026-11-08: browser check on release builds made with `--lib-features timings` (working tree on 127.0.0.1:3915, parent on 3916; both stopped after). The pane was hidden, so the harness replaced `requestAnimationFrame` with a zero timeout to let the after-paint fill run; the machine was loaded (load average ~40 from other builds), so the timings were taken interleaved, tab by tab.
- 2026-11-08: `scripts/check.sh`: ALL-OK; `scripts/check.sh --full`: ALL-OK.

## Answer

Done 2026-11-08. A slot's picker list comes from the engine already presented; the UI only renders it.

- **Rules.** `Engine::slot_entries` returns `SlotEntry`s (name, chip, summary, story note, noun, open, the one reason) holding `EntryOption`s (the `SlotOption` with its status and edit, the value's name, its reason). `Catalog::pick_item` is the one lookup of the item a pick takes. `ParameterKind::with_article` and `Engine::domain_name` are new; `pick_category`, `pick_summary`, `story_conditions` and the public `pick_base_name` are gone.
- **UI.** `libs/ui` has no `groups`, `group_reason` or `value_noun`, and builds a slot list from the entries alone; the deity picker strips nothing.
- **Proof.** No behaviour difference over 2,162 Builds.
- **Browser** (release, working tree against parent, every picker list compared item by item: name, detail, reason, locked, current, and the chips): identical on full fighter's Combat feat (Weapon Focus held: its value list with Heavy Mace current and the values already taken), the level-1 and level-17 general slots, the Bonus feat, the Exotic Weapon Proficiency and Improved Critical value lists (EWP's "Choose an exotic weapon." apart, choice 4); full cleric's two Abadar-limited domain slots ("Not a domain of Abadar", Earth already taken in the other) and level-5 feat; the deity picker's domains ("Earth, Law, Nobility, Protection, Travel"); a fighter 17 Trickster → Gold Dragon's two breath slots (the four "(Story: …)" notes, "Needs Mythic rank 9 (have 8)" at MR 8, open at MR 9) and a mythic ability slot. Picking Empowered Breath filled its slot. No console errors.
- **Timings** (`timed(...)` medians of 7 interleaved opens on full fighter, parent `options` plus its grouping → working tree `options`, which now includes the grouping and labels): Combat feat 6.2 → 6.2 ms, level-1 general 7.1 → 6.9, level-17 general 5.1 → 5.0. The parent's grouping alone was 0.1–0.2 ms.
- **Tests:** `libs/rules/tests/engine/slot_entries.rs` (12), `deities.rs`' domain name; moved tests in `labels.rs`, `class_rules.rs` and the real-data `class_rules.rs`.
