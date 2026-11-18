# Rescue retros

The `subagent-rescue` skill appends one entry per rescue: the ticket, the minutes it took, what made it slow, and the change that would have prevented it, marked applied or proposed. A cause seen twice gets a mechanical fix.

## 2026-11-17: ticket 306i (exploits, arcana, mercies), about 65 minutes

- **Slow because:** it stalled 33 minutes, with no writes and no compiles, before its first full test run. Then that full `seeds` `game_data` run took about 5 minutes and showed 11 failures, which it iterated with full runs rather than filtered ones. It had also set `Status: resolved` before writing any Answer.
- **A move away from the game, caught by the rescue:** moving the six greater exploits into the ordinary exploit lists dropped the game's `ArcanistGreaterExploitsFeature` requirement. The brief didn't say that moving an option into a wider list carries over its owner's gating.
- **Changes:**
  - (proposed) `ticket-builder.md` step 3: iterate with `cargo test -p <crate> --test <bin> -- <filter>`, and run a crate's whole suite only at the end. The step says this already; repeat it in briefs for data slices, whose suites are slow.
  - (applied: research/306/decisions.md) Family-slice briefs: an option moved from a gated selection into a wider list keeps the gate as a `Requires` (cite it), or the census shows offers moving away from the game.
  - (applied) `ticket-builder.md` step 8: set Status only together with the Answer, in one edit.
  - (applied) The rescue helper and its 30-minute watchdog in the `ticket-queue` skill.

## 2026-11-17: ticket 306j (hexes, with the Sylvan Trickster's fey tricks), about 120 minutes

