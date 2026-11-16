# The importer, stage 2: selections and the per-class splits

Type: build (data, through the importer)
Status: open
Blocked by: 293
Model: Opus to build: one bug touches hundreds of entries.
Map: [WotR Build Planner](../map.md)

## Problem

Stage S2 of the importer ([293](293-the-importer.md); staging decided 2026-11-16, user: "Yes, start it", then the three answers in the design). The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md): its §2 area for this stage, §3 (the user's facts and the registry), §4.2 (each stage's proof) and the user's answers at its end.

## Change

- `--write` writes the selection option lists from the game (§2.3), the ~65 per-class splits and the rows that grant them, the Extra feats' own lists (302's first-had lists and `needs_slot_of` become the game's: the game opens the slot whenever the feat is taken), and new option entries, each named with a one-line summary in our own words (user, Q1). Hidden carriers are S3's.
- Each split that changes a key a link names gets an alias, so 298's migration opens the old link. Identify each selection through what grants it, never by name (298's correction).
- The offers the game's lists remove are applied (game wins); the 62 open corpus Builds that hold one open with the pick refused, listed. 298's link test changes to "the same picks, an alias heir, or a refusal this stage lists".
- `once_per_option` generalised across selections where the third elemental focus split needs it (302's Q5).

## Done when

- The design's §4.2 proof, all nine points: the game check exits 0 with this stage's `known.txt` lines gone; a second `--write` writes nothing; the change census attributes every differing line to a change-log row (0 unattributed); old links counted three ways, each refused one listed by Build; the sampled hand check of 100 fields; the self-checks' long runs, alone on the machine; every gold-build change shown to the user before the merge; the ignored tests of this stage's areas come off ignore or name their gap ticket.
- The scoped checks pass. Estimate: two Opus builds.
