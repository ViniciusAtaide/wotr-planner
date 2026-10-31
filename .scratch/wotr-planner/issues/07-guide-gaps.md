# Filling the guide's gaps

Type: grilling
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

The guide is the source of truth, but [How the guide is laid out](01-guide-anatomy.md) found gaps and found content we haven't scoped yet (see [research §5](../research/guide-anatomy.md)). What does the planner do about each?

**Scope**

- **Prestige classes:** are the guide's 13 prestige classes in? Their entry requirements include skill ranks, BAB, feats, minimum character level and alignment.
- **Alignment:** do we check it as a prerequisite? Classes and prestige classes restrict it, but the map's list of checked prerequisites doesn't include it.
- **DLC content:** do we show it always, hide it, or let the user toggle it? The guide marks it only in prose.
- **Path-specific picks:** do these count as mythic choices in the planner? Examples: Aeon gazes, Trickster bonus feats, Devil and Swarm That Walks.

**Data the guide doesn't give**

- ~~**Combat Feats:** there's no "Combat Feat" tag, yet fighter bonus slots accept "any Combat Feat".~~ Answered; see Comments.
- **Caster level:** prerequisites are often missing, for example Spell Penetration. Base classes have no CL column.
- **Feats without prerequisites:** 55 feats have no Prerequisites paragraph. Are they genuinely free, or is the text missing?
- **Skill ranks:** the maximum ranks per skill isn't stated.

For each of these: does the user supply the fact from game knowledge, do we allow a second source, or do we leave the check out? Where are facts that don't come from the guide recorded? Possibly an override file with a `source:` field.

**Committing data**

- Is it acceptable to commit the extracted game facts (names, numbers, prerequisites) next to our own summaries and links? The guide's notice permits personal use only.

## Comments

**2026-10-31, user:** A combat feat is basically any feat outside the guide's class, racial, skill, teamwork and magic categories.

On the guide's `feats` page, that makes combat feats everything under:

- `#weapon-feats`
- `#ranged-feats`
- `#unarmed-combat-feats`
- `#armour-and-shield-feats`
- `#miscellaneous-feats`
- `#mounted-combat-feats`
- `#combat-manoeuvres`

The word "basically" leaves room for exceptions; those go in the override file.

**2026-10-31, user:** `#miscellaneous-feats` counts as combat. The combat feat rule is settled.

**2026-10-31, user:** answers to the remaining questions.

- **Prestige classes:** in scope. Source: the guide's `prestige-classes` page.
- **Alignment:** checked as a prerequisite. The user pointed to the [fextralife Alignments page](https://pathfinderwrathoftherighteous.wiki.fextralife.com/Alignments).
  - That page lists only the 9 alignments (LG, NG, CG, LN, N, CN, LE, NE, CE) and the paladin example: a paladin who stops being Lawful Good loses all paladin spells and features until LG is restored. It has no per-class restriction table.
  - The per-class and per-prestige-class restrictions come from the guide instead: the class characteristics list and the prestige entry requirements.
  - The fextralife robots.txt also blocks ClaudeBot and forbids building datasets, so agents don't fetch it again.
- **DLC:** no gating. All DLC content is included, because the user owns every DLC.
- **Caster level vs spell level:** a spell-level prerequisite is not a caster-level prerequisite. Most spellcasting prestige classes require *access to a spell level*. Examples:
  - Eldritch Knight: martial weapon proficiency + can cast 3rd-level arcane spells
  - Loremaster: 3rd-level spells + the listed feats

  Use the guide's data as it is: spell-level access comes from the class spells-per-day tables. Don't add caster-level prerequisites the guide doesn't state.
- **Feats with no prerequisites listed:** use the guide as it is. All 55 are treated as having no prerequisites. (Inferred from "continue your plan with the existing data in gamefaqs".)
- **Skill ranks:** the maximum is 20, or 40 on Legend.
- **Committing data:** OK to commit the extracted facts. The project becomes a git repo.

**2026-10-31, user:** path-specific mythic picks count as planner choices. That covers Aeon gazes, Trickster bonus feats and the picks for Devil, Swarm That Walks, Gold Dragon and so on.

## Answer

The guide stays the only source, with three facts from the user added on top. Facts that don't come from the guide go in the override file with `source: user`.

**Scope**
- **In scope:** prestige classes and path-specific mythic picks.
- **DLC:** all content included, with no gating.
- **Alignment:** checked as a prerequisite. The restrictions come from the guide's class and prestige class pages.

**Data rules**
- **Combat feats:** every feat outside the class, racial, skill, teamwork and magic categories. Miscellaneous counts; exceptions go in overrides.
- **Spell-level access vs caster level:** these are separate prerequisites. Spell-level access is worked out from the spells-per-day tables. Caster level is checked only where the guide states it.
- **Feats with no prerequisites listed:** they have none.
- **Skill rank cap:** 20, or 40 on Legend.

**Data handling**
- Committing the extracted facts is OK. The project is a git repo, and `guide-cache/` is ignored.
- Agents never fetch GameFAQs or fextralife.
