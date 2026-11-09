# Initiative and skill totals

Type: design
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) two picks depended on numbers the sheet doesn't give:
- **Initiative.** Ambuscading Spells gives flat-footed enemies −2 on their saves, so acting first matters. The run took Improved Initiative, Improved Initiative (Mythic) and the Pickpocket background (+2) for it, but couldn't state the total.
- **Skill totals.** Arcane Spirit is a Persuasion (Bluff) check against the target's Perception. `sheet` gives ranks, not the total (ranks + ability modifier + class skill + racial and background bonuses + Skill Focus).

## Options

- **A. Nothing:** the agent adds them up.
- **B. Totals in `sheet`:** an `initiative` line (Dex + feats + background + mythic feat at MR) and a `total` beside each skill's ranks, each with its sources. Unconditional only (no gear, no buffs), as for 114 and 119.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. The sources are a handful of items whose summaries already state a flat number; the sheet already holds the ranks, the scores and the class skills.
2. **Where do the bonuses live?** *Recommend:* the same per-item bonus data as 119's question 2 (a flat `bonus` with a target: initiative, a skill, a save), so both tickets share one shape.
3. **Saves too?** Iron Will, Mythic Resolve and the halfling's +1 change Will, and `sheet` gives saves from class progressions only. *Recommend:* yes, in the same pass.

## Done when

