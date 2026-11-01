# Impossible states instead of validation

Type: task
Status: resolved
Blocked by: 38
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-01:** the backend serves only this frontend, so drop any validation of the payload that the UI can make an impossible state. Example: with Power Attack picked, the UI offers Cleave and what Power Attack enables, so nothing needs to expect a Build holding Cleaving Finish without Cleave. Remove that validation from the service layer.

## What exists today (survey, 2026-11-01)

- **No server-side validation.** The server has one server fn, `get_game_data`. A Build never reaches the backend: the URL fragment is the Build (ticket 13), and `rules` runs in the browser.
- **One check, used twice.** `rules::engine::check_pick` decides whether a pick fits a slot at a point of the Build. It runs:
  1. in `Engine::options` (and `class_options`, `switch_options`): which options the picker enables, with the reason a disabled one shows ("needs Power Attack");
  2. in the walk, over every pick already in the Build, producing `Sheet.problems`. Those feed `Engine::validate` (the stale-link banner, ticket 13 Q9) and `Engine::would_break` (an edit to an earlier choice that would break a later pick is disabled, ticket 12), and a broken pick doesn't count as held.
- `Issue` has 18 variants (`rules/src/problem.rs`). Some are reasons a picker shows (`Unmet`, `AlreadyHeld`, `MaxTimes`, `ClassLevelCap`, `NotFromPath`). Others only arise from a Build the UI can't produce: `NotAllowed`, `NoSuchSlot`, `ParameterMissing`, `WrongParameter`, `LevelCap`, `SkillRankCap`, `SkillPointsOverspent`, `PointBuy`, `ScoreOutOfRange`, `NotChoosable`, `NoAbilityIncrease`, `UnknownKey`.
- A Build the UI can't produce enters only by a link: a stale one (the game data changed since) or a hand-edited one.

## Decisions

**User, 2026-11-01:**

