# How the GameFAQs guide is laid out

Research for [01-guide-anatomy](../issues/01-guide-anatomy.md). Sampled on 2026-10-31 by reading pages in a normal browser session (one visit per page, non-essential cookies declined). Guide: "Guide and Walkthrough (PS4)" by chris-williams, **Version 2.3, updated 10/29/2026**.

`BASE` = `https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/`

Pages sampled in full: `introduction`, `character-creation`, `feats`, `rogue`, `fighter`, `barbarian`, `witch`, `paladin`, `prestige-classes`, `mythic-paths`, `aeon`, `trickster`, `other-mythic-paths`. Everything else below comes from the table of contents on `BASE/introduction`.

## TL;DR

- **70 pages and 454 TOC entries.** Each page is one URL slug; subsections are `#anchor`s on that page. About **38 pages** matter to the planner (listed below). The rest are the walkthrough, items, crusade, trophies and spell lists.
- **The HTML is easy to parse.** All content is a flat list of sibling elements inside `div#faqwrap`: `h2/h3/h4/h5`, `p`, `table.ffaq`, `ul`, `blockquote`. Every heading carries its own `<a id="slug">`. Nothing is paginated within a page.
- **Progression data is in real tables.** Each class has a BAB/saves table and a level-progression table, and every archetype has its own full progression table. Each mythic path has a rank-progression table. Feats, mythic abilities, mythic feats and feature selections are **prose entries** (a heading, then paragraphs) with no tables.
- **Prerequisites follow a loose convention.** They sit in `<p>Prerequisites:<br>line<br>line</p>`, one condition per `<br>` line, and the lines are ANDed. Inside a line the text is free: OR, "One of:", "Two of:", mixed `or`/`+`, missing colons, abbreviations and typos (even in the label). You'll need a tolerant parser and a hand-correction pass.
- **Blocker for automated fetching:** the site returns 403 to our fetch tool. robots.txt disallows `ClaudeBot`, `anthropic-ai` and `Claude-Web`. The GameFAQs (Fandom) terms forbid "unauthorized … scraping" and allow personal, non-commercial use only. The author forbids reproduction except for personal use. **Recommendation:** the user saves the ~38 pages once by hand from their own browser into a git-ignored cache, and the extractor parses those files offline. Commit only the derived facts, our summaries and links.

## 1. Where each kind of data lives

All URLs are relative to `BASE`.

