# Races

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: none
Model: Sonnet to build: heritages and familiarities read from the race blueprints.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `follow_ups.rs`; half B part 4) differ on `races.ron`:
- **The half-elf's heritage** (`HalfElfHeritageSelection` 9df7b68d60544bcf8e5b56c0a4688e04) offers Adaptability, Dual-Minded or Kindred-Raised; the planner has Adaptability only (its Skill Focus list is [255](255-a-parametrised-feat-at-one-value.md)'s). Adaptability's list doesn't ignore prerequisites in the game.
- **Weapon familiarity (seen, not asserted):** a Human-Raised half-orc keeps the double axe as martial (keyed to `HalfOrcRace` 1dc20e195581a804890ddc74218bfd8e); the halfling's (`HalflingRace` b0c3ef2729c498f47970bb50fa1acd30) Sling Staff as martial is missing.

## Findings (the game check, `known.txt` section 291)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / ignores prerequisites or not | 1 |
| **all** | **1** |

The 1 entries: `feat list half-elf/adaptability`.

The report lists each finding under its group, marked `[known: 291]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The half-elf's two missing heritages join; Adaptability's list stops ignoring prerequisites; the two familiarities become the game's; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `follow_ups.rs`: the half-elf's race slot test (two heritages missing).
- `option_follow_ups.rs`: `a_dual_heritage_or_kindred_raised_half_elf_has_no_adaptability_pick` (re-read with the game's heritages).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 291 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
