# Drop what a switch or a removed level takes away

Type: task
Status: resolved
Blocked by: 40
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 40](40-stale-links-generic-error.md)'s two decisions (from the user's words, 2026-11-01: "I don't care to store what feats you chose before selecting Legend"; the Build holds only what the sheet shows):

1. **Remove level drops the path and the Path switch** once their rank (MR 3, MR 8) isn't reached any more, as it already drops those ranks' picks.
2. **A Path switch drops the picks it takes away at earlier, reached ranks:** Legend's MR 3–7 general and path picks, Gold Dragon's and Swarm's old-path picks of MR 3–7, Devil's gaze picks, and anything else ticket 26's `kept_through_rank` / `path_kept_through_rank` / dropped grants cover. They leave the `Build`; the Mythic track still shows those ranks as dropped or partly dropped, with the lost slots empty. Undoing the switch doesn't bring them back.

## Build

AFK, test-first (`tdd`), in `rules`; no schema, game-data or link-format change expected. Only agent in the repo.

- Both go through `Engine::apply` (ticket 40), next to its dropping of unreached ranks' picks.
- With these picks gone from the Build, simplify whatever only existed to carry them: for example whether `dropped_slots` can now be read straight from the rank rules rather than from held picks, and whether `Engine::is_possible` should now refuse a Build that holds a pick a switch takes away (a link can't carry one any more). Keep what the sheet needs to draw a dropped rank.
- Removability: the switch's lock (`BreaksLater`) should now only fire when a later choice needs a pick the switch drops, as ticket 40 did for MR 8–10.
- **Docs:** ticket 12's Mythic track line if it mentions dropped picks being kept, `GLOSSARY.md` (Mythic track, Path switch), the map's line for ticket 26 if it says the picks are kept, and a map "Decisions so far" line for ticket 41.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: removing level 9 drops the path (and level 17 the switch); Legend's switch drops MR 3–7's general and path picks and the track still marks those ranks dropped; Gold Dragon/Swarm drop the old path's MR 3–7 picks and keep the general ones; Devil drops the gaze picks; a link holding a pick a switch takes away is a stale link.

## Progress log

- 2026-11-01: started. Baseline: HEAD 5e7e106, clean tree; scratch in `t41/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t41/`), `old-src/` is `git archive HEAD`. Read the switch handling: `RankSheet.dropped_slots` came from the held picks' `GrantSource.slot`, so once the picks leave the Build it would always be empty; it has to come from the rank rules. The walk held a taken-away pick until MR 8 and then released it with its grants (`GrantSource.selection` for named selections, the holdings dropped with their pick); with no such pick left in a possible Build, none of that is needed.
- 2026-11-01: tests first. `path_rules.rs` (4 new, 4 rewritten, 1 removed), `stale_links.rs` (1 new, 2 extended), `mythic.rs` (1 rewritten), `option_levels.rs` (1 rewritten). Red run: `t41/red-run.txt` (12 failures, each on the new behaviour).
- 2026-11-01: built. `Engine::apply` drops the path and switch below their ranks and the picks a switch takes away (`Engine::switch_takes`, by the rank rules); the walk fails such a pick where it's filled and never holds it; `dropped_slots` and the rank status come from the rules; `check_unreached` fails a path or switch whose rank isn't reached; `Recheck` walks from the first level `apply` changed. Two tests that only checked the walk releasing a taken-away pick's grants removed. Green: rules 293 + unit 2.
- 2026-11-01: proof, docs, Verification. The proof found one more difference: a slot the switch takes away still offered a picker whose picks `apply` discards, so `Engine::options` is `None` there now (asserted in `path_rules.rs`, added with the change, not in the red run). The real-data Devil test (`data/tests/game_data.rs`) failed in the workspace run and was rewritten. Proof passes on both sets; docs and Verification below.

## Answer

Done 2026-11-01, uncommitted. The Build no longer holds what a switch takes away or what Remove level leaves unreached. `Engine::apply` drops the path and the switch once their ranks (MR 3, MR 8) aren't reached, and the picks a switch takes away at earlier ranks. The walk never holds such a pick and fails it, so a link holding one is a stale link. The Mythic track marks the lost slots from the path rules, empty and with no picker. No schema, game-data, link-format or `Build` change.

