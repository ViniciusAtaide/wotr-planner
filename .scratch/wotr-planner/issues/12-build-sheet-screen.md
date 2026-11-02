# Build the planning sheet screen

Type: task
Status: resolved
Blocked by: 09, 10, 11, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42
Map: [WotR Build Planner](../map.md)

## Question

Build the Leptos screen chosen in [Level-up screen behaviour](06-level-up-screen.md) (variant C, Sheet) on top of the real `GameData` and rules engine. The [prototype](../prototypes/level-up-screen/index.html) is the reference for behaviour only: rewrite it properly and don't port it.

- **Build start:** race, alignment and point buy, ahead of the grid.
- **Levels grid:** levels 1–20 (1–40 on Legend). A class button opens the class/archetype picker; prestige entry asks once which spellcasting class it advances. There's one chip per slot from the rules engine.
- **Mythic track tab:** MR 1–10 at their fixed levels, the path at MR 3, the Path switch at MR 8. Ranks never reached show as such. A rank a switch drops or partly drops marks the picks it lost (`RankSheet::dropped_slots`): they stay in the Build, held until the switch, and an empty slot there still has a picker ([ticket 42](42-keep-a-switchs-earlier-picks.md)).
- **Picker:**
  - a bottom sheet below 600px and a modal above
  - search and category chips
  - locked options hidden behind "Show N locked", with reasons
  - a pick shows a remove ("×") control only when no later pick depends on it, so a chain is removed from the leaves inward (Cleaving Finish, then Cleave, then Power Attack); class changes and other earlier choices with dependants offer no change control (`Engine::removable`). A picker locks an option that would break a later choice (`Issue::BreaksLater`: Toughness at level 1 while level 3 holds it), and any other control asks `Engine::allows` for its edit, so the sheet only ever holds a possible Build ([ticket 39](39-impossible-states-not-validation.md)). Every edit goes through `Engine::apply`, which also drops the picks of Mythic ranks the Build no longer reaches (Remove level, a Legend switch: [ticket 40](40-stale-links-generic-error.md)) and the path and switch once their ranks aren't reached ([ticket 41](41-drop-what-a-switch-takes-away.md)); the picks a switch takes away at earlier ranks stay ([ticket 42](42-keep-a-switchs-earlier-picks.md))
- **The sheet is the summary** (user, 2026-10-31): there's no separate summary screen. A finished Build's summary is the same grid and Mythic track, showing class, feats, mythic feats and abilities up to level 20 (40 on Legend). A shared link opens on the same sheet.
- **Skill ranks:** per level, capped at character level (`Build::apply` refuses ranks above it, and an ability increase off every 4th level).
- **State:** the Build lives in the URL fragment (`/#b=…`) and is rewritten with `history.replaceState` on every edit. `/` with no fragment is a blank Build. A link opens through `Engine::open_link`: one that doesn't decode, or whose Build the current game data can't hold (a stale or hand-edited link), shows one generic error ("This link no longer works with the current game data") and a blank Build, with no banner, marks or reasons ([ticket 40](40-stale-links-generic-error.md)). The rules engine runs in the browser on `GameData` loaded once.
- **Fold 7:** check it at cover and inner sizes; tap targets at least 44px.

AFK: driven by the agent. End-to-end smoke tests come later (see the map).

## Progress log