1. **Changing an earlier pick: remove from the leaves inward.** A pick shows a remove ("×") control only when no later pick depends on it. With Power Attack → Cleave → Cleaving Finish, only Cleaving Finish shows ×; once it's removed, Cleave shows ×, and so on. No edit is offered that would break a later pick, so there's no "Would break: …" message and the Build the UI holds is always possible. Read generally (my reading, not the user's words): every other choice a later pick depends on (the class at a level, an archetype, a point-buy score, a skill rank, the path) offers no change or remove control until its dependants are gone. The engine answers "may this choice be removed/changed" per choice; it doesn't list what would break.
2. **Stale links keep the banner** (ticket 13 Q9 stands). A link whose game data changed opens as is, with broken picks marked with their reason and removed keys shown as "removed from game data". A broken pick always shows ×, whatever depends on it, since removing it is how the player fixes the link.
3. **Error reasons only an impossible Build can produce go.** Where it's cheap, make the bad state unrepresentable in the types (for example a point-buy score, or a parameterised pick that must name a value of the right kind), with the link decoder refusing a link that can't be read into those types. Keep the reasons a disabled option shows (unmet prerequisites, already held, max times, class maxed out, path switch not open from the Build's path).

**How 2 and 3 fit together** (mine, so the agent should check it): since stale links still load, the reasons a *game-data change* can cause stay too, because the banner shows them (a key gone, a prerequisite changed, a feat leaving a slot's list, a class table shortened). What goes is what only a *hand-edited* link or a buggy UI could produce: a score outside 7..=18, an overspent point buy, overspent skill points or ranks above the cap, a level above the cap, an ability increase at a level without one, a value of the wrong kind or a missing one. Those become unrepresentable types, a decode error ("this link can't be read"), or simply aren't checked. List each `Issue` variant in the Answer with what happened to it and why.

## Build

AFK, test-first (`tdd`), in `rules` (and the codec); no schema or game-data change is expected. Only agent in the repo.

- **Removable choices.** Replace `would_break` (a list of broken later picks for an edit) with what the UI needs: whether each choice in a Build may be removed or changed now. A pick is removable when no later pick depends on it (it is a leaf), or when it is itself broken. Prefer a dependency answer that doesn't walk the whole Build once per choice, if that stays simple; otherwise correctness first, and note the cost.
- **The walk** keeps checking picks only for what a stale link can contain (decision 3), so the banner still works; a broken pick still doesn't count as held.
- **Issues and types** per decision 3.
- **Tickets 12 and 13:** update their text to match (ticket 12's "disabled with Would break: …" line becomes the × rule; ticket 13's stale-link paragraph stays, plus "a broken pick always shows ×").
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: in the Power Attack → Cleave → Cleaving Finish chain only Cleaving Finish is removable, then Cleave, then Power Attack; a class level, archetype or point-buy score a later pick depends on isn't changeable; a stale link's broken pick is removable even with dependants; a hand-edited link with an out-of-range value doesn't decode into a Build; every removed `Issue` variant is gone from the code and tests.

## Progress log

- 2026-11-01: started. Baseline: HEAD 4beeb8a, clean tree; scratch in `t39/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t39/`), `old-src/` is `git archive HEAD`. Checked the "How 2 and 3 fit together" paragraph against the code, variant by variant, asking whether a game-data change can produce each one (then a stale link holds it and the banner needs it). It holds for the point buy (score range and budget), ranks above the character level and an ability increase off every 4th level: all are rules in code that no game data changes. It doesn't hold for four variants it lists as going: `LevelCap`, `SkillPointsOverspent`, `ParameterMissing` and `WrongParameter` (see Questions). Those are kept as they were, and the question is written below rather than guessed. `NotChoosable`, which the survey lists as UI-impossible, is game-data reachable too (a path gains or loses a switch rule), so it stays under the paragraph's own rule.
- 2026-11-01: a gap in the Build section: filling an earlier empty slot, or replacing a pick, can break a later choice too (Toughness at 1 while level 3 holds it; a `Not`, an option group, a selection only without an option, option-row slot indices shifting). `would_break` used to cover that per tapped option. Decided: the pickers lock such an option with a new picker reason, `Issue::BreaksLater`, and `Engine::allows` answers any other edit.
- 2026-11-01: tests first. `removable.rs` (12, replacing `would_break.rs`), 2 in `edits.rs`, 1 in `codec.rs` and 1 codec unit test, the point-buy test in `validation.rs`; `backgrounds.rs`, `bonus_feats.rs` and `parameters.rs` rewrote their would-break tests as removability. Red runs: `t39/red-compile.txt` (nothing compiles: no `PointBuy`, `removable`, `allows`, `BreaksLater`, `Choice::AbilityScore`), `t39/red-run.txt` (the types in place, `removable` and `allows` stubbed: 15 failures), `t39/red-fixed-rules.txt` (the fixed rules switched off: the 5 codec, edit and point-buy tests fail).
- 2026-11-01: built. `PointBuy`, `Build::keeps_fixed_rules` (decoder and `Build::apply`), `Engine::removable` and `Engine::allows` over one paused walk (`Recheck`), `BreaksLater` in `options`, `class_options`, `path_options` and `switch_options`; `would_break` and 4 `Issue` variants gone. Green: rules 288 + unit 2. One existing test changed its expectation (`path_rules::gold_dragons_dragon_feats_…`: it now asks the second Dragon Feat slot, since every other feat in the first would break Cleaving Finish in the third, and Cleaving Finish is `BreaksLater` there).
- 2026-11-01: proof, docs, Verification. Proof (`t39/proof-{old,new}` on one shared `t39/proof-common.rs`, `t39/compare.py`): 445 Builds, both outputs byte-identical (11862 lines). Docs: `GLOSSARY.md` (Problem; Removable replaces Would-break), tickets 12 and 13. No schema, game-data or link-format change, so no DB reset and the v1 vector stays. Verification below.

## Answer

Done 2026-11-01, uncommitted. The sheet can now hold only possible Builds: the engine says which choices may be removed or changed (from the leaves inward), the pickers lock what would break a later choice, and the rules no game data changes are kept by the types, `Build::apply` and the link decoder instead of being reported. Stale links still open with their problems. Four `Issue` variants are gone, one picker reason is new, and four that the "How 2 and 3 fit together" paragraph lists as going are kept, because a game-data change can produce them (Questions).

### Removable choices

- `Engine::removable(build) -> BTreeSet<Choice>` replaces `would_break`. It returns the choices the sheet may remove or change now: those whose taking-back breaks no other choice, plus every broken choice (a broken pick always shows ×, whatever depends on it). In Power Attack → Cleave → Great Cleave → Cleaving Finish only Cleaving Finish is removable, then Great Cleave, then Cleave, then Power Attack.
- Taking a choice back means: a pick cleared; a level taken by no class (its row's slots and grants go, and later levels of its class move down one); a class's archetype or advanced classes unchosen; a point-buy score lowered to 7; an ability increase or a skill's ranks at a level removed; the race (with subrace and racial ability), subrace, alignment, background, path or Path switch unchosen. A level's own skill points don't count against changing its class; the class picker locks classes with too few. `Choice::AbilityScores` became `Choice::AbilityScore { ability }`, so each score has its own answer.
- `Engine::allows(build, edit) -> bool` is the general rule: an edit is allowed when every choice that holds up still holds up, apart from the choices it sets. Choices a stale link already broke don't count. This is `would_break(..).is_empty()` without the list.
- **Pickers:** `options`, `class_options` (at a level the Build has), `path_options` and `switch_options` give an option that is open at its point but would break a later choice `Err(Issue::BreaksLater)`. Examples: Toughness at level 1 while level 3 holds it, anything but Cleave in Cleave's slot while Great Cleave needs it, wizard at fighter 4 while its bonus slot holds Great Cleave, Gold Dragon for a Build past level 20. A level after the last breaks nothing, so the next level's class picker isn't checked.
- **Cost (the removability answer):** one walk, paused before every level (`Recheck`: a `Walker` clone per level), then for each choice or option a walk from the first level its edit changes. The Build start resumes from level 0. This is still one walk per choice, though a shorter one, and not a dependency graph. Tracking which later check reads which earlier grant would mean provenance for every state value (BAB, scores, option levels, slot indices), and it isn't simple. Measured on the real data, native release (`t39/bench/`):

  | | 20-level fighter, 32 picks | fighter 20 + rogue 20 on Legend, 54 picks |
  |---|---|---|
  | `evaluate` | 0.05 ms | 0.15 ms |
  | `removable` | 2.5 ms | 15 ms |
  | `options` at level 1 (885 candidates) | 9 ms | 27 ms |
  | `options` at the last level | 1.8 ms | 2.1 ms |
  | `class_options` at level 1 | 11 ms | 33 ms |

  Expect these to be several times slower in the browser's wasm.

### Fixed rules: types, edits and the decoder

- **`PointBuy`** (in `BuildStart.ability_scores`) holds only scores from 7 to 18 costing at most `POINT_BUY_BUDGET` (25; it moved to `build.rs`). It is built with `TryFrom<AbilityScores>` or `with(ability, score) -> Option<Self>`, and has `get`, `scores` and `cost`. Its serde form is `AbilityScores`, so the link format is unchanged and `V1_LINK` still decodes without being regenerated. A link with a score of 19 or 34 points fails with `DecodeError::Format`.
- **`Build::keeps_fixed_rules`:** a skill's ranks never exceed the character level, and an ability increase sits only on every 4th level. The decoder refuses a link that breaks either (`DecodeError::Format`, now "doesn't describe a Build the planner can make"). `Build::apply` does nothing for an edit that would break either.
- The walk no longer checks these rules. It reads only an increase on a level that has one.

### Issue variants

| Variant | Now | Why |
|---|---|---|
| `UnknownKey` | kept | Stale: a key removed from the game data ("removed from game data" on the banner). |
| `Unmet` | kept | Picker reason. Stale: a prerequisite changed. |
| `AlreadyHeld` | kept | Picker reason. Stale: a class row or background starts granting a feat a pick takes, or a selection starts sharing an option. |
| `MaxTimes` | kept | Picker reason. Stale: a feat's or option's times lowered. |
| `ParameterMissing` | kept (Q1) | Stale: a feat gains a parameter, as ticket 27 did. Which feats take which kind of value is game data, so no type can hold it, and the decoder never sees game data. |
| `WrongParameter` | kept (Q1) | Stale: a feat's kind changes, an option's value list shrinks, or a weapon becomes exotic (ticket 32's Sawtooth Saber). Same reason as above. |
| `NotAllowed` | kept | Picker reason. Stale: a feat leaves a slot's list, an option leaves a selection, or a class stops casting what a prestige class advances. Its hand-edit-only cases (a pick of the wrong kind for its slot, a subrace or archetype under another owner) share the variant and cost nothing. |
| `NoSuchSlot` | kept | Stale: a class or option row loses a slot. |
| `LevelCap` | kept (Q1) | Stale: Legend's cap is lowered, or its switch is no longer taken. It also locks taking back a switch that the levels past 20 need. |
| `ClassLevelCap` | kept | Picker reason. Stale: a class table is shortened. |
| `SkillRankCap` | **gone** | The cap is the character level, a rule in code. `Build::apply` refuses ranks above it, and the decoder refuses a link with them. |
| `SkillPointsOverspent` | kept (Q1) | Stale: a class's or archetype's skill points, or an Intelligence modifier, changed. It also locks lowering Int, or changing a level's class to one with fewer points. |
| `PointBuy` | **gone** | The `PointBuy` type holds only scores that fit, and a link with another doesn't decode. |
| `ScoreOutOfRange` | **gone** | Same as `PointBuy`. |
| `NotChoosable` | kept | Stale: a path gains or loses a switch rule. The survey had it as UI-only, but game data can cause it. |
| `NotFromPath` | kept | Picker reason. Stale: a switch's `from` changes. |
| `NoAbilityIncrease` | **gone** | Every 4th level is a rule in code. `Build::apply` refuses it, the decoder refuses it, and the walk reads only an increase on a level that has one. |
| `BreaksLater` | **new** | A picker reason only, never a Build's problem. |

### Tests

- **Removed:** `would_break.rs` (7 tests: Cleave chain breaks, the reason at the broken pick, class and archetype changes, the prestige entry, the edited choice itself, the picker asking per option). Also `validation::skill_ranks_are_capped_at_the_character_level` (`SkillRankCap`) and `validation::the_point_buy_must_fit_the_budget` (`PointBuy`).
- **Rewritten as removability:** `backgrounds::a_background_whose_feat_a_pick_needs_is_not_changeable`, `bonus_feats::a_bonus_list_pick_never_depends_on_anything` and `parameters::weapon_focus_stays_while_the_specialization_in_its_weapon_needs_it` (Weapon Focus (Dagger) is `BreaksLater`).
- **Changed:** `path_rules::gold_dragons_dragon_feats_…` now asks the second Dragon Feat slot, and Cleaving Finish there is `BreaksLater`. Six tests set scores through `PointBuy::with` or `point_buy([..])`.
- **New:**
  - `removable.rs` (12): the chain from the leaves inward; class levels; the archetype; point-buy scores; skill ranks for Arcane Trickster; the Legend switch past 20; a stale link's broken pick; the option, replacement, class and switch pickers; `allows` with Int and skill points.
  - `edits.rs` (2): increases only on every 4th level, ranks never above the level.
  - `codec.rs`: ranks and increase links don't decode. The proptest now makes possible Builds only.
  - A codec unit test: a hand-edited point buy doesn't decode.
  - `validation::a_point_buy_holds_only_scores_from_7_to_18_within_the_budget`.
- **Counts:** rules 281 + unit 1 → 288 + unit 2.

### Proof

`t39/proof-old/` (HEAD 4beeb8a) and `t39/proof-new/` build one shared source, `t39/proof-common.rs`.

- **Builds.** `proof-old gen` writes 445 Builds as links. They are ticket 38's 413, plus 32 filled ones: every base class to 20 on a path, and six to 40 on Legend. Each filled Build has a Mobility rank a level, +1 Str every 4th level, and every slot filled with the first open option that keeps it problem-free.
- **What each side prints.** Per Build: problems, the removable choices and the path and switch pickers. Per level: each slot's options (open, breaks or locked), the class picker and the state. Per rank: its slots. Then the next level's class picker.
- **How the old side answers.** It derives everything from the old engine's `would_break` rule and the withdrawals above.
- **Result.** `compare.py`: the two outputs are byte-identical, 11862 lines.
- **What it exercised.** 8462 option keys `BreaksLater`, 229410 class-picker options `BreaksLater`, 216 path or switch options `BreaksLater`, 11474 removable choices (1355 picks, 5364 of 6522 class levels), and 74 Builds with problems.

This shows two things. Resuming from a paused walk gives the same answers as walking each edited Build whole. And apart from the new statuses and removability, nothing in the walk changed.

### Docs

- `GLOSSARY.md`: **Problem** says only a stale link has them and an impossible link doesn't open. **Removable** replaces **Would-break**.
- Ticket 12's picker line now covers the × rule and `BreaksLater`; its skill-ranks line covers `Build::apply`. Ticket 13's stale-link paragraph gains "a broken pick always shows ×" and a line on hand-edited links.

### Decided

1. **The pickers lock options that would break a later choice** (`Issue::BreaksLater`). The Build section only asked for removability, but filling an earlier empty slot or replacing a pick can break a later one. Without the lock, the sheet could still make an impossible Build.
2. **What "taking back" means for choices that aren't picks** (the list above). A score goes to 7. A level becomes no class, ignoring its own skill points. The race goes with its subrace and racial ability.
3. **`Engine::allows` is public**, so the sheet can check any other control (a score's +/−, a skill's ranks, the background) without a list of what would break.
4. **The hand-edit-only cases of `NotAllowed` stay** under the kept variant rather than becoming a new path.

### Verification

- `cargo test --workspace`: pass (rules 288 + unit 2, extract 88, data files 25 + game_data 119 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 39 backgrounds).
- No schema or game-data change, so no DB reset. The link format is unchanged, so `V1_LINK` stays and still decodes.
- The removed variants (`SkillRankCap`, `PointBuy`, `ScoreOutOfRange`, `NoAbilityIncrease`) and `would_break` appear nowhere in the code or tests.
- Proof: passes (above).

### Questions

1. **Four reasons the paragraph drops can come from game data.** "How 2 and 3 fit together" lists a level above the cap, overspent skill points, and a missing value or one of the wrong kind as things only a hand-edited link produces. A game-data change produces each of them:
   - `LevelCap`: Legend's cap is lowered, or its switch is no longer taken.
   - `SkillPointsOverspent`: a class's skill points or an Int modifier changes.
   - `ParameterMissing`: a feat gains a parameter, as ticket 27 did.
   - `WrongParameter`: a feat's kind or an option's values change, or a weapon becomes exotic, as in ticket 32.

   Decision 3's example, "a parameterised pick that must name a value of the right kind", can't be a type: which kind a feat takes is game data, and the decoder never sees game data. They are kept as stale-link reasons. Should they stay on the banner, or be left off? The walk must still check `LevelCap` and `SkillPointsOverspent` either way, because they are what lock taking back the Legend switch and lowering Int.
2. **Picks of unreached ranks.** Remove level keeps the removed level's mythic picks (ticket 14). A Legend switch also ends the ascensions and leaves the picks of MR 9 and 10 in the Build. The walk never checks such picks, so removability can't see what they depend on. If the level is added back or the switch is taken back, they may come back broken; they then show as broken, with ×. Should Remove level, and a switch that ends the ascensions, drop them instead? That needs the game data's ascension table, so it would be an engine call rather than `Build::apply`.
