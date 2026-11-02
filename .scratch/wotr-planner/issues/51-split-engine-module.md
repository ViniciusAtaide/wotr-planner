# Split engine.rs into submodules

Type: task
Status: resolved
Blocked by: 47 (and the lead's test-only-helpers change that follows it)
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02** (asked from a forked session): `rules/src/engine.rs` is about 3,000 lines; organise it the way `rules/src/engine/labels.rs` already is: a child module holding a plain `impl Engine { … }` block. **No traits**: there is one engine type and nothing to swap.

Suggested split, each file its own `impl Engine` block (adjust where the code's real seams differ, and say so in the Answer):

- `engine/lookups.rs`: `Engine::new` and the lookups by key;
- `engine/walk.rs`: the `Walker` and `evaluate`;
- `engine/pickers.rs`: `options`, `class_options`, `path_options` and `switch_options`;
- `engine/removable.rs`: `removable`, `allows` and `Recheck`;
- `engine/switch.rs`: the path-switch rules.

**Also (user, 2026-11-02): break up `Engine::new`.** It builds about 20 lookup tables inline (key → position maps for classes, archetypes, races, subraces, selections, options, feats, skills, weapons, paths, backgrounds, mythic abilities and feats; feature facts: bonuses, option rows, daily choices, floors, names, shared names; `LinkKeys`). Move them into an `Indexes` struct (or a few) built by one small function per group (`index_classes`, `index_races`, `index_features`, …), in `engine/lookups.rs`, so `new` is about ten lines: canonicalise the data, build the indexes, done. No dependency injection or traits: the engine takes plain `GameData` and does no I/O. Same lookups, same behaviour.

`engine.rs` keeps the `Engine` struct, the public types it returns (`Sheet`, `LevelSheet`, `RankSheet`, `Slot`, `SlotOption`, …) unless they sit better beside their code, the module docs, and the `mod` lines. Private items shared between files become `pub(super)` (or `pub(crate)` where another module of the crate already uses them), never wider than needed.

## Build

AFK; the only agent in the repo. A **move-only refactor**: no behaviour change, no renamed public item, no new public item, the same doc comments moving with their code. Prefer moving code as whole items (`git diff --stat` and a reviewer should see moves, not rewrites). Run before [ticket 48](48-ui-performance.md), which reworks the walker and `Recheck`.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; the public API of `rules` is unchanged (compare `cargo doc`'s item list, or `cargo public-api` if available, before and after); no file under `rules/src/engine/` is a trait module; `engine.rs` is a short root module; the test counts are unchanged.

## Progress log

- 2026-11-02: baseline at HEAD `01daefd` saved in the scratchpad (`t51/`): rustdoc JSON public API with and without `--features testing`, `cargo doc` item list (122 pages, 0 warnings), `cargo test -p rules -- --list` (318 + 4 unit), and the ticket-47 proof walk with HEAD's `rules` (`t51/old-src`, `git archive HEAD`).
- 2026-11-02: `t51/split.py` cut the HEAD `engine.rs` into the new files by whole-item line ranges, so the moved code is the original text. Then the private field reads moved behind `indexes` / `features`, visibility was widened only where the compiler refused (`pub(super)`), the glob imports were replaced by explicit ones, and `cargo fix` + `cargo fmt` tidied them.
- 2026-11-02: the walker's shared types (`Held`, `GrantSource`, `RankGrant`, `Holding`) moved into `walk.rs`. As private items of the parent they're visible to `walk/*` without widening any field.
- 2026-11-02: verification passed (see Answer). Nothing committed.

## Answer

**Done.** `engine.rs` is now a 317-line root module. Each child module is a plain `impl Engine { … }` block, or `impl Walker { … }` under `walk/`. There are no traits, and `Engine::new` is 9 lines.

### Layout (lines)

| File | Lines | Holds |
| --- | ---: | --- |
| `engine.rs` | 317 | module docs, `mod` lines, `BASE_LEVEL_CAP` / `PATH_CHOICE_RANK` / `PATH_SWITCH_RANK`, the `Engine` struct (now `data`, `indexes`, `features`, `link_keys`), `MythicPickKind` + `mythic_pick_kind`, `EffectiveRow` + `effective_row` + `effective_rows`, `Sheet` / `LevelSheet` / `RankSheet` / `RankStatus` / `Slot` / `SlotKind` / `SkillPoints` / `SlotOption` / `ClassOption` / `PathOption`, `impl Sheet`, `with_bonuses`, `class_level_cap` |
| `engine/lookups.rs` | 525 | `new`, `data`, the lookups by key (`class`, `race`, `background`, `feat`, `weapon`, `feat_list`, `selection`, `option`, `mythic_ability`, `mythic_feat`, `path`, `archetype_of`, `row`, `casts`, `spontaneous`), the values (`parameter_values`, `value_fits`, `every_value`, `option_parameter`, `option_values`), `daily_choices`; `Indexes` + `index`, `index_classes`, `index_selections`, `index_options`, `index_races`, `index_feat_lists`; `FeatureFacts`; `ClassAt`, `SelectionAt` |
| `engine/slots.rs` | 320 | what a slot accepts: `offers`, `pools`, `slot_lists`, `slot_accepts_in`, `slot_ignores_in`, `single_option`, `slot_accepts`, `slot_ignores_prerequisites` (still `cfg(any(test, feature = "testing"))`), and the free `check_pick` + `check_repeat` |
| `engine/pickers.rs` | 251 | `options`, `story_conditions`, `class_options`, `class_status`, `path_options`, `switch_options` |
| `engine/removable.rs` | 267 | `allows`, `removable`, `withdrawals`, `first_level`, `Recheck`, `LOWEST_SCORE`, `no_class` |
| `engine/switch.rs` | 82 | `chosen_path`, `chosen_switch`, `apply`, `reaches`, `rank_level`, `ascension_level` |
| `engine/walk.rs` | 443 | `evaluate`, `probe`, `is_possible`, `open_link`, `state_before` (still `cfg`-gated); `Held`, `GrantSource`, `RankGrant`, `Probe`, `Probed`, `Walker`, `Holding`, `Walk`; the walk's own steps (`new`, `fail`, `run`, `finish`, `check_unreached`, the Build start `start` / `background`, and `level`) |
| `engine/walk/level.rs` | 510 | a Level-up's steps: `check_class_entry` … `fill_pending`, the Tables section (`recompute_tables`), `advancing_levels` |
| `engine/walk/options.rs` | 282 | the "Options with rows" section: `take_option` … `grant_option_row` |
| `engine/walk/hold.rs` | 320 | `fill_all`, `fill`, `check_pick`, `hold_pick`, `hold`, `release`, `removes_penalty`, `refresh_floors`, `refresh_ability_scores`, `add_feature` |
| `engine/walk/ascension.rs` | 228 | the "Mythic ascension" section: `ascend`, `grant_rows_before_the_choice`, `switch_path`, `switch_drops` |
| `engine/labels.rs` | 617 | unchanged except the field reads below |

### `Engine::new` and `Indexes`

`new` canonicalises the data and builds three things:

- `Indexes::new(&data)`: the key → position maps. One function builds each group: `index_classes` (classes, archetypes), `index_races` (races, subraces), `index_selections`, `index_options`, `index_feat_lists`, and the generic `index` for skills, weapons, feats, mythic abilities, mythic feats, paths and backgrounds.
- `FeatureFacts::new(&data)`: bonuses, option rows, daily choices, floors, names and shared names, from the same feature list as before.
- `LinkKeys::new(&data)`, as before.

`index_options` resolves each indexed selection through the new `SelectionAt::selection(data)`, which holds the match that `Engine::selection` had. `Engine::selection` is now one line over it, so the options map is built exactly as before, from the selection index.

### Departures from the suggested split, and why

1. **`slots.rs` is new.** What a slot accepts (offers, pools, lists, bonus lists, the one option) and `check_pick` are used by both the walk and the pickers, and they belong to neither. Putting them in `lookups.rs` would have made it about 850 lines.
2. **`switch.rs` is the engine side of the path and switch.** It holds which path and switch the Build takes, the ranks' character levels, and what `apply` drops. The walker's side of a switch (`switch_path`, `switch_drops`, `GrantSource`, `RankGrant`) stays with the walker in `walk/ascension.rs` and `walk.rs`, because it reads the walker's private state.
3. **The walker is split under `walk/`**, along the section comments it already had (Level-up and Tables, Options with rows, holding, Mythic ascension). A single `walk.rs` would have been about 1,700 lines. Those files hold `impl Walker` blocks, not `impl Engine` blocks.
4. **The walker's shared types live in `walk.rs`, not beside their section.** As private items of the parent, their fields stay private and are still visible to `walk/*`.
5. **Root-module helpers.** `with_bonuses`, `effective_row` and the other public functions stay in `engine.rs`, so no public item needs a re-export. `class_level_cap` and `effective_rows` also stay there, since several children use them.
6. **Two `Engine` fields instead of one `Indexes`.** The feature facts are a second struct (`features`) rather than more fields on `Indexes`, so a read is `self.features.bonuses` rather than `self.indexes.feature_bonuses`.

### What isn't a pure move

- **Field reads.** `self.<map>` became `self.indexes.<map>` or `self.features.<fact>`: 26 sites in the moved code (14 of them in `lookups.rs`) and 7 in `labels.rs`. The private facts dropped their prefix: `feature_bonuses` became `bonuses`, `shared_feature_names` became `shared_names`.
- **Visibility.** It was widened only where the compiler required it, always `pub(super)`:
  - Engine helpers: about 30, such as `class`, `feat`, `selection`, `option` and `probe`.
  - Walker steps and fields: `build` on `Walker`, `state` and `kind` on `Probed`, `failed` on `Walk`.
  - Index fields: 4 on `Indexes` (`archetypes`, `subraces`, `skills`, `options`) and 5 on `FeatureFacts`.
  - `check_pick` stays `pub(crate)` as it was.
- **New module docs** on each new file.
- **Two intra-doc links** now name their target's path, because their file no longer imports it:
  - `apply`'s `RankSheet::dropped_slots` now reads `(super::RankSheet::dropped_slots)`.
  - The walker's two `LevelFrom::GrantingOption` links now read `(crate::LevelFrom::GrantingOption)`. One of those lines is rewrapped.

  Rendered doc text is unchanged.
- **rustfmt reflow** where a longer path wrapped a line.

### API comparison

- **rustdoc JSON (`cargo rustdoc -p rules -- -Z unstable-options --output-format json`).** The scratchpad's `t51/api.py` lists every item reachable from `rules`' root: path, kind, signature (ids resolved to paths), fields, variants, and inherent and trait impl items. The lists before and after are **identical**:
  - 2,535 rows without features.
  - 2,540 rows with `--features testing`, which adds `state_before`, `slot_ignores_prerequisites`, `LinkKeys::of_build` and `Unmet::conditions`.
- **Public item docs.** The same listing with docs differs only in `Engine::apply`, and only in the link target above.
- **`cargo doc -p rules --no-deps`.** The `all.html` item list is the same 122 pages, with 0 warnings. `--document-private-items` also gives 0 warnings.

### Proof (ticket 47's walk)

The scratchpad's `t51/proof-old` and `t51/proof-new` are ticket 47's `proof-common.rs` and `main.rs`. `proof-old` is built on HEAD's `rules`/`data` (`t51/old-src`) and `proof-new` on the working tree. Each walked `game-data` in `walk` mode, which covers every class and archetype, the prestige Builds, the option, background, talent and follow-up Builds, and each slot's label.

Both outputs are 6,561 lines and **byte-identical** (sha1 `2609af9cfc87b046b94256f8570a53a011eebbf8`).

### Verification (ticket 22's checks)

- `cargo test --workspace`: pass.
  - rules: 318 + 4 unit, unchanged; the `--list` output is identical before and after.
  - data: files 25, game_data 150, postgres 26.
  - extract: 91.
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- The acceptance grep (`"(feat|skill|…):`) over `game-data`, `data/tests/fixtures`, `rules`, `extract` and `data/src` finds nothing.
- No DB change, so no reset or reseed.
- No file under `rules/src/engine/` declares a trait.
