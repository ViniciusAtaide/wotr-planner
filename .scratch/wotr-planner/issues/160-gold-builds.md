# Gold builds: Jack-in-the-Green and the Enforcer as the standard to compare against

Type: task
Status: resolved
Blocked by: 159, 165
Model: Opus to build (careful conversion, and judging each disagreement against the data before filing it); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "I agree we should use a gold standard to check. that's why I downloaded some builds from gamefaqs", then "it's Jack-in-the-green and the Enforcer. They should be bring back in a format that's easily parseable so we can compare them".

The proof and the snapshot check ([159](159-snapshot-check.md)) compare the engine with itself. Neither says the engine matches the game. The two write-ups do: real builds, which the user knows work on this version of the game. Ticket [88](88-qa-full-builds.md) checked them once by hand in the browser and then deleted them. This ticket is ticket [146](146-checking-the-data-against-the-guide.md)'s option B.

**The write-ups:** the deleted file is recovered from git blob `101964c4a6d049c7e57f6938c4e46ef198bbcdac` (`git cat-file -p 101964c4`, 155 lines, until git prunes it). A copy made on 2026-11-11 is at `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/builds-recovered.txt`; if both are gone, ask the user for the file.

Decided in the grilling session (user, 2026-11-11):

- **A refusal is never the write-up's fault.** "I already know those builds works on this version so let's not care about if gamefaqs is stale or not". There is no mechanism for pinning a write-up error.
- **Format:** one RON file per build, with the write-up's own names, and the original text committed beside them.
- **What is compared:** everything the write-up lists at a level or rank is gained there on the engine's sheet, chosen or granted. The engine may show more.
- **The gold file doesn't mark chosen against granted.** The test's Build is written in Rust; the gold file is only the expected side.
- **Known engine gaps:** the test merges with the lines the engine gets wrong listed, each naming its ticket. The list can only shrink.

## Build

AFK, with one question for the user below. Test files only; an engine or data fix belongs to the bug ticket it names.

- **Files**, under `libs/seeds/tests/game_data/gold/`:
  - `source.txt`: the recovered text, unchanged. It keeps the prose, the companion builds and the suggested-spells tables, which nothing checks.
  - `jack-in-the-green.ron` and `enforcer.ron`, converted by hand once. Each holds:
    - the start: race, background, classes with their level counts, attributes, skills, alignments;
    - one entry per level: the class, and the list of gains;
    - one entry per Mythic rank: the path (Mythic Hero at ranks 1–2), and the list of gains.
  - **Names** are the write-up's, with its label prefixes dropped and its parameters kept: "Bonus Feat - Spell Penetration" is `"Spell Penetration"`, "Spell Focus - Evocation" stays whole, "Charisma +1" stays whole. The conversion is checkable line by line against `source.txt`.
  - **Bloodline spells** go in a field of their own per level, apart from the gains. Spells known aren't Build choices (map, Out of scope), so the test compares them only if the engine's sheet has them; say in the Answer whether it does.
