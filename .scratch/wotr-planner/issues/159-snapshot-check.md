# A cheap behaviour check: snapshots of a small corpus

Type: task
Status: done
Blocked by: 158
Model: Opus to build (it designs the projection and the message every later ticket reads); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "my idea is to simplify the proof concept to be something very cheap to do instead of resource heavy. I was thinking about a simple tree diff simple comparison between builds and a generic message telling what's the difference between the builds."

What the proof costs (`scripts/proof.sh`): comparing takes seconds. The cost is producing two sides: two release builds of the workspace, and a record of 2,163 Builds per side (about 60–70 s each since commit `b573ef9`), most of it the `allows` battery and the consistency checks. Its record hashes most fields, so a difference names a Build and a field, never the values.

Decided in the grilling session (user, 2026-11-11):

- **Two tiers.** This cheap check runs on every change. The proof stays, and runs once at the end of a queue of tickets ([161](161-proof-once-per-queue.md)).
- **One side.** The check runs the current engine only. The other side is snapshot files committed in the repo, so an intended change shows up as a snapshot diff in the commit.
- **The claim is weaker than the proof's:** nothing changed in these Builds. It has no `allows` battery and no consistency asserts; those stay in the proof.

## Build

AFK. A new test and its files: no engine, data or schema change.

**Wait for [158](158-faster-engine-walk.md)** (an agent is live in its worktree, user 2026-11-11). This ticket adds files only, but its snapshots record engine output, and 158 must not change them: take the snapshots after 158 is merged.

- **Where:** `libs/seeds/tests/game_data/snapshots.rs`, declared with `mod snapshots;` in that folder's `main.rs`. It uses the real data and engine that `support.rs` loads once, and `scripts/check.sh` already runs it.
- **Corpus**, written in Rust with `support.rs`'s helpers, never as stored link codes (the bench links have gone stale before, ticket 86):
  - the 6 bench Builds (`support.rs`'s `bench()`);
  - one Build per class at level 20 with no picks;
  - one Build per mythic path at Mythic rank 10 with no picks beyond the path.
  
  Assert each one is possible: an impossible Build yields no options and would snapshot as empty.
- **The projection:** a small module in the test, about 150–200 lines, that turns a Build's engine output into flat `path = value` lines in tree order. It uses only public engine calls (`evaluate`, `options`, `totals`, `caster_line`, `choice_label`, `pick_name`, `feat_name`, `Address`'s `Display`). The engine's types stay serde-free.
  - **The sheet, for every Build:** the start picks; per level the class, BAB, saves, scores, skill points and ranks, and the feats and features *gained at that level* (not the whole state, so one new feature is one line); per Mythic rank the same; the last level's totals and caster lines. Copy the shape of the MCP's `SheetResult` (`apps/mcp/src/shapes.rs`); the test can't call it, since `mcp` depends on `seeds`.
  - **The options, for the pick-less class and path Builds only:** one line per slot, `<slot address>.open = key, key, …`, listing the keys `Engine::options` returns as open. A parameterised pick is `key(value)`. The bench Builds get the sheet only: all their slots are filled.
- **Snapshot files:** one per Build, `libs/seeds/tests/game_data/snapshots/<name>.txt`. Expect about 15–25 KB for a sheet and 45–80 KB with every slot's options, 2–3 MB in all.
- **Regenerating:** an environment variable, `SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`, rewrites the files. No `insta`.
- **The message.** One `#[test]` compares every Build's stored and current lines, and on any difference panics with:
  - one line per changed path and value pair, with the number of Builds it affects and the first three names (`level.5.saves.will: 4 → 5 in 12 Builds: …`);
  - `.open` lists expanded to one path per key, so a new option reads `+ slot.feat.5.general.0.open/power-attack`;
  - a second grouping with numbers wildcarded (`slot.feat.*.general.0.open/power-attack: + in 31 Builds, 212 slots`), so one newly legal feat prints one line;
  - a missing or extra snapshot file named as such;
  - the regeneration command as its last line.
- **Test the message itself** on made-up line sets (a changed value, an added and a removed option, a missing file), so its grouping is pinned without the real data.
- Measure the test's wall time in a debug build on a quiet machine and record it in the Answer. The estimate from recorded timings is 2–4 s when the Builds run on threads. If it is over 10 s, say so and stop for a decision before trimming the corpus.
- Document it in `libs/game-data/README.md`, where the real-data tests are described: what the check covers, how to read its message, and how to regenerate.

## Done when

- `cargo test -p seeds --test game_data -- --list` shows the snapshot test and the message's own tests.
- The corpus covers every class and every mythic path in `libs/game-data/`, found from the data's own lists, so a new class without a snapshot fails the test.
- A throwaway change to one feat's prerequisite (reverted) fails the test with a grouped message that names the feat, and `SNAPSHOTS=write` then makes it pass.
- The Answer records the corpus size, the snapshots' total size and the measured run time.
- `scripts/check.sh` passes.

## Answer

