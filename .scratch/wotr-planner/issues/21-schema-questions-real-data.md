# Answer the schema questions from the real data

Type: research
Status: resolved
Blocked by: 17, 18, 19, 20
Map: [WotR Build Planner](../map.md)

## Question

With the hand-fixed real `game-data/`, answer each item of the map's **Schema questions for the real guide data** with numbers. This is read-only: change no files except this ticket.

- How many archetype level rows differ from their base class, and in which columns?
- Do all classes follow full/¾/½ BAB and good/poor saves? List the exceptions.
- Are kept ranks always 1..n?
- Are any grants duplicated within one row?
- Are selection options shared across selections?
- `one_of` slot vs feat-granting selection: what does the data show?
- Are `level_cap` and `further_ascensions` independent?
- Is mythic `group_name` a closed set?

For each, recommend keep or change, and the schema or type change it implies. The user decides; this ticket only reports.

## Answer

Researched 2026-11-01, read-only. Every number comes from read-only SELECTs on `wotr-planner-postgres` (5434), whose stored seed hash (`9031395e…`) equals a fresh hash of `game-data/**/*.ron`, so the database holds exactly the hand-fixed, bare-key data at commit `0a9ec2d`. Where a number needed the guide (table columns, typos), I read the saved pages in `guide-cache/` offline. No repo file changed except this ticket and one line in `map.md`.

Scale: 26 base classes (520 rows), 196 archetypes (3920 rows, all 20 levels), 13 prestige classes (130 rows), 49 feature selections with 579 options, 4026 granted features, 687 granted selections and 618 feat slots.

### 1. Archetype level rows vs the base class

**Findings.** Of 3920 archetype rows, **2298 (58.6%) are identical** to the base class row at the same level. **1622 (41.4%) differ**, broken down by column:

| Column | Rows that differ | Archetypes |
| --- | --- | --- |
| `bab`, `fort`, `reflex`, `will` | **0** | 0 |
| `arcane_spell_level` | 20 | 1 (Eldritch Scoundrel gains arcane casting, 1→6) |
| `divine_spell_level` | 52 | 3 (Tandem Executioner, Divine Guardian and Warrior of the Holy Light lose casting; the guide agrees, e.g. "Loses all spellcasting") |
| granted features | 1428 | 196 |
| granted selections | 206 | 64 |
| feat slots | 221 | 70 |

- Every archetype changes at least one row. The median archetype changes 8 of its 20 rows (range 1–20), and level 1 is the most changed (173 of 196 archetypes).
- 45 rows differ only in spell levels; 1577 differ in their grants.
- The guide's archetype tables have no BAB, save or spell columns (`Level | Feats | Special 1 …`). Only each class's base table has `AB | Fort | Reflex | Will`. So the 3920 × 4 BAB/save numbers on archetype rows are copies of the base class's, not guide facts.
- The rows the archetypes copy have already drifted. Ticket 18 had to add sneak attack dice to the base class and 6 archetypes one by one, and the base wizard lacked the bonus-feat slots its archetypes had.
- `data/tests/files.rs`'s allow-list test runs only on the fixture. On the real data it would also flag the three archetypes that lose spellcasting (they're correct).

