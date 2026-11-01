# Hand-fix the extracted base classes

Type: task
Status: resolved
Blocked by: 10
Map: [WotR Build Planner](../map.md)

## Question

Stage of the split extraction (user, 2026-11-01). Resolve the `## Base classes` section of `game-data/EXTRACTION_REPORT.md` by editing the RON files directly. AFK.

- **Files you may edit:** `game-data/classes/*.ron` only. Plus the report's `## Base classes` section.
- **Focus:** Progression table cells, bonus-list slots (`ignores_prerequisites`, decision D1), archetype rows, class skills, feature selections and their options' prerequisites, and prose rules (totems, advanced rogue talents at 10, Major/Grand hexes).

**Shared rules for the hand-fix stages** (these four run in parallel, one agent each):
- **Your area:** fix only this stage's section of `game-data/EXTRACTION_REPORT.md`, and only this stage's files (below). Other agents are editing the other files at the same time.
- **Source:** the saved pages in `guide-cache/`. Never fetch GameFAQs or any site.
- **Copyright:** commit facts only (names, numbers, prerequisites, keys, anchors). A `summary` stays empty unless it's one short line in your own words.
- **Validating:** use the database-free check command from ticket 10's Answer. Never seed, reset or migrate the dev database; the main session seeds once all stages are done. No schema or `rules` type changes. Anything that needs one becomes a proposed follow-up in your Answer.
- **Unclear items:** anything you can't resolve accurately from the guide stays in the report, with a note on why. Don't guess.
- **Done when:** your section of the report holds only unresolvable items, the check command passes, `cargo test --workspace` passes, and there are no commits.
- **Recording:** append an `## Answer` with fixed and remaining counts and the follow-ups, set `Status: resolved`, and add one line at the end of map.md's Decisions so far using Edit. Keep a dated `## Progress log`, because runs can be cut off.

## Progress log

- 2026-11-01: subagent run (main session merges the report and records the map line; this run edits only `game-data/classes/*.ron` and this ticket). Fixed so far: arcanist Icy Tomb, hunter Push and Bite, alchemist Greater Sacred Incense (self-reference), rogue archetypes' Advanced Talents (archetype-local copies the talent prerequisites never saw), Sylvan Trickster and Underground Chemist talent slots, Underground Chemist Throw Anything, shaman spirit selection + spirit hex prerequisites (9 reported + 9 unreported wrong cross-page matches) + the 11 shared witch hexes. Check passes.
- 2026-11-01: bonus-list pass (Point-Blank Shot, bloodline lists, monk/ranger/cavalier/hunter/cleric/barbarian slots, base wizard and sorcerer slots), the coordinator's sneak-attack request (dice granted at every level the prose names), slayer talents, BAB/saves/spell-level and class-skill check of all 26 base classes. Check, key scan and `cargo test --workspace` pass. Done.

## Answer

Done 2026-11-01, uncommitted. `cargo run -p data --bin seed -- --check-only` passes, a scan finds every referenced key defined in `game-data/` (3082 keys, 0 dangling), and `cargo test --workspace` passes. Only `game-data/classes/*.ron` and this ticket were edited; the report and map line are left to the main session.

### Report items: 15 of 64 fixed, 49 remain

**Fixed (15):**