**Done**, 2026-11-11 on branch `ticket-159`, from `main` at `34143b5` (158 merged). The real-data test `snapshots` (`libs/seeds/tests/game_data/snapshots.rs`, `mod snapshots;` in its `main.rs`) and 42 snapshot files in `libs/seeds/tests/game_data/snapshots/`; no engine, data or schema change. Documented in `libs/game-data/README.md`, "The snapshot check".

### What was built

- **The corpus, 42 Builds:** the 6 bench Builds (`bench-<name>`: blank, start, fighter20, fighter20_nopath, multi_prestige20, legend40), the 26 classes of `DATA.classes` at level 20 with no picks (`class-<key>`) and the 10 mythic paths of `DATA.mythic_paths` on a fighter 20 with no pick beyond the path (`path-<key>`). Classes and paths are read from the data's lists, so a new one is a missing snapshot file and fails the test. Each is asserted possible before it is projected.
- **The projection** turns a Build into `path = value` lines in tree order, from `evaluate`, `options`, `totals`, `caster_line`, `choice_label`, `pick_name`, `feat_name`, `feature_name_apart`, `class_value_name`, `rank_features` and `Address`'s `Display` (the engine stays serde-free). The shape is the MCP's `SheetResult`: the Build start (`start.*`, with the point buy spent, and `mythic.first-ascension`/`path`/`switch`); per level the class (with archetype and advances), class level, BAB, the three saves, the six scores, skill points available and spent, the ability increase, the ranks the level adds per skill, each slot as `Label: key "Name"` (`-` while empty), and the feats and features gained at that level (`level.N.feat/<key>`, `level.N.feature/<key>`); per Mythic rank its character level, status, path, slots, features (`×2` for one given twice, the Trickster's Sneak Attack) and dropped slots; at the last level the caster level, spell levels, class skills, the totals (initiative, saves, each skill, each with its parts) and the caster lines. The pick-less class and path Builds also get `<slot address>.open = key, key, …` after each slot, the keys `options` returns as open, sorted, a parameterised one `key(value)` (`Parameter::value_key`).
- **Regenerating:** `SNAPSHOTS=write cargo test -p seeds --test game_data snapshots` rewrites every file and deletes the file of a Build no longer in the corpus, listing the files by name without reading them, so it also repairs a file that doesn't parse. Any other value of `SNAPSHOTS` fails the test.
- **Prestige classes, a guard:** `UNCOVERED_PRESTIGE_CLASSES` lists the 13 prestige classes no Build covers (all of today's; the bench's `multi_prestige20` takes the Assassin, which is listed all the same), and `snapshots_list_every_uncovered_prestige_class` fails when `DATA.prestige_classes` has a key not on it or the list names one that's gone, saying to add a Build for the new class or extend the list.
- **The message** (`snapshots::message`): `.open` lists are expanded to one path per key with no value, then each Build's stored and current lines are compared by path. The first grouping is one line per path and change with its number of Builds and the first three names, most Builds first (`level.5.saves.will: 4 → 5 in 4 Builds: a, b, c, …`, `+ slot.feat.5.general.0.open/power-attack in 36 Builds: …`, `- level.2.feature/x = X …`). The second wildcards the path's first number, its level or rank, keeping slot indices as the ticket's example does (`slot.feat.*.general.0.open/power-attack: + in 36 Builds, 357 slots`). Missing and extra files are named as such; a file that doesn't parse (a merge conflict's markers) is named with its first bad line (`class-wizard.txt: line 1 isn't `path = value`: <<<<<<< HEAD`); a file whose lines match by path but not as written (reordered, a line twice, an `.open` list out of order) is listed under "Same lines, different order or duplicates". The header counts every Build that differs, missing and unparseable files included, and the extra files after it (`… in 0 of 42 Builds, and 1 snapshot file with no Build`). The regeneration command is the last line. Each grouping shows 60 lines, then "… and N more".
- **The message's own tests** (14), on made-up line sets: nothing differs (`None`); a changed value grouped by Build; added and removed options with the wildcarded grouping; added and removed lines with their values; missing and extra files with the command last; what the wildcard replaces; one path changing to different values in different Builds kept apart in both groupings; the most Builds first; three names without an ellipsis, four with one; the 60-line cap; different changes never merged by the wildcard; an unparseable file; the header with only missing or only extra files; reordered and doubled lines.
- **Counts and statuses in the lines:** a feat or feature a level gives is written with `×N` when the level gives more than one copy, and a feat gained again at a later level (a second class's Simple Weapon Proficiency) gets a line there too, as a rank's do. A rank's status is a fixed word (`reached`, `partly-dropped`, `dropped`, `not-reached`), not the enum's `Debug`.

### Sizes and time

- **Corpus:** 42 Builds, 36 of them with options.
- **Snapshots:** 3,703,389 bytes (3.6 MB), 18,420 lines. The bench sheets are 1–32 KB (93 KB in all), the class Builds 67–126 KB (2.2 MB), the path Builds 128–221 KB (1.4 MB). Most of a pick-less file is its `.open` lines (fighter: 113 of 124 KB), since a parameterised feat is listed once per value. Above the 2–3 MB estimate, mostly because each path Build is a fighter 20 whose feat slots are listed too; kept, since a path's features can open feats (Legend's +4, the Swarm's Str and Dex).
- **Run time:** 1.5 s wall in a debug build (`snapshots::snapshots_match_the_engine` alone, 5 runs of the test binary: 1.51–1.54 s at a 1-minute load of 9–10; after the review fixes 1.48–1.51 s at 5), the Builds projected on 8 threads. Writing the snapshots takes the same. Under the 2–4 s estimate, so the corpus wasn't trimmed.

### The throwaway change

Power Attack's prerequisite in `feats.ron` lowered from Strength 13 to 10, then reverted (the snapshots regenerated after the revert are byte-identical to those committed). The test failed with:

```
the engine's output differs from the snapshots in 36 of 42 Builds

Changes, each with the Builds it's in:
  + slot.feat.1.general.0.open/power-attack in 36 Builds: class-alchemist, class-arcanist, class-barbarian, …
  + slot.feat.11.general.0.open/power-attack in 36 Builds: class-alchemist, class-arcanist, class-barbarian, …
  …
  + slot.feat.12.class.0.open/power-attack in 13 Builds: class-cavalier, class-fighter, class-warpriest, …
  …
  + slot.feat.9.class.0.open/power-attack in 1 Build: class-warpriest

The same, levels and ranks wildcarded:
  slot.feat.*.general.0.open/power-attack: + in 36 Builds, 357 slots
  slot.feat.*.class.0.open/power-attack: + in 14 Builds, 127 slots

If every change is intended, regenerate the snapshots:
SNAPSHOTS=write cargo test -p seeds --test game_data snapshots
```

(27 lines in the first grouping in all.) `SNAPSHOTS=write` then rewrote 36 files and the test passed.

### Judgement calls

- **"Every class" is every base class, and a guard for the prestige classes.** A prestige class has no pick-less Build: its entry needs feats and skill ranks, and it has 10 levels, not 20. Covering one means a hand-written Build per prestige class with the picks its entry needs, which the ticket's corpus doesn't describe; only the bench's Assassin (`multi_prestige20`) is covered. So the Done-when's "a new class without a snapshot fails the test" holds for prestige classes through the list of uncovered ones (review, 2026-11-11): a new one fails until it gets a Build or joins the list. Archetypes aren't in the corpus either (a pick-less archetype Build can be impossible, `arcanist/phantasmal-mage`).
- **A pick-less Build that's impossible gets an alignment.** Each class Build starts as `support::build` makes it (with a faith a deity-bound class needs: the cleric, inquisitor and warpriest follow Abadar); one still impossible gets the first alignment of `Alignment::ALL`, with a faith that fits, that makes it possible (`support::realigned`): the barbarian, druid, monk, paladin and shifter. Every class and path came out possible this way.
- **Path Builds:** a fighter 20 (MR 10 comes at 19), as `support::mythic` makes them. A path taken only by a switch (Devil, Gold Dragon, Legend, Swarm-That-Walks) is switched to from the first path, in key order, it can be switched to from (Aeon for all four); no first ascension.
- **The bench Builds come from `bench()`,** as the ticket says, so from `tools/proof/bench-links.tsv`'s link codes, not from Rust like the rest. `bench()` already opens a stale one as an atheist and asserts each possible.
- **Skill ranks per level are the ranks the level adds** (`level.N.skill.<skill>`, its choice), not the running totals, so one rank moved is one or two lines; the totals at the last level carry each skill's ranks.
- **Caster lines:** 15 per casting Build, for the spell traits whose bonuses the data distinguishes: each school, Evocation for each energy type and with metamagic, and Enchantment for charm and compulsion. The class, ability, caster level, spell penetration and concentration don't depend on the spell and are written once (`caster.*`). No attack line: the ticket's sheet doesn't list one.
- **Names beside keys:** values are `key "Name"` where the MCP gives both, so a renamed feat or feature shows up too.
- **The projection is about 400 lines** with the formatting, not 150–200: it writes the whole sheet, the totals and the caster lines, each a few lines per field.
- **One path per line:** the projection asserts no path repeats, since the comparison is by path.

### Review fixes (2026-11-11)

An independent review found no blocker and six things to fix, all made: write mode lists files without parsing them and compare mode names an unparseable file with its first bad line instead of panicking; the header counts missing and extra files; the prestige guard above; a file's line sequence is compared when its lines match by path; `×N` on feats and features a level gives, and feats gained again; fixed status words. Regenerating changed every file's 10 `rank.N.status` lines (`Reached` → `reached` and so on) and nothing else but: `bench-multi_prestige20` gains `level.2.feat/` lines for Armor Proficiency (Light Armor) and Simple Weapon Proficiency, `bench-legend40` `level.21.feat/` lines for those two and Brew Potions (feats a second class grants again, which the old projection skipped), and `path-trickster`'s `level.9.feature/trickster/sneak-attack` became `Sneak Attack (Trickster) ×2` (MR 3 gives two copies).

