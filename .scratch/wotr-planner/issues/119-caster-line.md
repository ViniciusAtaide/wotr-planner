# A caster line: spell DCs, caster level against spell resistance, concentration

Type: design
Status: resolved
Blocked by: none (option C waits on [113](113-effects-as-data.md) option C)
Map: [WotR Build Planner](../map.md)

## Question

The user asked an agent for "the strongest DC caster build possible: the maximum chance to debuff the maximum amount of enemies" (2026-11-09, the Max DC run: a Kitsune Sorcerer 20, Arcane bloodline with Fey as Second Bloodline, Azata). This is what [114](114-attack-line-and-damage.md) was for the martial run: the planner checked every pick but couldn't measure the goal.
- `sheet` gives Charisma 25 and caster level 20, but no save DC. The agent added up "about DC 35 for a 9th-level compulsion with metamagic" by hand and had to label it as its own knowledge.
- `compare` can't say which of two branches has the higher DC (Arcane against Crossblooded, Azata against Aeon).
- The same applies to the other two caster numbers the run cared about: caster level against spell resistance (Spell Penetration, Greater, Mythic) and concentration (Combat Casting).

The sources the run stacked, each one stated in a summary today:

| Source | Bonus | Applies to |
| --- | --- | --- |
| Casting ability modifier | Cha (sorcerer), Int (Sage), Wis (Empyreal) | every spell |
| Spell Focus, Greater Spell Focus, Spell Focus (Mythic) | +1 each | one school |
| Expanded Arsenal | the Spell Focus bonuses of one school | another school |
| Kitsune / Gnome / Oread | +1 | enchantment / illusion / acid spells |
| Arcane bloodline arcana | +1 | spells with metamagic |
| Arcane School Power | +2 | one school, from sorcerer 15 |
| Fey / Infernal bloodline arcana | +2 | compulsion / charm spells |
| Elemental Focus, Greater | +1 each | one energy type |
| Aeon's Save DC gaze | 1 + MR/3 | allies' spells, conditional on the gaze |

## Options

- **A. Nothing:** DCs stay the agent's arithmetic, labelled as such.
- **B. A caster line in `sheet` and `compare`**, as 114's option B did for one weapon: for one school (and optionally a subschool, an energy type and "with metamagic") the agent names, at each level, the DC of each spell level, broken down by source, plus caster level against spell resistance and the concentration bonus. Unconditional numbers only: no gaze, no gear, no buffs. A small, fixed table of DC sources in `rules` (the rows above, keyed by item and scope), with a fixed `leaves_out` list, like the attack line's.
- **C. Through a modifier model** ([113](113-effects-as-data.md) option C): DCs fall out of effects as data. Large; not needed for B.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It is the same shape as the attack line, it answers the run's goal, and the source table is small and every row comes from a summary the guide backs.
2. **Where does the DC source table live?** *Recommend:* in the game data, as a `dc_bonus` on the items that give one (value, scope: school, subschool, energy, metamagic, all), since it is a fact from the guide; the engine sums those held. The alternative, a table in `rules`, hides game facts in code.
3. **Subschools:** compulsion and charm need the spell's subschool, which only matters with spells. *Recommend:* the line takes the subschool as an input ("compulsion") and adds the arcana when it's named; no spell data needed ([121](121-spells-as-game-data.md)).
4. **Which spell levels?** *Recommend:* every spell level the Build can cast at that character level, plus a "heightened to the highest" row, since Heighten Spell is how a debuffer uses low-level spells.

## Done when

