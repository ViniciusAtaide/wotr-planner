# One name per choice: address, label and value key in the rules

Type: task
Status: open
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