- Prerequisite lines (11): `arcanist#icy-tomb` → Ice Missile exploit; `hunter#push-and-bite` → New Favorite Toy (the guide swaps the words); the 9 shaman spirit lines → a new `selection:shaman/spirit` (Battle, Bones, Flame, Frost, Life, Nature, Stone, Waves, Wind) picked at level 1 by the base class and the five archetypes whose table has "Spirit", and spirit hexes require `SelectionOption("selection:shaman/spirit/<spirit>")` (Battle hexes also accept the Wildland Shaman, whose spirit is fixed to Battle).
- `monk#student-of-stone`: the bonus feats at 1 and 2 were missing; every Student of Stone slot is now the monk list plus that level's Shaitan feats (Earthblast only from 18, not 14).
- `cavalier#tactician-bonus-feat`: the 1/9/17 slots are `Category(Teamwork)` (prerequisites checked, as the page says), base and archetypes.
- `ranger#nomad`: the Archery list (2: Point-Blank Shot, Precise Shot, Rapid Shot; 6: + Improved Precise Shot, Point-Blank Master, Manyshot; 10: + Improved Critical), ignoring prerequisites. The Stormwalker ("may only select Archery feats") had no slots at all and gets the same.
- `slayer#advanced-talents`: the slayer-talent selection held only its 2 own options, so a slayer could never pick a rogue talent. It now also holds a copy of all 36 rogue talents (`selection:slayer/slayer-talents/<talent>`, guide `rogue#<talent>`); the advanced ones require `feature:slayer/advanced-talents`, and Slayer Camouflage requires Terrain Mastery and Advanced Talents.

**Remaining (49):**

