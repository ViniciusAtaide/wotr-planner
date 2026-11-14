# Two data guardrails: lists that name a selection's options, and archetypes in the snapshot corpus

Type: tooling
Status: resolved
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

## Answer

User and main session, 2026-11-14 ("do every recommendation"; the main session settled the open questions of the research phase): the literal rule with an exception table, no test of sharers, the eight archetypes as named.

1. **What was built.** `libs/seeds/tests/game_data/option_lists.rs` (four tests) and eight archetype Builds in the snapshot corpus (`ARCHETYPE_BUILDS` in `snapshots.rs`, 44 Builds become 52).
2. **The rule.** For every feat with `extra_pick_of` whose prerequisite names a selection option, the prerequisite names every option its extra pick may be of (the own and shared options of each selection it names; Extra Rage Power's two and the Battle Scion's 38 shared are the union), less an exception table `(feat, option-key prefix, reason)`, and names none the pick can't be of. A second rule covers every other prerequisite of a feat, mythic feat or mythic ability: an `Any` naming two or more of one selection's own options names all of them (Extended Aspects, Second Mystery, Witch Wandering Hex and the rest passed with no exception). Options', classes' and archetypes' `Any`s are out: the kineticist's elements, the plague mutations and the Dragon Disciple's ten Draconic bloodlines are deliberate subsets.
3. **The feats of the shape and the gaps.** Nine feats have `extra_pick_of`; seven hold an `Any` of options; Extra Slayer Talent (a slayer level) and Extra Wild Talent (a kineticist level) name none and are pinned as such. No gap is a bug, so no data fix: the four exceptions are the Battle Scion's Teamwork Feat (a teamwork feat is no rage power, ticket 189), and the witch hexes in Extra Arcana and Extra Rogue Talent and the rogue talents in Extra Discovery, each reached through a Hexcrafter, Sylvan Trickster, Vivisectionist or Underground Chemist branch of the prerequisite (ticket 64).
4. **Sneak Attack.** Accomplished Sneak Attack's and Mythic Sneak Attack's lists each name the 14 class, archetype and path features whose key ends in `/sneak-attack` or `/sneak-attack-<archetype>`; a test holds both to that set (the Knife Master's Sneak Stab and the like don't match).
5. **Left out, by decision.** No sharer-pin test (a `shared_options` list is a deliberate subset). The Stonelord (a dwarf, with a subrace perhaps) and the Defender of the Hearth have no Build of their own; a later ticket. An extra-pick feat whose prerequisite is met through another selection's slot is ticket 197's item 15.
6. **Redundant assertions deleted:** `extra_rogue_talent_counts_every_rogue_talent_option` and its helper (`borrowed_selections.rs`), the Extra Rage Power list assertion (`instinctual_rage_powers.rs`), its Teamwork Feat half (`skald_rage_powers.rs`), Extra Arcanist Exploit's list assertion (`mercies_and_exploits.rs`). The behavioural tests stay.
7. **The archetype Builds** (pick-less, level 20, with open options, named `class-<class>-<archetype>`): Invulnerable Rager (426 lines), Standard Bearer (425), Feyspeaker (427), Two-Handed Fighter (423), Hospitaler (433), Eldritch Scoundrel (437), Battle Scion (450), Shieldbearer (450). All were possible pick-less: each has no condition of its own and took only its class's alignment and faith (neutral good and atheism for the Rager and the Feyspeaker, lawful good and atheism for the Hospitaler, Abadar for the Shieldbearer, none for the others).
8. **Data fix riding along:** the wizard's `wizard/school` guide link `wizard#wizard` (no such anchor) is the page link `wizard`: `libs/game-data/notes/204-wizard-guide-link.md`.
9. **Tests:** `extra_feats_name_every_option_of_their_selections`, `every_exception_covers_an_option_the_prerequisite_leaves_out`, `an_any_of_a_selections_options_names_all_of_them`, `the_sneak_attack_feats_name_every_sneak_attack_feature`; all four are in `-- --list`. With an empty exception table the first fails naming the 26 hexes, the 36 rogue talents, the 26 hexes and the Teamwork Feat. Removing Blinding Strike from Extra Rogue Talent's `Any` fails the first and the third naming `rogue/rogue-talents/blinding-strike`; the data was restored (`git diff -- libs/game-data/feats.ron` empty).
10. **Snapshot message** before regenerating: "differs from the snapshots in 8 of 52 Builds", the eight as "Missing snapshot files … no snapshot file (a new Build)", no changed line. The README's snapshot paragraph describes the new corpus. **The queue's proof** should show nothing from this ticket: no engine or data behaviour moved.
