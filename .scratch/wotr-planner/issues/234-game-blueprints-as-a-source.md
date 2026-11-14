# The game's blueprints as a source of truth

Type: design
Status: resolved
Blocked by: none
Model: no agent. A design ticket: the user decides, in a grilling session.
Map: [WotR Build Planner](../map.md)

## Question

The map's Notes make the GameFAQs guide the source of truth: "When the game and the guide disagree, the guide wins." On 2026-11-14 research for tickets 139, 213, 219 and 222 hit questions the guide doesn't answer. The answers came from the game's own data, its **blueprints**. The user asked whether that is a better source than the guide ("wow this sounds much better source of truth than the gamefaqs guide").

**What blueprints are.** Owlcat's games keep their game data as serialized objects, each with a name and a GUID: every class, archetype, feat, selection, buff, unit, item and natural weapon. The game reads them from a `blueprints.zip` in its install folder. They are data; the rules that apply them are the game's C# code.

**Where they can be read:**
- the user's own install (`blueprints.zip`): the primary source;
- **BubblePrintsData** (https://github.com/factubsio/BubblePrintsData), a community extraction per game patch (release `2.7.0x_0` for patch 2.7.0x). It is the data the BubblePrints explorer (https://github.com/factubsio/BubblePrints) loads;
- the game's decompiled code (https://github.com/xADDBx/WrathDecompiled), for how the rules apply the data.

**What they settled on 2026-11-14** (`research/numbers-the-guide-lacks-2026-11-14.md`):
- Aivu's class is `DragonClass` (d12, full BAB, all saves good), and her natural weapons are named.
- The Skeletal Champion is `UndeadClass` (d8, ¾ BAB, good Will) at size Medium.
- Greater Power Attack with Power Attack (Mythic) gives +6 a step.
- The Seven Virtues ring works by path, unlike its text.
- Slashing Grace allows 13 weapons, not the 8 the planner inferred.
- In one case the guide is wrong: the guide puts Aivu's Large size at MR 6, but both the game and the user's table put it at MR 7.

**What weighs against it:**
- **It's data, not rules.** Some answers need the code: whether Aivu's "Large" weapons scale to her Small size could not be read from the data.
- **It includes the game's bugs.** The map models "the game's intended, current (patched) behaviour".
- **It's noisy.** NPC, army, cutscene and test blueprints sit beside the player-facing ones. The player's options are found by following the selection lists.
- **Copyright and terms.** Facts (numbers, prerequisites) can be committed, as the guide's extracted facts are. The game's text strings and the raw dump are Owlcat's content, and the planner is public on GitHub Pages. Owlcat's EULA has not been checked.
- **The guide stays useful.** Its curation, its names, the summaries the data is written from, and the `guide_url` links the planner shows users all come from it.

## To decide

1. **The rule.** When the guide and the game's blueprints disagree, which wins? Options:
   - (a) the game, with each known bug ruled on by the user;
   - (b) the guide, as today, with blueprints only filling the guide's gaps;
   - (c) case by case, each disagreement the user's.
2. **Which copy.** The user's own install, BubblePrintsData, or both (the dump to search, the install to confirm)? Which patch is pinned, and how does the data record it?
3. **Bugs.** When the game's behaviour differs from its own text (the Seven Virtues ring), which is modelled: the behaviour, the intent, or both with a note? Does the map's "intended behaviour" rule change?
4. **The first use.** Ticket [146](146-checking-the-data-against-the-guide.md) checks the data against the guide. Should it check `libs/game-data/` against the blueprints instead or as well: each feat's prerequisites, each class table, selection lists, item bonuses, with a list of differences for the user to rule on?
5. **Filling gaps.** May a ticket take a fact from the blueprints where the guide is silent, cited by blueprint name and GUID in the data's note? Does `GameData` gain a source field per fact (guide, blueprint, the user)?
6. **Tooling.** Is a blueprint reader part of the repo (a script under `scripts/` or a crate), or only a research aid kept outside it? Where does the extracted dump live (never committed: git-ignored like `guide-cache/`)?
7. **What is committed.** Facts and own-words summaries only, never blueprint text or the dump, as with the guide? Should the user check Owlcat's EULA before any of this?
8. **The guide's role after.** Do the summaries, names and `guide_url` links stay the guide's? Does a fact the guide gets wrong keep its guide link with a note?
9. **Agents.** Do the MCP's honesty boundary and the `wotr-build` skill change: may an agent cite a blueprint fact the planner holds as the planner's fact, and say where it came from?

10. **The decompiled code.** The user (2026-11-14): "there's a decompiled game code we can also use to grep the rules". Blueprints give the data; `xADDBx/WrathDecompiled` gives how the rules apply it: how Aivu's natural weapons scale with her size, how bonuses stack, what a component does. Is it a source of truth for rules, as the blueprints would be for data? How is it read (cloned locally and git-ignored, or fetched per question)? Is a rule taken from it cited by file and method? The same copyright question as 7, sharper for code: read for facts, never copied into the repo.

## Answer

**Decided** (user, 2026-11-14, in a grilling session; recommendations taken except where the user's words are quoted). The question numbers below are this ticket's "To decide".

**Facts gathered first:**
- **Owlcat's EULA** (owlcat.games/eula):
  - §2.6 forbids users to decompile, decode or reverse engineer the game's data, and to distribute the means to do so.
  - §2.9 asks anyone who wants to do those things to seek approval at `mod@owlcat.games`, for non-commercial work only.
- **Owlcat's OGL notice** (owlcat.games/OGLPFWotR) designates "the game mechanics" as Open Game Content. Proper nouns, characters, dialogue and locations are Product Identity.
- **The `.binz` file** is BubblePrints' own container (LZ4-compressed chunks of the JSON that project already extracted), not Owlcat's format.
- **The patch.** The game's last update is 2.7.0x (Steam, 2025-05-15). It is also the newest WotR release of BubblePrintsData (`2.7.0x_0`) and the newest commit of WrathDecompiled (`6ce4a6b`).

1. **The rule (1, 3).** The user: "game wins on rules and guide only wins if it has a more descriptive description but must not contradict".
   - The blueprints are the source for data, the decompiled code for how the rules apply them.
   - The game's code is taken to work always (user: "assume the game code works always"). So when the game's behaviour differs from its own text (the Seven Virtues ring), the behaviour is modelled, and the summary says where it differs from the in-game text.
   - Known exploits (a player's tricks) stay unmodelled.
2. **Which copy (2).** BubblePrintsData's `2.7.0x_0` release, kept in a git-ignored `blueprints/` folder in the repo. WotR isn't installed on the user's Mac. The patch is pinned at 2.7.0x, and it moves only when both sources have caught up with a new one.
3. **The decompiled code (10).** It is the source of truth for how rules apply. The user: "Clone it and let's use the latest version as the source of truth", and "put the clone inside our repo in a gitignored folder".
   - It is cloned as `decompiled/` (xADDBx/WrathDecompiled at `6ce4a6b`, "2.7.0x"), with `/decompiled/` in `.gitignore`.
   - A rule taken from it is cited by file and method, with the WrathDecompiled commit.
4. **What is committed (7).** Facts in our own words, with blueprint names and GUIDs as citations. Never the game's text strings, the dump or the game's code, not even as a quoted snippet in a ticket.
   - Code of our own that reads the community's `.binz` is allowed. The user went further than the recommendation (see 9).
   - The published site gains the OGL notice and Section 15 credit for the mechanics it holds. That was owed already for the facts taken from the guide.
5. **Your own facts.** The game wins over the user's tables too (skills, backgrounds, Aivu). But a difference with one of the user's facts goes to the user before the data changes.
6. **Applying a difference with the guide.** A ticket applies the game's fact without asking and lists every change. Only differences with the user's facts come to the user.
7. **The guide's role after (8).**
   - Each summary is written in our own words from whichever source says more, and never contradicts the game.
   - `guide` (the link) stays wherever a guide section exists, and becomes optional so the data can hold what the guide lacks.
   - A fact the guide gets wrong keeps its link, and the data's note records the difference.
8. **A blueprint GUID per entry (5).** Each feat, class, archetype, feature, selection, option, mythic ability, race and item gains an optional blueprint GUID, which `GameData::check` checks.
   - The planner's own keys stay, since links depend on them.
   - There is no per-fact source field; the exceptions go in the data's notes.
   - The user's question that led here: "aren't GUID's reference for you to search and create a tree of knowledge?" Blueprints refer to each other only by GUID, so walking from the classes, races, paths and feat selections reaches exactly what a player can get, past the dialogue and cutscene noise.
9. **Tooling and checks (6).** A game check in `tools/` reads the `.binz` from `blueprints/`. It compares each GUID-mapped entry's prerequisites, class table, progression rows and option lists with its blueprint, and prints the differences. It is run by hand.
   - **The regression checks go.** The user: "I would argue we don't even need proof.sh and check.sh as they are today anymore". Asked which checks remain (recommended: keep the snapshot check, make the proof optional), the user chose "c": the snapshot check and the proof go, and tests, the game check and the gold builds remain.
   - **Now.** The user: "the queue stopped so c remove them now" (recommended: after the game check exists).
   - **`check.sh`** keeps its build steps (fmt, clippy, tests, the wasm build, `check-data`).
10. **The oracle.** The engine's self-checks stay: the agreement sweeps (picker against walk), the record's consistency and shortcut checks, the `allows` battery, and the corpus they run on. They move out of `tools/proof`. The user: "we need to rethink the oracle if they contradict the game rules we go with the game rules. assume the game code works always."
    - A self-check never outranks the game.
    - A test's expected value comes from the game, cited by blueprint GUID or by code file and method, never from the engine's earlier output.
11. **The tests we already have.** The user chose "b" (recommended: update each test when a ticket changes its fact): a ticket re-derives every existing real-data test's expected value from the game.
12. **Ticket 146 (4).** Closed as superseded by this ticket. The game check replaces its spot checks (A). The gold builds (B) stay, and a gold build's gain that contradicts the game goes to the user, as in 5.
13. **Agents (9).** The honesty boundary is unchanged. The MCP instructions and the `wotr-build` skill say summaries are "the planner's notes on the game". `guide_url` stays a link for the user that agents never fetch. `describe` shows no GUIDs or code citations.
14. **A check of what exists.** The user: "not only re-research open numbers but also create tickets to exercise our existing logic". An agent re-answered the open numbers of [research/numbers-the-guide-lacks-2026-11-14.md](../research/numbers-the-guide-lacks-2026-11-14.md) from the code and the blueprints, and compared the data and the engine's rules with the game. Its findings are in [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md), and each area of differences is a ticket.

## Done when

The user has decided each question, the decisions are in the Answer and on the map (its Notes on the source of truth rewritten), and the build is ticketed.

Built by:
- [235](235-remove-the-regression-checks.md) removes the regression checks and keeps the self-checks (not blocked);
- [236](236-the-game-sources-in-the-repo.md) puts the game sources in the repo;
- [237](237-a-blueprint-guid-per-entry.md) adds a blueprint GUID per entry (after 236);
- [238](238-the-game-check.md) is the game check (after 236 and 237);
- [239](239-the-guide-link-becomes-optional.md) makes the guide link optional;
- [240](240-agents-read-notes-on-the-game.md) rewords what agents read;
- [241](241-ogl-notice-and-credit.md) adds the OGL notice and credit;
- [242](242-the-tests-expected-values-from-the-game.md) re-derives every test's expected value from the game (after 235 and 236);
- the difference tickets from 244 on, filed from [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md), applied without asking unless one touches the user's facts.
