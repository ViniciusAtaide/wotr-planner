# Hand-fix the extracted feats

Type: task
Status: resolved
Blocked by: 10
Map: [WotR Build Planner](../map.md)

## Question

Stage of the split extraction (user, 2026-11-01). Resolve the `## Feats` section of `game-data/EXTRACTION_REPORT.md` by editing the RON files directly. AFK.

- **Files you may edit:** `game-data/feats.ron` only. Plus the report's `## Feats` section.
- **Focus:** Unparsed prerequisite lines (OR, "One of / Two of", misspelt labels, class features, caster/spell level), wrong categories, and missing or extra feats vs the guide's `feats` page.

**Shared rules for the hand-fix stages** (these four run in parallel, one agent each):
- **Your area:** fix only this stage's section of `game-data/EXTRACTION_REPORT.md`, and only this stage's files (below). Other agents are editing the other files at the same time.
- **Source:** the saved pages in `guide-cache/`. Never fetch GameFAQs or any site.
- **Copyright:** commit facts only (names, numbers, prerequisites, keys, anchors). A `summary` stays empty unless it's one short line in your own words.
- **Validating:** use the database-free check command from ticket 10's Answer. Never seed, reset or migrate the dev database; the main session seeds once all stages are done. No schema or `rules` type changes. Anything that needs one becomes a proposed follow-up in your Answer.
- **Unclear items:** anything you can't resolve accurately from the guide stays in the report, with a note on why. Don't guess.
- **Done when:** your section of the report holds only unresolvable items, the check command passes, `cargo test --workspace` passes, and there are no commits.
- **Recording:** append an `## Answer` with fixed and remaining counts and the follow-ups, set `Status: resolved`, and add one line at the end of map.md's Decisions so far using Edit. Keep a dated `## Progress log`, because runs can be cut off.

## Progress log

- 2026-11-01: read the ticket, README, report `## Feats`, ticket 10's Answer, map Notes, glossary. Converted `guide-cache/feats.html` to text in the scratchpad. Feat set matches the guide (256 anchors + 5 Trickster bonus feats; `dlc-5-feats` is a heading, not a feat); every name and category matches its guide heading and section. Read every prerequisite against the guide. Found: Extra Mercy also accepts the witch's Mercy *patron* (false name match); the rogue Weapon Focus / Skill Focus talents ("the feat of the same name") count for some feats but not for Aldori Dueling Mastery, Fencing Grace, Slashing Grace, Acrobatic Spellcaster, Slayer's Feint. Report items: Armor Focus and Extra Channel resolvable; Disruptive (no arcane bloodline key) and Extra Slayer Talent ("Talented Slayer" is on no saved page) stay.
- 2026-11-01: edited `game-data/feats.ron` (8 feats). `seed --check-only` passes (261 feats), every key `feats.ron` references exists in `game-data/`, `cargo test --workspace` passes. Report and map left to the main session, as instructed.

## Answer

Verified 2026-11-01, uncommitted. Only `game-data/feats.ron` was edited. The main session merges the report and the map line.

### Report `## Feats`: 2 fixed, 2 remaining

Fixed (delete these lines from the report):
- `feats#armor-focus`: now `Any` of the light, medium or heavy Armor Proficiency feats, plus BAB 1. (The guide's "Light. Medium or Heavy" has a full stop for a comma.)
- `feats#extra-channel`: now `Any` of the cleric's channel (`feature:cleric/channel-energy`, plus the Angelfire Apostle's and Priest of Balance's own channel features, since those cleric archetypes replace it), the Empyreal Sorcerer's channel positive energy, and the Hospitaler's.

Remaining (keep in the report):
- `feats#disruptive`: "Fighter Level 6 or Arcane bloodline". There's no sorcerer bloodline selection in the data yet, because the bloodline tables aren't read (that's the base-class stage). Once an arcane bloodline key exists, this becomes `Any([ClassLevel(fighter, 6), <arcane bloodline>])`. It's also unclear whether the Sage Sorcerer (built on the arcane bloodline) or the bloodrager's Arcane bloodline count.
- `feats#extra-slayer-talent`: "Slayer Talents or Talented Slayer". "Talented Slayer" doesn't appear on any saved page. Also, `selection:slayer/slayer-talents` holds only 2 options, so an `Any` of its options would be too narrow.

### Whole-file check against the guide

I checked all 261 feats, not just a sample of 20. The set matches: 256 `feats` anchors plus the 5 Trickster bonus feats, none missing and none extra. Every name matches its heading and every category matches its guide section. Ability scores, BAB, skill ranks, class levels, feat chains, races, "One of" / "Two of" groups and the Trickster bonus feats (each needs Improved Mythic Trick → Perception II plus its guide prerequisite) all match. Errors found and fixed:
- **Extra Mercy** also accepted `selection:witch/patron/mercy`, which is a witch *patron* that happens to be called Mercy. It's now only `feature:paladin/mercy`.
- **The rogue talents Weapon Focus and Skill Focus** are "the feat of the same name" (rogue page). The extractor counted the Weapon Focus talent for Dazzling Display, Shatter Defenses, Weapon Specialization, Greater Weapon Focus and Penetrating Strike, but not for Aldori Dueling Mastery, Fencing Grace or Slashing Grace. It never counted the Skill Focus talent, so Acrobatic Spellcaster and Slayer's Feint missed it. I added the talent as an `Any` alternative to those 5 feats.

### Suspected extractor bugs (not changed)

1. **Names resolve to every entity of that name, in any class.** "Mercy" matched the witch patron as well as the paladin's Mercy. Any prerequisite name shared by a feature or selection option elsewhere can pick up a false alternative. This is worth a sweep in the class files too.
2. **Qualified names are treated differently from bare ones.** A bare "Weapon Focus" resolves to the feat *and* same-named selection options. "Weapon Focus (Dueling Sword)" and "Skill Focus - Mobility" drop the qualifier and resolve to the feat only.
3. **A distributive list inside parentheses isn't expanded** ("Armor Proficiency (Light. Medium or Heavy)"). It happened to work for Shield Focus/Shield Bash only because the qualifier is dropped.

### Follow-ups

- **Class proficiencies aren't proficiency feats** (needs a rule or schema decision). Classes grant `feature:<class>/<class>-proficiencies`, not the Armor/Shield Proficiency feats. So Armor Proficiency (Medium/Heavy), Tower Shield Proficiency, Shield Focus, Shield Bash, Shield Wall and now Armor Focus are blocked for a fighter, paladin and so on unless they spend a feat on the proficiency. Possible fixes: let a feature grant feats (schema), or have the engine treat named proficiency features as the feats.
- **Selection options that *are* a feat** (rogue Weapon Focus, Skill Focus, Intimidating Prowess) are handled now by an `Any` per prerequisite. An "option grants feat" link (schema) would make every check count them without per-feat edits.
- **Selective Channel** lists only the features literally named "Channel Energy". It doesn't include the Angelfire Apostle's or Priest of Balance's channel, or the paladin's Channel Positive Energy. The guide doesn't say whether they count, so I left it as it is.
- **Trickster's Knowledge (World) III** ignores feat prerequisites for the character. That's an engine rule for the mythic stage or the rules crate.
- Disruptive and Extra Slayer Talent depend on the base-class stage's sorcerer bloodlines and slayer talents.
