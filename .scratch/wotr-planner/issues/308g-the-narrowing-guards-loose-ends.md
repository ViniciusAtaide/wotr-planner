# Importer S4, 308g: the narrowing guard's loose ends

Type: build (the importer, stage S4: tooling)
Status: resolved
Blocked by: 308a
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

308a's second review (2026-11-17) passed the prerequisites writer and its narrowing guard (`tools/game-check/src/write/prerequisites.rs`, `tools/game-check/prerequisite-narrowing.txt`), with follow-ups that should land before the data slices 308c–f write with it:

1. **Stale allowlist lines are never reported** (`allows`, `prerequisites.rs` ~225), unlike `known.txt`. Once 308c–f write the data, the lines they used match nothing, and a stale line would silently re-allow a narrowing that comes back.
2. **A blocked narrowing prints `-> 308`, not its owner** (`prerequisites.rs` ~296), so its `known.txt` line and the Answer name the owner by hand.
3. **Two entries are blocked that the game allows:**
   - Extra Mercy: `selections/paladin/mercy.ron` is the only selection offering mercies, and the Divine Hunter grants it at 3 (`archetypes/paladin/divine-hunter-paladin.ron`), so everyone holding a mercy holds the selection.
   - Impossible Domain: all 7 cleric archetypes keep `cleric/domain` in row 1, and no cleric archetype removes `DomainsSelection` in the game (`refs`); the Divine Hunter and Living Deity map their own game selections. The user's answer (research/308/decisions.md item 1) stands.
4. **Missing tests:** the allowlist parser's refusals; the embedded file parses; the shared-own exemption passing with no allowlist line; a drop inside a nested `Not(Not …)` or an `Any` inside `All`.

## Change

- Report each allowlist line that matches no narrowing, as the game check reports a stale `known.txt` line, and fail on it (a data slice removes the lines it used).
- Print each blocked narrowing's owner ticket, from the gap's owner the writer already resolves (§1.2 of research/308/slices.md).
- Allowlist Extra Mercy's and Impossible Domain's narrowings, each with its reason and blueprint evidence.
- The four tests.

## Done when

`cargo test -p game-check` passes with the new tests listed, the game check exits 0 and reports no stale allowlist line, and a dry run lists Extra Mercy and Impossible Domain as written.

## Answer

**Built** (base `60659ba2`; no data written):

- **Stale lines** (`write/prerequisites.rs`): `changes` became a wrapper over `plan`, which also returns every narrowing a rewrite makes, by entry, allowed or not. `Narrowings` keeps each narrowing's line (number and text). The new `pub fn stale(dump, data, narrowings)` runs `plan` over every entry and returns each allowlist line no rewrite makes, as `line <n>: <line>`. `Report::with_stale_narrowings` (`report.rs`) lists them under "prerequisite-narrowing.txt lines that match no narrowing:" and fails the report. `main.rs`'s check path reads the data once as the files hold it (the writer reads `Requires`), checks a clone for the comparison, and attaches the stale lines. So the plain game check exits 1 on a stale line, as it does on a stale `known.txt` line. The module docs and the file's header say so.
- **Parser** (`Narrowings::parse`): a line ending in ` ->` is now refused as "an empty reason". That branch was unreachable before, because the trimmed line never held ` -> ` with nothing after it. An entry's reason with no narrowing of that entry is refused: it would be a stale line the per-narrowing check can't see.
- **Owners**: `narrowings_of` returns a `Narrowing { said, blueprint, owner }`. The owner is `owner(ctx, guid)` of the fact's blueprint, the same name-based owner a gap gets (§1.2), and 308 when it names none. The blocked list prints `-> <owner>`.
- **Allowlist** (`prerequisite-narrowing.txt`): Extra Mercy's 15 drops (each `paladin/mercy/*` option) and Impossible Domain's 5 (classes cleric and inquisitor, archetype `hunter/divine-hunter`, the Living Deity way, Pentamic Faith), each with its reason and evidence.
- **`known.txt`**: `mythic ability impossible-domain -> 294` is now `-> 265`, with the other mythic abilities 308d writes. The domains (294) no longer block it. The 308a fix-round comment is updated: Extra Mercy and Impossible Domain are allowlisted, and the writer prints `-> 308` for the five entries still blocked.

