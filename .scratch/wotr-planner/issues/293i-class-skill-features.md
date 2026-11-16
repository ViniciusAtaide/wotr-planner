# Importer S1, 293i: the features' ClassSkill that 262 needs

Type: build (the importer, stage S1)
Status: open
Blocked by: 293f
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

Form proof (base 17d92338 recorded before the first edit): `builds.tsv`, `slots.txt` and `sheets.txt` are identical; 2,161
Builds compared, 2,161 old links reopen, 0 offer differences. `numbers.txt` differs in 20 Builds (82 lines), every one a
`Class skill=3` line toward the game: Trickery for the Seeker Sorcerer's Builds (`SorcererSeekerTrapfinding`
b1c4f14bc87534e4da7ecd8e0b0658d2, which the list lacked) and Lore (Nature) from level 1 instead of 4 for Animal-domain
Builds (`AnimalDomainBaseFeature` d577aba79b5727a4ab74627c4c6ba23c at level 1 of `AnimalDomainProgression`
23d2f87aa54c89f418e68e790dba11e0). The planner's own Seeker (oracle) and Witch of the Veil lists still hold their skill.
Scoped checks: `cargo test -p seeds -p corpus` passes, `check-data` and `cargo fmt --all --check` pass.
