# Wild talent follow-ups

Type: task
Status: open
Blocked by: 46
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 45](45-kineticist-wild-talents.md)'s decisions 1 and 3 (user, 2026-11-02):

1. **The Skill Focus wild talent repeats per skill** (a fact from the user, who wins over the guide's silence): it can be picked at several wild talent levels, each time with a skill the character doesn't already have it for. Model it the way the data already models "once per value" (a feat's `repeat: PerValue`, or what options with a `chosen_feat` use), so a second pick for the same skill is refused and one for another skill is open. **The rogue talent Skill Focus follows the same rule** (user, 2026-11-02: "rogue talent follows the same rule as wild talent"): picked again at later rogue talent levels, each time for another skill. The slayer shares the rogue talents' options, so it applies to a slayer's talents too.
2. **Rogue talents Weapon Focus and Combat Trick** (user, 2026-11-02): "for the rogue talent weapon focus can be only taken once. but the combat trick -> weapon focus can be taken repeatable times as long as you select a different weapon".
   - The rogue talent **Weapon Focus** stays taken once (as today).
   - The rogue talent **Combat Trick** can be taken more than once, and each time grants a bonus Combat feat slot (today it's a bare option that grants nothing). Through those slots, Weapon Focus can be taken again for another weapon, as the feat's own once-per-weapon rule already says. Check the saved rogue page for what Combat Trick grants and whether its feat ignores prerequisites; if the page doesn't say, keep prerequisites. The slayer shares the rogue talents, so it applies to its talents too.
3. **The bonus-feat slot's label** reads "Wild talent feat" instead of "Elemental Focus feat". It's named after the selection by a generic rule in `rules/src/engine/labels.rs`; change the slot's label without renaming the Elemental Focus selection, and keep the other `OptionList` labels as they are ("Bloodline feat").

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same method as tickets 43–45 for the data change (extractor rule asserted against the guide page's shape, one-off rewrite of `game-data/`, fresh extraction matching, real-data tests first). If the types can't say "an option once per value" yet, add it test-first in `rules` (and the schema, with a DB reset per `CLAUDE.md`).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Skill Focus (Perception) again at 6 is refused; a rogue takes the Skill Focus talent for two different skills and is refused a second one for the same skill; the bonus-feat slot's label is "Wild talent feat"; the sorcerer's level-1 slot still reads "Bloodline feat"; a rogue takes Combat Trick twice, gets a Combat feat slot each time, and takes Weapon Focus (Longsword) in one and Weapon Focus (Dagger) in the other, while Weapon Focus (Longsword) twice is refused; the rogue talent Weapon Focus can't be taken twice.

## Progress log

## Answer