**Decisions:**
- **Where stale lines fail** (builder's): in the plain game check, which the main session runs after every data merge. A data slice's write uses its lines in that same run, so they only go stale afterwards. Only the check sees them then.
- **The five still-blocked entries keep their hand owners** (builder's): Extra Channel (267), Favored Enemy Spellcasting (275), Battle Master (278), the 13 Hexcrafter hexes (283) and Mantis Zealot (267). `owner` names none of their facts' blueprints (`ChannelEnergySelection`, `FavoriteEnemyRankUp`, `WildlandShamanArchetype`, `HexcrafterArchetype`/`MagusClass`, `DeitySelection`), so they print `-> 308`. Their `known.txt` lines name the tickets the 308a fix round found by reading the game, which is where the fix lands, so they're left as they are.
- **Impossible Domain's Pentamic Faith drop** (user, 2026-11-17, "as the game"; decisions.md item 1).

**Evidence** (verified with `game-check show/refs/granted-by` on the pinned dump):
- `ExtraMercySelection` 8a49abed5be9473da1e1fd1e2457562e: one `PrerequisiteFeature` (`Any`) of `SelectionMercy` 02b187038a8dce545bb34bbfb346428d. In the game, `DivineHunterArchetype` fec08c1a3187da549abd6b85f27e4432 (the paladin's) removes `SelectionMercy` at level 6 only, and `archetypes/paladin/divine-hunter-paladin.ron` grants `paladin/mercy` at 3. The mercy options are offered only by `paladin/mercy` and by `paladin/extra-mercy`, whose feat needs a mercy first.
- `ExtraDomain` 213a8480d22206b45acbfa0619ca5aaf: `Any` of `DomainsSelection` 48525e5da45c9c243a343fc6545dbdb9, `DivineHunterDomainsSelection` 72909a37a1ed5344f88ec9d1d31f5c5b and `FakeDivineSparkSelection` e6d337e709b4478cb6e5389d3f85b601.
  - `refs DomainsSelection`: `ClericProgression` and `InquisitorProgression` grant it, and no cleric archetype references it. The planner's cleric and its 7 archetypes grant `cleric/domain` at 1.
  - `FaithHunterArchetype` ac87f40932cfd324c90b441f93857c40 removes it at 1, and `faith-hunter.ron`'s row 1 lacks `inquisitor/domain`. The old tree let that inquisitor in, and the game doesn't.
  - `DivineHunterArchetype` f996f0a18e5d945459e710ee3a6dd485 (the hunter's) adds `DivineHunterDomainsSelection` at 1.
  - `MagicDeceiverWayOfLivingGodProgression` 464ffaf88140474a949618bec7955e78 grants `FakeDivineSparkSelection` only at 20, as the way's row 20 grants `arcanist/magic-deceiver/domain`.

**Tests** (`tools/game-check/tests/fixture/prerequisites_write.rs`; the world gains `domain-feat` 216 `PlantedDomainProgression`, `narrow-domain` 217, `nested-not` 218, `all-any` 219):
- `a_blocked_narrowing_names_its_facts_owner`: failed first with `-> 308` for `-> 294`.
- `an_allowlist_line_that_matches_no_narrowing_fails_the_check`: failed first, `left: []`.
- `the_allowlist_parser_refuses_a_malformed_line`: failed first on the empty reason.
- `the_embedded_allowlist_parses`, `a_shared_own_refusal_is_written_with_no_allowlist_line`, `a_drop_under_two_nots_is_left_whole_and_listed` and `a_drop_from_an_any_inside_an_all_is_left_whole_and_listed`: coverage of behaviour that already held, so they passed first.

**Runs** (after the last source edit):
- `cargo test -p game-check`: 158 passed.
- `cargo fmt --all --check`: clean. `scripts/test-modules.sh`: OK.
- The game check: exit 0, "3832 known in 199 groups", no stale line.
- `--write prerequisites --dry-run`: 254 fields to write, up from 252, with `feat extra-mercy` and `mythic ability impossible-domain` among them. 368 entries are left whole, down from 370, and 30 narrowing lines are blocked, down from 50.