- **The test,** `libs/seeds/tests/game_data/gold.rs` (`mod gold;` in `main.rs`):
  - builds each character in Rust with `support.rs`'s helpers, slot by slot. Ticket 88's Answer tables say which slot each pick went into and the choices it settled (Halfling (Hasty), Chaotic Good, Atheism, the Enforcer's extra skill points);
  - resolves each gold name to a planner item through the data's display names. A name that resolves to nothing fails the test. Where the planner names a thing differently ("Improved Critical (Battle Axe)" for "Improved Critical - Battleaxe"), put the rule or the pair in a small table in the test, each entry with the write-up's spelling beside the planner's;
  - asserts, per level and rank, that each gold gain is among what the engine's sheet shows gained there;
  - asserts the start: the attributes are reachable within the point-buy budget with the race's modifiers, each listed alignment is allowed, the listed skills are class skills or have ranks;
  - reports every disagreement in one panic, one line each (`enforcer level 19: Greater Two-Weapon Fighting: locked, "Needs Dex 19 (have 17)"`), not only the first.
- **Known engine gaps:** a list in the test of the gold lines the engine gets wrong today, each with its ticket. The test fails on a disagreement that isn't listed, and on a listed line that now agrees, so a fix is proved by deleting its entry. Start from ticket 88's tables and re-check each against the current engine: [89](89-life-bonding-friendship-teamwork-feats.md), [90](90-arcane-enforcer-saves.md), [91](91-first-ascension-path-ability.md) and [92](92-sorcerer-level-1-bonus-feat-list.md) may be resolved since. A disagreement with no ticket gets a new bugfix ticket in 88's style.
- **Add both Builds to 159's corpus** as sheet-only snapshots.
- **A question for the user, not to be settled here.** Ticket 88 found lines that fail because of decisions already on the map, not bugs:
  - Greater Two-Weapon Fighting at the Enforcer's level 19 needs Dex 19 and the Build has 17. In the game a belt covers it; gear is out of scope here.
  - Mythic Shield Focus at MR 10: the planner places MR 10 at level 19 (ticket [04](04-progression-model.md)), before Shield Focus at level 20.
  - Finesse Training - Light Shield: granted with no weapon choice (ticket [54](54-mercies-and-enforcer-exploits.md)).
  - Uncanny Dodge, which 88 calls the write-up's own mistake.
  
  The user's rule says the write-up is right, and those decisions say the planner won't model it. List each such line among the known gaps with the map decision in place of a ticket, mark them apart from the bugs, and put the question in the Answer: does the gold standard reopen any of those decisions?

  **Answered for gear** (user, 2026-11-11): "we need to introduce gear support so before implementing I have to plan how to introduce equipment". The plan is [163](163-equipment.md). The Greater Two-Weapon Fighting line is a known gap naming [168](168-gear-in-the-mcp.md), which gives the Enforcer a Dex +2 belt and removes the entry. This ticket doesn't wait for it. The other three lines are still open questions.
- Update ticket 146's Answer: option B is built by this ticket. Remove "146's golden-build write-ups" from the map's "Waiting on the user" line.

## Done when

- `cargo test -p seeds --test game_data -- --list` shows the gold test.
- Both RON files match `source.txt` line for line, checked by a second read.
- Every disagreement between the write-ups and the engine is either fixed or in the known-gaps list with a ticket or a map decision. The Answer has the list.
- Removing one entry from the known-gaps list (throwaway, reverted) fails the test with that line's message.
- Both Builds have snapshots in 159's corpus.
- `scripts/check.sh` passes.

## Progress log

- 2026-11-11: started on branch `ticket-160` in its own worktree, from `main` at `995564e` (base confirmed); `.env` from `.env.example`, the database created and migrated (no schema change). Read the repo's `CLAUDE.md`, the map's Notes, this ticket, tickets 88 and 146, the README's snapshot section and `snapshots.rs`. Gear (ticket 166) is being built in another worktree and isn't on this base; no gear code touched.
- 2026-11-11: `gold/source.txt` copied from the scratchpad copy, byte for byte the git blob `101964c4` (155 lines, no newline at the end, kept). Dumped both of ticket 88's final links through the current engine to read the slot ids and names: the Enforcer's opens and is possible; Jack's is no longer (ticket 92 moved the level-1 bonus feat), so both Builds are written afresh in `gold.rs`.
- 2026-11-11: judged each disagreement against the data and the saved pages (`guide-cache/slayer.html`, `sorcerer.html`, read only): the guide's tables list Studied Target once (slayer 1) and Draconic Resistances once (bloodline 3), the data matches them, and the game's level-up names each step, as the write-ups do; filed [170](170-scaling-features-granted-once.md). The rest have tickets or map decisions.
- 2026-11-11: throwaway check (reverted): with the Greater Two-Weapon Fighting entry removed and a made-up entry for a line that agrees ("jack level 1: Mounted Combat"), the test failed with `enforcer level 19: Greater Two-Weapon Fighting: locked in the Feat slot, "Needs Dex 19 (have 17)"` and `jack level 1: Mounted Combat: listed as a known gap (ticket 999) but now agrees: delete its entry`.
- 2026-11-11: both Builds added to the snapshot corpus; `SNAPSHOTS=write` added `snapshots/gold-jack.txt` and `snapshots/gold-enforcer.txt` and changed no other file (`git status`).
- 2026-11-11: review (no blocker) and the user's answers to the question on the four decision-backed lines: each now names a ticket on `main` (171–174). Review fixes: a known gap covers one failure on its line (a planner item not gained, or a name that resolves to none), not the whole line, with a test that a second failure on a listed line is still reported; "Mythic Hero" checks the first ascension's path; the start's skills check, which couldn't fail (both Builds rank every listed skill), is now the engine's skill points covering a rank in each listed skill at every level; the README says which lines test the engine; ticket 170's sweep names the other slayer archetypes and the bloodrager's resistances.

## Answer

**Built**, 2026-11-11 on branch `ticket-160`, from `main` at `995564e`. Documented in `libs/game-data/README.md` ("The gold standard").

### What was built

- **`libs/seeds/tests/game_data/gold/`:** `source.txt` (the write-ups, unchanged), `jack-in-the-green.ron` and `enforcer.ron` (the start, 20 levels, 10 ranks, in the write-ups' names).
- **`libs/seeds/tests/game_data/gold.rs`** (`mod gold;` in `main.rs`), six tests:
  - `gold_files_match_the_write_ups`: reads `source.txt` as the conversion does (cells split at semicolons outside brackets, a wrapped cell joined, the `LABELS` prefixes dropped, "Bloodline Spell - " items apart) and asserts each RON file says the same, item by item and in order: the second read, made mechanical.
  - `gold_builds_are_possible`: both Builds are possible and every pick has its slot on the sheet.
  - `gold_builds_gain_what_the_write_ups_list`: every gold line against the engine's sheet, the disagreements less `KNOWN_GAPS`, one panic with a line each; a listed line that now agrees fails too.
  - `the_second_bloodlines_spells_arent_in_the_data`: see "Bloodline spells".
  - `a_known_gap_hides_no_other_failure_on_its_line`: a Jack with an Elemental (Fire) second bloodline fails "(Dragon Resistances III; Wings)" on both Draconic Resistances and Wings; the entry covers the first, and the second is still reported.
  - `gold_names_tables_are_all_used`: every entry of the names tables (below) reads a name of the gold files, so neither keeps a stale spelling.
- **The snapshot corpus** (`snapshots.rs`) takes both Builds, sheet only: `snapshots/gold-jack.txt`, `snapshots/gold-enforcer.txt`.

"Gained there" is what the snapshot projection reads as a level's or rank's gains, through the same public engine calls: a level's slot picks (`Engine::pick_name`), its ability increase, and the feats (with their values), features and options its `CharacterState` holds more of than the level before's; a rank's slot picks and `Engine::rank_features`. Names are compared by display name, a feature by its name and by `feature_name_apart`'s.

### Known gaps

Ten lines, each covering one failure on its line (a planner item not gained there, or a name that resolves to no planner item; any other failure on the line is still reported), and each naming a ticket of one of four kinds, kept apart in the test (`Why`).

**Bugs:**

| Build, place | Write-up line | Covers: what the engine does | Ticket |
|---|---|---|---|
| Enforcer level 5 | Studied Target +2 | Studied Target not gained: only at level 1 | [170](170-scaling-features-granted-once.md) |
| Enforcer level 10 | Studied Target +3 | the same | 170 |
| Enforcer level 15 | Studied Target +4 | the same | 170 |
| Jack level 9 | (Dragon Resistances II) | Draconic Resistances not gained: only at level 6 (MR 1's Second Bloodline) | 170 |
| Jack level 15 | (Dragon Resistances III; Wings) | Draconic Resistances not gained (Wings is, and isn't covered) | 170 |

**Gear:**

| Build, place | Write-up line | Covers: what the engine does | Ticket |
|---|---|---|---|
| Enforcer level 19 | Greater Two-Weapon Fighting | not gained: locked in the Feat slot, "Needs Dex 19 (have 17)" | [168](168-gear-in-the-mcp.md) (the Enforcer's Dex +2 belt) |

**Map decisions the user reopened** (2026-11-11):

| Build, place | Write-up line | Covers: what the engine does | Ticket |
|---|---|---|---|
| Enforcer MR 10 | Mythic Shield Focus | Shield Focus (Mythic) not gained: locked in the Mythic feat slot, "Needs Shield Focus" (MR 10 comes at level 19, Shield Focus at 20) | [171](171-mythic-picks-after-the-levels.md) |
| Enforcer level 18 | Finesse Training - Light Shield | "Finesse Training (Light Shield)" names no planner item: Finesse Training has no weapon | [172](172-finesse-training-takes-a-weapon.md) |
| Jack level 1 | Animal Companion - Elk | "Animal Companion (Elk)" names no planner item: the companion has no animal | [174](174-animal-companions.md) (design) |

**The write-up against the guide:**

| Build, place | Write-up line | Covers: what the engine does | Ticket |
|---|---|---|---|
| Enforcer level 8 | Slayer Talents - Uncanny Dodge | Uncanny Dodge not gained: no empty slot at level 8 offers it | [173](173-uncanny-dodge-as-a-slayer-talent.md) (design; the question stays open there) |

The Uncanny Dodge line is not a user decision: the checked-in data files are the planner's source of truth, and they follow the guide's slayer talents (the rogue talents plus Trapfinding and Slayer Camouflage), which have no Uncanny Dodge; the write-up takes it as one. Ticket 173 decides which way it goes. The Elk is a line ticket 88 didn't count as a deviation ("not a bug"); by the user's rule it is a line the engine doesn't gain.

### Tickets 89–92, re-checked

All four are fixed on the current engine; every line ticket 88 blamed on them agrees: Life-Bonding Friendship's teamwork feats (Shake It Off and Allied Spellcaster for Jack, Shield Wall and Shake It Off for the Enforcer) in the superpower's slots at MR 6 and 7 (89); the Enforcer's Trickster first ascension gives Bit of Fun at MR 1 and Summon Will-o'-Wisp, Creative Approach and Hard to Kill at MR 2 (91); Spell Penetration is Jack's level-1 Magic feat and Mounted Combat his level-1 Feat (92). The write-ups list no saves, so 90 isn't on the gold lines; the Enforcer's sheet reads Fort 0, Ref 2, Will 2 at level 1 (good Reflex and Will), and `archetype_saves.rs` pins it. Ticket 88's other "done differently" lines agree too: Ascendant Element now takes Electricity (ticket 125), and Spell Specialization needs no spell.

### Bloodline spells

They're in each level's `bloodline_spells`, apart from the gains. The engine's sheet has the Sylvan Sorcerer's own (the data grants them as features, "Bloodline spell - Entangle" at 3, and so on to 19), so those are compared, and all nine agree. The bracketed "(+ …)" ones are the Draconic second bloodline's, which the data doesn't hold (it has bloodline spells only for the archetypes whose tables list them: Sylvan, Sage, Empyreal), so they aren't compared; `the_second_bloodlines_spells_arent_in_the_data` fails when the Draconic bloodline gains any, saying to compare them.

### Names

The test's `NAMES` table, the write-up's spelling beside the planner's (`gained` must be gained at the line's place; `held` only held by then):

| Write-up | Planner (gained) | Planner (held) |
|---|---|---|
| Bypass Epic DR | Bypass Epic Damage Reduction | |
| Metamagic (Bolstered) | Metamagic (Bolster Spell) | |
| Metamagic (Heighten) | Metamagic (Heighten Spell) | |
| Charisma +1 | +1 Charisma | |
| Draconic Bloodline Arcana | Draconic Arcana | |
| Dragon Resistances I, II, III | Draconic Resistances | |
| Soul of the Fey (Power of Wyrms) | Soul of the Fey; Power of Wyrms | |
| Second Bloodline - Bronze Dragon | Second Bloodline; Draconic (Bronze) | |
| Extra Mythic Ability - Abundant Casting | Extra Mythic Ability; Abundant Casting | |
| Extra Mythic Ability - Greater Enduring Spells | Extra Mythic Ability; Greater Enduring Spells | |
| Life-Bonding Friendship (Shake It Off / Allied Spellcaster / Shield Wall) | the teamwork feat | Life-Bonding Friendship |
| Menacing Style - Intimidating Prowess / Shatter Defenses / Dreadful Carnage | the feat | Menacing Feats |
| Studied Target +1, +2, +3, +4 | Studied Target | |
| +1d6 sneak | Sneak Attack | |
| Mythic Shield Focus | Shield Focus (Mythic) | |
| Andoren Diplomat (the background) | Andoran Diplomat | |

and two rules: "Name - Value" is the planner's "Name (Value)" ("Spell Focus - Evocation", "Spell Focus (Evocation)"), with `VALUES` for a value spelled otherwise ("Battleaxe", "Battle Axe"); a group in brackets, "(Dragon Resistances I)", is the second bloodline's, each item gained at its level or, when the Second Bloodline's rank comes later, there (level 6 for Jack). Every other name is the planner's own. A name that resolves to no planner item (an item's name, or an item's with a skill, weapon, school, energy or element) is a disagreement.

### Judgement calls

- **The conversion:** a prefix is dropped as a label only when it names a slot or a list ("Bonus Feat", "Bloodline Feat", "Exploit", "Slayer Talents", "Advanced Talents", "Fey Tricks", "Superpower"); one that names an item gained or held stays ("Second Bloodline - Bronze Dragon", "Extra Mythic Ability - Abundant Casting", "Menacing Style - Intimidating Prowess", "Animal Companion - Elk", "Finesse Training - Light Shield"). A bracketed group stays one item, brackets included, so the gold file carries the write-up's mark and the test reads it. The bloodline spell keeps its bracket ("Entangle (+ Mage Armor)"). Levels with "-" have no gains. `source.txt`'s rank 4 cells wrap onto the next line with an empty line after; the reader joins them.
- **Bracketed items** are the second bloodline's: the write-up lists them at the sorcerer level that would give them, and the game gives them with MR 1's Second Bloodline. The test expects each at the later of its level and the Second Bloodline's rank's level; the Draconic Arcana and Grow Claws (level 1) and Dragon Resistances I (level 3) are gained at 6 and agree.
- **"Life-Bonding Friendship (X)"** is the teamwork feat X gained at that rank with the superpower held (at MR 7 the write-up repeats the superpower's name for its second feat); **"Menacing Style - X"** the feat X with the Menacing Feats style held.
- **Rank gains** are the rank's slot picks and its path row's features; "Mythic Hero" (MR 1 and 2) is the first ascension's path (the MR 3 path's without one), which the sheet shows there.
- **The Builds:** ticket 88's settled choices (Halfling (Hasty), Chaotic Good, atheism, one rank a level in each listed skill, the Enforcer's rogue levels' extra points in Mobility and Stealth); Jack's level-1 Bonus Feat in the Magic feat slot (ticket 92) and his Bloodline Feats in the class slots at 7, 13 and 19; Jack's first ascension Azata (Instrument of Freedom at MR 1), the Enforcer's Trickster (Bit of Fun); the Enforcer's level-13 exploit in the Greater Exploits slot, leaving the Arcanist Exploits slot empty (ticket 88's reading); the Combat Style Feat talents at 2, 6 and 12 with Menacing Feats at 2; Iceplant through the Fey Tricks slot (`witch/hex/iceplant`); the slots the known gaps leave empty: level 8's Slayer Talents, level 19's Feat and MR 10's Mythic feat.
- **The start's checks:** the race, background, classes and racial +2 are the Build's own input, compared by name only. The engine's part: the attributes are the level-1 scores the engine makes of the point buy with its racial modifiers (Jack 12/14/14/12/8/17, the Enforcer 16/16/14/10/8/12, both 25 of 25); each listed alignment is set on the whole Build (atheist) and the engine must allow it; the engine's skill points give a rank in each listed skill at every level. The first version asked whether each listed skill was a class skill or ranked at level 1, which couldn't fail (both Builds rank every listed skill); the review had it replaced.
- **Which lines test the engine:** a picked line (a feat, an option, a mythic pick, an ability increase) agrees by construction, proving only that the engine gives the slot there and accepts the pick; the granted lines (features and options gained between levels, rank path-row features, the bloodline spells) are the real comparison. Said in the README.

### The question for the user, answered

Four lines failed because of decisions on the map, and the Answer asked whether the gold standard reopens any. The user (2026-11-11):

1. Mythic Shield Focus at MR 10: "you can select mythic feats after leveling all base levels so it's perfectly fine to level mythic shield focus mythic 10 at level 20." Ticket [171](171-mythic-picks-after-the-levels.md).
2. Finesse Training - Light Shield: "make finesse training take a weapon". Ticket [172](172-finesse-training-takes-a-weapon.md).
3. Uncanny Dodge: "let's deal with it in a subsequent ticket". Ticket [173](173-uncanny-dodge-as-a-slayer-talent.md) (design; the question stays open there).
4. Animal Companion - Elk: "Add animal companions as a subsequent ticket". Ticket [174](174-animal-companions.md) (design).

So every known gap names a ticket.

### Done when

- `cargo test -p seeds --test game_data -- --list` lists `gold::gold_builds_gain_what_the_write_ups_list` and the five other `gold::` tests.
- Both RON files match `source.txt` item for item: `gold_files_match_the_write_ups` reads the source and compares, and a second read by hand agreed.
- Every disagreement is in the known-gaps list, covering its one failure, with a ticket (above).
- Removing one entry fails the test with that line's message (progress log).
- `snapshots/gold-jack.txt` and `snapshots/gold-enforcer.txt` are in the corpus.
- `scripts/check.sh`: ALL-OK.