- **2026-11-02, rules labels (TDD):** `rules/src/engine/labels.rs` gives the sheet its words: names by key (falling back to the key), `pick_name` with the value ("Weapon Focus (Longsword)"), `slot_label`, `pick_category`, `pick_summary`, `is_prestige`, and `issue_text` / `unmet_text` / `prerequisite_text` ("Needs Power Attack, BAB +4 (have +1)"); `Ability`, `Alignment` and `FeatCategory` names, `Alignment::ALL`, `ParameterKind::noun`. The engine keeps a feature-name map. 11 tests in `rules/tests/engine/labels.rs`, 2 real-data tests in `data/tests/game_data.rs`.
- **Slice 1, GameData + Engine in the browser, URL, Build start:** `SheetPage` loads `get_game_data` once through a `LocalResource` (SSR renders "Loading the game data…", so hydration matches) and builds the `Engine` in the browser. `Planner` holds the Build in a signal, the `Sheet` and `removable` as memos (once per edit), and every edit goes `Engine::apply` → `history.replaceState` (`/` for a blank Build, `/#b=…` otherwise). The fragment opens through `Engine::open_link`; a failure (or any other fragment) shows the one generic error and a blank Build; a pasted link (`hashchange`) opens the same way. Build start: race/subrace, "+2 to" for a chosen racial modifier, alignment, background (with its note), point buy with −/+ per score (PointBuy::with and `Engine::allows`). Checked headless at 412×920: `s1-*.png`.
- **Slice 2, levels grid + class picker:** one row per level (class button, a chip per slot, ability increase, skills, ✦ MR badge), "+ Level N" up to `Sheet::level_cap` and "Remove level N" (`allows(RemoveLevel)`). The class button opens the class/archetype picker (`Engine::class_options`) only while `removable` has `Choice::Class`; a class first taken at the level gets the picked archetype. A prestige class whose rows advance spellcasting, first taken at the level, asks which held classes it advances (the choices for which the edited Build `is_possible`, and None). `s2-*.png`.
- **Slice 3, pickers, locks and ×:** a slot's picker computes `Engine::options` after the picker has painted, groups a parameterised feat's values under one entry that opens a value list, files options under category chips (feat category, defining selection, mythic group), hides locked ones behind "Show N locked" with each `Issue` as a reason, shows Story conditions as a note, and offers "× Remove" only for a pick `Engine::removable` gives. `s3-*.png`.
- **Slice 4, skills and ability increases:** the "Skills a/b" chip opens a per-level panel (class skills marked, total and this level's ranks; + needs points left, a Build `Build::apply` changes, and `allows`; − needs `allows`). The "Ability" chip at every 4th level picks one of six (`allows`), × when removable. `s4-*.png`.
- **Slice 5, Mythic track:** MR 1–10 with their level and path; the path chip at MR 3 (`path_options`) and the switch chip at MR 8 (`switch_options`), enabled once the Build has the rank's level; Partly lost / Lost / Not reached badges from `RankStatus`; picks in `dropped_slots` are struck through and tagged "lost", and every slot keeps its picker. `s5-*.png`.
- **Slice 6, Fold sizes and polish:** cover 412×920, inner 840×930 and desktop 1280×900 (dark), bottom sheet below 600px and centred modal above, unfolding with a picker open keeps it and its search (`s6-fold-*.png`), no visible button under 44px tall, type-erased views so the release build compiles, release timings measured.

## Answer

The Sheet is built: `app/src/` (`planner.rs` state and page, `start.rs` Build start and skill ranks, `grid.rs` levels grid and summary, `mythic.rs` Mythic track, `picker.rs` every picker, `browser.rs` fragment and timings) and `style/main.css` (tokens, light and dark). Run it with `cargo leptos watch` (dev DB `wotr-planner-postgres` on 5434) and open http://localhost:3900.

### What's built

- **Build start** ahead of the grid, open while the Build has no levels, folded to one line after: race and subrace, the "+2 to" ability for a race that chooses it, alignment, background and its note, point buy (scores 7–18, points left, racial modifier and total).
- **Levels grid**: a row per character level with the class (and archetype on its first level), one chip per slot of `LevelSheet::slots` labelled by what it takes, the ability increase, skill points spent/available and the Mythic ranks reached there; a summary above it (classes, BAB, saves, scores, class skills, daily choices as a note). Up to 20 levels, 40 after a Legend switch.
- **Mythic track** tab: MR 1–10 at their levels, path at MR 3, switch at MR 8, the ranks' status, lost picks marked.
- **Picker**: bottom sheet below 600px, centred modal from 600px; search (more than 8 entries), category chips (more than one category), "Show N locked" with readable reasons, a value list for parameterised feats and options, "× Remove" only for a removable choice. Escape and the backdrop close it.
- **Every edit** goes through `Engine::apply`; the pickers' options carry the engine's statuses (`options`, `class_options`, `path_options`, `switch_options`); every other control asks `Engine::allows` (Build start options, point-buy steps, racial ability, ability increases, skill ranks, Remove level). The class button offers no change while a later choice depends on the class (🔒).
- **URL**: rewritten with `history.replaceState` on each edit (no history entries; checked), `/` for a blank Build, links through `Engine::open_link` with the one generic error; a link pasted into the same tab opens on `hashchange`.

### Performance (headless Chromium on the dev Mac, `performance.now`, ms)

| | dev wasm | release wasm |
|---|---|---|
| `Engine::new` (once) | 14–16 | 3.3 |
| `evaluate` (per edit) | ≤ 2.8 | ≤ 1.1 |
| `removable` (per edit, 20 levels) | median 6.4, max 17 | median 1.1, max 3.4 |
| `removable` (40-level Legend Build) | | median 5.2, max 6.8 |
| `options` (feat slot, 20 levels) | 17–23 | 3.2–4.3 |
| `options` (level 1 slot of a 40-level Build) | | 24–28 (picker listed 62 ms after the tap) |
| `class_options` | ≤ 28 | ≤ 6.2 |
| `allows` | ≤ 1.9 | ≤ 0.6 |

Opening a 20-level link and showing the sheet took ~90 ms locally. Nothing is too slow for a tap, so the engine is unchanged. Sizes: release wasm 1,437,912 bytes (546,434 gzipped); the game data is 1,486,874 bytes of JSON (111,868 gzipped), served uncompressed.

### Decided

- **× lives in the picker's header** ("× Remove"), not on the chip: a chip stays one 44px tap target and the dense grid stays readable at 412px; it shows only for a choice `Engine::removable` gives. The Build start's choices, the ability increase, the path and the switch use the same header control.
- **Changing an archetype** is the class picker at the class's first level, so it needs that level's class to be changeable (removable); no separate archetype control.
- **Prestige advancement prompt**: offered after picking a prestige class first taken at the level whose rows advance spellcasting; it lists only the choices for which the edited Build is possible (one class, two, or None), since the walk gives no reason for the others.
- **Options with values**: one entry per feat or option ("Weapon Focus ›"), opening its values; it's locked when no value is open, with a reason other than "already taken" when any value has one.
- **Locked Build start options and steps** show "Would break a later choice": `allows` gives no reason.
- **Skill rank +** also needs points left at that level (the walk accepts no overspent level, and `allows` doesn't check the edit's own level).
- **A fragment that isn't `#b=…`** is a stale link (the generic error), and the error clears on the next edit.
- **The Build start** opens by default only while the Build has no levels.
- **Timings** are recorded only when the page defines `window.__wotrTimings` (the smoke scripts do); nothing else reads them.

### Screenshots

All in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t12/`: `s1-blank-cover.png`, `s1-race-picker-cover.png`, `s1-background-picker-cover.png`, `s1-start-cover.png`, `s1-stale-link-cover.png`; `s2-grid-cover.png`, `s2-class-picker-cover.png`, `s2-advances-cover.png`, `s2-grid20-cover.png`, `s2-grid-inner.png`; `s3-feat-picker-cover.png`, `s3-locked-cover.png`, `s3-values-cover.png`, `s3-grid-cover.png`, `s3-locked-inner.png`, `s3-*-inner.png`; `s4-skills-cover.png`; `s5-mythic-cover.png`, `s5-switch-picker-cover.png`, `s5-legend-cover.png`, `s5-legend-inner.png`, `s5-legend-desktop.png` and `s5-switch-picker-desktop.png` (dark); `s6-grid-cover.png`, `s6-grid-inner.png`, `s6-40-levels-release.png`, `s6-40-levels-picker-release.png`, `s6-fold-before-cover.png`, `s6-fold-after-inner.png`. The smoke scripts that made them are `smoke1.js`–`smoke5.js` there.

### Known gaps

- "Would break a later choice" never names the later choice (the engine doesn't say which); the prototype's "Would break: Cleave (L1)" would need an engine addition.
- The game data goes over the wire uncompressed (1.5 MB; 112 KB gzipped). Fine on the LAN; a compression layer on the server would fix it.
- No copy-link or new-Build button: the address bar is the link, and `/` is a blank Build.
- The value list of Exotic Weapon Proficiency says "Choose a weapon" (the pick's value can't tell an exotic weapon from any weapon).
- No end-to-end tests yet (the map's later ticket); the smoke scripts above are throwaway.
- Feats have no summaries in the data, so feat entries show only their names.

### Verification

- `cargo test --workspace`: 568 passed (rules engine 307 with 11 new label tests, data game_data 121 with 2 new).
- `cargo clippy --workspace --all-targets`: clean; also clean for `app` with `hydrate` on wasm32 and with `ssr`.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 39 backgrounds.
- `cargo leptos build --release`: pass; wasm 1,437,912 bytes (546,434 gzipped).
- No schema, game-data or `Build` change; the dev DB was not touched.