### What the sheet drops now

- **Remove level** drops the path with level 9 (MR 3) and the switch with level 17 (MR 8), as it already dropped those ranks' picks. Adding the level back brings neither back.
- **The rule is general:** after any edit, `apply` drops a path or switch whose rank the Build doesn't reach. So a path or switch set before its rank leaves the Build as it was. The sheet doesn't offer one there, and without this the pickers would allow an edit that makes the Build impossible.
- **A Path switch** drops the picks it takes away at MR 1-7, by `Engine::switch_takes`. It reads the rank rules only, never the held picks:
  - the general pick (and the selections its mythic ability grants) past `kept_through_rank`;
  - the old path's feat slots and selections past `path_kept_through_rank`;
  - a selection the switch drops by name, at any rank.
- **Per path,** that is:
  - Legend drops all of MR 3-7's picks, besides MR 8-10's (ticket 40);
  - Gold Dragon and the Swarm drop the old path's MR 3-7 picks and keep the general ones;
  - Devil drops the Aeon's gaze picks.
- **Undoing:** taking the switch back doesn't bring the picks back.
- **Every level, not only from MR 8.** The picks are gone from the Build, so the sheet doesn't hold them at levels 9-16 either.

### The walk, the track and the pickers

- **`Walker::fill`** fails a pick the switch takes away where it's filled, and doesn't hold it. `Engine::is_possible` is false for such a Build, and `Engine::open_link` gives `StaleLink`.
- **`check_unreached`** now also fails a path whose MR 3 isn't reached and a switch whose MR 8 isn't reached. This reverses ticket 40's "checked whether reached or not". An unknown key there still fails too.
- **`RankSheet.dropped_slots`** lists the slots `switch_takes` names, picked or not, so with the picks gone they're always empty.
  - The status uses them too: a rank with a lost slot is partly dropped at least.
  - This changes two sheets of Builds that held nothing there: an empty slot of the old path at MR 3-7 under Gold Dragon or the Swarm is now listed, and under Devil from Aeon each rank with an empty gaze slot (MR 7 in the tests) is `PartlyDropped`, not `Reached`.
  - The statuses of other ranks don't change.
- **`Engine::options`** is `None` for a slot the switch takes away: there is nothing to pick there.
- **The switch's lock:** `Recheck` applies the switch through `Engine::apply`, then walks from the first level whose picks `apply` dropped, not from MR 8. So the switch picker locks a switch (`BreaksLater`) only when a later choice needs a pick it takes away, at any level.

### Simplified or removed

- **`GrantSource.slot`** is gone. Only the pick-based `dropped_slots` used it.
- **`GrantSource.selection`** is gone, and with it the `dropped_selections` check in `switch_drops`. A pick in a selection the switch drops is never held now; `switch_takes` covers those slots.
- **The holdings a switch dropped** (an option a taken-away pick took, its option level and feat list) are gone from `switch_path`, since no such holding is ever made.
- **`switch_drops` is smaller:** it now drops only grants that don't come from a pick: the old path's rows past the cut-off, features dropped by name, and what those rows grant.
- **`Engine::first_level`'s slot case** moved to `Engine::slot_level`, which `Recheck` also uses.

### What stays

`RankStatus::{PartlyDropped, Dropped}`, `RankSheet.dropped_slots` and the walk's release of dropped row grants (`rank_grants`, `release`) stay. The track still has to draw a dropped rank, and the old path's rows (Trickster Tricks, Extra Gaze) are still held until the switch.

### Tests

- **Removed:**
  - `path_rules::a_trickster_who_becomes_a_legend_keeps_the_feats_perception_ii_unlocked`. A feat that needs Perception II now locks the switch; see Questions.
  - `path_rules::a_switch_dropping_an_option_drops_its_feats_but_not_the_same_feats_held_otherwise` and `parameters::a_switch_that_drops_an_every_value_grant_keeps_a_value_a_pick_holds`. Both checked the walk releasing a taken-away pick's grants, which no longer happens.
