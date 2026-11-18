# The 308 queue's review fixes

Type: build (tests and tooling)
Status: resolved
Blocked by: 323
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

The code review of the queue's merged range (`2e044a3c..0a10cf89`, 2026-11-17) found five things the queue caused:

1. **One approval clears every guarded change in a range.** `scripts/guarded-files.sh` (~150) accepts any added `User approved:` line under `issues/` for every guarded hit in `<base>..HEAD`, so 308e's Enforcer approval would also pass another ticket's gold or held-fact change in the same range.
2. **A test still ignored that 308c made passable.** `libs/seeds/tests/game_data/mercies_and_exploits.rs` ~601 waits on 293 for Extra Arcanist Exploit's prerequisite as the game's selections, which 308c wrote (`Has(Selection("arcanist/exploiter-exploits"))`).
3. **An exception by key alone.** `tools/corpus/tests/links.rs` ~250, ~302: 306n's `SKALD_BONUS_FEAT_LACKS` lets its 10 keys through in any slot, against `CODING_STANDARDS.md` ("names its entry and its slot kind, never a key alone").
4. **Docs of 306aa's removed rule.** `mercies_and_exploits.rs` ~555-556 ("ticket 49's rule: the selection it had a slot of first"), `extra_picks.rs` ~313 ("the selection she had a slot of first"); a stray `/// A Feature selection by key.` opening `picks_of`'s doc in `support.rs` ~1054.
5. **A weak assertion.** `option_follow_ups.rs` ~577 asserts `Err(Issue::Unmet(_))` where the removed assertions checked a Magic Deceiver's domain levelled by arcanist levels at 6 and 8.

## Change

1. Each guarded hit needs an approval that names it: the gold Build's function (`jack`, `enforcer`) for a pick or expected value, the gap's number for a `KNOWN_GAPS` line, the fact's key for a `held` line, `gold/` file or comparison function by name. The script lists each hit's name and fails on a hit no added `User approved:` line names (case-insensitive). 308e's line ("the Enforcer gold build … gap 173") still passes the queue's range. Tests for a hit named, a hit unnamed beside another named, and the queue's range.
2. Un-ignore the test, run it, and fix its reason or its data if it fails (by the game).
3. Scope `SKALD_BONUS_FEAT_LACKS` to the skald's level-1 bonus-feat slot, and say so in the comment.
4. Make the docs true.
5. Assert the condition the game refuses with (the domain selection), as the test's name says.

## Done when

`cargo test -p seeds -p corpus` pass, the guard's test passes with the new cases, `bash scripts/guarded-files.sh 2e044a3c` exits 0, fmt and `scripts/test-modules.sh` pass.

## Answer

Built on base c977f7eb (Opus builder). The five items are the ticket's; the forms below are the builder's unless marked. No data and no engine changed.

**1. Each guarded hit needs an approval that names it** (`scripts/guarded-files.sh`). Each guarded line now carries a name, and an added `User approved:` line clears only the hits whose name it holds as whole words, case-insensitively (an awk matcher, not `grep -w`: ugrep's and GNU's word rules differ). The names (the header lists them):
- a line inside a gold Build or comparison function, a line of an assertion, or a test's `#[test]`/`#[ignore` line: the function's name (`jack`, `enforcer`, `compare`, the test's name);
- `impl Gains`/`impl Covers`: `Gains`/`Covers`; `const NAMES`/`const VALUES`: `const NAMES`/`const VALUES` (builder: the bare words are common English);
- a `KNOWN_GAPS` entry, every line of it from `gap(` to its closing parenthesis: `gap <n>`, its `Why`'s number; the list's own lines `KNOWN_GAPS`;
- a `held` user fact: its last ` / ` field (`gear item belt-of-giant-strength-2`);
- a `gold/` file: its file name.
A failure prints each unnamed hit with its name and the diff's approvals. The queue's range has one hit, the Enforcer's level-8 pick (`enforcer`), named by 308e's line; `bash scripts/guarded-files.sh 2e044a3c` exits 0. New cases in `scripts/guarded-files-test.sh` (the fixture gains `fn enforcer` and a multi-line gap 173): an approval that names nothing fails; one naming `jack` passes; `Jack-in-the-Green` (another case) passes; `jackal`/`jack_pick` fail; Jack's pick named beside the Enforcer's unnamed fails; an added gap with another gap named fails, with `gap 5` named passes; a held fact named passes; a range approval that names nothing fails, one naming `gap 2` passes; the 308 queue's shape (a pick added to `enforcer`, gap 173 deleted, 308e's exact line) passes. The `unset` of git's variables and the decoy case are kept.

**2. The passable test** (`mercies_and_exploits.rs`, `an_exploiter_1_takes_extra_arcanist_exploit_with_no_exploit_picked`): un-ignored. It failed in its own body, not by the game: `why_not` panics on a possible Build, and the Build is possible. Its data is the game's: ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a has PrerequisiteFeature ArcanistExploitSelection or ExploiterExploitSelection 2ba8a0040e0149e9ae9bfcb01a8ff01d (Group Any), which `feats/extra-arcanist-exploit.ron` writes. It now asserts the general slot's status of the feat is `Ok(())` on an Exploiter 1 with no exploit picked, and the Build with it is possible.

**3. The skald exception** (`tools/corpus/tests/links.rs`): `SKALD_BONUS_FEAT_LACKS` passes a refused pick only in a feat slot of `skald/bonus-feat` (new `SKALD_BONUS_FEAT`), read from the Build's sheet; the skald's and its four archetypes' level-1 rows alone grant it. The Battle Scion link refused at `Feat { level: 1, source: Class, index: 0 }` with Dispel Synergy still passes. The rogue-talent and Battle Prowess exceptions are unchanged (outside the ticket).

**4. The docs:** `extra_arcanist_exploit_gives_an_exploiter_wizard_one_more_exploit_slot`'s doc and `extra_picks.rs`'s witch comment say the slot is of the selection the feat grants (306aa); `/// A Feature selection by key.` moved from `picks_of` to `any_selection`.

**5. The assertion** (`option_follow_ups.rs`, `a_living_deity_below_arcanist_20_cant_take_impossible_domain`): the removed assertions (308d, 5b1878f3) checked the Nobility domain levelled by arcanist levels (powers at 6 and 8), which the game now refuses before 20. The test asserts the refusal exactly: `Unmet::Any` of `Any(cleric/domain, inquisitor/domain, inquisitor/sacred-huntsmaster/domain)`, `arcanist/magic-deceiver/domain` and `hunter/divine-hunter/domain`, each `Prerequisite::Selection`. Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf, three PrerequisiteFeature (Group Any): DomainsSelection 48525e5da45c9c243a343fc6545dbdb9 (the three domain selections whose entries name it), DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b, FakeDivineSparkSelection e6d337e709b4478cb6e5389d3f85b601 (`game-check show`).

**Checks:** `cargo test -p seeds -p corpus` exit 0 (game_data 646 passed, 89 ignored); `bash scripts/guarded-files-test.sh` OK; `bash scripts/guarded-files.sh 2e044a3c` exit 0; `bash -n` on both scripts; `cargo fmt --all --check` clean; `scripts/test-modules.sh` OK.

**Not done:** `CODING_STANDARDS.md`'s `User approved:` bullet doesn't yet say the line names each change (outside this lane's files).