- 39 progression tables under h4 headings (sorcerer bloodlines 9, wizard schools 9, bloodrager bloodlines 10, cavalier orders 5, Magic Deceiver ways 6): need the follow-up below.
- `character-creation#bloodlines`: the bloodline feat slot is one list for all bloodlines. It's now the right list per class (see below), but which bloodline you hold can't narrow it until the follow-up.
- `sorcerer#crossblooded`, `#empyreal-sorcerer`, `#geomancer`, `#nine-tailed-heir`, `#sage-sorcerer`, `#sylvan-sorcerer` level 1 "Bonus Feat" (6): every sorcerer table has it at level 1, but the guide never says what it is (only the Seeker's 5/10/15 magic feats are explained). Still stored as `General` on those archetypes; the base sorcerer has none. Left alone.
- `cleric`: alignment within one step of the deity. Deities aren't data.
- `barbarian#rage-powers` (totem exclusivity) and `kineticist#kineticist-bonus-feat`: unchanged. The kineticist has no selections at all (wild talents and Elemental Focus aren't data), so its wild talent bonus feats can't be placed either.

**New items for the report** (found here, not fixable with current types or not stated clearly enough):

- `rogue#fey-tricks`: the Sylvan Trickster's even-level pick is a rogue talent or a witch hex (major from 10, grand from 18). A slot takes one selection, so only rogue talents are offered now.
- `rogue#discovery`: from 10 the Underground Chemist may take six alchemist discoveries instead of rogue talents. Only rogue talents are offered (level 10 had no talent slot at all before).
- `slayer#slayer-talents`: a slayer may instead take a ranger combat style feat three times (at 2 or 4, 6 or 8, 10+). Not offered: options can carry a feat, but nothing limits how many.
- `cleric#crusader-bonus-feats`: only the BAB and fighter-level prerequisites are skipped; the data skips all of them (a slot can't skip some).
- `rogue#skill-focus-master-of-all` (and every "Skill Focus" pick): one `feat:skill-focus` key can't be held more than once, so the Master of All's three Skill Focus grants stay features.
- `shaman`: the Second Spirit mythic ability and Wandering Hex give another spirit's hexes; not modelled.
- `rogue#sneak-attack-eldritch-scoundrel`: read as dice at 1, 3, 7, 11, 15, 19 ("at 1st level … an additional 1d6 at 3rd level … 7, 11, 15, 19"). Check it.

### Other fixes (not in the report)

- **Shaman hexes had wrong prerequisites** (extractor bug): "Wind" matched the oracle Wind Whisperer's Wind, "Bones" the lich's Indestructible Bones, and "Flame" an `Any` of an angel power, an arcanist exploit and Ward of Flame itself. Replaced with the spirit selection. Shaman hexes also lacked the 11 witch hexes the guide says they share (Ameliorating, Aura of Purity, Evil Eye, Fortune, Healing, Iceplant, Misfortune, Nails, Protective Luck, Slumber, Ward); added as `selection:shaman/hex/<hex>` with `witch#` guide links.
- **Rogue archetypes never qualified for advanced talents**: each archetype minted its own `feature:rogue/<archetype>/advanced-talents`, but the 16 talent prerequisites name only `feature:rogue/advanced-talents`. The archetypes now grant the base feature (guide link fixed to `rogue#advanced-rogue-talents`). Sylvan Trickster (every even level) and Underground Chemist (10) were missing their talent slots; Underground Chemist gains Throw Anything at 2.
- **Sneak attack** (coordinator request): granted once per die at every level the prose gives: rogue base and 6 archetypes and Vivisectionist 1, 3, …, 19; Master of All 1, 5, 7, 11, 13, 17, 19; Eldritch Scoundrel as above; slayer base and 7 archetypes and Cult Leader 3, 6, …, 18; Imitator 3, 9, 15; Inciter 2, 7, 12, 17; Sanctified Slayer 4, 7, …, 19; Divine Scion 5, 10, 15, 20 (Shadow Shaman was already right). `Feature { count }` prerequisites such as "Sneak Attack 2d6" now work. No base class row sets `advances_*` (the "+0" bug doesn't reach base classes).
- **Bloodline feat lists were one mixed list** (extractor bug): sorcerer and bloodrager slots both held the same 33 feats, the union of both classes' lists minus every name that didn't resolve (Empower/Reach/Quicken/Extend Spell, Spell Focus, Point-Blank Shot, Deft Hands, …). Now: bloodrager 28 (its 10 lists; Hag-Riven only Arcane/Elemental/Fey/Hag, Reformed Fiend only Abyssal/Infernal), sorcerer 34 (its 9 lists; Empyreal = Celestial, Sage = Arcane, Sylvan = Fey). Both still check prerequisites (bloodrager page says so; D1 for sorcerer). The base sorcerer gains the 7/13/19 slot its archetypes already had.
- **Point-Blank Shot was missing from every ranger combat-style list and the Zen Archer list** (the guide writes "Point Blank Shot", the feat is "Point-Blank Shot"; the extractor dropped the name silently).
- Monk: Sohei's slots were `OneOf([Mounted Combat])`; now the monk list plus all five mounted combat feats.
- Base wizard: bonus magic feat at 1, 5, 10, 15, 20 and Scribe Scrolls at 1 (the prose and all 9 school tables agree; the archetypes already had them).
- Hunter: Precise Companion (Precise Shot or Outflank at 2, prerequisites ignored) on base and 6 archetypes; Urban Hunter's Captor lists at 6–18; the teamwork slots accept Precise Shot too, as the page says (`OneOf` of the 13 teamwork feats plus Precise Shot, prerequisites checked).
- Demonbane Priest 4/8: any teamwork feat, Spell Penetration or Greater Spell Penetration (was Spell Penetration only). Pack Rager: teamwork slots exclude Shake It Off.

### Bonus-list flags (D1)

Every fixed list a class grants ignores prerequisites, except the bloodline lists and the two lists that stand in for a teamwork category slot (hunter teamwork-or-Precise-Shot, Pack Rager teamwork without Shake It Off), which check like the category slot they replace. The pages that mention prerequisites agree: ranger, hunter Precise Companion/Captor and kineticist say they're skipped; bloodrager and cavalier say they're checked. The Crusader is the one partial case (listed above). Note the two teamwork lists are frozen copies of today's 13 Teamwork feats.

### Spot checks

BAB, saves and highest spell level of all 26 base classes match their guide tables. The alchemist stores no spell level (the page says extracts aren't arcane spells) and the ranger has spell level 1 from 4 (the "0" cells, bonus spells only): both intended. Class skills of all 26 base classes match. Selection levels match the tables for rogue, barbarian, alchemist, arcanist, magus, witch, shaman, slayer, warpriest, oracle, shifter, hunter and inquisitor.

### Suspected extractor bugs

1. Name resolution matches across pages and by substring (shaman Wind/Bones/Flame, an option requiring itself in Greater Sacred Incense and Ward of Flame) and isn't reported.
2. Names in feat lists that don't resolve exactly (punctuation, "Empower Spell" vs "Metamagic (Empower Spell)") are dropped silently.
3. The `character-creation#bloodlines` list was built from both classes' bloodline lists and reused for both.
4. An archetype feature whose base feature has no matching heading (Advanced Talents vs "Advanced Rogue Talents") gets a per-archetype key, so prerequisites naming the base key miss archetypes.
5. Recurring grants stated only in prose (sneak attack dice, a wizard's bonus feats under school tables) are read from the table only.
6. Bonus feat grants stated in a feature's prose (Precise Companion, Captor, Stormwalker's Archery) aren't turned into slots.
7. "Mounted combat feats" resolved to the single feat Mounted Combat (Sohei).

The prestige stage should check the Dragon Disciple bloodline list and the feats/mythic stages the "Bloodrager Bloodline" and "Arcane bloodline" lines: they likely depend on bug 3 or on the follow-up below.

### Follow-up: options that grant over levels (needs a `rules` type and schema change)

The current types can hold the **choice** (a `FeatureSelection` with options, as now done for shaman spirits), but a `SelectionOption` grants at most one feat. Nothing can say "this option grants these features, feat slots and selections at these class levels", so a bloodline, school, order, spirit or way can't give anything. Proposed shape:

```ron
(key: "selection:sorcerer/bloodline/abyssal", name: "Abyssal", guide: "sorcerer#abyssal",
 features: [(key: "feature:sorcerer/bloodline/abyssal/grow-claws", name: "Grow Claws", guide: "character-creation#grow-claws-abyssal"), ...],
 levels: [
   (level: 1, features: ["feature:sorcerer/bloodline/abyssal/grow-claws", ...]),
   (level: 7, feat_slots: [OneOf(feats: ["feat:augment-summoning", ...])]),
   ...
 ])
```

- `SelectionOption` gains `features: Vec<Feature>` and `levels: Vec<OptionLevelRow>`, where `OptionLevelRow { level, features, feat_slots, selections }` is a `ClassLevelRow` without BAB, saves and spells. `level` is a class level of the class whose row granted the selection slot. The engine grants a held option's rows as that class reaches each level (rows at or below the level when picked apply at once).
- Schema: an `option_level` table `(option_key, level)`, plus a fourth owner on the feature, feat-slot and selection grant tables (their exactly-one-owner CHECKs gain it); option features in `feature` with an option owner.
- An archetype that changes what a bloodline gives (Crossblooded, Nine-Tailed Heir, Seeker, Overwhelming Mage, Geomancer, Primalist, Mixed-Blood Rager) gets its own selection with its own options; fixed-bloodline archetypes (Empyreal, Sage, Sylvan) keep their rows as now. Nested choices (dragon or element type) are a selection an option row grants.
- The bloodline feat slot moves from the class rows into each option's 7/13/19 (bloodrager 6/9/…/18) rows, which resolves `character-creation#bloodlines`; prerequisites such as Dragon Disciple's "Draconic bloodline" and Disruptive's "Arcane bloodline" become `SelectionOption`.
- Affects: sorcerer (9 bloodlines; draconic and elemental subtypes), bloodrager (10), wizard (9 schools; the base wizard's school features), cavalier (5 orders, including the Mounted Mastery and Braggart bonus feats), arcanist Magic Deceiver (6 ways), shaman (9 spirits at 1/8/16/20 plus Manifestation), magus Eldritch Scion and fighter Dragonheir Scion (bloodlines), and likely oracle mysteries, domains, blessings and witch patrons (not checked here).

Two smaller follow-ups: a selection slot that offers the options of two selections (Fey Tricks, Underground Chemist's discoveries; it would also replace the 36 copied slayer options), and repeatable feats (Skill Focus, Magical Tail).