`sheet` gives initiative, each skill's total and saves with their flat feat and race bonuses, each broken down by source, with real-data tests; `scripts/check.sh` passes.

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (the user's brief: "take the recommended answers"). Questions 1–3:

1. **Option B**, recommended answer, taken while the user slept, 2026-11-09: `Engine::totals(start, state)` (`libs/rules/src/engine/totals.rs`), shown by `sheet` (MCP) as `totals` at the last level, with `totals_leave_out`. Not in the web app (not trivial: the sheet screen has no place for a breakdown yet; the engine call is there for it).
2. **The bonuses live in 119's per-item bonus data**, recommended answer, taken while the user slept, 2026-11-09: `bonuses.ron` / `rules::ItemBonus`, widened (below), loaded through a new migration that recreates `item_bonus` (`20261109075346_120-124-totals-and-interactions.sql`).
3. **Saves too**, recommended answer, taken while the user slept, 2026-11-09: each save's total beside initiative and the skills.

Settled while building (the smallest choice consistent with the map's Notes, taken while the user slept, 2026-11-09):

4. **The extended bonus shape** (`libs/rules/src/bonus.rs`):
   - `BonusItem` gains `Background(BackgroundKey)` (the Build start's background) and `MythicAbility(MythicAbilityKey)`.
   - `BonusTarget` gains `Initiative`, `Save(Save)`, `AllSaves`, `Skill(SkillKey)`, `AllSkills` and `Held`: the skill or stat the item is held for (Skill Focus (Persuasion); Stat Focus (Initiative), (Fortitude), (Reflex), (Will); Stat Focus's other stats count nowhere). `BonusTarget::adds_to_skill`, `adds_to_save`, `adds_to_initiative` say whether a bonus held for a value adds to a number.
   - `BonusAmount` gains `TenRanks { amount, with_ten_ranks }` (the higher number with 10 or more ranks in the skill: Skill Focus 3/6, Alertness, Deceitful, Deft Hands, Persuasive, Stealthy 2/4; skill targets only) and `AbilityModifier(Ability)` (Divine Grace's Charisma to every save; Cunning Initiative's and Fervent Reflexes' Wisdom and Lightning Draw's Intelligence to initiative; Divine Insight's Charisma to the four Knowledge and Lore skills; the oracle final revelations'). The 119 question said "unconditional flat numbers"; an ability modifier from the character's own score is as unconditional, and leaving Divine Grace out would make a paladin's saves wrong as a fact (ticket 122 sums saves next).
   - `requires` covers steps at a class level: the Duelist's Improved Reaction (+2, +4 with `ClassLevel(duelist, 8)`), Powerless Prophecy (+4 from oracle 5).
   - Schema: `item_bonus` recreated with `background_key`, `mythic_ability_key`, `save`, `skill_key` (a foreign key to `skill`), `amount_with_ten_ranks` and `ability`, the wider `target` and `amount_kind` CHECKs, and a constraint per new column (a save target has its save, a skill target its skill, ten ranks both numbers, an ability modifier its ability).
   - The holding rule is 119's (`Engine::held_bonuses`, now `pub(super)` and shared): a background counts when it's the Build start's, a mythic ability when held; `amount` (shared too) takes the skill whose ranks `TenRanks` reads.
5. **The sums.** Initiative: Dexterity's modifier, then each bonus. A save: the base save of the class progressions (`state.fort`, …), the ability's modifier (Constitution, Dexterity, Wisdom), then each bonus. A skill: its ranks, its ability's modifier, +3 when it's a class skill with at least one rank (the map's Notes: "a flat bonus added to a trained skill's total"; ticket 37/38's `state.class_skills`), then each bonus. Every skill of the data is listed, trained or not. Under Perfect Mind (the one feature with `skill_ranks_at_least_mythic_rank`), the ranks counted are at least the Mythic rank and every skill is a class skill, as its summary says.
6. **The sources** (all from summaries checked against the saved guide pages; backgrounds from the user's list): Improved Initiative +4, Improved Initiative (Mythic) +MR, Warrior Priest +1, Pickpocket +2, Halfling (Hasty) +2; Great Fortitude, Iron Will, Lightning Reflexes and their Improved forms +2, Mythic Resolve (a mythic feat in the data) +1 to every save, Halfling Luck +1 (halfling and Hasty), Half-Orc (Tribal) +1, Halfling (Craven) +2 Reflex, Half-Elf (Dual Heritage) +2 Will, Unstoppable Dwarf +1 Fortitude, Farmhand/Lumberjack/Miner/Smith +1 Fortitude and Varisian Explorer +1 Reflex (the user's list; the guide's table shows the +1 Fortitude beside the Farmhand only), the Loremaster's secrets +2, the Duelist's Grace +2 Reflex; every race's and subrace's skill bonuses from the guide's racial traits (a subrace's in place of its race's); the skill feats above, Breadth of Experience, Discerning Eye, Elven Spirit, Endurance and Stony Step; Nexian Scholar and Osirian Historian +2 Knowledge; Nature Sense, Canny Observer, Well Hidden, Diabolic Harbinger; Creative Approach +3 to every skill, Knowledge (World) I +MR/2 ("when you or an ally" makes a check), All-skilled +MR. The mythic Iron Will, Great Fortitude and Lightning Reflexes are rerolls: no number.
7. **Left out**, in `TOTALS_LEAVE_OUT`: gear; buffs, songs and auras; abilities switched on in a fight (rage, aspects, judgments, stances); bonuses against one kind of effect or foe; **bonuses to one use of a skill** ("Persuasion when used for diplomacy", for bluff or to intimidate: the aasimar's, the tiefling heritages', the dhampir's, the half-orc's, the Andoran Diplomat's), the smallest choice since `Persuasion` is one skill and the guide scopes these to one use; bonuses in one place or with one weapon; bonuses that grow with class level; **a background's other ability for a skill** (Acolyte, Healer: their notes; modelling it needs a new background field, out of this ticket); rerolls and taking 10; armour check penalties and size.
8. **MCP shape** (`apps/mcp/src/totals.rs`): `sheet` gives `totals: {initiative, fortitude, reflex, will: {total, counts}, skills: {<skill key>: {ranks, total, counts}}}` at the last level (not per level, to keep the sheet small: the engine call takes any state, so `compare` or a per-level line can follow) and `totals_leave_out`. The instructions' honesty boundary says the totals are fact, like the attack and caster lines.

## Progress log

- 2026-11-09: read the map's Notes, this ticket, 119 (its Answer and settled choice 5: how 120 plugs in), 124, `bonus.rs`, `caster.rs`, `bonuses.ron`, `libs/seeds/src/bonuses.rs`, the sheet's MCP side. Read the races' traits, the backgrounds' table and the feats, mythic and prestige pages in `guide-cache/` for every source.
- Test first in `rules` (`tests/engine/totals.rs`, 7 tests): initiative with a mythic feat by rank, a background and Stat Focus; saves with base, ability, a race's every-save bonus, an ability-modifier bonus and Stat Focus; skills with ranks, ability, the class skill bonus only when trained, `TenRanks` below and at 10 ranks, Skill Focus by value; Perfect Mind; `requires`; the data check (`HeldBonusWithoutSkillOrStat`, `TenRanksWithoutSkill`, an unknown skill or background) and the interactions lookup (ticket 124).
- Data: 141 new rows in `bonuses.ron` (initiative 14, saves 25, skills 102). The seed's check and foreign keys passed on the first seed.
- Schema: migration `20261109075346_120-124-totals-and-interactions.sql` (`item_bonus` recreated, `item_interactions` new); `libs/seeds/src/bonuses.rs` writes and reads both; rebuilt the worktree database with CLAUDE.md's four steps and re-prepared `.sqlx`. The fixture's `bonuses.ron` gained every new shape and an `interactions.ron`, so the files → database → files round trip covers them; the schema test checks the new constraints.
- Real-data test `libs/seeds/tests/game_data/totals.rs` (4 tests): the Max DC run's Build at 20: initiative 19 (Dexterity 3, Pickpocket 2, Improved Initiative 4, its mythic feat 10), Will 15 (base 12, Wisdom 1, Iron Will 2), Persuasion 30 (20 ranks, Charisma 7, class skill 3), Mobility 5 with the Kitsune's +2; a halfling Farmhand paladin 2's Fortitude (base, Constitution, Halfling 1, Farmhand 1, Great Fortitude 2, Divine Grace from paladin 2); Skill Focus (Persuasion) +3 at 9 ranks, +6 at 10.
- MCP: `totals.rs`, `sheet`'s `totals` and `totals_leave_out`, the tool description and instructions; `tests/server/totals.rs` (2 tests).
- Behaviour proof, `scripts/proof.sh both` against `main` (e3c6c13): 2,163 Builds on each side, none only in one, none differing (empty `compare.txt`): the sheets are unchanged; the totals are a new call beside them. Run with a private build folder under the agent's scratchpad (`CARGO_TARGET_DIR` for the proof only), as 119 did, since the script's shared one may be in use by other worktrees.
- `scripts/check.sh --schema`: ALL-OK (fmt, clippy, wasm clippy, the workspace tests, the rules wasm build, the seed check, the sqlx check against a new migrated file).

## Answer

Done 2026-11-09 (option B). `Engine::totals(start, state)` gives initiative, each save and each skill's total, each broken down by source (ability modifiers, base saves, ranks, the class skill bonus and each item bonus held), with a fixed `leaves_out` list. The sources are 119's per-item bonus data, widened: `BonusItem::Background` and `MythicAbility`; `BonusTarget::Initiative`, `Save(Save)`, `AllSaves`, `Skill(SkillKey)`, `AllSkills`, `Held`; `BonusAmount::TenRanks { amount, with_ten_ranks }` and `AbilityModifier(Ability)`; loaded through migration `20261109075346_120-124-totals-and-interactions.sql`. `sheet` shows them at the last level as `totals`, and the instructions let an agent state them as fact. The Max DC run's Build has initiative +19, Will +15 and Persuasion +30 at level 20.
