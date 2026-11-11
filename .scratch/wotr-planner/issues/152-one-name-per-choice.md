# One name per choice: address, label and value key in the rules

Type: task
Status: resolved
Blocked by: 148
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 2, *Strong*). The user asked for a ticket for each strong candidate.

A Build's choices are `rules::Choice`. The MCP server holds a second naming of them, and the labels and value keys are written in several places with different casing.

- **Addresses.** `apps/mcp/src/address.rs` (370 lines) has its own `Address` enum. `Address::choice` maps it to `rules::Choice` one to one (`address.rs:63–91`), apart from `RacialAbility`, which the engine keeps with the race. It has its own grammar (`slot.feat.N.class.I`, `level.N.skill`: `Display` and `FromStr`), its own `place` and its own `label` (`address.rs:118–144`).
- **Labels.** The same choices are named in at least four places:
  - `Address::label`: "Race", "Racial ability bonus", "Strength (point buy)", "Level 4: ability increase", "MR 3: Mythic path".
  - `apps/mcp/src/sheet.rs`, which ignores `Address::label` and hard-codes "Class" and "Ability increase" (`sheet.rs:296, 306`).
  - `libs/ui/src/picker.rs:1063`: `<Head title="Ability increase" …>`.
  - "+1 {ability}" in both `apps/mcp/src/options.rs:217` and `sheet.rs:308`.
  - The rules crate has `Engine::slot_label` and about 40 `*_name` functions (`engine/labels.rs`), but nothing that names a non-slot choice.
- **Value keys from display names.** `apps/mcp/src/values.rs:41` builds an alignment's key from its display name (`name().to_lowercase().replace(' ', "-")`). `validate/declare.rs:191–210` parses it back by searching `Alignment::ALL` for a match. The faith's `"atheism"` is hard-coded in both directions (`values.rs:46–51`, `declare.rs:191–197`). Renaming a display name would break a key.

**Shape.** One codec in `libs/rules`, beside `engine/labels.rs`, owns for every `Choice`:

- its address (the grammar the MCP uses today, which agents and tests already rely on, unchanged), parsed and printed;
- its label (with its place: "Level 4", "MR 3");
- for a value without a game-data key (alignment, faith, ability), a stable key that isn't derived from its display name, in both directions.

`apps/mcp/src/address.rs` and `values.rs` fold into it, and the MCP's `todo`, `sheet`, `options` and `validate` take labels and keys from it.

- The web app's picker headings and the Build sheet's ([ticket 87](87-build-sheet-export.md), `engine/build_sheet.rs`) use its labels where they name the same choice, with the sentence case the UI uses today. Where the MCP and the UI word a label differently today, list each one in the Answer, take the UI's, and note it.
- `RacialAbility` needs an answer: either the codec gains it as an address beside `Choice`, or it becomes a `Choice` of its own. Say which, and why.
- Names are suggestions. Pick what reads best next to the glossary, and add the term to `GLOSSARY.md` if it's new.

Out of scope: [ticket 97](97-slot-options-presented.md)'s option names and summaries, and the link format (`codec.rs`). The address grammar doesn't change, so no agent instruction, skill text or test address changes.

## Build

AFK. A refactor: what tools return doesn't change, apart from any label casing the Answer lists.
- Blocked by [148](148-naked-numbers.md): both change `apps/mcp/src/{sheet,server,shapes}.rs`, so run this one after 148 has merged.
- `apps/mcp/tests/server/address.rs` unit-tests `mcp::Address`, the only reason `Address` is `pub`. It moves to the rules crate's tests and becomes a round trip over every `Choice` of the test world: address → parse → the same choice, and value key → parse → the same value.
- No schema, data or link-format change.

## Done when

