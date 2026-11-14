# Refusals and labels from the MCP QA session

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build; Opus to review the wording.
Map: [WotR Build Planner](../map.md)

## Problems

Found by a QA session through the MCP server (2026-11-13); the first four were rechecked on `main` at `ce9cc3b`. Each reason is correct but leaves out what the reader needs, or reads oddly.

1. **A lowered score doesn't name what it breaks.** A sorcerer with Dodge held through a Belt of Physical Perfection +2, Dexterity lowered from 11 to 10: "At Dexterity 11 → 10: Would break a later choice, which must be withdrawn first …" (`BREAKS_LATER`, `apps/mcp/src/options.rs`). The gear refusal for the same Build names it ("Dodge needs Belt of Physical Perfection +2"). Name the first choices it would break, as ticket 171's locked rank moves do.
2. **"Already taken"** for an option taken once per value (`Issue::AlreadyHeld`, `libs/rules/src/engine/labels.rs`): Finesse Training (Rapier) chosen again at rogue 19 says only "Already taken". Say where: "Finesse Training (Rapier) is taken at level 3".
3. **A rank level below the rank under it** (`choose` `mythic.4.level` 12 with MR 3 at 13): "`12` isn't an option for `mythic.4.level`, and no offered value is close to it. `options` lists what it takes, with `values: true` for a parameterised feat's values." (`apps/mcp/src/choose.rs`). 13 is offered and one away, the cause (MR 3 is taken at 13) is missing, and the parameterised-feat hint doesn't apply to a level.
4. **Withdrawing a rank already at its earliest level:** "The Build holds nothing at `mythic.4.level`" (`apps/mcp/src/withdraw.rs`). MR 4 is held, at level 11; say it is already at the earliest level it comes at.
5. **"Combat Style Feat feat"**, the ranger's slot label at 2, 6, 10, 14 and 18 (`format!("{} feat", …)` in `labels.rs`, on the selection "Combat Style Feat").
6. **"(no Bloodline yet, no Bloodline yet)"** at the head of the Dragon Disciple's prerequisite in `search` ("one of 11 alternatives ((no Bloodline yet, no Bloodline yet), Draconic (Black), …)"), from `labels.rs`'s `no {} yet`. Say what the alternative is (a sorcerer or a magus with no bloodline chosen yet, if that's it) once.
7. **"Needs Perception II"** for Stat Focus at level 13 when Perception II is held, but at MR 4 moved to level 14. The same-level case already says where the pick is ("… in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level 11's pick order"); the later-rank case should say the rank and its level.
8. **"Stat Focus (AC) needs MR 3 earlier than level 13"** on `options` for `mythic.3.level`: Stat Focus needs Perception II in MR 4, and moving MR 3 breaks it only by taking MR 4 along. Name MR 4 and that it moves with MR 3.

Items 7 and 8 come from a subagent's report and were not rechecked on `ce9cc3b`: reproduce them first (an Empyreal Sorcerer 20, Trickster, Perception I at MR 3 and Perception II at MR 4).

## Done when

Each reason or label says what is held, where, and why, without the stray hint; tests pin the new texts; the snapshot check's reason lines move only for these; `scripts/check.sh` passes.

## Answer

Done 2026-11-13 (branch `ticket/192-refusals-that-dont-say-why`, from `main` at `5858eb4`, merged with `main` at `e4dce36`). Opus reviewed the wording; the texts below are its settled ones. Not merged.

### The texts, one example each

1. **A refusal that breaks a later choice names it** (`apps/mcp`: `Breaks` in `options.rs`, `choose`, `withdraw`).
   - Score: "power-attack needs Strength above 12"; in `choose`: "At Strength 13 → 12: power-attack needs Strength above 12. Nothing was changed."
   - Skill ranks: "deft-maneuvers needs more than 0 Mobility ranks at level 1".
   - Any other address: "cleave depends on what `slot.feat.1.class.0` holds now"; a parameterised entry listed without its values says the same from its first value's edit.
   - `withdraw`: "cleave depends on `slot.feat.1.class.0`, so it can't be withdrawn now: withdraw that choice first."
   - The first three names, then "and N more". `BREAKS_LATER` stays as the last fallback only. Rules gained `Engine::refused_by`, `refusals` and `choice_names_within`, which the gear picker and the rank picker share.
2. **"Already taken" says where** (`Engine::held_where` in `libs/rules/src/engine/pickers.rs`, used by `slot_entries`, `options`, `choose` and `validate`): "Finesse Training (Light Shield Bash) is taken in level 3's Finesse Training slot"; in a rank's slot "dodge is taken in Mythic rank 4's trickster/bonus-feats slot, at level 11". A grant keeps "Already taken". `Issue::AlreadyHeld` and `issue_text` are unchanged, so no status hash the proof reads moves. A value's place is never reused for the next value.
3. **`choose` of a rank level that isn't offered:** "MR 4 can't be taken at level 10: MR 4 comes at level 11 at the earliest (the rank table's level). `options` on `mythic.4.level` lists the levels."; with the rank below, "(MR 3 is taken at level 13, and ranks keep their order)"; past the last level, "the Build has 13 level(s), and a rank is taken at a level the Build has". No "closest" note for a level; the `values: true` clause only for slot addresses. 191's `validate` text takes the same parenthetical.
4. **`withdraw` of a rank at its earliest level:** "MR 4 comes at level 11 at the earliest (the rank table's level) and is taken there: there is nothing to withdraw at `mythic.4.level`."; pushed by the rank below, with "To take MR 4 earlier, take MR 3 earlier: …"; unreached and no-such-rank texts as `validate`'s. The code stays `no_such_choice`.
5. **"Combat Style Feat feat"** is fixed in the data, not in code: the ranger's selection `ranger/combat-style-feat` is named "Combat Style" (the key stays), so its feat slots read "Combat Style feat" and the level-2 slot "Combat Style", like the slayer's "Combat Style (Slayer)". A code fix (stripping a trailing "Feat") would have left the level-2 slot and its feat slot differing only by case. The slayer's talent options named "Combat Style Feat (2nd or 4th level)" are options and keep their names.
6. **"(no Bloodline yet, no Bloodline yet)":** inside an `All`, identical texts read once and a group left with one text loses its brackets; an `Any` and an `AtLeast` keep every alternative. `search`: "(one of 11 alternatives (no Bloodline yet, Draconic (Black), Draconic (Blue), …)), 5 ranks in Knowledge (Arcana), …"; `describe`: "(no Bloodline yet or Draconic (Black) or …)"; the unmet reason: "Needs (no Bloodline yet or Draconic (Black) or …), 5 ranks in Knowledge (Arcana) (have 0)".
7. **A prerequisite held by a later rank's pick** (`Engine::later_rank_pick`, `later_rank_text`, `rank_level_that_lets_in`): "Needs dodge: dodge is in Mythic rank 4's trickster/bonus-feats slot, taken at level 14, after this slot's level 13". The MCP (`options`, `choose`, `validate`) appends ". MR 4 taken at level 12 opens it: `choose` `mythic.4.level` 12 first"; the web app appends " · MR 4 taken at level 12 opens it". `validate` tries the level on its declaration without the failing pick and says no level when another choice fails beside it. The rank's own hint from 171 wins when it fires.
8. **A locked rank move that fails for a pick in a rank moving along:** "crane-style needs MR 4's dodge earlier than level 13: MR 4 moves with MR 3", only when every named choice fails for the one same pick, held in a rank in `moves`; otherwise 171's text, with its tests untouched.

### What differs from the settled texts

- None in wording. Item 1's skill-rank line reads "At Mobility ranks at level 1 1 → 0: …" in `choose` because the step prefix is older text, left alone.
- Item 7's tail is the MCP's; the web app's has no address, as it names none anywhere.

### Timing (item 1)

`options` with `only_open: false` on a fighter's Power Attack slot, real data, debug build: 261 options of which 57 locked, 9.9 ms before and 11.0 ms after at 3 levels; at 20 levels 10.2 ms before and 13.9 ms after. With `values: true` (896 options, 167 locked): 33.0 ms before, 35.4 ms after at 3 levels; 32.6 ms and 44.8 ms at 20. One extra walk per locked option, never two.

### Snapshot check

Only `class-ranger.txt` moved, by the rename:

```text
the engine's output differs from the snapshots in 1 of 44 Builds

Changes, each with the Builds it's in:
  slot.feat.10.class.0: Combat Style Feat feat: - → Combat Style feat: - in 1 Build: class-ranger
  slot.feat.14.class.0: …  slot.feat.18.class.0: …  slot.feat.2.class.0: …  slot.feat.6.class.0: …
  slot.selection.2.0: Combat Style Feat: - → Combat Style: - in 1 Build: class-ranger
```

### Tests

Rules: `libs/rules/tests/engine/{labels,slot_entries,rank_levels}.rs` (an and of alike texts; the slot label; held where, by value and in a rank's slot; the move naming the pick of a rank along; the later rank's pick). MCP: `apps/mcp/tests/server/{options,choose,withdraw,finesse_training,rank_levels}.rs` (the score, skill, slot and entry reasons; `withdraw`; the rank-level refusals; Finesse Training twice in `choose` and `validate` on the real data; the later rank's pick in `options`, `choose` and `validate`). Seeds: `report_gaps.rs` (the ranger's labels; the Dragon Disciple's three texts). 191's pinned parenthetical changed.

### What the queue's proof should show

The Dragon Disciple's class reasons (`class_reasons`) change text, and nothing else: `AlreadyHeld`, `BreaksLater` and every option's status are unchanged, and the new texts live beside the engine's reasons (`EntryOption.reason`, the MCP, the web app), not in what the proof hashes.

### Checked

`scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, test with the snapshot check, rules-wasm, data-check). `cargo fmt --all --check` passes. Not run, by the queue's rule: `scripts/check.sh --full` and `scripts/proof.sh`. Not done yet: the README note for the ranger rename, and its line 386 prose, which wait for ticket 189.

### After ticket 189 (2026-11-13, part 2b)

- **README** (`libs/game-data/README.md`): a note for the ranger's selection named "Combat Style" (its option-list feat slots read "Combat Style feat"), and the ticket 62 line about a held curse now says the picker and `options` say where it is held ("Plagued is taken in level 1's Oracle's Curse slot"), while `issue_text` still reads "Already taken".
- **`choose`'s skill-rank prefix** reads "At Mobility ranks at level 1, 1 → 0: …" (a comma after the skill's place; a score's prefix is as it was: "At Strength 13 → 12"). The pinned text in `apps/mcp/tests/server/choose.rs` is updated.
- **`validate` words an `Any` as the picker does.** With 189's gates, a barbarian 4 declaring Clear Mind read "Needs Barbarian level 8 (have 4) or Skald level 8 (have 0)" in `validate`'s failure, where the picker says "Needs Barbarian level 8 (have 4)". `Wording::unmet` now passes the issue through `Engine::reason_for` (made `pub`) with the state at the slot's level, so the reason reads "Needs Barbarian level 8 (have 4)"; pinned by `real_data_a_locked_rage_power_names_only_the_class_the_character_has` in `apps/mcp/tests/server/validate.rs`. The failure's `needs` list is untouched: it still gives the prerequisite as the data states it.

### Code review fixes (2026-11-13, branch `ticket/192-review-fixes`, from `main` at `255fb7a`)

A review of the merged range found seven defects in its code; five were reproduced through the running server. Each was reproduced by a test first.

1. **A breaks-later reason that blamed the slot for something else.** `Breaks::reason` worded every other edit "{names} depend(s) on what `{address}` holds now". It now words by what each refused choice fails for (`Breaks::cause`): a pick that would be held twice says where it is held, in item 2's phrase ("toughness is taken in level 3's Feat slot": Toughness at level 1 with Toughness at level 3 and slot 1 empty); "depends on what `slot.feat.1.class.0` holds now" only when the address is a slot that holds a pick and every refused choice fails a prerequisite that is not a negation (`Engine::unmet_negates`); anything else says only "Would break combat-trick" (a class swapped under the rogue talent its level gives). `withdraw` words by the same causes. Tests: `a_pick_that_would_be_held_twice_says_where_it_is_held_not_what_the_slot_holds`, `a_change_that_breaks_a_pick_for_another_reason_says_only_what_it_would_break` (`choose.rs`).
2. **The later-rank clause named a pick of another value.** `names_pick` compared keys only. It now reads the value: `FeatFor` the value it names, `FeatForSameValue` the value of the pick that fails, `FeatIfValue` its feat only for that value (`later_rank_pick` takes the refused pick). Weapon Specialization (Dagger) at level 13 names MR 4's Weapon Focus (Dagger) at level 14; the other weapons say nothing of it (152 options carried the clause before, 7 after on the real data). `Issue` and `Unmet` are untouched. Test: `a_later_ranks_pick_is_named_only_for_the_value_it_would_meet` (rules).
3. **The one-line entry spoke for two values.** `SlotEntry::shown` (shared by the engine's `entry_reason`, the MCP and the web app) is the value whose refusal the entry shows; its issue, pick and edit stand for the entry, so a first value already held no longer lends its edit or drops the "MR n opens it" tail. The web app takes the entry's clause from that value. Test: `a_parameterised_entry_speaks_for_the_value_whose_reason_it_shows` ("cleave depends on what `slot.feat.1.general.0` holds now", where it read "power-attack and cleave depend…").
4. **`choose` of a level written as text.** A rank level given as a key that reads as a number is that number: `"13"` is taken at level 13 when offered, and `"10"` is refused as 10 is. Test: `choose_reads_a_level_given_as_a_string`.
5. **"moves with MR n" for three refusals of more.** `pick_moved_along` now looks at every refusal, not only the three the reason names; the special text needs all of them to fail for the same pick of the same rank. No new test: the case needs four refusals of which three fail for one pick and one for another, which the test world's feats can't make without new data; the existing test (two refusals) and the untouched tests at `rank_levels.rs` 283 to 386 pass.
6. **Walks repeated or spent early.**
   - (a) `Engine::later_rank_levels` works out, once for a later rank, the levels below the slot the engine allows with the Build moved and the ranks along; `level_that_lets_in` is one possibility check per option. The MCP asks only when a reason is shown (`options`' closed lines, a `choose` refusal; never under `only_open: true`), through `LaterLevels`, with the clause itself built without a walk; the web app keeps a map of the levels for the slot. Timing on the finding 2 Build (fighter 14, MR 4 at level 14 holding Weapon Focus (Dagger), `options` on level 13's feat slot, debug build, real data, median of five runs): `only_open: false, values: true` (896 options) 100 ms before and 37 ms after; `only_open: true, values: true` (249 options) 78 ms before and 13 ms after; `only_open: false` without values (261 options) 15 ms before and 14 ms after.
   - (b) `Wording::unmet` asks `later_rank_pick` before it clones the Build and checks it is possible.
   - (c) `Engine::slot_entries_in` takes the sheet when the caller has it (the MCP and the web app do); `slot_entries` walks it once, lazily, as before.
   - (d) `rank_level_not_offered` reads `Engine::rank_floor`, not the whole picker.
7. **The unreached-rank wording in four places.** `apps/mcp/src/unreached.rs`: `Unreached::of` says no such rank, too few levels (the level the rank comes at and the Build's levels) or a Legend switch; `options`, `choose`, `validate` and `withdraw` use it, so they read alike: "The Build doesn't reach MR 9, so `mythic.9.level` has no level to take it at: its Path switch (Legend) ends the ascensions at MR 8." (`withdraw`: "…so there is nothing to withdraw: …"). `options` no longer says "MR 9 comes at level 17 … and not after a Path switch" without the cause. Test: `options_choose_validate_and_withdraw_say_alike_why_a_rank_is_not_reached`.

No data, snapshot or seeds test moved.

A second pass found one more: **`sheet` and `compare` dropped a second grant at one level.** The Trickster's MR 3 row lists Sneak Attack twice; `sheet`'s level 9 `features` listed it once (the count rose by two) and the rank's own `features` listed the same entry twice, so `compare` (which tells lists apart by their entries) saw no difference between gaining one and gaining two. A feature gained more than once now carries `times` (`Gained` in `apps/mcp/src/shapes.rs`; left out for one) in a level's `features`, a rank's `features` (one entry with `times`, not the entry repeated), `compare`'s level `features` and its `path_features`: `{"key": "trickster/sneak-attack", "name": "Sneak Attack (Trickster)", "times": 2}`. No tool description or skill text describes these lists in a way that needed the count; the schema's field docs say it. Tests: `real_data_the_sheet_says_a_feature_gained_twice_at_a_level_twice`, `real_data_compare_sees_a_feature_gained_twice_against_none` (`apps/mcp/tests/server/gained.rs`) and `a_feature_gained_twice_differs_from_the_same_one_gained_once` (`compare.rs`).

Third pass (the reviewer's re-check of `3338256`):

- **A group exclusion reads as a negation.** `Engine::unmet_negates` counts `NoOtherOptionOfGroup` and `NoOptionFrom` beside `Not`, so a Draconic (Blue) pick refused because a held Draconic (Red) excludes it says "Would break …", not "… depends on what `slot.selection.1.0` holds now". Test: `a_group_exclusion_counts_as_a_negation_and_a_plain_need_does_not` (rules; the sorcerer's two-bloodline Build needs a mythic pick, so it is pinned on the condition, not through the server).
- **A change at an address that isn't a slot now reads "Would break …".** The settled text for "any other address" was "{names} depends on what `{address}` holds now"; that wording is true only for a slot that holds a pick, so a class, race or alignment change that breaks picks (and any slot change whose broken choices fail something other than a plain need) says "Would break combat-trick", which is all that is known to be true.
- **`validate` reads a rank level written as text,** as `choose` does (`"13"` is 13; text that is no number keeps "takes a number"). Test: `validate_reads_a_level_given_as_text_as_choose_does` (`rank_levels.rs`, beside 191's).
- **Two doc comments** (`Offer.like`: the shown value's edit; `LaterLevels`: one slot of one call).

### QA session fixes (2026-11-14)

Branch `ticket/192-qa-fixes`, from `main` at `8a457a0`. Each defect was reproduced by a test first. No `Issue`, `Unmet` or `issue_text` changed, no issue code, no data, snapshot or seeds test.

1. **`validate` words a rank the Build doesn't reach as the other tools do,** whatever level is declared and whichever of `not_allowed` and `not_reached` the walk raised (`Wording::unreached_rank`, over `Unreached::of` of the declared Build without the rank's own level). 13 levels, `mythic.5.level: 20`: "The Build doesn't reach MR 5, so `mythic.5.level` has no level to take it at: MR 5 comes at level 14 at the earliest, and the Build has 13 levels."; a Legend Build declaring `mythic.10.level: 20`: "The Build doesn't reach MR 10, so `mythic.10.level` has no level to take it at: its Path switch (Legend) ends the ascensions at MR 8." A rank the Build reaches, declared past the last level, keeps "MR 4 can't be taken at level 20: the Build has 13 levels, and a rank is taken at a level the Build has."
2. **"1 level", "13 levels"** (`apps/mcp/src/words.rs`: `levels`, over `plural`, which moved there from `search.rs`) in `unreached.rs`, `choose.rs`, `options.rs`, `validate/declare.rs` and `validate/wording.rs`: "The Build has 1 level: `level.N.class` takes N from 1 to 2, the last adding a level."
3. **`choose` of a rank level that is no number** says what `validate` says, and where the levels are listed: "`mythic.4.level` takes a number. `options` on `mythic.4.level` lists the levels." (`"fourteen"`; a text that reads as a number is still that number.)
4. **`withdraw` says what to withdraw:** "cleave depends on `slot.feat.1.class.0`, so it can't be withdrawn now: withdraw cleave first." for one dependent (named, so "it" can only be the address just named); "cleave and great-cleave depend on `slot.feat.1.class.0`, so it can't be withdrawn now: withdraw them first." for several, and also when the names end in "and N more" ("cleave, critical-mastery, great-cleave and 1 more depend on `slot.feat.1.class.0`, so it can't be withdrawn now: withdraw them first."). The "Would break" form has not named the address yet, so it keeps it: "Would break toughness, so `slot.selection.1.0` can't be withdrawn now: withdraw toughness first." The rank-level form likewise ("… needs MR 4 later than level 11, so `mythic.4.level` can't be withdrawn now: withdraw toughness-mythic first.").
5. **One "and" in a name list:** `Engine::choice_names_within` joins by commas alone when "and N more" follows: "Metamagic (Quicken Spell), Toughness, Great Fortitude and 2 more"; "Level 18: Skill ranks, Level 19: Skill ranks, Level 20: Skill ranks and 2 more". Lists without "and N more" are as they were. The count that grew by two per level past 20 is not a double count: moving MR 4 later on a Legend Build puts every level past 20 under the cap, so for each such level both its class (`Level 21: Fighter`) and its skill ranks (`Level 21: Skill ranks`) are refused; two different choices. Nothing changed for it.
6. **`validate`'s later-rank hint ends in `validate`'s terms:** "Needs dodge: dodge is in Mythic rank 4's trickster/bonus-feats slot, taken at level 14, after this slot's level 13. MR 4 taken at level 12 opens it: declare `mythic.4.level` 12" (`options` and `choose` keep "`choose` `mythic.4.level` 12 first"; `Fix` in `options.rs`).
7. **`validate`'s `needs` list reads as its `reason`:** `needs` for Clear Mind on a barbarian 4 is `["Barbarian level 8 (have 4)"]`, not "… or Skald level 8 (have 0)". `needs` is documented as "each prerequisite not met, in short", not as the data's prerequisite in full, and the same trimming serves: `Wording::trimmed` gives the issue as the picker words it, and the `Node` carries its `needs` from it; the tree is still placed by the untrimmed issue.

Tests: `validate_says_a_rank_is_not_reached_whatever_level_is_declared`, `a_level_count_of_one_reads_level_not_levels`, `choose_says_why_a_level_is_not_one_a_rank_can_be_taken_at` (changed), `withdraw_names_one_dependent_and_says_them_for_several`, `withdraw_says_them_first_when_the_names_end_in_and_n_more`, `withdraw_names_the_one_choice_it_would_break_when_no_need_is_unmet`, `a_choice_something_depends_on_is_not_removable` and `withdraw_names_what_depends_on_a_choice` (changed), `validate_says_where_a_later_ranks_pick_is_and_which_level_would_open_it` (changed), `real_data_a_locked_rage_power_names_only_the_class_the_character_has` (extended), `a_count_of_levels_is_singular_only_for_one` (`words.rs`), and in rules `a_list_of_names_that_goes_on_is_joined_by_commas_and_one_and`.

