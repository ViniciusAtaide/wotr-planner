# WotR Build Planner

Label: wayfinder:map

## Destination

A working, locally run Leptos app at `~/code/wotr-planner` for planning Pathfinder: Wrath of the Righteous builds. You build a character level by level, and each level offers only the choices the game would allow at that point: class, archetype, feats, mythic feats, mythic abilities, feature selections and skill ranks. The app ends with a summary up to character level 20, or 40 on the Legend mythic path. Builds live entirely in the URL, which updates as you edit and is what you share. It works well on a phone, specifically the user's Galaxy Z Fold 7: both the folded ~21:9 cover screen and the unfolded ~6:5 inner screen. The source of truth is the [GameFAQs guide](https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/introduction).

## Notes

- **This map includes building the app**, not just planning it. Tickets may build things, not only decide them.
- **Stack:**
  - cargo-leptos SSR on Axum, using a dated nightly pinned in `rust-toolchain.toml`.
  - Postgres in docker compose; local only.
  - sqlx with typed serde structs.
- **Game data:**
  - The source of truth is checked-in data files extracted once from the guide and corrected by hand.
  - A seed step loads them into ordinary relational tables, prerequisites included (no JSONB).
  - Store prerequisites, a short summary in our own words and a link to the guide, never the guide's full text.
  - When the game and the guide disagree, the guide wins.
  - **Never fetch GameFAQs or the fextralife wiki automatically.** Both robots.txt files disallow ClaudeBot. Agents parse only the pages the user saved to `guide-cache/`, which is git-ignored.
  - Include all DLC content, with no DLC gating. Prestige classes are in scope.
  - Committing extracted facts is OK.
  - **One migration while prototyping** (user, 2026-10-31). Schema changes edit the initial migration, then the dev database is dropped and reseeded; there are no fix-up migrations. Only one agent at a time does this. The steps are in the repo's `CLAUDE.md`.
  - **No derived values in the database** (user, 2026-10-31). Anything computable from other columns or a simple rule belongs in Rust, not in a column. Examples: a `position` that equals `rank − 1`, or ability/feat counts that follow `rank % 2`.
- **Rules followed:** the game's intended, current (patched) behaviour. Known exploits are not modelled.
- **Extraction in stages** (user, 2026-11-01): [Extract the guide into data files](issues/10-extract-guide.md) is stage A, where the extractor writes `game-data/` and the seed passes. Then four parallel hand-fix stages, one per category: [feats](issues/17-handfix-feats.md), [base classes](issues/18-handfix-base-classes.md), [prestige classes](issues/19-handfix-prestige-classes.md), [mythic](issues/20-handfix-mythic.md). They validate without the database, and the main session seeds once. Then [Answer the schema questions from the real data](issues/21-schema-questions-real-data.md). Commit a checkpoint after each stage.
- **Order** (user, 2026-10-31): finish the data layer (saved guide pages, extraction, checking the data against the guide) before the UI. [Build the planning sheet screen](issues/12-build-sheet-screen.md) waits on [Extract the guide into data files](issues/10-extract-guide.md).
- **Rules:** the rules live in a plain Rust crate with no DB or Leptos code, running in the browser. The browser loads the game data once, filters choices immediately, and decodes and encodes the Build from the URL.
- **Mobile:** design mobile-first. The main targets are the Galaxy Z Fold 7 cover screen (folded, tall and narrow, ~21:9) and inner screen (unfolded, nearly square, ~6:5). Desktop is secondary. Both Fold screens must work for touch: no hover-only interactions, and tap targets large enough for a thumb. The layout should adapt when the phone is folded or unfolded without losing the user's place.
- **Builds and links:** no accounts and no saved-build table. The URL fragment encodes every choice, updates live, and is the share link. `/` with no fragment is a blank Build.
- **Prerequisites checked:**
  - race, ability scores (point-buy and the +1 every 4 levels)
  - BAB
  - caster level, only where the guide states it
  - spell-level access (for example "can cast 3rd-level arcane spells"), which is a different check from caster level
  - skill ranks
  - alignment
  - other feats, class features
- **Facts confirmed by the user** (not researched):
  - Characters gain a feat at every odd character level.
  - The Legend path raises the level cap to 40.
- **Skills to use:**
  - `tdd` for the rules crate and the guide extraction; screens get end-to-end smoke tests only.
  - `grilling` + `domain-modeling` for design tickets; keep [GLOSSARY.md](../../GLOSSARY.md) current.