- `apps/mcp/src/address.rs` and `values.rs` are gone (or each is a few lines of re-export), and `lib.rs` re-exports no `Address`.
- A grep of `apps/mcp/src` and `libs/ui/src` finds no hard-coded choice label ("Ability increase", "Class", "+1 {}"), and no key built from `name()`.
- The rules crate's round-trip test covers every `Choice` variant.
- The MCP server tests pass, unchanged apart from any label casing the Answer lists.
- `cargo test --workspace -- --list` lists the same tests, apart from the ones moved.
- `scripts/check.sh` passes.

## Progress log

- 2026-11-11: worktree fast-forwarded to `main`'s `8d8be9b` (ticket 148 merged); own `.env` and database. Baseline `cargo test --workspace -- --list` saved before any edit (1,113 tests).
- 2026-11-11: test first. `libs/rules/tests/engine/addresses.rs` written against a `rules::Address` that didn't exist yet (27 compile errors), with the MCP's four address tests moved into it; then `libs/rules/src/engine/address.rs` made it pass (10 tests).
- 2026-11-11: the MCP server moved onto it: `apps/mcp/src/address.rs`, `values.rs` and `tests/server/address.rs` deleted; the web app's picker headings, start rows, Mythic chips and level chip, and the build sheet's start lines, take the labels from it. MCP server tests (148) pass unchanged.
- 2026-11-11: checks, proof and the test-list diff (see Answer).
- 2026-11-11: review fixes, test first where they change behaviour: the faith labelled "Faith"; the Build sheet's v1 line keys fixed again, with a test that its text is byte for byte what `8d8be9b` wrote (captured by running that commit's `build_sheet.rs` beside the new one); the class picker's advancement step keeps its level; one racial-ability label for the web app and the MCP server; a refused slot labelled by the sheet's slot; `placed` on a level; one feat-source table both ways; `Faith::from_key` refusing a blank key; the class value's name in `labels.rs`; the round trip's parameter kinds exhaustive, and Metamagic covered on the real data (a metamagic feat in the test world broke 15 `pick_values` and `canonical` tests, so it isn't there).

## Answer

**Done.** A choice's address, its label and the keys of its values without a game-data key live in one place, `libs/rules/src/engine/address.rs` (beside `engine/labels.rs`), as `rules::Address`. The address grammar is unchanged, and so is every MCP answer apart from the label wording listed below.

### The codec

- **`Address`**, the MCP's enum moved as it was: one variant per choice, `choice()` (to the engine's `Choice`), `From<&Choice>`, `Display` and `FromStr` (the ticket 104/107 grammar, `NotAnAddress` for anything else, now also a `std::error::Error`).
- **`Place`** (`Start`, `Level(n)`, `Rank(r)`) from `Address::place()`: `Display` is how a label begins ("Level 4", "MR 3"), `heading()` how a picker's heading says it ("Build start", "Level 4", "Mythic rank 3"). A class's archetype and advanced classes have none. `Address::placed(what)` puts a level's or rank's place before a text ("Level 3: 2 of 4 skill points unspent"); a level itself is its own place, so `placed` leaves its text alone (never "Level 4: Level 4").
- **Labels**, `impl Engine`: `choice_label(address, slot)`, sentence case, unplaced, as the sheet's lines and the pickers' headings name a choice ("Class", "Ability increase", "Athletics ranks", a slot's `slot_label`); `placed_label(address, slot)`, as `todo` and `validate` name it ("Level 4: Ability increase", "MR 3: Mythic path"). `Ability::increase_name()` ("+1 Strength") and `increase_short_name()` ("+1 Str", the level's chip).
- **Value keys**, written out per variant, never derived from a display name: `Ability::key`/`from_key` (`str` …, and the full name in any case as before), `Alignment::key`/`from_key` (`lawful-good` …), `Faith::key`/`from_key` (a deity's key, or `atheism`; `None` for an empty or blank key, which `validate` refuses as `not_an_option`), and `Parameter::value_key` with `Engine::parameter_from_key(kind, key)` for a school, energy type, armour type, stat or element (`attacks-of-opportunity`, `negative-energy`). The keys are the same strings agents already used (a test pins them). A feat slot's source token (`race`, `class`, `general`) comes from one table, `FEAT_SOURCES`, read both ways and checked whole when compiled, so a new `FeatSource` fails to compile until it has a token.

### Where it's used

- **MCP server**: `address.rs` and `values.rs` are gone and `lib.rs` re-exports no `Address`. `todo` (`code.rs`), `validate` (`wording.rs`, `tree.rs`) take `placed_label`, `validate` with the sheet's slot for its declaration's refusals too (`Wording::label`, so a refused slot is "Level 3: Feat" like a failed one, not "Level 3: slot"); `sheet` takes `choice_label` for every line (its hard-coded "Race" … "Class", "Ability increase", "{skill} ranks", "+1 {}" are gone); `options` and `sheet` write values with the keys above; `validate`'s declaration reads them back with `from_key` and `parameter_from_key`. What `values.rs` built from MCP shapes became `ChoiceValue::{pick, ability, race, class}` in `shapes.rs`, and the class value's name ("Armiger (Fighter)", "Eldritch Knight, advancing Wizard") is `Engine::class_value_name` in `engine/labels.rs`, which `options` and `sheet` use; the Build sheet shares its ", advancing …" (`Engine::advancing`) and keeps its own v1 order for the rest ("Fighter (Armiger)").
- **Web app**: every picker heading takes its title and place from the codec (`picker::heading`): race, deity, alignment, background, ability increase, class, slot, first ascension, path, switch, and the skill ranks panel. The Build start's rows, its racial ability's field and the Mythic track's chips take `choice_label`; the ability increase's items `increase_name`, its grid chip `increase_short_name`. The class picker's advancement step heads "Level N: Spellcasting advancement".
- **Build sheet** (`engine/build_sheet.rs`): none of the labels. Its line keys ("Name", "Race", "Subrace", "Racial +2", "Deity", "Alignment", "Background", "Point buy", "First ascension", "Mythic path", "Path switch", and a level's "Ability" and "Skills") are its own constants, the "WotR build v1" format's, so rewording a label never changes an export. `build_sheet::the_v1_text_is_byte_for_byte_the_one_ticket_87_wrote` pins five test-world Builds' text against what `8d8be9b` wrote.

### Decisions

- **`RacialAbility`: an address beside `Choice`.** The ticket's lean. The engine keeps the racial ability with the race: the walk checks it with the Build start, and a race's Withdrawal takes it along. A `Choice` of its own would reach into the walk's failures, `removable` and the withdrawals, a behaviour change this refactor doesn't need; as an address the codec knows, it has a label and a key like any choice, and `choice()` is `None` for it alone (the MCP's `withdraw` still refuses it, as before).
- **Name: `Address`**, as the MCP, its tests and the skill already called it. Added to `GLOSSARY.md` (callers see it in every tool).
- **Places**: "MR 3" in a label, as the ticket's shape and the MCP's labels have it; "Mythic rank 3" in a picker's heading, as the web app has it. Both come from one `Place`.
- **Racial ability: one label, "Racial ability bonus"**, the codec's, for the MCP server and the web app (whose field said "+2 to"); the Build sheet keeps "Racial +2:", its v1 key.
- **Value keys for parameters** too: they are values without a game-data key like the three the ticket names, and the MCP built them from display names the same way (`key().to_lowercase().replace(' ', "-")`).

### Label changes (old → new)

Where the MCP and the web app worded a label differently, the web app's wording was taken, except the faith's and the racial ability's (below):

| Choice | MCP before | Now (MCP and web app) | Where the MCP shows it |
| --- | --- | --- | --- |
| `level.N.class` | Level N: class | Level N: Class | `validate` |
| `level.N.increase` | Level N: ability increase | Level N: Ability increase | `todo`, `validate` |
| `level.N.skill` | Level N: skill points | Level N: Skill ranks | `validate` (`todo` keeps "N of M skill points unspent") |
| `mythic.first-ascension` | MR 1: first ascension | MR 1: First ascension | `todo`, `validate` |

The faith keeps the MCP's "Faith", the glossary's term (a deity, or atheism: "Deity: Atheism" would be a category error), and the racial ability the MCP's "Racial ability bonus". The web app changes with them and in its class picker, which heads like the others (title and place):

| Web app before | Now |
| --- | --- |
| "Deity" (Build start row, picker heading) | "Faith" |
| "+2 to" (racial ability field) | "Racial ability bonus" |
| "Class at level N" | "Class" over "Level N" |
| "Level N: add a class" | "Add a class" over "Level N" |
| "Class at level N" over "Spellcasting advancement" | "Class" over "Level N: Spellcasting advancement" |

No MCP server test asserted any of the MCP's changed labels, so none changed; `builds.rs` now pins "Faith".

### Done when

- `apps/mcp/src/address.rs` and `values.rs` are gone; `lib.rs` re-exports no `Address`.
- Grep of `apps/mcp/src` and `libs/ui/src` for `"Ability increase"`, `"Class"`, `+1 {`, `"Faith"`, `"atheism"` and `to_lowercase().replace`: only doc comments and a `validate` unit test's declared value (`"start.faith": "atheism"`). (`rank.rs`'s lower-cased subschool and energy are spell descriptions, not keys.)
- Round trip: `addresses::every_choice_of_the_test_world_reads_back_from_its_address` takes every choice of the test world (the Build start's, each ability score, levels 1–40 with each skill's ranks and every level slot kind, each class's archetype and advanced classes, MR 1–10 with every rank slot kind) through address → text → parse → choice, and checks all 17 `Choice` variants are among them (an exhaustive `match` fails to compile on a new one). `every_value_without_a_game_data_key_reads_back_from_its_key` does key → parse → value for every ability, alignment, faith (atheism and each deity) and each parameter kind's values, the kinds from an exhaustive `match` too. The test world has no metamagic feat, so `seeds`' `game_data::value_keys` does the same on the real data, Metamagic included.
- MCP server tests: 148 pass, unchanged; the review fixes added one (149).
- Test list: the same apart from the moved tests. Gone from `mcp`'s `server`: `address::{an_ability_may_be_named_in_full, every_engine_choice_has_an_address_and_back, every_kind_of_address_is_written_and_read_back, text_outside_the_grammar_is_no_address}` (152 → 148). New in `rules`' `engine` (567 → 577): `addresses::` the three kept by name, `a_key_is_read_in_any_case_and_an_ability_by_its_full_name` (the full-name test, with keys in any case), and six new: `every_choice_of_the_test_world_reads_back_from_its_address`, `every_value_without_a_game_data_key_reads_back_from_its_key`, `value_keys_are_the_ones_agents_already_use`, `each_choice_is_labelled_in_sentence_case_after_its_place`, `a_place_is_short_in_a_label_and_long_in_a_heading`, `an_ability_increase_is_named_by_its_ability`. The review fixes added `rules`' `engine` `addresses::an_empty_or_blank_key_names_no_value`, `build_sheet::the_v1_text_is_byte_for_byte_the_one_ticket_87_wrote` and `labels::a_class_value_is_named_with_its_archetype_and_what_it_advances`; `mcp`'s `server` `validate::an_empty_faith_is_no_faith`; and `seeds`' `game_data` `value_keys::{every_parameter_value_reads_back_from_its_key, every_metamagic_feat_is_a_metamagic_value_by_its_own_key, every_faith_reads_back_from_its_key}`.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm clippy, tests, rules wasm, seed check).
- Proof: `scripts/proof.sh rules` against HEAD `8d8be9b`: 2,163 Builds on each side, none differing; `compare.txt` empty. (The proof records the engine, and the engine's answers didn't change; the label changes are the MCP's and the web app's.) After the review fixes, `scripts/proof.sh rules` against `ec384e3`: 2,163 Builds on each side, none differing; `compare.txt` empty. `scripts/check.sh`: ALL-OK again.
- No schema, data or link-format change.
