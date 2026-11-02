# Stale links get a generic error

Type: task
Status: resolved
Blocked by: 39
Map: [WotR Build Planner](../map.md)

## Question

Build the user's answers to [ticket 39](39-impossible-states-not-validation.md)'s open questions (2026-11-01):

1. **A stale link gets a generic error.** "If a stale link happens to appear just give a generic error. Don't try to reason about it." A link that doesn't decode, or that decodes into a Build that isn't possible with the current game data (a key gone, any pick that doesn't fit, any rule broken), opens nothing: the sheet shows one generic message ("This link no longer works with the current game data") and a blank Build. No banner, no broken picks marked, no reasons, no partial repair. This reverses ticket 13's Q9 and ticket 39's decision 2.
2. **Picks the Build no longer reaches are dropped.** "I don't care to store what feats you chose before selecting Legend." Removing a level drops the picks of that level and of the mythic ranks it no longer reaches (reversing ticket 14's keep), and a Path switch that ends ascensions (Legend) drops the picks of the ranks it takes away. The Build only ever holds what the sheet shows.

## Build

AFK, test-first (`tdd`), in `rules` (and the codec); no schema or game-data change expected. Only agent in the repo.

- **One yes/no check for a loaded link.** Whether a decoded Build is possible is a single boolean (or a `Result<Build, StaleLink>` with no detail) at the link boundary. Reuse the walk; don't keep any code whose only job was explaining or marking a stale Build.
- **What goes:** everything that existed only for stale links:
  - the "a broken pick always shows ×" case in `Engine::removable`, and any "choices a stale link already broke don't count" handling in `allows`;
  - `Issue` variants that only a stale Build can produce (ticket 39's table marks which: `UnknownKey`, `NoSuchSlot`, `ParameterMissing`, `WrongParameter`, `NotChoosable`, and any other whose only remaining use is the stale case). Keep what a picker shows on a locked option and what the locks need (ticket 39 says the walk uses `LevelCap` and `SkillPointsOverspent` for locks: keep them if so, or replace them with something smaller if the picker never shows them);
  - `Problem`/`Choice` reporting, `Engine::validate` and `Sheet.problems`, if nothing but the stale-link check uses them after this; otherwise shrink them to what's used. Say which in the Answer.
- **Unreached picks:** `Edit::RemoveLevel` and the Path switch (choosing or changing a switch that ends ascensions) drop the picks of levels and ranks no longer reached. If that needs the ascension table or other game data, make it an engine call rather than `Build::apply`, and have the sheet use it.
- **Docs:** update ticket 12 (the State line: no "Game data changed" banner; a stale link shows the generic error and a blank Build) and ticket 13 (Q9 and the stale-links paragraph, noting the reversal), `GLOSSARY.md` (Problem, stale link), and the map's lines for tickets 13 and 39 where they describe the banner.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: a link with a removed key, one whose pick no longer fits, and one that breaks a fixed rule each come back as the generic stale error; a link that still fits loads unchanged; removing a level drops its picks and the unreached ranks' picks; choosing Legend's switch drops the picks of the ranks it takes away; the removed variants and code are gone.

## Progress log

- 2026-11-01: started. Baseline: HEAD db5465a, clean tree; scratch in `t40/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t40/`), `old-src/` is `git archive HEAD`. Read the walk for what only explained, marked or tolerated a stale Build. `Recheck` (removability and the pickers' `BreaksLater`) reads only *where* the walk finds a choice failing, never *why*; the pickers' reasons come from `check_pick` and `class_status`, never from `Sheet.problems`. So the walk can record failing `Choice`s and nothing else, and `Problem`, `validate` and `Sheet.problems` can go. The walk ignored picks of unreached ranks and levels, class choices of classes no level takes, and a path or switch whose rank isn't reached, so a link could hold a gone key there and still look possible.
- 2026-11-01: tests first. `stale_links.rs` (6) and 2 in `path_rules.rs` (Legend's switch and Remove level drop the unreached ranks' picks). Red runs: `t40/red-compile.txt` (no `StaleLink`, `open_link`, `is_possible`, `Engine::apply`), `t40/red-run.txt` (those stubbed as decode-only, `validate().is_empty()` and `Build::apply`: 7 failures).
- 2026-11-01: built. `Engine::is_possible`, `Engine::open_link` and `StaleLink`; `Engine::apply` drops the picks of ranks no longer reached, and `Recheck` applies edits through it; the walk records failing `Choice`s (`Walker::fail`) and checks at the end what no level checked (`check_unreached`); `Problem`, `Engine::validate`, `Sheet.problems`, the broken-choice case of `removable` and `allows` and 7 `Issue` variants are gone; `problem.rs` became `choice.rs`. Old tests rewritten (below); a test helper `why_not` asks the picker for the reason a pick that alone makes a Build impossible can't be taken. Green: rules 291 + unit 2, data `game_data` 119.
- 2026-11-01: proof, docs, Verification. Proof (`t40/proof-{old,new}` on one shared `t40/proof-common.rs`, `t40/compare.py`) over ticket 39's 445 Builds: passes (below). Docs: `GLOSSARY.md` (Stale link replaces Problem; Removable; Mythic track), tickets 12 and 13, the map's lines for 13 and 39 and its Order note. No schema, game-data or `Build` change, so no DB reset and `V1_LINK` stays. Verification below.

## Answer

Done 2026-11-01, uncommitted. A link opens through one call, `Engine::open_link`, which gives the Build or one generic error, `StaleLink`, with no reason. The walk now says only whether a choice fails, so `Problem`, `Engine::validate`, `Sheet.problems`, the stale-link × and seven `Issue` variants are gone. Edits go through `Engine::apply`, which drops the picks of Mythic ranks the Build no longer reaches (Remove level, a Legend switch). Two things the user's answers don't settle are kept as they were and asked below: the path and switch on Remove level, and the picks a switch takes away at earlier ranks.

### Stale links

- **`Engine::open_link(code) -> Result<Build, StaleLink>`** decodes the link and checks the Build with `Engine::is_possible`. A link that doesn't decode (damaged, a format version this planner doesn't know, a hand-edited one breaking a fixed rule) and one whose Build isn't possible are both `Err(StaleLink)`. `StaleLink` is a unit struct; it displays as "This link no longer works with the current game data". The sheet then shows a blank Build.
- **`Engine::is_possible(build) -> bool`** is one walk: every key known, every pick in a slot the walk reaches and fitting there, every rule kept. The walk records the failing `Choice`s (`Walker::fail`) for `Recheck`, never a reason. At its end, `check_unreached` fails what no level checked:
  - a pick for a slot the walk never reached: one a level doesn't have, one past the last level, or one at a rank not reached;
  - a class choice for a class no level takes;
  - the path and the Path switch, checked whether their ranks are reached or not: the path a known path without a switch rule, the switch a known path with one, open from the Build's path.
- **Removed:** `Problem` (and `Problem::is_unknown_key`), `Engine::validate`, `Sheet.problems`, the "a broken pick is always removable" case in `Engine::removable`, and "choices a stale link already broke don't count" in `Engine::allows` (`Recheck.broken`).

### Picks the Build no longer reaches

- **`Engine::apply(build, edit)`** is how the sheet applies every edit: `Build::apply`, then it drops the picks of levels and Mythic ranks the Build doesn't reach. A rank is reached when the Build has its ascension's character level and, under a switch that ends the ascensions, it comes before the switch rank. It needs the ascension table, so it's an engine call. `Build::apply` stays public for the edits that need no game data, and its docs point to `Engine::apply`.
- **Remove level** drops the removed level's picks (as before) and the picks of the ranks it reached: MR 10 with level 19, MR 8 and 9 with level 17. This reverses ticket 14's "mythic picks stay".
- **A switch that ends the ascensions** (Legend) drops the picks of MR 8, 9 and 10. Legend's MR 8 is never reached either: the switch happens there and "stays at MR 3". Gold Dragon, Swarm and Devil ascend on, so they drop nothing.
- `Recheck` applies edits through `Engine::apply`. So the switch picker doesn't lock Legend just because MR 8 to 10 hold picks. It locks Legend (`BreaksLater`) only when a later choice needs a dropped pick.

### What remains, and why

| Piece | Now | Why it's still needed |
|---|---|---|
| `Issue::Unmet` | kept | Picker reason ("needs Power Attack"). |
| `Issue::AlreadyHeld` | kept | Picker reason. |
| `Issue::MaxTimes` | kept | Picker reason. |
| `Issue::ClassLevelCap` | kept | Class picker reason (the class's table is done). |
| `Issue::NotFromPath` | kept | Switch picker reason (Devil only from Aeon or Azata). |
| `Issue::BreaksLater` | kept | Picker reason: the option would break a later choice. |
| `Issue::NotAllowed` | kept, wider | No picker shows it, since a picker offers only what its slot takes. `check_pick` still needs a "doesn't fit" answer for the walk: an earlier edit can leave a pick in a slot that no longer takes it (another class at the level, another option held), and that is what locks the edit. It now also covers a key the data lacks and a value of the wrong kind or none, which had their own variants. |
| `Issue::UnknownKey` | **gone** | Only a stale link holds one. The walk fails the choice; `check_pick` says `NotAllowed`. |
| `Issue::ParameterMissing`, `Issue::WrongParameter` | **gone** | No picker offers such a pick (each value is its own option). `check_pick` says `NotAllowed`. |
| `Issue::NoSuchSlot` | **gone** | No picker can be asked for a slot the Build doesn't have. The walk fails any pick for a slot it didn't reach. |
| `Issue::NotChoosable` | **gone** | The path and switch pickers list only paths without or with a switch rule. The walk fails the path or switch. |
| `Issue::LevelCap`, `Issue::SkillPointsOverspent` | **gone, replaced by something smaller** | No picker shows them; the locks show `BreaksLater`. The walk still fails `Choice::CharacterLevel` and `Choice::SkillPoints`, which is what locks taking back the Legend switch past level 20 and lowering Int. |
| `Choice` | kept (all variants) | `Engine::removable` answers with it, and `Recheck` needs to know where a walk fails so an edit's own choices don't count against it. `CharacterLevel` and `SkillPoints` are only such places. |
| `Problem`, `Engine::validate`, `Sheet.problems` | **gone** | Only the banner used the reasons. `Engine::is_possible` is the yes/no that remains. |
| `Unmet` | kept | The shape of `Issue::Unmet`. |
| `DecodeError` | kept | The codec's own contract (ticket 13's tests per variant). It never reaches the sheet, since `open_link` turns it into `StaleLink`. |
| `RankSheet::dropped_slots`, `RankStatus::{PartlyDropped, Dropped}` | kept | They show what a switch takes away at earlier ranks (ticket 26), not anything stale. |

`problem.rs` is now `choice.rs` (it holds `Choice`, `Issue` and `Unmet`).

### Tests

- **Removed:**
  - `removable::a_stale_links_broken_pick_is_removable_even_with_dependants`
  - `backgrounds::an_unknown_background_is_reported`
  - `validation::keys_missing_from_the_game_data_are_reported_not_fatal`, `validation::picks_that_no_longer_qualify_are_reported_with_their_reason` and `validation::a_pick_for_a_slot_that_no_longer_exists_is_reported`. `stale_links.rs` covers all three.
- **Rewritten:**
  - `validation.rs` became `possible.rs`, keeping its other six tests. Each asserts `is_possible`, or a picker's reason: the class picker's `Unmet` for Arcane Trickster and the Eldritch Scoundrel, the path pickers leaving out a switch-only path, `AlreadyHeld`.
  - `codec::a_link_with_keys_the_game_data_lacks_decodes_and_reports_them` became `…_still_decodes`.
  - `option_levels::a_pick_for_an_option_rows_slot_the_build_lacks_is_reported` became `…_makes_it_impossible`.
  - `parameters::a_parameterised_feat_needs_a_value_of_its_kind`: each bad value alone makes the Build impossible, and no picker offers it.
  - The advancement tests (`advances.rs`, `class_rules.rs`, the real-data Loremaster) compare a Build with other advances instead of reading the problems at `Choice::Advances`. Three got a Build whose entry holds, so only the advances fail:
    - the Mystic Theurge two-arcane test gained cleric levels;
    - "must already be taken" became an Arcane Trickster advancing a witch taken after it;
    - the Loremaster gained Skill Focus (Knowledge (Arcana)) and Extend Spell.
  - Every other `validate(..) == []` or `problems == []` became `is_possible`. Every single-problem assertion became `why_not(build, slot)`: the Build must be impossible, possible without that pick, and the picker gives the reason (or doesn't offer the pick: `None` where it was `NotAllowed`, `NoSuchSlot`, `ParameterMissing` or `WrongParameter`).
  - Test fixtures: `path_rules::mythic_build` puts general picks only at ranks the Build reaches. The Legend fixtures (`perception_legend`, mythic.rs's 40-level Legend) switch through `Engine::apply`. The Devil-from-Trickster test no longer asserts what the sheet of an impossible Build shows.
- **New:**
  - `stale_links.rs` (6): a link that still fits loads unchanged; a key gone (feat, class, skill, race, background, archetype, path, switch); a pick that no longer fits (Power Attack needing Str 18); a rule broken (overspent points, level 21 without Legend, a switch-only path at MR 3, two ranks at level 1, an increase at level 3); a choice the Build doesn't reach (no such slot, past the end, an unreached rank, an unused class choice); a link that doesn't decode. Each is `Err(StaleLink)`.
  - `path_rules.rs` (2): choosing Legend's switch drops MR 8 to 10's picks and keeps MR 3 to 7's, and Gold Dragon drops none; Remove level drops MR 10's picks at 19, then MR 8 and 9's and level 17's at 17, and adding the levels back brings the ranks back empty.
- **Counts:** rules 288 + unit 2 → 291 + unit 2; data `game_data` 119 → 119.

### Proof

`t40/proof-old/` (HEAD db5465a, ticket 39's engine) and `t40/proof-new/` (the working tree) build one shared source, `t40/proof-common.rs`. Both read ticket 39's 445 Builds (`t39/builds.txt`).

- **What each side prints.**
  - Whether each Build is possible (old: no problems; new: `is_possible`).
  - For each possible Build, everything ticket 39's proof printed: the removable choices; the path and switch pickers; per level its slots with every option's status (open, breaks, locked), the class picker and the state; per rank its status, slots and dropped slots; the next level's class picker.
  - The edits that change: Remove level again and again down to no level, and each Path switch (and none), as the sheet applies them (old `Build::apply`, new `Engine::apply`), with the picks left and whether the result is possible.
- **What `compare.py` requires.**
  - Every line identical except the edits' picks.
  - An edit's picks differ only by picks of ranks the edited Build no longer reaches.
  - The one allowed difference in an edit's possibility: a switch the switch picker already locks (`NotFromPath`) on a Build that doesn't reach MR 8 is now impossible, since the switch is checked whether reached or not. The sheet never makes that edit.
- **Result.** It passes (`t40/compare.txt`):
  - 16511 lines, and both sides agree on which Builds are possible: 371 of 445.
  - On the possible Builds, removability (9258 removable choices), every option status (271470 open, 8418 breaks, 707380 locked), every class picker status and every path or switch status (214 breaks) are byte-identical.
  - Of 6920 edits, 581 dropped 7113 picks of unreached ranks (6955 by Remove level, 158 by a switch). All 5065 Remove-level results are possible on the new side. 147 picker-locked switch edits became impossible before MR 8.

Run time is the same (51 s each), so the end-of-walk check costs nothing that shows.

### Docs

- `GLOSSARY.md`:
  - **Stale link** replaces **Problem**: one generic error, a blank Build, no reasons.
  - **Removable** loses the broken case.
  - **Mythic track** notes that an unreached rank holds no picks.
- Ticket 12:
  - The picker line loses "a broken pick always shows ×" and says edits go through `Engine::apply`.
  - The State line has `Engine::open_link`'s generic error and a blank Build instead of the "Game data changed" banner.
- Ticket 13: Q9 is marked reversed. The stale-links and hand-edited-links paragraphs became one: any link that doesn't decode or isn't possible opens nothing, with the generic error.
- The map: the Order note, ticket 13's line and ticket 39's line.

### Decided

1. **The link boundary is `Engine::open_link`, plus a public `Engine::is_possible`.** The ticket allowed a boolean or a `Result` with no detail; both are cheap, and the tests use the boolean. `decode_build` and `DecodeError` stay as the codec's own API.
2. **The walk records where a choice fails, not why.** That is all `Recheck` needs. `LevelCap` and `SkillPointsOverspent` became plain failing choices (the "something smaller" the ticket offered), since no picker shows them.
3. **`NotAllowed` absorbs unknown keys and wrong or missing values in `check_pick`.** The walk still needs a "doesn't fit" answer there, and no picker shows any of them.
4. **A Build holding something no level checks is impossible:** a pick for an unreached slot or rank, a class choice for a class no level takes, and a path or switch the Build couldn't take even while unreached. This follows from "a key gone, any pick that doesn't fit, any rule broken" and "the Build only ever holds what the sheet shows". Without it, such a link would open and the sheet could later hold an impossible Build. For example, the class picker for a class not yet taken keeps an unused choice's advances. And an unreached path picked while a Devil switch is held would be found broken only on reaching MR 8, after the next-level class picker (which checks nothing past the end) had added the level.
5. **Remove level keeps the path and the switch**, as before, since they aren't picks. Asked below.
6. **The picks a switch takes away at earlier ranks stay**, shown as dropped (ticket 26), since they're reached. Asked below.

### Verification

- `cargo test --workspace`: pass (rules 291 + unit 2, extract 88, data files 25 + game_data 119 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 39 backgrounds).
- No schema, game-data or `Build` change, so no DB reset, and `V1_LINK` stays and still decodes.
- `Problem`, `validate`, `Sheet.problems`, `is_unknown_key` and the removed variants (`UnknownKey`, `NoSuchSlot`, `ParameterMissing`, `WrongParameter`, `NotChoosable`, `LevelCap`, `SkillPointsOverspent`) appear nowhere in the code or tests. `app/src/lib.rs` used none of them.
- Proof: passes (above). The full output is in `t40/verification.txt`.

### Questions

1. **The path and the Path switch when Remove level takes their rank away.** They aren't picks, so Remove level keeps them: a Build of 8 levels may hold the path chosen at MR 3 (level 9), and adding level 9 back reaches it again. They're checked whether reached or not, so a link can't hide a gone path there. Should Remove level drop them too once their rank isn't reached? "The Build only ever holds what the sheet shows" suggests yes, if the sheet won't show a path for an unreached MR 3.
2. **The picks a switch takes away at earlier ranks.** Legend drops MR 3 to 7's general and path picks. Gold Dragon and Swarm drop the old path's picks of MR 3 to 7, and Devil the gaze picks. These ranks were reached, so the picks stay in the Build and the Mythic track marks them as dropped (ticket 26's `dropped_slots`). "I don't care to store what feats you chose before selecting Legend" may mean these rather than MR 8 to 10's. Should a switch drop them from the Build too (then the track shows those slots empty)?

## Decisions on the questions

Taken from the user's own words (2026-11-01), "I don't care to store what feats you chose before selecting Legend", and the rule that the Build holds only what the sheet shows; the user can still overrule them:

1. **Yes:** Remove level drops the path and the Path switch once their rank (MR 3, MR 8) isn't reached.
2. **Yes, later reversed (ticket 42):** a switch drops from the Build the picks it takes away at earlier, reached ranks (Legend's MR 3–7, Gold Dragon's and Swarm's old-path picks, Devil's gaze picks). The Mythic track still shows those ranks as dropped, with their slots empty.

Built by [ticket 41](41-drop-what-a-switch-takes-away.md).
