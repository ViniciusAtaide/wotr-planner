# Interaction notes: the cases the Max DC run couldn't answer

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

[113](113-effects-as-data.md) took option A (interactions live in the summaries' words) and left option B, a short `interactions` note on an item, waiting "until an agent run or a user shows where two summaries aren't enough". The Max DC run (2026-11-09, see [119](119-caster-line.md)) is such a run. Each of these decided a pick or a claim, and neither summary settles it:

| Question | The summaries say | What it decided |
| --- | --- | --- |
| Do Favorable Magic and Persistent Spell stack, or does one replace the other? | Favorable Magic: "Enemies roll saves against your spells twice and keep the worse". Persistent Spell: "a target that saves must save again". | The run's core claim; the agent had to say it didn't know. |
| Does Expanded Arsenal copy Greater Spell Focus and Spell Focus (Mythic), or only Spell Focus? | "your Spell Focus bonuses from one other school also apply to it" | Two mythic feat slots went to it (Conjuration, Necromancy). |
| Does Second Bloodline give the second bloodline's arcana? | "Take a second sorcerer bloodline with all its benefits" | The Fey +2 on compulsion, the run's biggest single DC source. |
| Does Bloodline Ascendance give the second bloodline's 20th-level power, or only the first's? | "Gain your sorcerer bloodline's 20th-level power." | Whether it was worth a mythic ability on a sorcerer 20 (not taken). |
| Do the Fey (+2 compulsion) and Infernal (+2 charm) arcana stack with School Power (+2 school)? | Each states its own bonus; only School Power says "stacking with Spell Focus". | The DC total the agent gave. |

## Options

- **A. Still nothing:** the agent states each as unknown.
- **B. Build 113's option B now**, for these five items: an optional `interactions` text on an item, shown by `describe` and `search`, from the saved guide pages only.
- **C. Fold the stacking ones into [119](119-caster-line.md)'s DC sources:** a DC bonus row says whether it stacks, and the caster line sums only what stacks. Covers rows 2, 3 and 5, not 1 or 4.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B for rows 1 and 4, and C for 2, 3 and 5 if 119 is built (else B for all five).
2. **Source:** *Recommend:* the saved guide pages only, as 113 decided. Where the guide says nothing, the note says "the guide doesn't say", which is itself worth stating: the agent then knows it's guessing.
3. **More items:** *Recommend:* only these, and new notes as later runs find them; no sweep.

## Done when

`describe` of Favorable Magic, Persistent Spell, Expanded Arsenal, Second Bloodline and Bloodline Ascendance gives each one's note (or "the guide doesn't say"), with a real-data test that they're present; `scripts/check.sh` passes.