- **Verdict:** not stuck. It committed `421d1e28` at 03:24 with the full `seeds` and `corpus` suites passing and the game check at exit 0 (0 differences not known). The rescue had nothing to unblock.
- **Slow because the machine slept.** The build took about 33 minutes of work (01:25 to 01:58). Its final full run started at 01:58 and ended at 03:05, but the tests' own times are 45 s (`links`) and 183 s (`game_data`). `pmset -g log` shows the Mac, on battery, in "Maintenance Sleep" from 02:14:49, waking for a few seconds about every 15 minutes. A frozen process looks like a hung one: `ps` showed `links` at 16 minutes elapsed and load averages of 83 to 112 at each wake. The rescue's own read-only commands took 15 to 35 minutes each for the same reason.
- **Second cause, seen in 306i too:** the tests that pin a shape were found only by a full run. The first full run (cut at 01:52) showed 15 failures: about 10 pins of a count or a key (262 feats, 36 repeating feats, 634 `Requires`, the Hexcrafter's row 4, the Extra feat tables), and 5 real omissions (the new feat missing from `selections/feats.ron` and `gold-dragon/dragon-feats.ron`; `witch-wandering-hex`'s any-of and its planner selection missing the eleven copies and the channel). The section in `slices.md` named one test. All 15 were fixed in two minutes once seen. That log held no failure messages, since the run was cut before its summary.
- **Left for the reviewer** (none blocks the commit):
  - a Hexcrafter still takes `extra-hex` but gets no slot (the game refuses the feat: `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35 needs `WitchHexSelection` or `WinterWitchWitchHexSelection`; 264's `known.txt` line);
  - the corpus generators still build a shaman's and a Hexcrafter's Extra Hex with `extra-hex` (`tools/corpus/src/builds/feats.rs:47`, `talents.rs:798`), so a regenerated corpus has 2,155 Builds, not 2,159, and 3 that no longer open;
  - `witch/extra-hex` holds no list of its own (the writer skips a selection with none; the game's holds 27), which 306aa needs;
  - the Answer doesn't explain the census's `ok -> BreaksLater` groups (the 40 largest, in 24 to 28 Builds) or `witch-wandering-hex` going to Unmet in 21 Builds.
- **Changes:**
  - (applied) `ticket-queue` skill: keep the machine awake for the length of a queue (the app's keep-awake request, or `caffeinate -i` around it), and say so before dispatching on battery.
  - (applied) `subagent-rescue` skill, step 2: before judging a process hung, compare its elapsed time with the test's own "finished in" and read `pmset -g log` for `Sleep` lines since the builder started. When the machine slept, the verdict is "wait" and no rescue is needed.
  - (applied: `scripts/pinning-tests.sh`, a cause seen twice: a mechanical fix) a script, `scripts/pinning-tests.sh <key>…`, that lists the test functions in `libs/seeds/tests/game_data/` naming a selection, feat or option key the slice touches, or the totals it moves (the feat count, the repeating feats, the `Requires` count). The research puts its output in the section's **Tests**, and the builder runs those filtered before the one full run.
  - (queued on ticket 306ab) `game-check --write option-lists`: list a selection named in `--only` that it skipped for holding no list, rather than skipping it silently.
  - (proposed) the form proof's `compare`: when the regenerated corpus loses Builds or holds Builds that no longer open, name them and the generator's line, so the slice updates the generator.

## 2026-11-17: ticket 306k (the small splits: Aeon gaze, Elemental Witch, Nine-Tailed Heir, discoveries, angel), about 35 minutes at the check

- **Verdict:** progressing; the watchdog fired on a builder two steps from its commit. Nothing to unblock.
- **How the minutes split** (file times; the lane's base is 08:33, the Mac woke at 08:32 and didn't sleep after):
  - 08:33 to 08:38: the form proof's `record` on the base (49 s of run, the rest compile);
  - 08:38 to 08:53: the data edits and the writer (`lists.log`);
  - 08:53 to 09:00: tests and the game check (`gc.txt`);
  - 09:00 to 09:04: `record` on the tip, `compare`, the refused links listed;
  - 09:04 on: the one full `seeds`/`corpus` run (`links` 45 s; `game_data` about 3 minutes), then the Answer and the commit.

  All of it was work: no stall over a few minutes between writes, no sleep. Waiting was only the runs themselves, about 6 minutes of 35.
- **`pinning-tests.sh` and filtered runs did cut the time.** Ten test files under `libs/seeds/tests/game_data/` were edited before the first full run started at 31 minutes; 306i and 306j each found their pins (11 and 15 failures) from a full run and iterated from there. 306i took 65 minutes, 306j 33 of work with two full runs; 306k reached its first full run at 31. That run still showed one pin the script missed: `pick_order::the_selections_outside_the_features_band_are_the_games` lists every selection with `band: Path`, and the new `sorcerer/nine-tailed-heir/bloodline` (`band: Path`) is not in it. The script searches for the keys a slice touches; a test that lists all entries of a kind names no new key. One failure instead of 11 or 15, but it costs a second 4-minute run.
- **The estimate was the miss, not the builder:** `slices.md` says 20 minutes for five splits, a new feat, a new option, a census and a full run. The fixed costs alone (record, compare, the full run, the game check) are about 10 minutes.
- **Seen in the census, for the reviewer:**
  - 7 refused link lines where the section's Proof says 3 (2 Vivisectionist, 1 Underground Chemist): 6 Vivisectionist lines (t64 "discovery" and "talent only", sample and `full`) and 1 Underground Chemist (`full t49`). All are Extra Discovery, all listed in `306-refused.txt`; none is an Aeon link or a slot-order refusal (0 through an heir, 0 changed). The Answer should say why the count is 7.
  - the regenerated corpus has 2,156 Builds, not 2,158 (it lost `full t64 vivisectionist talent only` and its `/ filled`), and 1,990 open, not 1,994: the generator still gives a Vivisectionist `extra-discovery`. The same cause as 306j's shaman.
  - every offer status change (2,112 in 172 groups, all `-> BreaksLater`) sits in the 7 refused Builds; no other Build's offers changed status.
- **Changes:**
  - (declined: the user set the watchdog at 30 minutes; a medium-effort check is cheap) `slices.md` estimates for the remaining 306 slices: add 10 minutes of fixed cost per data slice, or move the watchdog to 45 minutes for a slice with a census, so it fires on a stall and not on a full run.
  - (applied: research/306/decisions.md; ticket 306af for compare) the corpus generator follows the split: a slice that gives a class its own Extra feat updates `tools/corpus/src/builds/` in the same diff, and `compare` names the Builds the regenerated corpus lost with the generator's line (306j's proposal, still open).
  - (applied: research/306/decisions.md) the research's Proof counts refusals in link lines (sample and `full`, each `/ filled` variant), the unit `306-refused.txt` and `compare` use, not in Builds.
  - (applied) `scripts/pinning-tests.sh`: also list the tests that enumerate every entry holding a field the slice's new entries set (`band: Path` → `pick_order::the_selections_outside_the_features_band_are_the_games`), from a small table of field → test in the script.

## 2026-11-17: ticket 306aa, fix round (an Extra feat's prerequisites are the game's), about 31 minutes at the check

- **Verdict:** slow but progressing. The builder started at 12:14:45; at 12:46 it was live (last transcript entry 12:46:14, no stall over 3 minutes), with no cargo process between its runs, no lock, and no sleep since 10:35. Load 6 and falling (it was 40 a few minutes before, with 308b compiling in `lane-game`).
- **How the minutes split** (file and log times):
  - 12:14 to 12:27: the edits, steps 1 to 4 (three feats' prerequisites, Evasion's gate, the generator's revert, four test files), and the first `compare` (`fix/compare.log`, 12:27:56);
  - 12:27 to 12:46: attributing `compare`'s offer differences. 6 `compare` runs (38 to 52 s each) and 24 runs of a throwaway `tools/corpus/tests/tmp_dbg.rs`; one `git stash` round trip at 12:39 to run `compare` on f336cb77 alone (`fix/compare-306aa.log`), popped (the stash list is empty);
  - not started: the scoped checks (only filtered `seeds` runs and `corpus --test links` so far) and the commit.
- **What the 19 minutes were about:** on the base's tip `compare` gives 43 differences, 671 offer differences, statuses changed in 16 Builds; with the fix it gives 53 differences, 9,020 offer differences, 8,782 statuses changed in 211 groups over 305 Builds, 5 links refused. The brief said "run `compare`" and gave no expected counts, so the builder set out to explain each group by hand. At 12:46 it had them down to the three feats' own rows (`extra-rogue-talent` in 208 Builds, `extra-hex` in 70, `extra-discovery` in 45, to and from `Unmet`) and `weapon-focus: ok -> BreaksLater` (1,326 in 5 Builds, the refused ones).
- **Seen in the diff, for the reviewer:**
  - `306-refused.txt` gains 15 lines where the brief says nine;
  - `tools/corpus/tests/tmp_dbg.rs` is untracked and must not reach the commit;
  - Extra Discovery's new prerequisite agrees with the game (ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66: one PrerequisiteFeature, DiscoverySelection cd86c437488386f438dcc9ae727ea2a6, group Any, and the rank cap). The other two weren't checked here.
  - `git stash` in a lane touches every changed file's time and forces a rebuild, and a failed pop would leave the fix out of the tree.
- **Changes:**
  - (proposed) a fix round's brief that changes prerequisites says what `compare` should show: "every status change is one of the three feats' own rows or sits in a Build the refused list names; put the counts in the Answer; anything else is a finding, at most 10 minutes on it". 306k's retro found the same shape by hand (every status change in the 7 refused Builds).
  - (proposed) `compare` prints that split itself: status changes whose option is a key the diff touched, those in Builds whose links are refused, and the rest. With it this step is one run, not 30. The same cause as 306k's census reading and as ticket 306af (`compare` names what the corpus lost): second entry, so it gets the mechanical fix; add it to 306af.
  - (proposed) `.claude/agents/ticket-builder.md`: no `git stash` in a lane; a debug test goes in the scratchpad as its own crate or is deleted before the checks, and `git status` shows no untracked file at the commit.

## 2026-11-17: ticket 306s (the general and group feat lists), about 31 minutes at the check

- **Verdict:** slow but progressing. The builder started at 13:49; at 14:20 its first full `cargo test -p seeds --test game_data` was 4 minutes in at 600% CPU (611 of 695 tests done, 8 failed), with a transcript entry every few seconds, no lock, and no sleep since 10:35. Load 40 to 48 from that run itself (no cargo or rustc ran under `lane-tests` at the check).
- **How the minutes split** (file times in the scratchpad's `306s/`):
  - 13:49 to 13:50: `record` with offers on the base (42 s; 2,156 Builds, 1,990 open);
  - 13:50 to 14:01: the game check before, the writer (`306s.tsv`), and reading `ExtraFeatMythicFeat` against `BasicFeatSelection` (`basic.txt`, `extra.txt`);
  - 14:01 to 14:13: the refused links: `bench-links.tsv` and `fixture-base.tsv` regenerated (the `fighter20` bench links were refused), `306-refused.txt` +73 lines, `corpus --test links` green (6 passed, 42 s);
  - 14:16 on: the full `game_data` run;
  - not started: `compare`, the scoped checks, the commit.
- **What made it slow: the slice was larger than its edit list.** `slices.md` says "run `--only` the five, `known.txt`, un-ignore two tests, 10 min". Un-ignoring `extra_feat_offers_the_general_slots_feats_but_the_tricksters_five` needed what the list doesn't name: a new selection `selections/mythic/extra-feat.ron` (the game's Extra Feat list, 274 feats against the general slot's 278), `mythic/feats/extra-feat.ron` naming it, and a `rules` change (`check.rs`: a mythic feat's selection counts as naming a feat list). That brings `cargo test -p rules` and the wasm build into the scoped checks. `selections/feats.ron` itself has no diff, where the Proof counts 17 for it.
- **The pins came from the full run again:** 8 failures at 611 of 695 (`bonus_feat_choices::the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites`, `data::every_feat_list_holds_two_feats_or_more_or_every_feat`, `first_ascension::every_older_link_decodes_to_the_same_build_and_takes_its_mr_3_paths_first_ascension`, `mythic::extra_feat_and_extra_mythic_ability_are_the_mythic_feats_that_give_a_pick`, `mythic::extra_feat_at_mr_2_gives_a_feat_slot_at_level_7_offering_what_the_general_slot_there_does`, `nested_selections::extra_mythic_ability_and_extra_feat_grant_their_selections`, `nested_selections::the_category_selections_list_the_feats_of_their_category`, `repeats_from_the_header::extra_feat_mythic_is_taken_twice`). `pinning-tests.sh` was run (it is in the transcript), but five of the eight pin Extra Feat's selection, a key the slice's edit list doesn't touch, so the script wasn't asked about it. Third entry with this cause (306i, 306j, 306k).
- **Seen in the diff, for the reviewer:**
  - `306-refused.txt` gains 73 lines where the Proof says 70 open Builds (link lines against Builds, 306k's unit note);
  - `known.txt` keeps `mythic/extra-mythic-ability` as `in game only -> 306v` (SecondMysteryEnlightenedPhilosopher), so option lists fall by less than the Proof's 66; the Answer should give the count;
  - `first_ascension::every_older_link_decodes…` failing may mean an older link's Extra Feat pick no longer decodes: an alias or a refusal to list, not a pin to rewrite.
- **Changes:**
  - (proposed) `slices.md`: a slice that un-ignores a test lists what the test needs to pass, found by running it un-ignored on the base during research; 306s's line should have named the new `mythic/extra-feat` selection and the `check.rs` change, and estimated 30 minutes.
  - (proposed) `scripts/pinning-tests.sh` takes the working tree's diff (`git diff --name-only` plus untracked `.ron`) when given no keys, so keys the builder adds beyond the edit list are searched too. Same cause as 306i, 306j and 306k: third entry, so the mechanical fix.
  - (proposed) `.claude/agents/ticket-builder.md`: after a full run fails, iterate with a filter of the failing names (`cargo test -p seeds --test game_data -- <name> <name>`), and run the full file once more only before the commit: a full `game_data` run is 4 to 5 minutes at load 40.

## 2026-11-17: ticket 308c (the feats' prerequisites as the game's), about 31 minutes at the check

- **Verdict:** on track. The builder started at about 15:18 (base 73bafe1d merged 15:18:00, `record.log` 15:19:52); at 15:49 it was live (a transcript entry every few seconds, last 15:49:19), with no cargo or rustc process, no lock, and no sleep since 10:35. The slice's own estimate is 75 minutes. Load 45, from the full run that ended at 15:48 and `spotlightknowledged.updater` at 120% CPU.
- **How the minutes split** (file times in the scratchpad's `308c/`):
  - 15:19: `record` with offers on the base (38.6 s);
  - 15:20 to 15:28: the dry run, the write (`write.txt`, `log.tsv` 15:21), the game check twice (`check1.txt`, `check2.txt`);
  - 15:28 to 15:44: the pins, the census and the refused links: `compare.txt` 15:34, `306-refused.txt` +77 lines, a `wfprobe` crate in the scratchpad at 15:38, `compare2.txt` 15:44;
  - 15:45 to 15:48: the one full `seeds` + `corpus` run (`full.txt`): `game_data` 598 passed, 1 failed, 176.6 s; `corpus --test links` 6 passed;
  - in progress: the one failure, `option_lists::extra_feats_name_every_option_of_their_selections` (the pinned set lacks `extra-arcana`, `extra-arcanist-exploit`, `extra-hex-shaman`, `extra-mercy`, `extra-rage-power`); `option_lists.rs` is already in the diff;
  - not started: the last `compare`, fmt, `check-data`, the commit.
- **The background command** it said it waited on was that full run; it ended on its own in about 3 minutes. Not hung, and no loop: one full run, one failure.
- **Seen in the diff, for the reviewer** (83 files changed, 2 untracked: the data note and `feat_prerequisites.rs`, declared in `main.rs`):
  - `compare2.txt`: census 0 unattributed (4,020 lines in 175 Builds), 93 links refused, 200,273 statuses changed in 875 groups over 2,154 Builds. The slice expects four refusal patterns; the Answer must list the 93 by Build;
  - `compare` still fails on 2 differences: the generator in `tools/corpus/src/builds/talents.rs:420` was renamed ("t57 exploiter extra exploit with no exploit" to "t57 wizard extra exploit with no exploit selection"), so one Build reads as lost and one as new. The count is held, the name isn't;
  - `prerequisite-narrowing.txt` loses 91 lines, `known.txt` 145 changed;
  - `spell-chain` gains `Has(Feat("spell-penetration"))` (`ok -> Unmet` in 2,153 Builds); not checked against the blueprint here.
- **Changes:**
  - (proposed) the watchdog compares the elapsed time with the slice's estimate in `slices.md` and fires at the estimate, not at a flat 30 minutes: this rescue found a 75-minute slice on schedule at minute 31.
  - (proposed) `pin-feats.txt` missed `option_lists::extra_feats_name_every_option_of_their_selections`, which enumerates every Extra feat whose prerequisite names no option. Same cause as 306i, 306j, 306k and 306s (a pin found by the full run): fourth entry; the mechanical fix proposed there (`pinning-tests.sh` reads the working tree's diff, with the field → test table) covers it.
  - (proposed) `compare` matches a renamed Build by its link when the name differs, or 306's decisions say a generator fixed toward the game keeps its name, so a rename doesn't read as a lost Build. Related to 306af (`compare` names what the corpus lost).

## 2026-11-17: ticket 308d (the mythic abilities', classes' and archetypes' prerequisites as the game's), about 31 minutes at the check

- **Verdict:** slow but progressing, inside its estimate (50 minutes in `slices.md`). The builder started at about 16:13 (base a165d7a2 merged 16:12:39, `record.log` 16:13:59); at 16:43 its one full `cargo test -p seeds -p corpus` was 2 minutes in, `game_data` at 690% CPU with 667 tests done, a transcript entry at 16:41:41, no lock, no sleep since 10:35. Load 13, from that run alone.
- **How the minutes split** (file times in the scratchpad's `308d/`):
  - 16:13: `record` with offers on the base;
  - 16:16 to 16:21: the dry run, the write (`log.tsv` 16:18), the game check twice;
  - 16:23 to 16:31: the pins by filtered runs (`t1.txt` to `t3.txt`), blueprints read (`show.txt`), a second write (`log2.tsv`);
  - 16:32 to 16:37: `compare` three times, the last still failing (`compare3.txt`: statuses changed in 1,378 Builds, `left: 138587, right: 0`);
  - 16:37 to 16:40: the refused links (`306-refused.txt` +51, `corpus --test links` green);
  - 16:41 on: fmt, then the full run: the corpus tests green, `game_data` with 3 failures so far (`first_ascension::every_older_link_decodes_to_the_same_build_and_takes_its_mr_3_paths_first_ascension`, `negated_prerequisites::every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out`, `ranks_and_pools::links_made_before_the_splits_open_with_the_same_picks`);
  - not started: the last `compare` after those fixes, `check-data`, `cargo test -p rules` and the wasm build, the commit.
- **The stop condition didn't fire:** all seven `gold::` tests pass, `gold_builds_gain_what_the_write_ups_list` included; `defensive-study` goes `BreaksLater -> Unmet` in 25 corpus Builds only.
- **Seen in the diff, for the reviewer** (40 files changed, 2 untracked: the data note and `mythic_class_prerequisites.rs`, declared in `main.rs`):
  - a `rules` change the slice doesn't name: `Repeat::derive` in `libs/rules/src/parameter.rs` reads a `Not` of an `Any` naming the entry as `Once`, with a test in `libs/rules/tests/engine/repeats.rs`;
  - of the slice's 26 entries, `abundant-smite-smite-chaos`, `greater-enduring-spells`, `mantis-zealot` and `hag-of-gyronna` have no diff (the note leaves the last two with 267; Greater Enduring Spells is ticket 208's, staged in the main checkout); `weapon-focus-mythic` has one though the slice re-homes the mythic feats: the Answer should say why for each;
  - two of the three failures are older links no longer opening the same: a refusal to list or an alias, not a pin to rewrite (306s's note).
- **Changes:**
  - (proposed) the watchdog fires at the slice's estimate, not at a flat 30 minutes: second entry with this cause (308c), so the mechanical fix: the `ticket-queue` skill's watchdog reads the minutes from the slice's heading.
  - (proposed) `pinning-tests.sh` reads the working tree's diff: the three failures came from the full run again, though the filtered runs before it kept that run to one. Sixth entry with this cause (306i, 306j, 306k, 306s, 308c).
  - (proposed) `slices.md`: a slice whose written prerequisite takes a new shape (`Not` of an `Any`) names the `rules` reader it needs, found by `check-data` on a dry run during research; 308d's edit list has no `rules` line.

## 2026-11-17: ticket 308e (the rogue, slayer, alchemist, magus and arcanist options' prerequisites as the game's), about 31 minutes at the check

- **Verdict:** on track, inside its estimate (70 minutes in `slices.md`). The builder started at about 17:53 (`record-before.log` 17:54:59); at 18:24 it was live (a transcript entry every few seconds, last 18:24:33; scratchpad writes at 18:22:50), with no cargo or rustc process, no lock, and no sleep since 10:35. Load 2.5.
- **How the minutes split** (file times in the scratchpad's `308e/`):
  - 17:54: `record` with offers on the base;
  - 17:57 to 18:06: the dry run, the write (`write.txt`, `308e.tsv` 18:03), the game check twice, the two Uncanny Dodge entries by hand (18:06);
  - 18:07 to 18:17: the un-ignored tests (`ign1.txt`: 4 failed) and the pins by filtered runs: `t1.txt` 18:10, 7 failed of 108; `t2.txt` 18:17, 110 passed, 0 failed; the new test file `rogue_slayer_alchemist_prerequisites.rs`, declared in `main.rs`;
  - 18:21 to 18:24: `compare` (`compare.txt`: census 0 unattributed, 1,803 lines in 87 Builds; 77 links refused; statuses changed in 388 Builds), then the refused links (`new-refused.txt`, `listed.txt`) and the `BreaksLater` rows;
  - not started: the one full `seeds` + `corpus` run, the last `compare`, `check-data`, fmt, the data note (none in `libs/game-data/notes/` yet), the commit.
- **Seen in the diff, for the reviewer** (63 files changed, 1 untracked):
  - `t1.txt` had the Enforcer impossible at all three starts (`gold_builds_are_possible`, `gold_builds_gain_what_the_write_ups_list`, `the_enforcers_mythic_shield_focus_needs_mr_10_taken_at_level_20`), a gold change beyond the expected one. The builder didn't stop: at 18:12 it ran `git checkout --` on `ranger/combat-style-feats-10/` (the writer's five entries, each an `Any` of the 6th-level style option and two ranger selections) and re-homed them in `known.txt` under 293 (lines 68 to 73). Gold is green since. The Answer must say this, and the main session decides whether it was a stop;
  - `gold.rs` drops the `Why::WriteUpMistaken` variant with the 173 entry and picks Uncanny Dodge at level 8, which the slice puts "after the user sees it";
  - `compare` still fails on a lost and a new Build: `full targeted ranger10-slayer2` is gone and `full t60 sanctified 8 combat style` is new, though no file under `tools/corpus/` is in the diff: the names come from the data;
  - `offers`: `ok -> BreaksLater` on options outside the slice (witch hexes in 22 Builds, `defensive-study` in 15, the aeon's Enforcing Gaze in 11): to explain in the Answer by the Builds' refused later picks.
- **Changes:**
  - (proposed) the watchdog fires at the slice's estimate: third entry with this cause (308c, 308d). The mechanical fix proposed there isn't applied yet; this rescue again found an on-schedule builder.
  - (proposed) `.claude/agents/ticket-builder.md`: a gold Build that turns impossible is a gold change and a stop, even when a revert of the written entries clears it; the builder reports the entry and the Build before it re-homes them.
  - (proposed) `slices.md` for 308e names the combat-style lists a slayer reaches through `combat-style-feat-10`: the writer's ranger-only `Any` closes them to the slayer, which a dry run with `gold::` would have shown at research.
