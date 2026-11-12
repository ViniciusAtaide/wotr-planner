# Gold builds: Jack-in-the-Green and the Enforcer as the standard to compare against

Type: task
Status: open
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
