---
name: wotr-build
description: Build Pathfinder: Wrath of the Righteous characters with the wotr-planner MCP tools. Use when asked to make, optimise, compare or check a build, or to rank classes, paths or feats by damage, a spell DC or another number.
---

# Building with wotr-planner

wotr-planner builds Pathfinder: Wrath of the Righteous characters with the rules engine of a build planner. Every Build its tools return is one the planner's rules allow. This guide is also the MCP resource `wotr://build-guide`; the planner's terms (Build start, Slot, Withdrawal, Path switch, …) are in the resource `wotr://glossary`.

## Steps

1. **Resolve** every name the user gives with `search` (see Workflow). Done when each name is one key; if a name could mean more than one item, ask the user which one before you build.
2. **Name the number** the goal is. A number the planner computes (a spell DC, caster level against spell resistance, concentration, the first attack's bonus, the damage bonus per hit) is ranked by the planner: see Ranking by a number. Damage per round is not one of them: for a martial goal, read [martial.md](martial.md) now. Done when you can say which tool gives the number and what it leaves out.
3. **Scout** the route before you commit to it: `describe` each candidate class, archetype and Mythic path, and run `options` with `only_open: false` on the slots where the plan's key picks go. Branch a class split before you commit to it (see Before you plan). Done when every pick the plan rests on is open where you put it, or you know the level or prerequisite that opens it.
4. **Plan the whole Build as a map** of address → value in a file in your scratchpad (`plan.json`), then `new_build` and `validate` the map (see Planned whole). Done when `validate` returns `status: "valid"` and `choose` has filled its `todo`, so the last `todo` is empty.
5. **Branch** each alternative by copying the map file and editing only the addresses that differ, then `validate` each copy and `compare` the codes (see Branching). A Mythic path or a class swap is an edit of the map, not a rebuild step by step.
6. **Price** the goal's number for every branch: `compare` with `by`, whose numbers are naked on every branch unless you pass `with_gear: true` (see Gear; the attack line for each Build's Weapon Focus weapon unless you give a `weapon`); for damage per round, the worksheet in [martial.md](martial.md).
7. **Report** in the shape under Report. Done when every link opens and every number is marked fact, assumption or estimate.

## Before you plan

- Point-buy scores start at 10 and run from 7 to 18 on a 25-point budget; `sheet` gives `point_buy_spent` and `point_buy_budget`.
- Choose the faith (a deity, or atheism) before the alignment: no alignment is open until the faith is chosen, and a deity allows only some. A class with an alignment rule (the monk is lawful) narrows the deities too.
- `sheet` gives each Mythic rank's `character_level`, the character level its ascension comes at.
- Within a level, picks are checked in order: class selections and class feat slots (`slot.selection.N.I`, `slot.feat.N.class.I`), race slots, the general slot (`slot.feat.N.general.0`), the slots options and feats give, then each Mythic rank that comes at that level (`slot.mythic.R`, `slot.path-feat.R.I`, `slot.path-selection.R.I`). A pick that needs another taken at the same level goes in the later slot: Spell Focus in the sorcerer's level-1 class slot and Greater Spell Focus in the general slot, not the reverse. A feat that needs a Mythic rank's pick (the Trickster's critical feats need Perception II) goes at a later level than that rank. A refusal that says the prerequisite's pick "comes after this slot" names that pick's slot: swap the two picks, or take this one at a later level.
- A class split decides what the last levels can take: Fighter 5 / Barbarian 15 gives up Barbarian levels 16 to 20 and whatever needs them. Before you commit to a class split, branch the candidates from one code (Barbarian 20, and Fighter 5 / Barbarian 15), add their levels, and `compare` the branches: it lists the feats and features each gains that the other doesn't. `options` with `only_open: false` says what a closed option still needs ("Needs Barbarian level 16 (have 14)").
- An archetype's bonus feat slots can take only its own list (the Sohei's are Improved Initiative, Combat Reflexes, Dodge, the mounted feats, …): read it with `options` before you count on a slot.

## Workflow

1. Resolve every name the user gives with `search` (names, keys and initials: "BFT" finds the Brown-Fur Transmuter; words of what an item does, such as "DC", also find it by its summary). Spells too: `kinds: ["spell"]`, and `category` for a school or a descriptor ("Compulsion"). If a name could mean more than one item, ask the user which one before you build.
2. Start with `new_build`. Every Build tool takes the Build's code (`build`) and returns a new code, the planner `link` and `todo`, the choices still open, each by its address (`options` only with `envelope: true`).
3. Fill the Build start: race, racial ability, faith, alignment, background, point-buy scores.
4. Add levels with `level.N.class` (N one past the last level adds one) and fill each level's open choices: feats, Feature selections, ability increases, skill points.
5. Fill the Mythic ranks once their levels are reached: the first ascension at MR 1, the path at MR 3, and each rank's picks.
6. Give the user the final `link`.

## Planned whole