| Planner data | Page(s) / anchor(s) | Form |
| --- | --- | --- |
| **Races** (+ subraces) | `character-creation#race` (H4 race, H5 subrace, e.g. `#elf-blightborn`) | One small 2-column table per race/subrace: "Ability Modifiers", "Racial Traits", sometimes "Exclusive Archetype". Prose follows. |
| Point buy, attribute increases | `character-creation#attributes`, `#point-buys`, `#attribute-bonuses` | Cost table (7 → +4 … 18 → -17). Prose says 25 points and +1 at levels 4/8/12/16/20. |
| **Skills** (WotR's 11) | `character-creation#skills` | Table of skill × attribute. Table of class/archetype × skill points per level + class skills (only archetypes that differ from the base class are listed). Table of prestige-class skill-rank requirements. |
| Bloodlines, deities/domains, animal companions | `character-creation#bloodlines`, `#deities-and-domains`, `#animal-companions-and-familiars` | Prose entries (H4/H5) |
| **Feats** (general) | `feats` — H3 categories: `#weapon-feats`, `#ranged-feats`, `#unarmed-combat-feats`, `#armour-and-shield-feats`, `#magic-feats`, `#miscellaneous-feats`, `#mounted-combat-feats`, `#combat-manoeuvres`, `#teamwork-feats`, `#skill-feats`, `#class-feats`, `#racial-feats` | 256 H4 feat entries. No tables. |
| **Base classes** (26) | One page per class: `alchemist`, `arcanist`, `barbarian`, `bard`, `bloodrager`, `cavalier`, `cleric`, `druid`, `fighter`, `hunter`, `inquisitor`, `kineticist`, `magus`, `monk`, `oracle`, `paladin`, `ranger`, `rogue`, `shaman`, `shifter`, `skald`, `slayer`, `sorcerer`, `warpriest`, `witch`, `wizard` | Characteristics `ul` (HP/level, BAB speed, good saves, skill points, alignment, caster info). Table 1: `Lvl/AB/Fort/Reflex/Will` (+ spells per day). Table 2: `Lvl/Feat/Special 1..n`. H4 per class feature. |
| **Archetypes** (~196 by TOC count) | H3 on the class page, e.g. `rogue#dark-lurker`, `paladin#divine-guardian`. Name clashes are disambiguated in the title: `paladin#divine-hunter-paladin`, `sorcerer#seeker-sorcerer` | A short "Loses/Gains" `ul`, then a **full** progression table and H4s for new features. **What each one replaces is never stated**; it has to be derived by diffing against the base table. |
| **Prestige classes** (13) | `prestige-classes` — H3 per class (`#aldori-swordlord` … `#winter-witch`) | Entry requirements as a `ul` ("Required feats/skills/BAB/special", minimum character level, alignment). Progression table includes `BAB` and, for casters, `CL`. |
| **Mythic ranks** | `mythic-paths#mythic-rank` (table: Rank / story Event / approx. Lvl), `#first-ascension`, `#mythic-path` | Ranks come from story events, not XP. |
| **Mythic abilities** | `mythic-paths#mythic-abilities` (H4 groups: Melee/Ranged/Magic/Healing/Misc/Class) | 63 H5 entries. "gained at odd-numbered mythic ranks". |
| **Mythic feats** | `mythic-paths#mythic-feats` (H4 groups) | 56 H5 entries. "at even-numbered mythic ranks". |
| **Mythic paths** | `aeon`, `angel`, `azata`, `demon`, `lich`, `trickster`, `gold-dragon`; `other-mythic-paths#legend`, `#devil`, `#swarm-that-walks` | Rank-progression table (`Rank/Abilities/Feats/Special…/Summons`) on the main path pages. Path-specific picks are H4/H5 (e.g. Aeon's 17 "Enforcing Gaze - …" options). `trickster#trickster-bonus-feats` holds extra feats. |
| **Legend** | `other-mythic-paths#legend` | **No table.** Prose plus a "Playing a Legend" blockquote. |
| **Feature selections** | On each class page as H4 + H5 children, e.g. `rogue#rogue-talents` (20) + `#advanced-rogue-talents` (16), barbarian "Rage Power" (33 H5), `witch` "Hex" (13) / "Major Hexes" / "Grand Hexes", `monk#ki-powers`, `kineticist#infusions`, `#wild-talents`, `oracle#oracle-mysteries`, `shaman#spirits`, fighter `armiger` "Order" | Same entry shape as feats. Many talents just link to a feat ("This is the feat of the same name"). |
| Sample/default builds | `character-building#sample-builds`, `appendix-default-builds` | Possible golden builds for "Checking the data". |

Out of scope but present: spell lists (`cleric-spells`, `druid-spells`, `wizard-spells`, the `#…-spells` anchors on class/path pages), companions, walkthrough chapters, items, crusade, trophies.

### Full page list (70 slugs)

Planner-relevant (38): `character-creation`, `feats`, the 26 base-class slugs above, `prestige-classes`, `mythic-paths`, `aeon`, `angel`, `azata`, `demon`, `lich`, `trickster`, `gold-dragon`, `other-mythic-paths`.
Useful for test fixtures (2): `character-building`, `appendix-default-builds`.
Not needed (30): `introduction`, `cleric-spells`, `druid-spells`, `wizard-spells`, `companions`, `prologue-trapped-in-the-darkness`, `prelude-through-the-ashes`, `chapter-1-the-wardstone-legacy`, `chapter-2-sword-of-valor`, `chapter-3-the-fifth-crusade`, `chapter-4-the-midnight-isles`, `interlude-the-lord-of-nothing`, `chapter-5-city-of-locusts`, `a-dance-of-masks`, `chapter-6-threshold-of-the-end`, `mythic-quests-aeon`, `mythic-quests-angel`, `mythic-quests-azata`, `mythic-quests-trickster`, `mythic-quests-legend`, `mythic-quests-swarm-that-walks`, `miscellaneous-locations`, `the-treasure-of-the-midnight-isles`, `postlude-inevitable-excess`, `items`, `items-lord-of-nothing`, `items-inevitable-excess`, `crusade-mode`, `appendix-ascension`, `trophies`.
(Source: `.toc_menu` on `BASE/introduction`: 454 links, 70 distinct paths.)

## 2. How prerequisites are written

**Container** (`BASE/feats`): a `<p>` whose first line is the label, with one condition per `<br>` line, e.g. `<p>Prerequisites:<br>Strength: 13</p>`. Lines are ANDed. The same shape appears on class pages (`BASE/barbarian`, `BASE/rogue`) and for mythic feats/abilities (`BASE/mythic-paths`; there, the first line after the label is often empty).

**Coverage on `feats`:** of 256 feat entries, 201 have a prerequisite paragraph and 55 have none. The label is misspelt in 16 of the 201: `Prerequisites` 185, `Prerequsities` 10, `Prerequsites` 6. A parser matching only "Prerequisites" would silently drop those 16.

**Clean cases** (`BASE/feats`):
- Power Attack: "Strength: 13"
- Improved Two-Weapon Fighting: "Dexterity: 17" / "Two-Weapon Fighting" / "Base Attack Bonus: 6"
- Cornugon Smash: "Power Attack" / "Persuasion: 6" (skill ranks)
- Greater Weapon Focus: "Fighter level: 8" / "Weapon Focus" (class level)

**Messy cases** (all short quotes from `BASE/feats` unless noted):

| Pattern | Example |
| --- | --- |
| OR across kinds | Crane Style: "Monk level: 1 or Base Attack Bonus: 2" |
| OR on a class feature | Pummeling Style: "Flurry of Blows or Base Attack Bonus: 6" |
| Mixed `or` / `+`, precedence unclear | Dirty Trick: "Combat Expertise or Kinetic Warrior + Intelligence 13" |
| N-of-list | Agile Maneuvers: "One of: Bull Rush, Dirty Trick, Sunder Armor, Trip"; Critical Mastery: "Two of: Blinding Critical, Exhausting Critical, …" |
| Class feature (not a feat) | Extra Rogue Talent: "Rogue Talents or Fey Tricks"; Accomplished Sneak Attacker: "Sneak Attack: 1" |
| Class feature tied to an archetype | Extra Channel: "Channel Positive Energy (Empyreal Sorcerer / Hospitaler)" |
| Class feature or bloodline | Disruptive: "Fighter Level 6 or Arcane bloodline" |
| Caster ability (free text) | Arcane Strike: "Ability to cast arcane spells"; Warrior Priest: "Ability to cast 1st level divine spells" |
| Parametrised feat | "Weapon Focus (Dueling Sword)", "Spell Focus (any)", "Skill Focus - Mobility", "Elemental Focus (any)" |
| Race; a comma means OR | Breadth of Experience: "Elf, Dwarf or Gnome"; Discerning Eye: "Elf, Half Elf" |
| Character level | Wings: "Aasimar" / "Character level: 10" |
| Inconsistent syntax | "Fighter level: 5" vs "Fighter level 5"; "Mobility 1" vs "Mobility: 3"; "BAB: 8" vs "Base Attack Bonus: 8"; "Constitution 13" vs "Wisdom: 13" |
| Typos in names | Extra Discovery: "Dicovery"; Armor Focus: "(Light. Medium or Heavy)" |
| Commas inside names (`BASE/barbarian`) | Beast Totem: "Beast Totem, Lesser" / "Barbarian level: 6" |
| Rules only in prose or blockquotes | Barbarian: "Totem" powers are exclusive (blockquote). Rogue: advanced talents "unlock at 10th level" (prose under the H4). |
| Tier gate from a mythic path (`BASE/trickster`) | Trickster bonus feats open "At Perception II" (a Trickster tier, not skill ranks). |
| Bonus-slot lists in prose (`BASE/paladin`) | Divine Guardian bonus feats: "(Improved) Great Fortitude, (Improved) Iron Will, …" (shorthand that expands to two feats) |
| Prestige entry requirements (`BASE/prestige-classes`) | A `ul`, not the `Prerequisites:` paragraph: "Required skills: Mobility (4), Trickery (4), Knowledge: Arcana (4)"; "Required special: Sneak Attack (2d6), 2nd level arcane spells"; "Alignment must be Lawful"; Winter Witch: "Druid (Winter Child), Shaman (Frost Spirit) or Witch (Winter Patron)". Skill names are spelled differently from the skills table ("Knowledge: Arcana" vs "Knowledge (Arcana)", once "Knowlege"). |

**Conclusion:** the format is consistent enough to auto-parse about 80–90% of lines as `Key: N`, `<feat name>` or `<race>`. The rest (OR/N-of/`+`, class features, caster ability, prose-only gates) needs a small grammar plus a hand-maintained override file. That fits the map's "extracted once … corrected by hand" approach.

## 3. HTML structure (parser facts)

From `BASE/introduction`, `BASE/feats`, `BASE/rogue` and `BASE/character-creation`:

- **Container:** `div#faqwrap.ffaq.ffaqbody`. Its direct children form a flat list (`h2`, `h3`, `h4`, `h5`, `p`, `table`, `ul`, `blockquote`) with no section wrappers. Nesting must be rebuilt from heading levels: page title = H2, subsection/archetype/category = H3, entry/class feature = H4, sub-entry/selection option = H5. The levels shift by context: a rogue talent is an H5 under H4 "Rogue Talents", while a feat is an H4 under an H3 category.
- **Anchors:** each heading contains `<a id="kebab-slug"></a>`, e.g. `<h3>Copyright and Acknowledgements<a id="copyright-and-acknowledgements"></a></h3>`. These give stable deep links to store in our data (`BASE/rogue#rogue-talents`). Slugs are unique only within a page (`rogue#weapon-focus` is a rogue talent; `feats#weapon-focus` is the feat).
- **TOC:** `ul.toc_menu` sits in a "Table of Contents" H3 on every page. Links are relative (`feats#weapon-feats`). This is the canonical page list.
- **Tables:** plain `<table class="ffaq">`, with a header row of `th` cells and no `thead`. Gotchas:
  - Iterative BAB in one cell: "+6 / +1".
  - Progression row `0` means "granted when you take the class". Rows 1–20 are class levels.
  - The `Feat` column marks odd levels for a **single-class** character. It is the general character-level feat, not a class bonus feat, so ignore it. Class bonus feats appear in a `Special n` column as text ("Bonus Combat Feat", "Bonus Feat").
  - Header spellings vary: `Lvl/Feat` (rogue) vs `Level/Feats` (witch); `AB` (base classes) vs `BAB` (prestige).
  - Footnote digits are glued onto cell text: "Rogue Proficiencies1" (Eldritch Scoundrel).
  - Columns can be misused: Trickster's `Feats` column holds "Sneak Attack x2" at rank 3 (`BASE/trickster`).
  - Repeated selections show as the same text in many rows ("Rogue Talent", "Mercy", "Hex").
- **Pagination:** none within a page (no `page=` links). Pages chain with Prev/Next links. `link[rel=canonical]` is the clean page URL.
- **Ads and consent:** a OneTrust cookie banner and ad markup sit outside `#faqwrap`, so selecting `#faqwrap` drops them.
- **Size:** the biggest pages are around 100 KB of text (`character-creation`: about 103k characters, 104 tables; `feats`: about 66k characters, 0 tables).

## 4. Fetching: robots.txt, terms, author's notice

- **robots.txt** (`https://gamefaqs.gamespot.com/robots.txt`):
  - `User-agent: *` gets `Allow: /` plus `Content-Signal: search=yes,ai-train=no,use=reference`; `/search` and `/user/` are disallowed.
  - `ClaudeBot`, `anthropic-ai` and `Claude-Web` are each `Disallow: /`, alongside GPTBot, CCBot and others.
  - The file calls its content signals express rights reservations under EU Directive 2019/790 Art. 4.
- **Terms of Use** (linked from the guide footer: `https://www.fandom.com/terms-of-service-pp1`, revised August 22, 2025):
  - It prohibits "unauthorized spidering, 'scraping,' data mining or harvesting of Content".
  - Content may be accessed "for personal, non-commercial purposes", and you may not "build a business or other enterprise" on it.
- **Author's notice** (`BASE/introduction#copyright-and-acknowledgements`): no reproduction "for anything other than personal use" without permission; permission is extended to gamefaqs.com.
- **In practice:** our fetch tool got HTTP 403 on guide pages, and an attempt to fetch from the shell was blocked locally as bot-check circumvention. Pages load normally in a browser.

**Feasibility verdict:** parsing is easy; fetching is the constraint. An automated crawler conflicts with robots.txt (for Claude agents), the ToS and the site's bot protection, so don't build one. Instead:

1. The user saves the ~38 pages manually in their browser (`#faqwrap` is all that's needed), into a **git-ignored** `data/guide-cache/` folder, for personal use.
2. The extractor (TDD, per the map) parses those files offline.
3. Commit only facts (numbers, names, prerequisites), our own one-line summaries and deep links. No guide prose.

This also matches the map's rule against reproducing the guide. Whether the derived game facts are OK to commit is a judgement for the user, not something this research settles.

## 5. Gaps: what the planner needs that the guide lacks

1. **No "combat feat" tag.** Fighter bonus feats are "any feat from the list of Combat Feats" (`BASE/fighter#bonus-combat-feat`), but the feats page groups feats by theme and never marks which are combat feats. Other bonus-slot lists are prose with shorthand: Divine Guardian (`BASE/paladin`) is confirmed, and others such as Pack Rager's "Bonus Teamwork Feat" (`BASE/barbarian`, not read in detail) are likely the same. Each **bonus slot's eligible list** needs a hand-built table.
2. **Archetype replacements aren't stated.** We have to diff the archetype table against the base table and read the "Loses/Gains" bullets. Changes to skill points, class skills, proficiencies or BAB are only in bullets or the skills table.
3. **Caster level is mostly implicit.**
   - Base classes have no `CL` column, only prose like "Caster level progression from level 4" (`BASE/paladin`). Prestige tables do have `CL`.
   - Caster-level prerequisites seem to be missing from feats. For example Arcane Armor Training, Spell Penetration and the Metamagic feats have no prerequisite paragraph at all. Verify these against the game.
4. **"No prerequisite paragraph" doesn't mean "no prerequisites".** The 55 entries without one include feats where in-game requirements are likely. They need a spot check.
5. **Mythic rank minimums.** The guide gives no rank gates for individual mythic abilities or feats, and doesn't say which belong to which path beyond the path pages. Ranks are tied to **story events**, not character level. The rank table gives an approximate character level per rank (rank 3 ≈ level 9, ranks 9–10 ≈ level 20) (`BASE/mythic-paths#mythic-rank`). The planner must decide how to place mythic ranks on the level timeline (a design question for 04-progression-model).
6. **Legend has no progression table.**
   - What the guide does say, in prose: you reach rank 8 on another path first, then lose the rank 3–8 mythic abilities and feats, and your "rank is stuck at 3".
   - Also in prose: a cap of 20 levels per class and 10 per prestige class, BAB rising past 20, and CL capped at 28 (`BASE/other-mythic-paths#legend`).
   - It doesn't say whether attribute increases continue after level 20 (`character-creation#attributes` lists only 4/8/12/16/20).
   - Per-level tables stop at class level 20.
7. **Skill rank rules are incomplete.** The guide gives skill points per level and the Intelligence bonus rule, but never states a **maximum ranks per skill** (e.g. ranks ≤ character level) (`BASE/character-creation#skills`).
8. **Alignment** limits classes and prestige classes (Paladin "Alignment must be Lawful Good", Hellknight "Lawful", Assassin "neutral or evil"), but alignment isn't in the map's list of checked prerequisites.
9. **Exclusivity and tier rules live in prose or blockquotes** (barbarian totems, rogue advanced talents at 10, witch Major/Grand hexes, Trickster tiers). They need hand-encoded rules.
10. **DLC gating** appears only in prose and blockquotes (`feats#dlc-5-feats` lists Lord of Nothing feats; archetype blockquotes such as `rogue#dark-lurker`). Decide whether the planner shows DLC content.
