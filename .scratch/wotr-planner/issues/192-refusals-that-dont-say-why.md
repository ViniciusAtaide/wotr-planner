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
