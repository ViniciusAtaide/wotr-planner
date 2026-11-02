# Keep a switch's earlier picks

Type: task
Status: resolved
Blocked by: 41
Map: [WotR Build Planner](../map.md)

## Question

Undo half of [ticket 41](41-drop-what-a-switch-takes-away.md) (user, 2026-11-02). The picks a Path switch takes away at earlier, reached ranks (Legend's MR 3–7, the old path's MR 3–7 on Gold Dragon and the Swarm, the Aeon's gaze on Devil) stay in the Build again. They were held from their rank until the switch, so they count for everything before it, and the Mythic track shows them as lost from the switch on, as [ticket 26](26-mythic-path-rules.md) had it. In particular, ticket 26's guide rule holds again: a Trickster who becomes a Legend keeps the feats Perception II unlocked.

Keep the rest of ticket 41 and all of ticket 40:
- the path goes once MR 3 isn't reached and the switch once MR 8 isn't, after any edit (`Engine::apply`), and `is_possible` refuses a path before MR 3 or a switch before MR 8;
- Remove level and Legend's switch drop the picks of ranks no longer reached (MR 8–10);
- a stale link gets the one generic error.

## Build

AFK, test-first (`tdd`), in `rules`; no schema, game-data, link-format or `Build` change expected. Only agent in the repo.

- Restore, for the earlier-rank picks only, the behaviour at 5e7e106 (ticket 40): kept in the Build, held until the switch, then released by the switch with what they granted; `dropped_slots` and `RankStatus` as they were; an empty slot at an earlier rank still has a picker (it's filled before the switch). Bring back what ticket 41 removed only to support dropping them (`GrantSource.slot`/`selection`, the switch's release of holdings from taken-away picks, …) where it's needed again; keep ticket 41's other simplifications (`slot_level`) if they still fit.
- `Engine::apply` no longer drops them; `Walker::fill` no longer fails them; `switch_takes` goes or shrinks to what the track still needs. A link holding such a pick is possible again, not stale.
- The switch lock (`BreaksLater`): back to ticket 40's behaviour for these picks. Legend is no longer locked for the Perception II Trickster.
- Tests: restore ticket 41's three removed tests (`path_rules::a_trickster_who_becomes_a_legend_keeps_the_feats_perception_ii_unlocked`, `path_rules::a_switch_dropping_an_option_drops_its_feats_but_not_the_same_feats_held_otherwise`, `parameters::a_switch_that_drops_an_every_value_grant_keeps_a_value_a_pick_holds`) and bring the rewritten ones back to ticket 40's expectations for earlier-rank picks (`git show 5e7e106:<path>` has them); drop or invert `stale_links::a_link_holding_a_pick_a_switch_takes_away_is_stale` and the Gold Dragon/Swarm "drop the old path's picks" test. Keep ticket 41's tests for the unreached path and switch.
- **Docs:** `GLOSSARY.md` (Path switch, Mythic track), ticket 12's Mythic track and picker lines, the map's notes on tickets 26, 40 and 41, and a "Decisions so far" line for ticket 42.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus: the Perception II Trickster switches to Legend and keeps its feats; Legend, Gold Dragon, Swarm and Devil keep their earlier-rank picks in the Build, marked lost from the switch on; the unreached path/switch tests of ticket 41 still pass; the proof (ticket 41's Build sets) shows that, against 5e7e106, the only differences left are ticket 41's path/switch drops.

## Progress log

- 2026-11-02: started. Baseline: HEAD 75d8135, clean tree; scratch in `t42/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t42/`). Read `git diff 5e7e106 98c66d7 -- rules/`: ticket 41's two halves touch different code. The path/switch half is `Engine::apply`'s two `reaches` checks, `Engine::reaches` and `check_unreached`'s rank checks. The rest (`switch_takes`, the walk failing a taken-away pick, rule-based `dropped_slots`, `options` returning `None`, `Recheck` walking from the first dropped pick, the removed `GrantSource` fields and holdings release) exists only for the earlier-rank picks.
- 2026-11-02: tests first. `path_rules.rs` (2 restored, 4 back to ticket 40's expectations, 1 inverted, 1 rewritten), `parameters.rs` (1 restored), `option_levels.rs` and `mythic.rs` (1 each back to ticket 40), `stale_links.rs` (1 inverted), `data/tests/game_data.rs` (1 back to ticket 40). Red run: `t42/red-run.txt` (12 rules failures + 1 data failure, each on the restored behaviour).
- 2026-11-02: built. `engine.rs` is 5e7e106's again plus ticket 41's path/switch half; `build.rs`'s `Edit::PathSwitch` doc is ticket 40's, `RemoveLevel`'s keeps ticket 41's. Green: rules 296 + unit 2, data `game_data` 119.
- 2026-11-02: proof, docs, Verification. The proof passes on all three sets (below). Docs and Verification below.

## Answer

Done 2026-11-02, uncommitted. The picks a Path switch takes away at earlier, reached ranks stay in the Build again. They're held from their rank until the switch and lost from it on, and the Mythic track marks them as ticket 40 did. A Trickster who becomes a Legend keeps the feats Perception II unlocked. Ticket 41's other half stays: `Engine::apply` drops the path once MR 3 isn't reached and the switch once MR 8 isn't, and `is_possible` refuses either below its rank. Ticket 40's MR 8–10 drops and the one stale-link error are unchanged. No schema, game-data, link-format or `Build` change; `V1_LINK` stays.

### What changed

- **`rules/src/engine.rs`** is ticket 40's (5e7e106) plus ticket 41's path/switch half:
  - `Engine::apply` drops the path and switch below their ranks, then the picks of unreached ranks, as ticket 40 did. It no longer drops what a switch takes away.
  - `Engine::reaches` and `check_unreached`'s rank checks stay from ticket 41.
- **Back from ticket 40:**
  - `GrantSource.slot` and `.selection`, and `switch_drops`' check of the selections a switch drops by name.
  - The switch's release of the holdings a taken-away pick made (the option, its option level and feat list).
  - `RankSheet.dropped_slots` comes from the held picks again. A lost slot holds its pick; an empty one isn't listed.
  - The rank status: Devil's empty-gaze ranks are `Reached` again.
  - `Walker::fill` holds such a pick until the switch and doesn't fail it, so a link holding one is possible, not stale.
  - `Engine::options` gives a picker on an empty earlier-rank slot: it's filled before the switch.
  - `Recheck` walks from the edit's own first level, so the switch lock (`BreaksLater`) is ticket 40's. A switch locks only when a choice from level 17 on needs what it takes away. Legend is open for the Perception II Trickster.
- **Gone:**
  - `Engine::switch_takes`. With the picks kept, nothing needs it.
  - `Engine::shows_pick` (ticket 40's `retain` again).
  - `Engine::slot_level`. Its other caller was the `Recheck` fold, so it went back inline into `first_level`.
- **`rules/src/build.rs`:** `Edit::PathSwitch`'s doc is ticket 40's. `Edit::RemoveLevel`'s and `Build::apply`'s mention the path and switch drop only.

### Tests

- **Restored as at 5e7e106:**
  - `path_rules::a_trickster_who_becomes_a_legend_keeps_the_feats_perception_ii_unlocked` (fixture still switches through `Engine::apply`);
  - `path_rules::a_switch_dropping_an_option_drops_its_feats_but_not_the_same_feats_held_otherwise`;
  - `parameters::a_switch_that_drops_an_every_value_grant_keeps_a_value_a_pick_holds`.
- **Back to ticket 40's expectations:**
  - `option_levels::a_path_switch_takes_away_a_general_picks_selection_with_the_option_it_took` is `a_path_switch_drops_an_option_a_dropped_pick_took` again, as at 5e7e106.
  - `mythic::a_legend_switch_at_mr_8_keeps_mr_1_and_2_and_stays_at_rank_3`, as at 5e7e106.
  - `data/tests/game_data.rs::a_devil_from_aeon_loses_the_gaze_picks_and_keeps_the_rest`, as at 5e7e106 plus ticket 41's check that MR 3's gaze slot is listed dropped.
  - `path_rules`:
    - `gold_dragon_keeps_the_general_picks_of_mr_1_to_7_but_the_old_paths_grants_of_mr_1_and_2_only`: the tricks and bonus feat are held at 16. The picked slots of MR 3–5 are listed dropped and keep their picks. MR 7's empty trick slot isn't listed and has a picker.
    - `a_legend_switch_keeps_the_old_paths_mr_1_and_2_rows_and_drops_every_pick_of_mr_3_to_7`: MR 1–7's picks stay and are held at 16. MR 3–7 are `Dropped`. A slot there is listed dropped exactly when it holds a pick.
    - `choosing_the_legend_switch_drops_the_picks_of_the_ranks_it_takes_away`: Gold Dragon keeps every pick, and Legend drops MR 8–10's only. Taking the switch back leaves MR 1–7.
    - `devil_from_aeon_keeps_mr_1_to_7_but_loses_the_gaze_picks_and_extra_gaze`: the gaze picks stay and are held at 16. MR 3–6 are `PartlyDropped` with their gaze listed, and MR 7 (empty gaze) is `Reached` with a picker.
  - These keep ticket 41's switching through `Engine::apply`, as do the Legend fixtures and `mythic::a_switch_that_keeps_ascending_takes_over_from_mr_8`.
- **Inverted:**
  - `path_rules::gold_dragon_and_the_swarm_drop_the_old_paths_picks_of_mr_3_to_7_and_keep_the_general_ones` became `gold_dragon_and_the_swarm_keep_the_old_paths_picks_of_mr_3_to_7_in_the_build`: the switch is open, applying it leaves the picks unchanged, and the result is possible.
  - `stale_links::a_link_holding_a_pick_a_switch_takes_away_is_stale` became `a_link_holding_a_pick_a_switch_takes_away_loads_unchanged`. It uses the same three cases (Legend with MR 3's general pick, Gold Dragon with the MR 3 trick, Devil with a gaze). Each is possible, opens unchanged, and is what `Engine::apply` makes from the Build without the switch.
- **Rewritten:** `path_rules::a_switch_is_locked_only_while_a_later_choice_needs_a_pick_it_takes_away` became `a_switch_is_locked_only_while_a_choice_from_its_rank_on_needs_a_pick_it_takes_away`.
  - Stat Focus at 15 (needing Perception II) leaves Legend, Gold Dragon and the Swarm open. Each switch keeps it and stays possible.
  - Stat Focus at 19 locks all three (`BreaksLater`).
  - A held switch stays removable.
- **Kept from ticket 41:**
  - `path_rules::removing_the_level_of_mr_3_or_mr_8_drops_the_path_or_the_switch` and `path_rules::a_path_or_switch_whose_rank_is_not_reached_is_not_kept`;
  - `stale_links`' path and switch on a 17-level Build, and its path before MR 3 and switch before MR 8.
- **Removed:** none.
- **Counts:** rules 293 + unit 2 → 296 + unit 2; data `game_data` 119 → 119.

### Proof

`t42/proof-41/` builds ticket 41's engine (`git archive 98c66d7` in `t42/src-41/`). `t42/proof-42/` builds the working tree. Ticket 40's engine is `t41/proof-old/`. All three run the same source as ticket 41's proof (`t41/side.rs` + `t41/proof-common.rs`). Their output on ticket 41's sets reproduces `t41/`'s byte for byte.

- **Three Build sets:** ticket 40's 445 (`builds.txt`), ticket 41's 614 switched (`switched.txt`), and a third 614 (`switched-42.txt`), each Build switched by the working tree. Ticket 41's switched set holds no taken-away pick (ticket 41's side dropped them). In the third set, 29 Builds hold one.
- **`t42/compare42.py`** writes ticket 26's switch rules out independently of the engine, and allows only what follows.
  - **Against 40:**
    - A Build may become impossible only by a path below MR 3 or a switch below MR 8.
    - Every line of a Build possible on both sides is identical, but the edits'.
    - An edit may differ only by the path dropped below level 9 or the switch below level 17, and its result's possibility only then. Its picks are identical.
  - **Against 41:**
    - A Build may become possible only by holding a pick its switch takes away.
    - On Builds possible on both sides, a rank's dropped slots may lose only empty slots its switch takes away, whose pickers come back. Its status may go `PartlyDropped` → `Reached` only then.
    - A switch picker may go breaks → open only when the Build holds a pick that switch takes away.
    - An edit may differ only by keeping picks the resulting switch takes away.
- **Result:** all six comparisons pass (`t42/proof.txt`).
  - **Against 40:**
    - On `builds.txt`, 371 Builds are possible on both sides. Removability (9258), option statuses, class pickers, path and switch pickers and rank lines are byte-identical; 147 switch edits on Builds below 17 become possible results.
    - On `switched.txt` and `switched-42.txt` (all 614 possible on both sides each), everything but the edits is byte-identical.
    - The edits differ only by ticket 41's path/switch drops: Remove level drops the path 261 times and the switch 85 times on `builds.txt` (10438 times on each switched set), and switch edits drop the switch 588 times on `builds.txt`.
  - **Against 41:**
    - On `builds.txt`, the 5 Builds ticket 41 made stale (`filled {alchemist,cavalier,inquisitor,skald,wizard} 40` under Legend) are possible again. On the other 366, everything is identical but the switch edits, which keep 499 picks ticket 41 dropped.
    - On `switched.txt`, 1261 empty slots are no longer marked dropped, and their 68914 picker options are back (33011 open, 35905 locked, 4 breaks; the 4 are All-skilled at an Azata's MR 4 under Legend, as ticket 40 had it).
    - On `switched-42.txt`, the 29 Builds holding taken-away picks are possible again, and on the other 585, 975 empty slots are unmarked.

### Docs

- `GLOSSARY.md`:
  - **Path switch:** the picks it takes away at earlier ranks stay in the Build, held until the switch (the Perception II example). The switch is locked while a choice from its rank on needs one of them.
  - **Mythic track:** the lost picks are marked from the switch on, and an empty slot there can still be filled. Remove level still drops the path or switch with its rank.
- Ticket 12: the Mythic track tab line (lost picks marked and kept, empty slot has a picker) and the picker line (`Engine::apply` drops the path and switch below their ranks; earlier-rank picks stay).
- The map: the Order note, notes on ticket 26's, 40's and 41's lines, and a "Decisions so far" line for ticket 42.
- Ticket 40's Answer already marks its decision 2 as reversed (planning commit). Ticket 41's Answer is left as written, with its "Decision on the open question".

### Decided

1. **`Engine::slot_level` is gone.** Its second caller was `Recheck`'s fold over the picks `apply` dropped before the edit's level. That fold only mattered for the taken-away picks, so `Recheck` is ticket 40's again and `first_level` holds the match inline as it did. The ticket allowed keeping it only "if it still fits".
2. **The empty earlier-rank slot's picker gives ticket 40's statuses,** whatever they are. On ticket 41's switched set, 4 of 68914 options are `BreaksLater`: All-skilled at an Azata's MR 4 under Legend. The proof shows they're byte-identical to ticket 40's.

### Verification

- `cargo test --workspace`: pass (rules 296 + unit 2, extract 88, data files 25 + game_data 119 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 39 backgrounds).
- No schema, game-data or `Build` change, so no DB reset; `V1_LINK` stays and still decodes.
- The full output is in `t42/verification.txt`.
