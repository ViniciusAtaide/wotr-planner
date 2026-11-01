# Hand-fix the extracted prestige classes

Type: task
Status: resolved
Blocked by: 10
Map: [WotR Build Planner](../map.md)

## Question

Stage of the split extraction (user, 2026-11-01). Resolve the `## Prestige classes` section of `game-data/EXTRACTION_REPORT.md` by editing the RON files directly. AFK.

- **Files you may edit:** `game-data/prestige/*.ron` only. Plus the report's `## Prestige classes` section.
- **Focus:** Entry requirements (skill ranks, BAB, feats, spell level, alignment, minimum character level), spellcasting advancement rows, and features.

**Shared rules for the hand-fix stages** (these four run in parallel, one agent each):
- **Your area:** fix only this stage's section of `game-data/EXTRACTION_REPORT.md`, and only this stage's files (below). Other agents are editing the other files at the same time.
- **Source:** the saved pages in `guide-cache/`. Never fetch GameFAQs or any site.
- **Copyright:** commit facts only (names, numbers, prerequisites, keys, anchors). A `summary` stays empty unless it's one short line in your own words.
- **Validating:** use the database-free check command from ticket 10's Answer. Never seed, reset or migrate the dev database; the main session seeds once all stages are done. No schema or `rules` type changes. Anything that needs one becomes a proposed follow-up in your Answer.
- **Unclear items:** anything you can't resolve accurately from the guide stays in the report, with a note on why. Don't guess.
- **Done when:** your section of the report holds only unresolvable items, the check command passes, `cargo test --workspace` passes, and there are no commits.
- **Recording:** append an `## Answer` with fixed and remaining counts and the follow-ups, set `Status: resolved`, and add one line at the end of map.md's Decisions so far using Edit. Keep a dated `## Progress log`, because runs can be cut off.

## Progress log

- 2026-11-01: checked all 13 classes against `guide-cache/prestige-classes.html` (entry lines, progression tables, feature prose). Edited the 10 `game-data/prestige/*.ron` files listed below. `seed --check-only` and `cargo test --workspace` pass; every key in `prestige/` resolves to a defined key and every `prestige-classes#…` anchor exists in the page. Per the main session, `EXTRACTION_REPORT.md` and map.md are left for it to update from this Answer. Nothing committed.

## Answer

Report section `## Prestige classes`: 7 items. **1 fixed, 2 narrowed, 4 left**, plus 9 new items found while checking (below). BAB and saves matched the guide's tables in all 13 classes.

### Changes

- **Spellcasting advancement, from the CL column** (a row advances when CL goes up from the previous row):
  - Eldritch Knight: level 1 no longer advances (CL +0). It advances at 2–10.
  - Dragon Disciple: levels 1, 5 and 9 no longer advance (CL 0, 3→3, 6→6, matching its prose "apart from 5th and 9th"). It advances at 2, 3, 4, 6, 7, 8, 10.
  - Arcane Trickster, Mystic Theurge (arcane and divine), Signifer, Loremaster and Winter Witch advance at every level (CL +1 to +10), as before.
- **Entry requirements:**
  - Loremaster: added "Metamagic (any)" as an `Any` of the 12 `feat:metamagic-*` feats. *(report item fixed)*
  - Dragon Disciple: added `SpellLevel(Arcane, 1)`. *(report item narrowed: "spontaneously" stays)*
  - Winter Witch: added `Any([Archetype("archetype:druid/winter-child"), SelectionOption("selection:witch/patron/winter"), ClassLevel(class: "class:shaman", level: 1)])`. The shaman branch is a stand-in, because shaman spirits aren't a selection yet. *(report item narrowed)*
  - Arcane Trickster, "Sneak Attack (2d6)": was 2 dice from one source. It's now `AtLeast(2, …)` over each source at count 1 and count 2, plus `Feat("feat:accomplished-sneak-attacker")`, which totals the dice across sources. This allows the guide's own Rowdy 1 + Accomplished Sneak Attacker route. The class's own sneak attack was dropped from its prerequisite, since it referred to itself.
- **Features:**
  - Assassin sneak attack is now granted at 1, 3, 5, 7, 9 and Arcane Trickster's at 2, 4, 6, 8, 10 (one grant per die, as the Trickster path does), per the prose.
  - Stalwart Defender: removed the four bogus features `feature:stalwart-defender/1`…`/4` (the AC column). Defensive Powers is now picked at 2, 4, 6, 8 and 10 (prose: every even level), not just 2. Fearless Defense and Immobile require `ClassLevel(stalwart-defender, 4)`, and Increased Damage Reduction and Mighty Resilience require level 6.
  - Guide anchors pointed at the class instead of the feature's own heading: assassin poison use, sneak attack and uncanny dodge; arcane spellcasting for Arcane Trickster, Dragon Disciple, Eldritch Knight and Mystic Theurge; Mystic Theurge divine spellcasting and combined spells; Signifer order and armor training; Stalwart uncanny dodge; Student of War "Know Your Enemy - Swift".

