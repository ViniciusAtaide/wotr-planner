# Wild talent follow-ups

Type: task
Status: open
Blocked by: 46
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 45](45-kineticist-wild-talents.md)'s decisions 1 and 3 (user, 2026-11-02):

1. **The Skill Focus wild talent repeats per skill** (a fact from the user, who wins over the guide's silence): it can be picked at several wild talent levels, each time with a skill the character doesn't already have it for. Model it the way the data already models "once per value" (a feat's `repeat: PerValue`, or what options with a `chosen_feat` use), so a second pick for the same skill is refused and one for another skill is open. The rogue talent of the same name is unchanged (the user said nothing about it).
2. **The bonus-feat slot's label** reads "Wild talent feat" instead of "Elemental Focus feat". It's named after the selection by a generic rule in `rules/src/engine/labels.rs`; change the slot's label without renaming the Elemental Focus selection, and keep the other `OptionList` labels as they are ("Bloodline feat").

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same method as tickets 43–45 for the data change (extractor rule asserted against the guide page's shape, one-off rewrite of `game-data/`, fresh extraction matching, real-data tests first). If the types can't say "an option once per value" yet, add it test-first in `rules` (and the schema, with a DB reset per `CLAUDE.md`).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Skill Focus (Perception) again at 6 is refused; the bonus-feat slot's label is "Wild talent feat"; the sorcerer's level-1 slot still reads "Bloodline feat".

## Progress log

## Answer