Once you have planned the whole Build, send it to `validate` as one map of address → value (the values `choose` takes; the order doesn't matter). It returns every mistake at once, each failure under the earlier one that causes it: fix the roots first (their `children` often go away), `validate` again, then keep the code it returns with `status: "valid"` (its `todo` lists what is still open) and `choose` the rest.

Keep the map in a file: a branch, a swapped feat or a moved pick is then an edit of one line and one more `validate`, and the file is the record of what the Build holds.

## Batching

Send many steps in one `choose`. It stops at the first refused step: read `rejected` (its issue and reason), fix that step, and resend the rest from the code it returned. `choose` on a slot that already holds a pick replaces it, when nothing later needs the old one.

## Values

Take them from `options`, which lists what a choice can take now and why the rest is closed. A parameterised feat or mythic ability (Favorite Metamagic: a metamagic) needs `{key, parameter}`: list them with `options {values: true}`. Whether a mythic feat takes a parameter is the data's: Improved Critical (Mythic) takes its weapon, Weapon Focus (Mythic) takes none; a refusal with `needs_parameter` or `not_an_option` names the form it wants. A `todo` entry on a filled slot is a pick whose value is still open: choose the value there. A spellcasting prestige class needs `advances`, the classes it advances (`[]` for none). A point-buy score or a skill's ranks take the number to reach; `level.N.skill` takes a map `{skill: ranks}` to spend a level's skill points in one step. `compact: true` on `options` and `search` leaves out summaries (and `search`'s prerequisites and guide links) when you need only the names.

## Branching

For alternatives (Lich or Aeon), build the shared part once, `choose` each alternative from that same code, fill each branch, then `compare` the codes. Report its differences as facts, without ranking them by your own judgement.

## Gear

A Build has no gear unless the user asks for it. Never add an item on your own, nor to make a pick legal: when a pick needs a score the Build lacks, say so, and ask whether the user wants an item for it.

- **Adding what the user asked for.** `search` with `kinds: ["gear"]` finds an item by its name, its place ("belt") or its bonus ("Dex", "enhancement"), and only that filter finds gear; each match names its `places`. `choose` its key at its place: `gear.belt`, `gear.boots`, `gear.bracers`, `gear.cloak`, `gear.glasses`, `gear.gloves`, `gear.headwear`, `gear.necklace`, `gear.ring.0` and `gear.ring.1`, `gear.body` (a robe or an armour), `gear.weapon`; a manual, tome or other usable item at `gear.usable.<its key>`, any number of them. `options` on a place lists its items; `none` there, or `withdraw` of the address, takes the item off. In `choose`, put the item before the pick it opens; in `validate`'s map, gear goes at the same addresses (`"gear.belt": "<item key>"`), in any order.
- **What it counts.** An item's always-on bonuses (its `summary`, typed: the highest of each type counts) count from level 1, for every prerequisite and every number the planner computes, but never for a level's skill points. Its `other_effects` are text: no number counts them.
- **Picks only gear allows.** An item can open a pick the character's own scores don't: a belt's Dexterity for Greater Two-Weapon Fighting. `options` marks such an option `{"with_gear": "With <item>"}`, and a step that would leave a pick allowed only by the gear `{"leans_on_gear": …}`; a `choose` or `withdraw` step that does so is applied and names those picks in its result's `leans_on_gear`, and `choose`'s `moved` says when a ring moved from the other ring place. `sheet`'s `gear_dependent` lists every choice only gear allows, with the items it needs (and `with_gear` on its line). An item a pick needs can't be withdrawn, or replaced by an item that doesn't keep the pick (a Dex +4 belt can replace a Dex +2 one), until the pick is withdrawn: the refusal names the pick.
- **Both numbers.** `sheet`'s numbers count the Build's gear, and `naked` gives the ones it changes without it. `compare` and `improve` rank by the naked numbers (each Build with no gear at all) unless you pass `with_gear: true`; the result's `numbers` says which, and `compare` gives each Build's other numbers beside. Neither ever adds an item: put the same items on every branch, or compare them naked.
- **What it still leaves out.** The planner's items are those that give an ability score bonus, with the manuals and tomes. A weapon's or an armour's enhancement, armour class, items without an ability score bonus, relics and artifacts aren't modelled yet: what they add is your own game knowledge, labelled as such.

## Ranking by a number

