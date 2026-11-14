# Interaction notes for the martial picks

Type: data
Status: open
Blocked by: none
Model: Sonnet to research and write; Opus to review against the guide.
Map: [WotR Build Planner](../map.md)

## Problem

Found by the third martial run (2026-11-14; see [212](212-athletics-iii-raises-bab.md) for the run). Both agents hit the same rules questions, and each changed their damage estimate by hundreds. Neither the summaries nor `interactions` (`libs/game-data/interactions.ron`, given by `describe`) answers any of them. The MCP agent had to guess. The guide-only agent read the pages, and found most still unanswered:

1. **Greater Power Attack with Power Attack (Mythic).** Greater Power Attack (Two-Handed Fighter 15) makes the damage ×2 rather than ×1.5. Power Attack (Mythic) makes it +5 per step with a two-hander (`mythic-paths#power-attack-mythic`). Does the guide say how they combine? The agents used 5 and 6 per step, a 200 damage-per-round spread.
2. **Demonic Rage with a barbarian's Rage**, and how long Demonic Rage lasts beyond its uses a day (`demon#demonic-rage`, `demon#playing-a-demon`).
3. **How many Demon aspects are active at once**: the minor ones with Extra Minor Aspect at 5 and 8, and the major one.
4. **Aspect of Incubus's damage at MR 9**: "+4, rising by 1 and 2 at ranks 6 and 9" read as +6 or +8.
5. **Knowledge (World) II and critical hits**: does the natural 1 that becomes a 20 threaten?
6. **The Trickster's and Aspect of Babau's sneak attack through Shatter Defenses**: does a target made flat-footed this way take it?
7. **Athletics III's fifth iterative attack**: [212](212-athletics-iii-raises-bab.md)'s question 2, as a note until that ticket decides it.
8. **Every +1 critical multiplier stacking**: Improved Critical (Mythic), Improved Improved Improved Critical Improved and Weapon Mastery to ×5.

## Build

- For each, read the guide's pages through `guide-cache/txt/` (its `INDEX.txt`), the item's own section and the "Playing a …" section of its class or path.
- Write an `interactions` entry on each item involved, in the planner's words: what the guide says, or that it doesn't say. An entry never answers from game knowledge: the data holds to the same honesty boundary the MCP instructions give the agents.
- The ticket's note goes in `libs/game-data/notes/215-martial-interactions.md`, with each question, the guide lines read and the entry written.
- Item 1's answer, where the guide gives one, also goes to ticket [139](139-attack-line-weapon-features-and-mythic-feats.md)'s "Questions for the build" (Greater Power Attack's trade).

## Done when

`describe` of each item above returns its `interactions`. A real-data test pins that each holds one. `cargo run -p seeds --bin check-data` and `scripts/check.sh` pass, and the snapshot check leaves its files untouched.
