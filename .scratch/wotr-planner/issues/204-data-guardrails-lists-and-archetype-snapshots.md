# Two data guardrails: lists that name a selection's options, and archetypes in the snapshot corpus

Type: tooling
Status: open
Blocked by: [195](195-archetype-rows-keep-what-the-base-row-gives.md) (same test crate; one after the other)
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retrospective of the queue of tickets 188 to 193 (2026-11-14; user: "do every recommendation"). Two mistakes that queue's reviews caught, which a test would have:

1. **A list that names a selection's options fell behind the selection.** Ticket 190 added two options to `rogue/rogue-talents`, and Extra Rogue Talent's prerequisite, an `Any` of the selection's options, wasn't extended: an Eldritch Scoundrel holding only Uncanny Dodge was refused the feat until the code review found it. Extra Rogue Talent and Extra Rage Power each have a test of their own now; no test covers the shape.
2. **The snapshot corpus has no Build with a class archetype,** so the per-ticket snapshot check said nothing for tickets 190 and 193 and for most of 188 and 189. (The corpus's two archetype Builds are a gold build and a sorcerer.)

## Build

Real-data tests in `libs/seeds/tests/game_data/`; no data change.

- **One data-wide test for the lists.** For every feat with `extra_pick_of`, when its prerequisite holds an `Any` whose children are options of one of those selections, the `Any` names every option of the selection (its own and its shared ones), less an exception table in the test, each entry with its reason (the Battle Scion's Teamwork Feat is not a rage power). Find every feat of that shape first and say which had gaps. If another kind of list has the same hazard and can be checked mechanically, add it; `shared_options` lists are deliberate subsets and stay out.
- **Archetypes in the snapshot corpus.** Add one Build per class for the classes whose archetype rows the stepping tickets edited most, each with the archetype that differs most from its class: barbarian (Invulnerable Rager), rogue (Eldritch Scoundrel), fighter (Two-Handed Fighter), skald (Battle Scion), paladin (Hospitaler), druid (Feyspeaker), cavalier (Standard Bearer), warpriest (Shieldbearer). About eight new snapshot files, named so they sort beside their class. Read how the corpus is built (`snapshots.rs`) and add them the way the `class-*` Builds are made.

## Done when

The list test fails when an option is removed from Extra Rogue Talent's `Any` (try it, then put it back) and passes on the data as it is; the new snapshot files are committed and the snapshot check's message names only them as added; `scripts/check.sh` passes.