- **Tracker:** local markdown. Tickets are in `issues/`; research write-ups are in `research/`.

## Decisions so far

<!-- one line per closed ticket: [title](link): gist -->
- [How the guide is laid out](issues/01-guide-anatomy.md): about 38 of 70 pages hold build data. The HTML is easy to parse, with tables for progressions and prose entries with `Prerequisites:<br>` lines (free text inside, so hand-correct them). Don't crawl it (robots/ToS); the user saves the pages by hand and we parse them offline.
- [Leptos and toolchain setup](issues/02-leptos-toolchain.md): Leptos 0.8.23 + cargo-leptos 0.3.11 + sqlx 0.9 on pinned `nightly-2026-10-30` with wasm32 (build verified). Workspace follows start-axum-workspace: app/frontend/server plus rules (both targets) and data (sqlx, native only, reached only via `app/ssr`). The pool goes through `leptos_routes_with_context`, migrations run at startup, `data/.sqlx` is committed for offline builds.
- [Filling the guide's gaps](issues/07-guide-gaps.md): prestige classes, path-specific mythic picks, alignment and all DLC are in scope. Combat feats are every non-class/racial/skill/teamwork/magic feat. Spell-level access is a separate check from caster level. Feats with no listed prerequisites have none. The skill rank cap is 20, or 40 on Legend. Facts from the user go straight into the data files, unmarked, and committing extracted facts is OK.
- [How a character progresses](issues/04-progression-model.md): A Build is a Build start (race, alignment, point buy), then Level-ups to 20 (40 on Legend), then Mythic ascensions at fixed levels 6/7/9/11/14/15/16/17×2/19. The path is chosen at MR 3, and a path switch at MR 8 (Legend keeps only MR 1–2). The archetype is fixed when you first take a class, and a prestige class's spellcasting advancement is chosen when you first take it. Skill ranks can't exceed character level. BAB, saves and spell-level access are calculated.
- [Create the project skeleton](issues/03-scaffold.md): the five-crate workspace builds and runs. `cargo leptos watch` serves on `0.0.0.0:3900` (reload on 3901, reachable from the LAN), and the hello page reads its seeded row with `query_as!` into `rules::Greeting`. Postgres 18 (`wotr-planner-postgres`) is on host port 5434. `cargo test -p rules`, `cd data && cargo sqlx prepare` and `SQLX_OFFLINE=true cargo leptos build` are all verified.
- [Game data schema and seeding](issues/05-game-data-schema.md): RON files in `game-data/` are generated once by a test-first extractor and then edited by hand, with no source marks. Text keys come from guide anchors. Classes and archetypes get full per-level progression rows (no diffing). Prerequisites are one relational `prerequisite_node` tree table (all/any/at_least + typed leaf conditions with CHECKs). A `seed` binary reloads everything in one transaction behind a data hash. The `rules` types are the single shape, and a round-trip test checks it.
- [Level-up screen behaviour](issues/06-level-up-screen.md): the Sheet won. It's a compact level × slot chip grid with a separate Mythic track tab, and a picker that's a bottom sheet when folded and a modal when unfolded. Locked options sit behind a "Show N locked" toggle with reasons, and edits that would break later picks are blocked with "Would break: …". The same sheet also serves as the end-of-build summary.
- [Saving and sharing Builds](issues/13-saving-sharing-builds.md): the URL is the Build. The fragment `/#b=…` holds a versioned, compressed encoding of every choice (text keys, optional name), updated live on each edit, with no Save step and no Postgres table. `/` opens a blank Build. Stale links open with broken picks marked and a banner.
- [Build the game data types, schema and seed](issues/09-game-data-foundation.md): `rules::GameData` is the one shape. `data` reads `game-data/**/*.ron` strictly, and `cargo run -p data --bin seed [DIR]` truncates and reloads every game table in one transaction (deferred FKs, `position` columns, owner columns with exactly-one CHECKs; prerequisites in `prerequisite_node` with per-kind CHECKs) and stores a SHA-256 of the files. `load_game_data` reads it back, and the round-trip test proves files → Postgres → `GameData` equals files → `GameData`. `get_game_data` serves it, the home page shows counts, and the server warns at startup when the hash is stale. `hello` and `Greeting` are gone. *(2026-10-31: superseded in part by ticket 14 below: no `position` columns, one migration edited in place, 25 tables, 26 when 09 closed.)*
- [Build the rules engine](issues/11-rules-engine.md): `rules::Engine::new(GameData)` walks a `Build` (Build start, Level-ups, picks keyed by stable `SlotId`) into a `Sheet`: per-level state, slots, Mythic track and problems. It also gives `options` (with `Unmet` reasons and `would_break`), `would_break(edit)` and `validate` (unknown keys reported, not fatal). Broken picks don't count as held, so breakage cascades. The link is base64url(version 1 ++ deflate(postcard(Build))), with a frozen v1 vector. On the user's request, the database stores no derived values: `position` is gone from every table (none kept), `mythic_ascension` is `(rank, character_level)` with the odd/even pick rule in Rust, and `GameData::canonicalize` makes equality order-free. *(2026-10-31: superseded in part by ticket 14 below: class choices moved to `Build.class_choices`, `options` carries no `would_break`, and the drop-positions migration was folded into the one migration.)*
- [Apply the schema and layering review](issues/14-review-fixes.md): archetype and spellcasting advancement live per class in `Build.class_choices` (validated: held casters, one arcane plus one divine), and rule facts (combat feats, MR 3/8, rank after a switch, class caps from table length, no row 0) moved from data into `rules`. `GameData::check` gates every seed, and the schema uses domains, a `path_switch` table and class→archetype composite FKs. The sheet gets `class_options`/`path_options`/`switch_options`, `RemoveLevel`/`SetName`, and a one-walk `options` with would-break asked per tapped option. The server serves one `Arc<GameData>`, and `data` hides sqlx behind `Database` and `data::Error`.
- [Apply the normalization review](issues/15-normalization-fixes.md): `ability_modifier` has no `kind` (a NULL ability is the chosen kind), the level cap lives on the switch (`PathSwitch.level_cap`, `path_switch.level_cap`), path ranks and kept ranks reference `mythic_ascension (rank)`, and `prerequisite_node` has one per-kind CHECK that names its columns and says what `amount` means. The loader now rejects class skills under an archetype that keeps its class's list, and prerequisite nodes outside exactly one owner's tree (`Trees::finish`). Still 25 tables.
- [Save the guide pages](issues/08-save-guide-pages.md): all 38 pages are saved in the git-ignored `guide-cache/` (stamped 2026-10-31), each verified by its canonical link; `character-building` and `appendix-default-builds` were dropped as unnecessary.
- [Bonus feats ignore prerequisites](issues/16-bonus-feats-ignore-prerequisites.md): a **Bonus-list slot** is `FeatSlot::OneOf { feats, ignores_prerequisites: true }` (column `feat_slot.ignores_prerequisites`, CHECK only on `one_of`; `GameData::check` allows it only on class/archetype level rows). The engine skips the feat's prerequisites there but still checks the list and already-held, the pick counts as held, so follow-on feats check theirs, and would-break never flags it. Fighter, category, general and other fixed-list slots still check.
- [Extract the guide into data files](issues/10-extract-guide.md) (stage A): `cargo run -p extract` wrote the real `game-data/` (26 classes, 196 archetypes, 13 prestige, 261 feats, 63 + 56 mythic, 10 paths) and seeded cleanly with no hand-fixes. `EXTRACTION_REPORT.md` holds 82 items under Feats / Base classes / Prestige classes / Mythic / Races and skills for the per-category stages (biggest gap: sorcerer and wizard base tables grant nothing). The stages validate with `cargo run -p data --bin seed -- --check-only`, which needs no database.

## Not yet specified

- **Checking the data:** how we confirm the extracted data matches the guide, by spot checks or golden builds. The guide's sample-build pages aren't saved (the user dropped them), so golden builds would have to come from elsewhere. That page does hold independent BAB, ability-score and skill-rank prerequisite tables, which would cross-check the extracted feats if it's ever saved.
- **End-to-end smoke tests:** tooling and which flows to cover.

## Out of scope

- Spells known or prepared: not part of the class/feat/mythic summary.
- Gear and companions: not part of a character's build choices. This includes gear bonuses toward ability-score prerequisites, which check natural scores only (user, 2026-10-31).
- Deployment: this map ends with the app running locally.
- User accounts: Builds are anonymous and exist only as their link.
- Reproducing the guide's full text: copyright; store a summary and a link instead.
