# The importer: the game data's mechanical fields from the blueprints

Type: design, then build
Status: open
Blocked by: 244 (it writes the per-blueprint files); 295–298 and 301–305 (S1 starts once 305 merges) (the component vocabulary, the data forms it writes and the link migration; user 2026-11-15 and 2026-11-16); the engine-rule tickets 247–254 and 256 (the importer writes data in the shapes they settle)
Model: Opus to build: one bug touches thousands of entries, so it is built and reviewed with care.
Map: [WotR Build Planner](../map.md)

## Staged (2026-11-16)

The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md) (main session's design research; the user's three answers at its end: names in our own words, the game's rows for step grants, carriers now and the full import later). It splits the importer into five stages, each its own ticket, review and merge:
- **This ticket is S1:** the `--write` mode of `game-check` (§1), the user-fact guard and its registry `tools/game-check/user-facts.txt` (§3), the change log, fixture tests; and it writes the class tables (with a new archetype `bab` field), the items (including the +8 headband the user approved, missing from `gear/`) and the features' `ClassSkill` 262 needs. About 67 findings.
- **S2–S5** are [306](306-importer-selections.md) (selections and the per-class splits), [307](307-importer-carriers.md) (the carrier entries), [308](308-importer-prerequisites.md) (prerequisites) and [309](309-importer-rows.md) (rows). Each proves itself as the design's §4.2 says.
- **The gap tickets** G1–G9 (§2.8) are numbered from 312 and filed at the queue's close. [310](310-corpus-draws-per-slot.md) makes the corpus's generated Builds stable under an unrelated offer change, so each stage's census shows only its own changes.
- **Follow-ups the form steps left here:** 302's Extra feats keep a first-had list and a `needs_slot_of` gate (the game opens the slot whenever the feat is taken: 306); `once_per_option` must reach across selections for the third elemental focus split (306); 305 keys held facts by kind and key, and 90 game blueprints are two planner entries (308 or a gap ticket); feature ranks uncapped since 243 (309, if 303 hasn't). From 303 (its review): Witch Wandering Hex carries a planner `plain_feature` (the game's WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 is a plain feature of Ranks 1, the planner's a selection); six options carry a hand-written `is_selection` (Combat Trick, the rogue Weapon Focus, both Skill Focus talents; planner Finesse Training and Battle Scion's teamwork feat), which the check should compare with the blueprint's type; and "no option left" ignores the options' prerequisites where the game's `FeatureSelectionExtensions.CanSelectAny` checks them (306).

The sections below are the ticket as first written; where they differ from the staging, the staging wins.

## Problem

Tickets 247–291 filed the 3,908 differences the game check found (238) against the data. 31 of them (261–291) hand-edit the data, one class file at a time, about 3,600 findings.
- **The game check already does the hard part.** It reads every prerequisite, progression row, option list, class table and item bonus from the blueprints, and maps each onto the planner's model.
- **Hand-copying them is slow and error-prone.** It's 30 Opus builders with review rounds, retyping values the computer already reads.
- **It would be done twice.** 244 rewrites every data file into one file per blueprint, so a fix made today to Sneak Attack's 13 copies would be redone there.

The user, 2026-11-15, agreed to replace the hand edits with an importer. The order is: the engine rules first, then 244, then this ticket. The modelling gaps it can't express become their own tickets.

234 allows it:
- code of our own that reads the community's `.binz` (item 4);
- facts committed in our own form, with blueprint names and GUIDs as citations;
- never the game's text strings, dump or code.

## Change

- **A writer beside the game check** in `tools/game-check`, or a sibling crate sharing its reader and its mapping. For each entry that names a blueprint, it writes the mechanical fields the check compares:
  - prerequisites;
  - class tables: BAB, saves, skill points, class skills;
  - progression rows and archetype additions and removals;
  - selection option lists;
  - mythic ranks;
  - item bonuses.

  It writes them into the entry's per-blueprint file (244), and reads them from the game as the check does, through the same normalisation.
- **What it never touches:** the planner's keys, names, summaries, guide links, `interactions`, the data notes, and the planner-made entries (`planner/`). It never writes the game's text strings.
- **What it can't express** (a component the planner's model has no form for) is left as it is. It is listed per kind, and the list becomes the modelling-gap tickets. The game check keeps reporting those findings, under their tickets.
- **Idempotent and deterministic.** It is a run by hand, like the check. A second run writes nothing, and its output is the same for the same dump and data.
- **After a run, the game check reports only the gaps.** `known.txt` loses each section of 261–291 whose findings are gone.
- **The tests waiting on 261–291** come off `#[ignore]` where the data now holds the game's value. Each one still waiting names its gap ticket.
- **The user's facts.** The seven decided on 2026-11-15, 262's class skills and 276's Improved Uncanny Dodge are applied by the run like any value. The importer never overwrites another of the user's facts: those differences go to the user.

- **The per-class selection splits and remaps** (user, 2026-11-16, "B: forms now, splits in 293"; from [298](298-selections-as-the-game-has-them.md)): `known.txt` section 293 holds them, 244's leftovers. The run writes each per-class selection as the game has it, read from its blueprint, with the game's own option list. An option key a link names that a split changes gets an alias, so 298's migration opens the old link. Research to start from: 298's build notes and `game/298/research/selections-spec.md` in the main session's scratchpad.
  - **Identify each selection through what grants it,** never by name: `game-check refs` on the progression or archetype. Two blueprints are named `BardTalentSelection`, and only 94e2cd84bf3a8e04f8609fe502892f4f is the bard's (298's correction).
  - **A list the game makes shorter removes offers** (the rogue-talent copies, the slayer's lists, the Instinctual Warrior's rage powers). That is applied as the game has it (234); a link whose pick the game no longer offers opens with that pick refused, as 251's game-correct changes did. The Answer counts the corpus Builds this touches.
- **Gap tickets** are numbered from 306 on.

## To decide (recommendations; the user's "build them all, in order" applies)

1. **Where it lives.** Recommended: a `--write` mode of `game-check`, sharing its reader and mapping, so the two can never disagree on what a field means.
2. **What a run may change.** Recommended: only the fields listed above, and only on entries with a blueprint. Any other change is a bug the run refuses.
3. **Review.** Recommended:
   - a sampled hand check of 100 entries across kinds against the blueprints;
   - the corpus self-checks' long runs;
   - the gold builds, with every change to a gold build's gains shown to the user.

## Done when

- One run brings the data to the game's values in the fields above. The game check's remaining findings are exactly the listed gaps, each under a ticket. Two runs: the second writes nothing.
- The tests that waited on 261–291 pass, or name a gap ticket.
- The self-checks' long runs and the gold builds pass, or their changes are shown to the user.
- The scoped checks and the game check pass.
