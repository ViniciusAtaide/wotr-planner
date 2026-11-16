# The importer, stage 3: the carrier entries

Type: build (data, through the importer)
Status: open
Blocked by: 306
Model: Opus to build: one bug touches hundreds of entries.
Map: [WotR Build Planner](../map.md)

## Problem

Stage S3 of the importer ([293](293-the-importer.md); staging decided 2026-11-16, user: "Yes, start it", then the three answers in the design). The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md): its §2 area for this stage, §3 (the user's facts and the registry), §4.2 (each stage's proof) and the user's answers at its end.

## Change

- An entry for each hidden blueprint a prerequisite or a row of S4/S5 names (≈400, §2.7), with only its grant components, named from its asset name (user, Q1). Nothing reads them yet: the census must be empty.
- The full import of every reachable feature (~3,458) is a gap ticket (G7), not this stage (user, Q3).

## Done when

- The design's §4.2 proof, all nine points: the game check exits 0 with this stage's `known.txt` lines gone; a second `--write` writes nothing; the change census attributes every differing line to a change-log row (0 unattributed); old links counted three ways, each refused one listed by Build; the sampled hand check of 100 fields; the self-checks' long runs, alone on the machine; every gold-build change shown to the user before the merge; the ignored tests of this stage's areas come off ignore or name their gap ticket.
- The scoped checks pass. Estimate: one Opus build.