Checked and correct as extracted: Aldori Swordlord, Duelist, Eldritch Knight, Hellknight, Hellknight Signifer, Mystic Theurge, Stalwart Defender and Student of War entry requirements. Known approximation, not counted: parameterised feats are stored bare (Weapon Focus (Dueling Sword), Skill Focus (Knowledge: Arcana or World), Skill Focus (Knowledge or Lore)).

### Report items that remain (for `## Prestige classes`)

Entry requirements:
- `prestige-classes#dragon-disciple`: "Must have a Draconic bloodline or no known bloodline". There's no "not" or "no bloodline" condition.
- `prestige-classes#dragon-disciple`: "Able to cast 1st level arcane spells spontaneously". `SpellLevel(Arcane, 1)` is stored, but there's no spontaneous-caster condition. The guide's list of qualifying classes is open-ended ("some Witch archetypes"), so an `Any` of classes would be a guess.
- `prestige-classes#winter-witch`: Shaman (Frost Spirit) is stored as `ClassLevel(class: "class:shaman", level: 1)`. It should become the Frost spirit's selection option once the base-classes stage builds shaman spirits (`shaman#frost`).
- `prestige-classes#assassin` *(new)*: "Alignment must be neutral or evil" is ambiguous. The data reads it as any alignment with a neutral part, or evil, which includes Neutral Good. Read as "not good", Neutral Good would be excluded.

Prose-only rules:
- `prestige-classes#hellknight-signifer`, `#loremaster`, `#winter-witch` (3, unchanged): each advances one class, arcane or divine. It's stored as both flags, so the engine accepts one arcane plus one divine class. The level rows themselves are right.
- `prestige-classes#increased-damage-reduction` *(new)*: "may be selected twice". A selection option can't be held twice.
- `prestige-classes#order`, `#order-hellknight-signifer` *(new)*: a Hellknight who wasn't an Armiger, and every Signifer, picks one of seven orders, but there's no selection for it. So Pentamic Faith (needs Order of the Godclaw, stored as the Armiger's option) is closed to a non-Armiger Hellknight.
- `prestige-classes#diverse-training` *(new)*: Eldritch Knight levels count as Fighter levels (and caster levels) for feat prerequisites.
- `prestige-classes#blood-of-dragons` *(new)*: Dragon Disciple levels continue the sorcerer bloodline progression, and a non-sorcerer picks a Draconic bloodline. Neither is in the data.
- `prestige-classes#frost-power` *(new)*: Winter Witch levels stack with Shaman and Witch levels for hexes, including Major and Grand hex access.

### Proposed follow-ups (type changes, not made)

- A per-class "advances one class, arcane or divine" flag (for example `advances: Either | Each` on `Class`) for Signifer, Loremaster and Winter Witch.
- A spontaneous-caster condition and a "no bloodline / Draconic bloodline" condition for Dragon Disciple.
- Repeatable selection options (Increased Damage Reduction).
- "Counts as class X levels" for feat prerequisites (Diverse Training, Frost Power), and bloodline progression from Dragon Disciple levels.

### Suspected extractor bugs

- `tables.rs`: a CL cell starting with `+` marks the row as advancing, `+0` included. A row should advance only when CL rises over the previous row.
- Non-standard table columns become features. Stalwart Defender's "AC" column gave features named "+1"…"+4".
- Feature anchors don't match disambiguated or grouped headings ("Sneak Attack (Assassin)", "Arcane Spellcasting (Mystic Theurge)", "Poison Use - …" under `#poison-use`, "Combined Spells (1st)" under `#combined-spells`) or the table's "Signifier" spelling of `#signifer-armor-training`. They fall back to the class anchor.
- Grants stated only in prose ("every odd level", "every even-numbered level") are missed when the table shows the first one only: sneak attack dice, Stalwart Defensive Powers.
- Option prose "Requires 4th level." isn't read as a prerequisite.
- **Cross-category (base classes):** `classes/rogue.ron` grants `feature:rogue/sneak-attack` at level 1 only, so `Feature(rogue/sneak-attack, count: 2)` (Arcane Trickster, and likely other feats) is never met by rogue levels. The other sneak-attack sources may share this.
