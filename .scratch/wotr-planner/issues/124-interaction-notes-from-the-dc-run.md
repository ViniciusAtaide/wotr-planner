# Interaction notes: the cases the Max DC run couldn't answer

Type: design
Status: resolved
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

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (the user's brief: "take the recommended answers"). Questions 1–3:

1. **B for rows 1 and 4, C for rows 2, 3 and 5**, recommended answer, taken while the user slept, 2026-11-09 ([119](119-caster-line.md) is built). C needed no new data: 119 already settled rows 2 and 3 in its bonus data, and row 5 needs no stacking column (below). Each of the five items (and row 5's two bloodlines) still carries a note, so `describe` answers all five as this ticket's "Done when" asks.
   - **Row 1** (Favorable Magic with Persistent Spell): the saved pages give each item's own rule and nothing on the two together (searched every page in `guide-cache/` for both names). Note on both: the guide doesn't say whether both apply or one replaces the other.
   - **Row 2** (Expanded Arsenal): answered by 119 (its choice 8, from `mythic-paths.html`): it copies the bonuses of all "Spell Focus feats" of one other school, so Spell Focus, Greater Spell Focus and Spell Focus (Mythic), and nothing in a school with its own; the guide doesn't say which school when several have them, and the caster line takes the one that adds most. The note says so.
   - **Row 3** (Second Bloodline's arcana): answered by 119's data (the arcana held through Second Bloodline count in the caster line). The guide's "gaining all its benefits", and `character-creation.html`'s note that two Draconic bloodlines are barred because both arcana would add damage, show the arcana counts. The note says so.
   - **Row 4** (Bloodline Ascendance with two bloodlines): the guide gives "the 20th level ability of your bloodline" and doesn't say which. The note says so, and that the planner (Bloodline Ascendance's `option_row`) grants each held bloodline's 20th-level ability.
   - **Row 5** (Fey and Infernal arcana with School Power): `character-creation.html` gives the arcana "+2" with no bonus type and School Power "stacks with the bonus granted by Spell Focus"; nothing on the arcana against School Power. So no stacking or bonus-type column (the brief: add one only if the guide gives one): every bonus held keeps adding, as now, and the note on the Fey and Infernal bloodline options says the guide doesn't say.
2. **Source: the saved guide pages only**, recommended answer, taken while the user slept, 2026-11-09; "the guide doesn't say" where it doesn't.
3. **Only these items, no sweep**, recommended answer, taken while the user slept, 2026-11-09: the five, plus the Fey and Infernal bloodline options for row 5 (the arcana are features, which `describe` doesn't describe on their own; the bloodline options are where an agent looks).

Settled while building (the smallest choice consistent with the map's Notes, taken while the user slept, 2026-11-09):

4. **One catalogue file keyed by item**, as 119 did for bonuses: `interactions.ron`, `rules::ItemInteractions { item: BonusItem, interactions: String }` (`libs/rules/src/bonus.rs`), `GameData::interactions`, `Engine::interactions(&BonusItem)`. No field on six item structs. The file is optional (data from before it reads as none). The check names each item and refuses two notes on one (`DataIssue::InteractionsTwice`).
5. **Schema in the same migration as 120's** (`20261109075346_120-124-totals-and-interactions.sql`): table `item_interactions`, one foreign-key column per item kind (exactly one set, each `UNIQUE`), `interactions` non-empty.
6. **MCP: `describe` only**, not `search`: another agent was editing `search` at the same time (the orchestrator's scope for this run). `describe` gives `interactions` for a feat, mythic feat, mythic ability, option, race or background that has a note; the instructions' honesty boundary says to cite it as the planner's note on the guide, and that where it says the guide doesn't say, any answer is the agent's own knowledge. `search` showing it is left for later (a one-line follow-up on `Hit`).

## Progress log

- 2026-11-09: read 113, 119 (choices 5 and 8), this ticket; searched every saved guide page for Favorable Magic, Persistent Spell, Expanded Arsenal, Second Bloodline, Bloodline Ascendance and the arcana's stacking (`azata`, `feats`, `mythic-paths`, `character-creation`, `sorcerer`).
- Rules: `ItemInteractions`, `GameData::interactions`, the check, `Engine::interactions`; a lookup test in `tests/engine/totals.rs` and the check's refusal there.
- Data: `interactions.ron`, seven notes. Schema: `item_interactions` in 120's migration; seeded and loaded by `libs/seeds/src/bonuses.rs`; the fixture's `interactions.ron` and the schema test's constraints.
- Real-data test: `libs/seeds/tests/game_data/totals.rs`'s `the_dc_runs_five_items_have_their_interactions_notes`.
- MCP: `describe`'s `interactions`; `tests/server/interactions.rs` (the five items, and none on Power Attack); the instructions (`tests/server/totals.rs`).
- Behaviour proof: run once with 120's (`scripts/proof.sh both` against `main`): no Build differs.
- `scripts/check.sh --schema`: ALL-OK.

## Answer

Done 2026-11-09. Rows 2, 3 and 5 are answered through 119's bonus data (Expanded Arsenal copies Spell Focus, Greater and Mythic of one school; Second Bloodline's arcana counts; the guide gives the arcana no bonus type, so no stacking column and every bonus keeps adding). Every item the run asked about carries an interactions note (`interactions.ron`, `rules::ItemInteractions`, table `item_interactions` in migration `20261109075346_120-124-totals-and-interactions.sql`), and `describe` shows it: Favorable Magic and Persistent Spell ("the guide doesn't say" whether both apply), Expanded Arsenal, Second Bloodline, Bloodline Ascendance (the guide doesn't say which bloodline; the planner grants each one's), and the Fey and Infernal bloodlines (their arcana against School Power: the guide doesn't say). `search` doesn't show it yet.