The run's Build shows its enchantment compulsion DC per spell level in `sheet` and `compare`, broken down by source, with a real-data test on that Build; `scripts/check.sh` passes.

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (the user's brief: "take the recommended answers"). Questions 1–4:

1. **Option B**, recommended answer, taken while the user slept, 2026-11-09: `Engine::caster_line`, shown in `sheet` and `compare` (MCP) for a `caster` argument. Not in the web app (the ticket asks for `sheet` and `compare`; the engine call is there for the UI when wanted).
2. **The DC sources are game data**, recommended answer, taken while the user slept, 2026-11-09: `libs/game-data/bonuses.ron`, loaded through the seed by a new migration (`20261109033141_119-caster-line.sql`, table `item_bonus`), summed by the engine for the items held.
3. **The subschool is an input**, recommended answer, taken while the user slept, 2026-11-09: `caster: {school, subschool?, energy?, metamagic?}`; no spell data ([121](121-spells-as-game-data.md) can feed it later). A subschool of another school is refused.
4. **Every spell level castable plus a heightened row**, recommended answer, taken while the user slept, 2026-11-09: levels 1 to the highest (cantrips left out), and `heightened` only when the Build holds Heighten Spell. Its DC equals the highest row's (Heighten Spell raises the spell level, and the Arcane arcana leaves Heighten Spell out), so it says "any spell of this kind reaches this".

Settled while building (the smallest choice consistent with the map's Notes, taken while the user slept, 2026-11-09):

5. **One general bonus shape, shared with [120](120-initiative-and-skill-totals.md)**, not a DC-only field: `rules::ItemBonus { item: BonusItem, target: BonusTarget, amount: BonusAmount, requires: Option<Prerequisite> }` (`libs/rules/src/bonus.rs`).
   - `BonusItem`: `Race(RaceKey)`, `Subrace(SubraceKey)`, `Feat(FeatKey)`, `MythicFeat(MythicFeatKey)`, `Feature(FeatureKey)`, `SelectionOption(SelectionOptionKey)`. A race's bonus counts without a subrace and a subrace's in place of its race's (as ability modifiers); a parameterised feat's or mythic feat's counts once per value held; a feature or option however it's held (Second Bloodline, Crossblooded).
   - `BonusTarget`: `SpellDc(SpellScope)`, `SpellPenetration`, `Concentration`. `SpellScope`: `All`, `School(SpellSchool)`, `Subschool(Subschool)`, `Energy(Energy)`, `Metamagic`, `Value` (the school or energy the item is held for).
   - `BonusAmount`: `Flat(i8)`, `HalfMythicRank`, `MythicRank`, `SpellFocusOfAnotherSchool` (Expanded Arsenal).
   - `requires`: a prerequisite checked against the state, like any other (Spell Penetration (Mythic): `HalfMythicRank` with `Not(Feat("greater-spell-penetration"))`, `MythicRank` with `Feat("greater-spell-penetration")`).
   - **One catalogue file keyed by item**, not a field on six item types: one shape for every giver, one table (`item_bonus`, one foreign-key column per item kind, exactly one set), no change to the item structs or their literals, and no overlap with ticket 121's edits.
   - **How 120 plugs in:** new `BonusTarget` variants (`Initiative`, `Skill(SkillKey)`, `Save(Save)`; their rows in `bonuses.ron`), new `BonusItem` variants where needed (`Background(BackgroundKey)`, `MythicAbility(MythicAbilityKey)`), a migration that recreates `item_bonus` with the wider `target` CHECK and the new columns (`skill_key`, `save`, `background_key`, …), and its own sum beside `Engine::caster_line` reusing the holding rule (`Engine::held_bonuses`, private today: make it `pub(super)` and generic over the target). Amounts that depend on ranks (Skill Focus +3, +6 at 10 ranks) fit `requires` (`SkillRanks`), or a new `BonusAmount` if the rank is the pick's value.
   - **How 124 plugs in:** stacking is not modelled; every bonus held adds (the guide's School Power and Fey Magic say they stack; nothing in `bonuses.ron` says otherwise). 124 can add a `stacks`/bonus-type column to `ItemBonus` and `item_bonus` and have the sum skip what doesn't stack.
6. **Casting ability as data:** `Class::casting_ability` and `Archetype::casting_ability` (`Option<Ability>`, the same migration). The guide's "primary attribute" where it names one (Sage: "Intelligence as a primary attribute", Empyreal "uses Wisdom", Stigmatized Witch "Primary attribute is Charisma", Tortured Crusader "Uses Wisdom rather than Charisma"), the core rules otherwise (Eldritch Scion, Ley Line Guardian, Living Grimoire, Eldritch Scoundrel, and the base classes whose primary attributes name two). The check refuses a class or archetype whose rows give spell levels without one, and one on a class that casts nothing of its own (prestige classes advance another's).
7. **The casting class:** the one with the highest caster level (class levels plus the prestige levels advancing it; the first entered on a tie); its spell levels and casting ability. A second casting class is in `leaves_out`.
8. **Expanded Arsenal** (the saved page `guide-cache/mythic-paths.html`): "Any bonuses from Spell Focus feats for another school apply to that school … Only the bonuses from one school are applied … You can take Expanded Arsenal in a school for which you have taken Spell Focus feats but this has no effect." So it copies **Spell Focus, Greater Spell Focus and Spell Focus (Mythic)** of one other school (all "Spell Focus feats"), the school where they add up to the most (the guide doesn't say which; the most, first in school order on a tie), and nothing in a school that has Spell Focus feats of its own. "Spell Focus feats" are, in the engine, the feats and mythic feats whose bonus is `SpellDc(Value)` and `Flat` for a school.
9. **Sources beyond the table**, because they are as unconditional as the table's and leaving them out would make the line wrong as a fact: Improved Elven Immunities (+1 enchantment), Tiefling (Spitespawn) (+1 enchantment), the Sage Sorcerer's Arcane arcana, the elf's +2 against spell resistance (not its subraces', whose traits the guide lists without it), Dual Heritage, Elven Spirit, Arcane Focus (+1 against spell resistance; its defensive-casting +2 is conditional), Fey Magic (+2 against spell resistance from Fey 15), Warrior Priest (+2 concentration; its initiative is 120's). Left out, in `leaves_out`: bonuses that grow with class level (Combat Casting Adept, Horn Call, Cold Specialization, Overwhelming Penetration, Haunting Refrain, Blessed Script), School Mastery's caster level for one school, abilities switched on in a fight (the Aeon's gaze, Demonic Rage and aspects, the Devil's decrees, Studied Target), gear and buffs, and what changes the target's roll rather than the DC (Persistent Spell, Favorable Magic, Ambuscading Spells).
10. **MCP shapes** in `apps/mcp/src/caster.rs`, as 114's in `attack.rs`: `sheet`'s levels and `compare`'s summaries get `caster` (`class`, `ability`, `caster_level`, `dcs`, `heightened`, `dc_counts`, `spell_penetration` with its counts, `concentration` with its counts), and `caster_leaves_out` once beside them. The honesty boundary says the caster line is fact, like the attack line, and that which spells exist (and their schools) stay the agent's knowledge.

## Progress log

- 2026-11-09: read the map's Notes, this ticket, 114, 120, 121, 124, the glossary, `Engine::attack_line` and its MCP side. Settled Expanded Arsenal from `guide-cache/mythic-paths.html` (choice 8) and checked the other sources' wording on `feats`, `character-creation` and `sorcerer`.
- Test first in `rules` (`tests/engine/caster_line.rs`, 13 tests): the DCs per spell level and their parts, spell levels by class level, no line without a caster, Spell Focus/Mythic/School Power by school, the highest casting class, prestige advancement, an archetype's casting ability, a subrace's bonuses replacing its race's, `requires`, subschool/energy/metamagic scopes, Expanded Arsenal (copies the best other school's; nothing where the school has its own), Heighten Spell, and the data check (`NoCastingAbility`, `CastingAbilityWithoutSpells`, `BonusValueWithoutParameter`, an unknown item).
- Data: `bonuses.ron` (39 rows), casting abilities on 17 classes and 8 archetypes. The check caught the Eldritch Scoundrel (a casting rogue archetype) on the first seed.
- Schema: migration `20261109033141_119-caster-line.sql`; `libs/seeds/src/bonuses.rs` writes and reads `item_bonus` (kept apart from `seed.rs`/`load.rs`); rebuilt the worktree database with CLAUDE.md's four steps and re-prepared `.sqlx`. The fixture gained a `bonuses.ron` with every shape and casting abilities, so the files → database → files round trip covers them; a schema test checks `item_bonus`'s constraints.
- Real-data test `libs/seeds/tests/game_data/caster_line.rs` (6 tests) on the Max DC run's Build, from its link: at 20, enchantment compulsion DCs 26–34 (Charisma +7, Kitsune +1, Spell Focus +1, Greater +1, Mythic +1, Fey Arcana +2, School Power +2), 35 with metamagic (Arcane Arcana), heightened 34; the DC by level as the picks arrive; Conjuration's Expanded Arsenal +3 from Enchantment; caster level 36 against spell resistance (Spell Penetration 2, Greater 2, Mythic 10, Fey Magic 2) and concentration 31 (Charisma 7, Combat Casting 4).
- MCP: `caster.rs`, the `caster` arguments, the instructions; `tests/server/caster_line.rs` (6 tests) on the real data: the sheet's levels, metamagic, no line without `caster`, `compare`'s summary, refused arguments, the instructions.

- Behaviour proof, `scripts/proof.sh both` against `main` (3b7aa71): 2,163 Builds on each side, none only in one, none differing (empty `compare.txt`): the sheets are unchanged, and the caster line is a new call beside them. The script's shared build folder (`$TMPDIR/wotr-planner-proof-target`) was in use by another worktree's run and failed twice with a missing `bindgen.rs`, so this run used a private one under the agent's scratchpad (`CARGO_TARGET_DIR` for the proof only).
- `scripts/check.sh --schema`: ALL-OK (fmt, clippy, wasm clippy, the workspace tests, the rules wasm build, the seed check, the sqlx check against a new migrated file).

## Answer

Done 2026-11-09 (option B). `Engine::caster_line(start, state, spell)` gives, for spells of a school (and optionally a subschool, an energy type and metamagic), the save DC of each spell level the casting class can cast, broken down by source (the casting ability's modifier and each item bonus held), the DC heightened to the highest level when the Build holds Heighten Spell, caster level against spell resistance and the concentration bonus, each with its parts, and a fixed `leaves_out` list. The sources are game data: `bonuses.ron` (`rules::ItemBonus`, a general shape with a target, an amount and an optional requirement, which ticket 120 extends with initiative, skill and save targets) and each casting class's `casting_ability`, loaded through migration `20261109033141_119-caster-line.sql`. `sheet` and `compare` take `caster` and show the line, and the MCP instructions let an agent state it as fact. The Max DC run's Build shows its enchantment compulsion DC 34 at 9th level (35 with metamagic), 36 against spell resistance and +31 concentration at level 20.
