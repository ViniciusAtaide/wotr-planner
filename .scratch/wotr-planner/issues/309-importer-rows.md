# The importer, stage 5: class, archetype and path rows

Type: build (data, through the importer)
Status: open
Blocked by: 306, 307, 308
Model: Opus to build: one bug touches hundreds of entries.
Map: [WotR Build Planner](../map.md)

## Problem

Stage S5 of the importer ([293](293-the-importer.md); staging decided 2026-11-16, user: "Yes, start it", then the three answers in the design). The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md): its §2 area for this stage, §3 (the user's facts and the registry), §4.2 (each stage's proof) and the user's answers at its end.

## Change

- `--write` writes class, archetype and mythic-path rows and mythic ranks from the game (§2.5).
- Step grants (user, Q2, "The game's rows"): the game's single grant replaces the planner's per-step grants on 107 features (557 findings; tickets 170–194 and the user's 188 choice for the cavalier). The sheet loses "rank n" on those features until gap ticket G9 shows steps the game's way.
- Feature ranks capped at the header's `ranks` as the game's `Feature.AddRank` does, if 303's derivation hasn't (305's Q2).

## Done when

- The design's §4.2 proof, all nine points: the game check exits 0 with this stage's `known.txt` lines gone; a second `--write` writes nothing; the change census attributes every differing line to a change-log row (0 unattributed); old links counted three ways, each refused one listed by Build; the sampled hand check of 100 fields; the self-checks' long runs, alone on the machine; every gold-build change shown to the user before the merge; the ignored tests of this stage's areas come off ignore or name their gap ticket.
- The scoped checks pass. Estimate: two Opus builds.