**Recommendation: change** (this reverses the user's ticket-05/14 choice of full rows, so it's the user's call). An archetype stores only the levels where it differs from its base class:

- **Rust:** `Archetype.levels: Vec<ArchetypeLevelRow>`, with `ArchetypeLevelRow { level, spell_levels: Option<SpellLevels>, features, feat_slots, selections }` and `SpellLevels { arcane: u8, divine: u8 }`. A row that's present replaces the base row's grants in full, and its `spell_levels` replace the base's when set. A missing level uses the base row. BAB and saves always come from the class (see 2). `GameData::check` gains "archetype row identical to the base row" and "spell override equal to the base".
- **Schema:** on `class_level`, `bab`/`fort`/`reflex`/`will` leave the table if 2 is adopted. Otherwise they become NULL on archetype rows, with `CHECK ((archetype_key IS NULL) = (bab IS NOT NULL))` and the same for the saves. The spell-level columns become nullable, with a CHECK that they're NOT NULL when `archetype_key IS NULL`. No grant table changes.
- **Saves:** 2298 `class_level` rows, 1030 `granted_feature`, 353 `granted_selection`, 253 `feat_slot` and 1389 `feat_slot_option` rows go away, and so do the 15 680 copied BAB/save numbers.
- **Trade-offs:**
  - Gains: a base-class fix reaches every archetype that doesn't override that level, and the files test and its allow-list go away.
  - Costs: an archetype's full table is no longer in one place in the RON file; the engine and UI merge it. The engine also needs one `effective_row(class, archetype, level)`.
  - **Middle option:** keep the full grant rows and drop only the copied numbers. That fixes the derived-value problem but not the drift.

### 2. BAB and save progressions

**Findings.** BAB has **no exceptions** in any of the 39 classes:

| BAB | Base classes | Prestige classes |
| --- | --- | --- |
| full | 9 | 6 |
| ¾ | 13 | 3 |
| ½ | 4 | 4 |

Saves follow good/poor in 108 of 117 class columns. The formulas are: base good 2 + ⌊L/2⌋, base poor ⌊L/3⌋; prestige good ⌊(L+1)/2⌋, prestige poor ⌊(L+1)/3⌋. Each deviation is a single cell, and every one matches the guide's own table, so they are the guide's cells (likely typos) and not extraction errors:

| Class | Cell | Guide | Formula |
| --- | --- | --- | --- |
| hunter | Will L7 | +3 | +2 (poor) |
| paladin | Will L20 | **+20** | +12 (good) |
| shifter | Reflex L2 | +2 | +3 (good) |
| warpriest | Will L20 | +11 | +12 (good) |
| arcane-trickster, loremaster, mystic-theurge, student-of-war, winter-witch | Will L4 | +3 | +2 (prestige good; Dragon Disciple and Signifer have +2) |

Saves aren't a prerequisite (ticket 14, decision 3), so the 9 cells only affect what the sheet displays. BAB, which is a prerequisite, has no exceptions.

**Recommendation: change.** Store the progression, not the numbers:
- **Rust:** `Class { bab: BabProgression, fort: SaveProgression, reflex: SaveProgression, will: SaveProgression, .. }`, with `enum BabProgression { Full, ThreeQuarters, Half }` and `enum SaveProgression { Good, Poor }`. `ClassLevelRow` loses `bab`, `fort`, `reflex` and `will`. The engine computes them with the base or prestige formula (`Class` already knows which).
- **Schema:** domains `bab_progression` and `save_progression`, four NOT NULL columns on `class`, and the four columns dropped from `class_level`. That removes 2600 class-row numbers, plus the archetype copies from 1.
- **User decision:** the 9 cells. My recommendation is to let the formula win and treat them as guide typos (paladin's +20 certainly is). The exact alternative is a `save_exception (class_key, level, save, value)` table with 9 rows, which is a guide fact and not derived, but it's clutter for display-only numbers.
- **Spell levels (aside):** the 222 base and archetype tables use only 6 spell-level curves (none, the 6-level, 9-level prepared and 9-level spontaneous curves, 4-level from L4 for bloodrager and ranger, and 4-level from L5 for paladin). The same rule could apply to them, but spell-level access is a prerequisite and the ranger/paladin starts were hand-checked, so I'd leave them for now.

### 3. Kept ranks

**Finding: always 1..n**, in all 4 switches: Legend `{1, 2}`, and Gold Dragon, Swarm That Walks and Devil `{1..7}`. No guide prose describes a gap.

**Recommendation: change.**
- **Rust:** `PathSwitch.kept_ranks: BTreeSet<u8>` becomes `kept_through_rank: u8`, and `rank_after_switch()` returns `kept_through_rank + 1`.
- **Schema:** drop `path_switch_kept_rank` and add `path_switch.kept_through_rank SMALLINT NOT NULL REFERENCES mythic_ascension (rank)`. The "below the switch rank" check stays in `GameData::check`. That's one table fewer (24).
- **Trade-off:** gaps can't be expressed, and none exist. **Do this together with the partial-keeps follow-up (C below):** that one needs a second cut-off for path picks, or a list of dropped selections, so the shape should be decided once.

### 4. Duplicated grants within one row

**Finding: yes, rarely, and they're meaningful.**
- Features: 2 rows. `cleric/separatist` L8 grants `domain-ability` ×2, and Trickster MR 3 grants `sneak-attack` ×2.
- Selections: 13 rows (27 grant rows). `warpriest/blessings` ×2 at L1 and L10 on the base class and its archetypes (11 rows), `demon/major-demonic-aspect` ×2 at MR 5, and `gold-dragon/improved-gold-dragon-breath` ×3 at MR 10.
- Feat slots: 1 row, Gold Dragon MR 9 with 3 identical `General` slots.

**Recommendation: keep the multiset meaning; optional schema-only normalization.**
- `granted_feature` and `granted_selection` could become `(owner, key, count u8 CHECK (count >= 1))` with `UNIQUE NULLS NOT DISTINCT (class_level_id, mythic_rank_id, key)` in place of the keyless surrogate `id`. The Rust `Vec`s stay as they are: the seed counts and the loader expands.
- `feat_slot` keeps its `id`, since `feat_slot_option` hangs off it.
- **Trade-off:** a natural key and no bag table, against a little grouping code in the seed and loader. Low priority.

### 5. Options shared across selections

**Finding: yes, by copying.** 47 of the 579 options (8%) are copies of another selection's option (same guide anchor):

| Selection | Copies | From | Prerequisites |
| --- | --- | --- | --- |
| `slayer/slayer-talents` | 36 | `rogue/rogue-talents` (ticket 18) | 20 identical; 16 differ only in naming the slayer's own `slayer/advanced-talents` (or a sibling copy) instead of the rogue's |
| `shaman/hex` | 11 | `witch/hex` | all 11 identical |

- Each copy also copies its prerequisite tree.
- Because a copy has its own key, a rogue/slayer multiclass can take the same talent twice today: the already-held check compares keys.
- Same name, different anchor (not copies): three barbarian rage powers and stalwart defensive powers (Clear Mind, Increased Damage Reduction, Internal Fortitude), plus two coincidences (Healing hex vs patron, Strength blessing vs patron).
- Wanted but not modelled:
  - Fey Tricks (rogue talents + witch hexes)
  - Underground Chemist (rogue talents + alchemist discoveries)
  - Second Spirit / Wandering Hex
  - Winter Witch's hexes

**Recommendation: change, to one option with many selections.**
- **Rust:** `FeatureSelection { options: Vec<SelectionOption>, #[serde(default)] shared_options: BTreeSet<SelectionOptionKey> }`. An option keeps the key of the selection that defines it, and the engine offers own ∪ shared.
- **Schema:** `selection_shared_option (selection_key → feature_selection, option_key → selection_option, PRIMARY KEY (selection_key, option_key), CHECK (NOT starts_with(option_key, selection_key || '/')))`.
- **Data:** delete the 47 copies and their trees. The slayer shares the 36 rogue talents and the shaman shares the 11 witch hexes. This also answers "a slot drawing on two selections": Fey Tricks becomes its own selection sharing both lists.
- **Rejected:** a whole-selection `includes` would be simpler, but it can't express the shaman's 11 of 26 hexes or the Underground Chemist's six discoveries.
- **User decision: context-dependent prerequisites.** The 16 advanced slayer talents need the *slayer's* Advanced Talents, and a Sylvan Trickster's Major hex is gated on rogue level 10, not witch 10. There are three ways to handle it:
  - (a) `Any` over each class's feature: simple, but lets a rogue 10 / slayer 2 take advanced talents through the slayer slot;
  - (b) gate in the slot: the class grants a separate "advanced" slot from level 10;
  - (c) a per-membership extra prerequisite, `selection_shared_option.prerequisite_root_id` (exact, but adds a prerequisite owner).
  - I'd take (b) where the guide has a level-based unlock and (a) otherwise.

### 6. `one_of` slot vs feat-granting selection

**Findings: they don't overlap, so keep both.**
- 307 of the 618 feat slots are `one_of` (208 bonus-list, 99 checked), drawing on only **70 distinct lists** that hold 3714 `feat_slot_option` rows.
- Only 12 selection options carry a `feat` (6 rogue talents and their 6 slayer copies), and no selection is made only of feat-granting options. So no selection duplicates a fixed-list slot.
- **27 of the 70 lists hold a single feat (91 slots).** These are fixed grants, not choices: Scribe Scrolls ×17, Brew Potions ×10, Improved Unarmed Strike ×9, Weapon Finesse ×8, Throw Anything ×8 and so on. Each still asks the player for a pick, and the Build (URL) stores it, which puts a derived value in the Build.
- `ignores_prerequisites` is a function of the list: no list appears with both values.
- 10 lists are shared across classes, and the other 33 multi-feat lists repeat level after level (the monk list on 6 levels × 9 monk tables).
- `ranger/combat-style-feat` (5 style options) is **granted by no row**. The ranger's slots offer the union of all styles' feats, so the style choice is the "options grant slots over levels" follow-up (A below).
- Ticket 17's "option grants feat" follow-up already exists: `selection_option.feat_key`, and `hold_pick` holds the feat. So the 14 `Any` alternatives that name the rogue Weapon Focus, Skill Focus or Intimidating Prowess talents are redundant (harmless).

**Recommendation: keep both concepts, plus two changes.**
1. **Granted feats (recommended):** `ClassLevelRow.feats: Vec<FeatKey>` (held without a prerequisite check, like a bonus-list pick) and `granted_feat (id, class_level_id → class_level, feat_key → feat)`. The 91 single-feat slots become grants.
   - Gain: 91 fewer slots and picks.
   - Gain: classes can then grant the real Armor/Shield Proficiency feats (follow-up D).
   - Cost: the engine must decide what happens when a granted feat is already held (no extra slot; the game behaves the same).
2. **Named feat lists (optional):**
   - **Rust:** `FeatList { key, name, guide, feats, ignores_prerequisites }` catalogs, with `FeatSlot::OneOf { list: FeatListKey }`.
   - **Schema:** `feat_list` + `feat_list_member`, and `feat_slot.feat_list_key` in place of `feat_slot_option` and `feat_slot.ignores_prerequisites`. That's 3714 → 535 member rows, and the bonus-list flag moves onto the list, where the data shows it belongs.
   - Cost: lists need a home in the RON layout (per class file, plus one shared file for the 10 cross-class lists), and a growing list (ranger Archery 2/6/10) is three lists.

### 7. `level_cap` and `further_ascensions`

**Finding: the data can't separate them.** There are 4 switches: Legend (cap 40, no further ascensions, keeps 1–2) and three others (no cap, further ascensions, keep 1–7). All three fields are perfectly correlated.

**Recommendation: keep both columns.** Each is a separate statement in the guide ("level cap 40", "no further mythic ranks"), and no stated rule derives one from the other. Collapsing them into a `Legend`-shaped enum would put a path's name into the rules, and "ascensions continue iff the switch keeps every lower rank" fits 4 rows by coincidence. No change.

### 8. Mythic `group_name`

**Finding: closed.**
- Mythic abilities use exactly the 6 h4 headings under the guide's "Mythic Abilities": Melee 7, Ranged 5, Magic 13, Healing 2, Miscellaneous 9, Class 27.
- Mythic feats use exactly the 6 under "Mythic Feats": Melee 13, Ranged 3, Armour and Shield 14, Magic 7, Miscellaneous 13, Combat Manoeuvre 6.
- That's 8 distinct strings repeated over 119 rows.

**Recommendation: change (low priority).**
- **Rust:** `MythicAbility.group: MythicAbilityGroup { Melee, Ranged, Magic, Healing, Miscellaneous, Class }` and `MythicFeat.group: MythicFeatGroup { Melee, Ranged, ArmourAndShield, Magic, Miscellaneous, CombatManoeuvre }`, each with a `name()` for display, as `FeatCategory` does.
- **Schema:** domains `mythic_ability_group` and `mythic_feat_group` on `group_name`.
- **Trade-off:** a typo becomes a parse error instead of a new group, and the labels live in Rust. A shared lookup table would be the alternative, but domains follow the existing precedent.

### Also: owner prefixes (deferred here by ticket 22)

If keys dropped their owner prefix, they would collide:

| Kind | Keys | Collisions without the prefix |
| --- | --- | --- |
| features | 1770 | 557 |
| selection options | 579 | 51 |
| feature selections | 49 | 1 |
| archetypes | 196 | 0 (but anchors like `paladin/divine-hunter-paladin` already disambiguate) |

**Keep them.** They're the identity, not a stored attribute. The cheap guard against drift is a `starts_with(key, owner || '/')` CHECK on `feature` and `feature_selection`, as `selection_option` already has. All 49 selections already pass it; 1768 of 1770 features do.

The two exceptions show an inconsistency worth a data fix:
- `mythic/hard-to-kill` (owned by Aeon, granted at MR 2 by 6 paths) is one shared key.
- Bypass Epic DR has 6 keys: 5 per-path `…/bypass-epic-dr` plus `mythic/bypass-epic-damage-reduction` for the Trickster.

### Follow-ups from the hand-fix tickets, prioritised

One candidate ticket per line. **U** = the user must decide something first; **I** = pure implementation once picked.

1. **(A) Selection options that grant over class levels**:
   - Covers bloodlines, schools, orders, spirits, ways, the ranger combat style, Dragon Disciple's bloodline continuation and Eldritch/Dragonheir Scion.
   - It's the largest gap: 39 report items, plus Disruptive, Winter Witch's Frost spirit, the bloodline feat lists and Dragon Disciple's entry.
   - **U:** approve ticket 18's `OptionLevelRow` shape, decide how archetypes that change a bloodline get their own selection, and decide whose class level drives an option's rows (Dragon Disciple levels continue sorcerer bloodline rows).
2. **MR 1–2 rows when the path is chosen at MR 3.** Today 12 path rows (30 feature grants, path-specific such as Force Reality) are never applied. **U:** a one-line rules decision. My recommendation: grant them retroactively on choosing the path, since every path page lists them. The code is small.
3. **(B) Shared options and a slot drawing on two selections.** Covers the slayer's 36 copies, the shaman's 11 hexes, Fey Tricks and the Underground Chemist; it fixes the double-pick bug and removes 47 copies. The shape is Q5 above. **U:** context-dependent prerequisites ((a), (b) or (c)).
4. **(D) Granted feats, and proficiency features vs proficiency feats.** Turn the 91 single-feat slots into `granted_feat`, then let classes grant the Armor/Shield Proficiency feats, which unblocks Armor Focus, Shield Focus, Shield Bash, Shield Wall, Tower Shield and the Medium/Heavy chain for martial classes. **U:** proficiencies as granted feats (data: each class's list) vs an engine alias from `<class>-proficiencies`. I'd choose granted feats.
5. **(C) Path switch rules, as one ticket:**
   - **I:** partial keeps (general picks vs path picks; Gold Dragon, Swarm and Devil drop the old path's MR 3–7 powers);
   - **I:** the previous-path condition (`PathSwitch.from: BTreeSet<MythicPathKey>`; Devil only from Aeon or Azata);
   - **U:** effects on switching (Legend's +4 to every ability score, and Gold Dragon's Perfect Body, Mind and Soul), which need a new "feature grants an ability bonus" concept, and whether such bonuses count toward ability-score prerequisites (natural-scores-only today);
   - **U:** whether Legend keeps the feats unlocked by Trickster's Perception II.
   - Combine with Q3's `kept_through_rank`.
6. **Feats taken more than once, and repeatable options.** Covers Skill Focus ×3 (Master of All), Magical Tail, the slayer's ranger combat feats "three times" and Increased Damage Reduction "twice". **U:** which feats repeat (a `max_picks` or `repeatable` fact on `Feat` and `SelectionOption`), and whether parameterised feats (Weapon Focus (X)) are modelled at all.
7. **Mythic-rank and story prerequisites.** **I:** a `MythicRank(n)` leaf (`prerequisite_node` kind `mythic_rank`, `amount` = rank) for the Gold Dragon breaths, also replacing the Aeon gazes' `CharacterLevel` stand-ins. **U:** whether story flags (Vow of Dahak, Uncorrupted) become Build-start choices or stay out of scope.
8. **"Advances one class, arcane or divine"** (Signifer, Loremaster, Winter Witch). **I:** a per-class `advances: Either | Each` (or derive "either" from the class), replacing the two always-true flags on these three.
9. **Prestige levels counting as another class.** Covers Diverse Training (Eldritch Knight as fighter levels for feats), Frost Power (Winter Witch stacks with witch/shaman for hexes) and Dragon Disciple bloodline levels (overlaps 1). **U:** scope: a `counts_as (class, as_class, for: feat prerequisites | hex progression)` fact, or just Diverse Training.
10. **Skipping only some prerequisites (Crusader).** The one archetype skips only BAB and fighter level. **U:** keep the over-permissive "skip all" (my recommendation) or add `ignores: All | Kinds(BTreeSet<PrerequisiteKind>)`.
11. **Druid Nature Bond.** **I, data only, now:** a `druid/nature-bond` selection (animal companion or domain) at L1, with Domain Zealot requiring its domain option. The domain's own powers wait on 1.

Smaller items in the same answers, also **I**:
- Gold Dragon's Dragon Feats: allow `ignores_prerequisites` on a path rank's general slot.
- Delete or grant the orphan `ranger/combat-style-feat` (it goes with 1).
- Unify the 6 Bypass Epic DR keys.
- Drop the 14 redundant rogue-talent `Any` alternatives.

Still **U** but only data: Assassin's "neutral or evil" reading, and Dragon Disciple's "no bloodline" and spontaneous-caster conditions.

### How this was measured

- Read-only `psql` through `docker exec` with `default_transaction_read_only=on`. The helper is `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/q.sh`, and the archetype comparison CTE is `cmp_cte.sql` next to it. It compares each archetype row with the base row by numbers and by the sorted multisets of features, selections and feat slots (kind, category, flag, sorted list).
- `patterns.py` (same folder) classifies BAB and saves against the formulas.
- `hash.py` reproduces `data::game_data_hash`, to confirm the seeded data is the current files.
- Guide checks parsed `guide-cache/*.html` offline.

## Decisions

**User, 2026-11-01: "All recommended".** Each recommendation as presented in the decision round, now the plan:

- **S1 Archetype rows:** sparse. An archetype stores only the levels where it differs from its base class; a missing level uses the base row. This reverses the full-rows choice of tickets 05 and 14. → [ticket 23](23-class-progressions-not-copies.md)
- **S2 BAB and saves:** each class stores its BAB and save progression types, and the engine computes the numbers. The 9 deviating cells stay as the guide prints them, in a small save-exception table (the guide is the source of truth). → ticket 23
- **S3 Small changes:**
  - `kept_through_rank` → [ticket 26](26-mythic-path-rules.md), with the other switch rules
  - grant counts, named feat lists, single-feat lists as granted feats, mythic group enums → [ticket 24](24-granted-feats-and-feat-lists.md)
- **S4 Shared options:** one option shared by several selections; a class-dependent prerequisite accepts either class's version with an "any of" (option (a)). → [ticket 25](25-shared-selection-options.md)
- **R1:** choosing a path at MR 3 grants its MR 1–2 rows. → ticket 26
- **R2:** classes grant the real proficiency feats. → ticket 24
- **R3:** Legend's +4 and mythic features' ability bonuses count toward prerequisites (the character's own scores, not gear), and a Trickster who switches to Legend keeps the feats Perception II unlocked. → ticket 26
- **R4:** repeatable feats are marked, and each copy records its skill or weapon, which prerequisites can name. → [ticket 27](27-repeatable-and-parameterised-feats.md)
- **R5:** story conditions (Vow of Dahak, Uncorrupted) are out of scope: allowed, with a note. Prestige levels counting as another class are in scope. Crusader keeps skip-all. → [ticket 28](28-class-rules-follow-ups.md)
- **R6:** Assassin's "neutral or evil" excludes neutral good: true neutral or any evil. → ticket 28
- **Bloodlines, schools, orders, spirits:** their own design ticket. → [ticket 29](29-design-option-level-grants.md)
- **Implementation items** (no decision needed) go with their tickets: owner-prefix CHECKs, unified Bypass Epic DR keys and the 14 redundant `Any` alternatives (24); partial keeps, the previous-path condition, the `MythicRank(n)` leaf and Gold Dragon's Dragon Feats ignoring prerequisites (26); advances one-or-each and Druid Nature Bond (28). The orphan `ranger/combat-style-feat` and Dragon Disciple's conditions go with bloodlines (29).
