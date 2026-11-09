# Effects as data: bonus types, conditions and "counts as"

Type: design
Status: resolved (option A; B and C not built)
Blocked by: [112](112-effect-summaries.md) (recommended: decide this once the summaries exist)
Map: [WotR Build Planner](../map.md)

## Question

The Demon Greatsword run (2026-11-08, see [112](112-effect-summaries.md)) had to decide how features interact, and the planner couldn't answer any of these:
- Does the Demon path's Demonic Rage count as rage for the barbarian's rage powers, and for Greater and Mighty Rage? This decided the Fighter/Barbarian split.
- Can more than one rage-power stance (Powerful, Reckless, Lethal) be active at once?
- Which bonuses stack? (A morale bonus from rage with an enhancement bonus, or two untyped bonuses.)

Summaries (112) say in words what one item does. Interactions are between items, and an agent reading two summaries can still guess wrong.

## Options

- **A. Nothing more:** summaries carry interactions in words ("counts as rage for rage powers").
- **B. An interaction note:** a short, optional `interactions` text on an item, for "counts as", "doesn't stack with" and "one at a time". `describe` shows it. It isn't checked against anything.
- **C. A modifier model:** each effect as data (bonus type, value, what it applies to, the condition: while raging, two-handed only, on a crit). The engine could then sum the bonuses that stack. This is what [114](114-attack-line-and-damage.md)'s option C needs.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* A while 112 is written, then B for the few items where words in two summaries aren't enough. C only if the user takes 114's option C. It's a large model, and the map's "Rules followed" note (intended, patched behaviour) means every interaction would need a source.
2. **Where do interactions come from?** *Recommend:* the saved guide pages only. Where the guide says nothing, the note says nothing, and the agent labels its answer as its own knowledge.

## Answer

Decided 2026-11-08: the user had every ticket of 112–118 built with its recommended options. So it's option A: interactions live in the words of [112](112-effect-summaries.md)'s summaries, and nothing more is built.
- Option B (an `interactions` note) waits until an agent run or a user shows where two summaries aren't enough.
- Option C (a modifier model) is not built: [114](114-attack-line-and-damage.md) took option B, which counts only unconditional bonuses.