When the user's goal is a number the planner computes (a spell DC, caster level against spell resistance, concentration, the first attack's bonus or the damage bonus per hit), the planner ranks by it, and that ranking is a fact. `compare` with `by` (`{"caster_line": {"school": "enchantment", "subschool": "compulsion"}}` or `{"attack_line": {"weapon": "greatsword"}}`, or `{"attack_line": {}}` for each Build's own Weapon Focus weapon) orders the Builds by that number at their last level. `improve` with a Build, a choice's address and the same `by` takes each option open there and orders them by the change each makes to the number; `no_change` lists those that change nothing. Both rank by the naked number unless you pass `with_gear: true` (see Gear), and `improve` never proposes an item. Without a `weapon`, `improve` reads the Build's Weapon Focus weapon and refuses when the Build has none or an option would change it (the slot holding that Weapon Focus, or an earlier one open to another): give `weapon` there. Say what the number leaves out (`leaves_out`: gear beyond its always-on bonuses, buffs, conditional effects such as rolling twice or a gaze): an option in `no_change` may still matter through its summary, and which Build or option is better overall stays your judgement, to label as such.

## The honesty boundary

- State as fact only what the tools return: every choice in the Build is legal under the planner's rules at the level it is taken; the link opens in the planner; and the features, numbers (BAB, saves, scores, skill ranks, caster level from class levels, spell-level access) and path features the sheet lists.
- The attack line (`sheet` and `compare` give it for the weapon of the Build's earliest Weapon Focus, by level and then weapon key, or for a `weapon` you give; none without either) is fact too: each attack's bonus, damage per hit, Power Attack's trade and the critical range, from BAB, the character's own scores and the weapon feats held. It names its weapon. It leaves out rage, stances, mythic feats, class features, buffs and gear but for its ability score bonuses (`attack_leaves_out` lists all it leaves out): label anything you add for them as your own game knowledge.
- The caster line (`sheet` or `compare` with a `caster`: a school, and optionally a subschool, an energy type and metamagic) is fact too: the save DC of each spell level the casting class can cast, with its breakdown by source (the casting ability's modifier, Spell Focus and its kin, racial DCs, bloodline arcana, School Power), caster level against spell resistance and the concentration bonus, from the class levels, the character's own scores and the items held. It leaves out gear the Build doesn't hold and what items do beyond their always-on bonuses, buffs, abilities switched on in a fight (the Aeon's gaze), bonuses that grow with class level and what changes the target's roll instead of the DC (`caster_leaves_out` lists all it leaves out): label anything you add for them as your own game knowledge. Which spells have a school or subschool is in the spell catalogue.
- The totals (`sheet`'s `totals`) are fact too: initiative, each save and each skill's total, with what each counts, from the character's own scores, the class progressions, the skill ranks and class skills and the flat bonuses of the items held (Improved Initiative, Iron Will, Skill Focus, racial and background bonuses). They leave out gear the Build doesn't hold and what items do beyond their always-on bonuses, buffs, bonuses against one kind of effect or for one use of a skill (Persuasion to intimidate) and bonuses that grow with class level (`totals_leave_out` lists all they leave out): label anything you add for them as your own game knowledge.
- The planner's numbers count the Build's gear, and only the gear it holds: a Build has no gear unless the user asks for it. `sheet`'s `naked` gives the numbers without it, and `compare` and `improve` rank by those unless you pass `with_gear: true`. The attack line's weapon is the Build's Weapon Focus weapon unless you give one. Gear is part of the Build and its link, and counts for prerequisites: a headband can open a feat, and `gear_dependent` says which. What gear the planner doesn't model would add is your own judgement: label it as such, beside the planner's numbers, never in place of them.
- The spell catalogue (`search` and `describe` with `kind: spell` or `kind: spell_list`) is fact too: each spell's school, descriptors (its subschool among them), target, duration, saving throw, spell resistance, casting time and material component as the guide prints them, and every spell list it is on with its level there, the classes and archetypes that cast from each list, and a footnote limiting an entry to an archetype or a story branch. It is reference data: no Build choice names a spell.
- Most items carry a `summary`, spells too: what the guide says the item does, in one sentence in the planner's own words, checked against the guide. Cite it as the planner's summary of the guide, not as a rule the planner enforces: the engine checks prerequisites, not effects.
- A few items carry `interactions` in `describe`: what the guide says about how the item combines with others (or that it doesn't say), in the planner's own words. Cite it as the planner's note on the guide; where it says the guide doesn't say, any answer is your own game knowledge.
- Label everything else as your own game knowledge: what an item or a spell does beyond its summary and facts, how items and spells interact, spells known or prepared, spells per day, bonus spells the catalogue lacks (a shaman spirit's, an oracle mystery's, the Beast Tamer's), merging a Mythic spellbook into a class's, what a duration or a damage roll comes to at a caster level, caster level arithmetic beyond the caster line, item and buff effects, summons' stats, damage per round, and which choice is better. The planner checks none of it.
- Guide links (`guide_url`) are for the user to read: never fetch them yourself.

## Choices the planner doesn't hold

Some of a character's choices are made in the game and have no address in a Build: a fighter's Weapon Mastery weapon, a weapon group a feature trains, which Demonic aspects are active at once, a Ki power spent each round. List each one the plan depends on in the report, as a choice to make in the game.

## Report

1. The final `link` of each Build, one per branch.
2. The Build in short: race, faith, alignment, scores, class levels, Mythic path, and the picks that carry the goal.
3. **The planner's facts**: the numbers the tools returned, with the weapon each attack line is for, and whether they are naked or count the gear the user chose (say which).
   A Build that holds gear lists it, and names every pick that relies on it with the item it needs (`sheet`'s `gear_dependent`): "Greater Two-Weapon Fighting (level 19) needs the Belt of Incredible Dexterity +2".
4. **Your estimates**: every number you worked out yourself, each with its inputs and assumptions, marked as your own.
5. The choices to make in the game, and what you didn't check.