- **Rewritten:**
  - `path_rules`:
    - `gold_dragon_keeps_the_general_picks_of_mr_1_to_7_but_the_old_paths_grants_of_mr_1_and_2_only` and `a_legend_switch_keeps_the_old_paths_mr_1_and_2_rows_and_drops_every_pick_of_mr_3_to_7` switch through `Engine::apply`. They check the picks are gone, MR 3-7's statuses, and the lost slots: empty, listed by the rules, and with no picker.
    - `choosing_the_legend_switch_drops_the_picks_of_the_ranks_it_takes_away`: Legend leaves only MR 1-2's picks, Gold Dragon drops the MR 3 trick, and taking the switch back brings nothing back.
    - `devil_from_aeon_keeps_mr_1_to_7_but_loses_the_gaze_picks_and_extra_gaze`: the gaze picks leave the Build, and MR 3-7 are all `PartlyDropped` with their gaze slot lost.
    - The Legend fixtures (`the_legend_switch_raises_every_score_by_4_from_the_switch_on`, `a_dropped_feature_takes_its_bonus_with_it_and_level_increases_still_count`, `after_a_legend_switch_the_mythic_rank_is_3`, `perception_legend`) switch through `Engine::apply`.
  - `mythic`:
    - `a_legend_switch_at_mr_8_keeps_mr_1_and_2_and_stays_at_rank_3`: at 16 only MR 1's ability is held.
    - `a_switch_that_keeps_ascending_takes_over_from_mr_8` switches through `Engine::apply`.
  - `option_levels::a_path_switch_drops_an_option_a_dropped_pick_took` became `a_path_switch_takes_away_a_general_picks_selection_with_the_option_it_took`: Legend drops Second Bloodline and its selection's pick, the option isn't held at 16 or 17, and a link holding them is stale.
  - `stale_links`:
    - `a_link_with_a_key_the_game_data_lacks_is_stale`: the gone path and switch are on a 17-level Build, since an unreached one is stale anyway.
    - `a_link_with_a_choice_the_build_does_not_reach_is_stale` gained a path before MR 3 and a switch before MR 8.
  - `data/tests/game_data.rs::a_devil_from_aeon_loses_the_gaze_picks_and_keeps_the_rest` (real data) switches through `Engine::apply`. The Build holding the gaze under Devil is impossible.
