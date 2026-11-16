# The importer, stage 4: prerequisites from the game

Type: build (data, through the importer)
Status: open
Blocked by: 306, 307
Model: Opus to build: one bug touches hundreds of entries.
Map: [WotR Build Planner](../map.md)

## Problem

Stage S4 of the importer ([293](293-the-importer.md); staging decided 2026-11-16, user: "Yes, start it", then the three answers in the design). The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md): its §2 area for this stage, §3 (the user's facts and the registry), §4.2 (each stage's proof) and the user's answers at its end.

## Change

- `--write` writes every blueprint entry's `Requires` from the game (§2.4), the 62 hand trees where §2.4 says.
- Shared-blueprint facts: 305 keys held facts by the planner's kind and key; where one game blueprint is two planner entries (Stunning Fist feat and the monk's feature, Wings, the rogue-talent Weapon Focus option; the full list of 90 is in 305's data note), the game's rule (a fact held once) is applied here or in its gap ticket, and each change that refuses a pick goes to the user if it touches a fact of theirs.

## Done when

- The design's §4.2 proof, all nine points: the game check exits 0 with this stage's `known.txt` lines gone; a second `--write` writes nothing; the change census attributes every differing line to a change-log row (0 unattributed); old links counted three ways, each refused one listed by Build; the sampled hand check of 100 fields; the self-checks' long runs, alone on the machine; every gold-build change shown to the user before the merge; the ignored tests of this stage's areas come off ignore or name their gap ticket.
- The scoped checks pass. Estimate: one and a half Opus builds.