- **New:**
  - `path_rules` (4):
    - `gold_dragon_and_the_swarm_drop_the_old_paths_picks_of_mr_3_to_7_and_keep_the_general_ones`;
    - `a_switch_is_locked_only_while_a_later_choice_needs_a_pick_it_takes_away` (Stat Focus at 15 needing Perception II locks Legend, Gold Dragon and the Swarm; without it they're open, and a held switch stays removable);
    - `removing_the_level_of_mr_3_or_mr_8_drops_the_path_or_the_switch`;
    - `a_path_or_switch_whose_rank_is_not_reached_is_not_kept`.
  - `stale_links::a_link_holding_a_pick_a_switch_takes_away_is_stale` (1): Legend with MR 3's general pick, Gold Dragon with the MR 3 trick, Devil with a gaze. Each pick fits without the switch, and the switch fits without the pick.
- **Counts:** rules 291 + unit 2 → 293 + unit 2; data `game_data` 119 → 119.

### Proof

`t41/proof-old/` (HEAD 5e7e106, ticket 40's engine) and `t41/proof-new/` (the working tree) build one shared source, `t41/side.rs` + `t41/proof-common.rs`. Both engines have the same API. It prints what ticket 40's proof printed, plus a first line per Build (levels, path, switch, picks) and the path in each edit's line.

- **Two Build sets.**
  - Ticket 40's 445 Builds (`t41/builds.txt`).
  - A second set, `t41/switched.txt`: each of them with each switch applied by the new side, where the result holds the switch and is possible. That is 614 Builds (224 Legend, 195 Gold Dragon, 195 Swarm). The first set has no possible Build that reaches a switch, so without this set the track change would go untested.
- **What `t41/compare.py` allows, and nothing else.** It writes the switch rules out independently of the engine.
  1. A Build possible before is impossible now only when it holds a pick its switch takes away, a path before MR 3 or a switch before MR 8.
  2. On a Build possible on both sides, every line is identical, but:
     - a rank's dropped slots may gain slots the rules take away that hold no pick, and their pickers are gone;
     - its status may go `Reached` → `PartlyDropped` only then;
     - a switch picker may go open → breaks only when the Build holds a pick that switch would take away.
  3. An edit may differ only by what the new side newly drops (picks of unreached ranks, picks the resulting switch takes away, the path below 9, the switch below 17). Its result's possibility may differ only when it dropped something more.
- **Result.** Both sets pass (`t41/compare.txt`, `t41/compare-switched.txt`).
  - **Ticket 40's set:**
    - 371 possible Builds before. 5 are now impossible, each holding a pick its Legend switch takes away (`filled {alchemist,cavalier,inquisitor,skald,wizard} 40`). No Build held a path or switch below its rank.
    - On the other 366, removability (8753), every option status (264355 open, 5532 breaks, 678590 locked), every class picker and every path or switch status are byte-identical.
    - Of 6695 edits:
      - Switch edits dropped 499 taken-away picks.
      - The switch was dropped from 588 switch edits on Builds below level 17. 147 of those results became possible: they are ticket 40's 147 picker-locked switches made impossible before MR 8.
      - Remove level dropped the path 216 times.
  - **The switched set:** all 614 are possible on both sides. Everything is byte-identical but the rank lines, where 1261 empty slots are newly marked dropped (68914 picker options gone with them). Remove level dropped the switch 10438 times and the path 261 times.

Run time: 52 s old, 45 s new.

### Docs

- `GLOSSARY.md`: **Path switch** says the picks it takes away leave the Build, aren't restored by undoing the switch, and lock the switch while a later choice needs one. **Mythic track** says the lost slots are marked empty by the path rules, and Remove level drops the path or switch with its rank.
- Ticket 12: the Mythic track tab line (lost slots empty, no picker) and the picker line (what `Engine::apply` drops).
- The map: the Order note, a note on ticket 26's and ticket 40's lines, and a "Decisions so far" line for ticket 41.
- Ticket 26's own Answer still describes the old release-at-the-switch mechanics (grants carrying the slot and selection), and was left as written.

### Decided

1. **The path and switch go after any edit that leaves their rank unreached, not only Remove level.** Setting one before its rank does nothing. The sheet never offers it, and a Build holding one is impossible (2), so the pickers' lock and `apply` agree.
2. **`is_possible` refuses a path before MR 3 and a switch before MR 8.** The ticket gives this reasoning for a taken-away pick ("a link can't carry one any more"), and the same holds for an unreached path or switch. It reverses ticket 40's "checked whether reached or not". No Build in ticket 40's set was affected.
3. **The walk fails a taken-away pick where it's filled, rather than holding it until the switch.** This is what makes the pick-release code removable. It only changes walks of Builds that are already impossible.
4. **`dropped_slots` and the rank status come from the rules.** Without this they'd always be empty, since no possible Build holds those picks. This changes Devil's empty-gaze ranks to `PartlyDropped` and lists empty lost slots.
5. **`Engine::options` is `None` on a lost slot.** Its picks would be discarded by `apply`, so offering them would show a choice the sheet can't hold. The proof found this: ticket 40's engine said "breaks" there.

### Verification

- `cargo test --workspace`: pass (rules 293 + unit 2, extract 88, data files 25 + game_data 119 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 39 backgrounds).
- No schema, game-data or `Build` change, so no DB reset; `V1_LINK` stays and still decodes.
- Proof: passes (above). The full output is in `t41/verification.txt`.

### Questions

1. **Perception II's feats on a Legend.** Ticket 26 modelled the guide: a Trickster who becomes a Legend keeps the feats Perception II unlocked, though Perception II goes. With Perception II now out of the Build, a feat that needs it can't be held at any level once Legend is chosen. So the switch picker locks Legend (and Gold Dragon and the Swarm) while such a feat is in the Build, and that guide case can't be planned any more. Is that acceptable? Keeping it would mean the Build holds a pick the sheet doesn't show, which is what this ticket removes.
