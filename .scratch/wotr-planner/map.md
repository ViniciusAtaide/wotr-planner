# WotR Build Planner

Label: wayfinder:map

## Destination

A working, locally run Leptos app at `~/code/wotr-planner` for planning Pathfinder: Wrath of the Righteous builds. You build a character level by level, and each level offers only the choices the game would allow at that point: class, archetype, feats, mythic feats, mythic abilities, feature selections and skill ranks. The app ends with a summary up to character level 20, or 40 on the Legend mythic path. Builds are saved in Postgres and shared by URL. It works well on a phone, specifically the user's Galaxy Z Fold 7: both the folded ~21:9 cover screen and the unfolded ~6:5 inner screen. The source of truth is the [GameFAQs guide](https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/introduction).

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
- **Rules:** the rules live in a plain Rust crate with no DB or Leptos code, used by both the browser and the server. The browser loads the game data once and filters choices immediately. The server checks the whole build again when it's saved.
- **Mobile:** design mobile-first. The main targets are the Galaxy Z Fold 7 cover screen (folded, tall and narrow, ~21:9) and inner screen (unfolded, nearly square, ~6:5). Desktop is secondary. Both Fold screens must work for touch: no hover-only interactions, and tap targets large enough for a thumb. The layout should adapt when the phone is folded or unfolded without losing the user's place.
- **Saved builds:** no accounts. Each save is a snapshot that can't be changed and gets its own share URL. Editing a shared build and saving it creates a new link.
- **Prerequisites checked:**
  - race, ability scores (point-buy and the +1 every 4 levels)
  - BAB, saves
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
- [Filling the guide's gaps](issues/07-guide-gaps.md): prestige classes, path-specific mythic picks, alignment and all DLC are in scope. Combat feats are every non-class/racial/skill/teamwork/magic feat. Spell-level access is a separate check from caster level. Feats with no listed prerequisites have none. The skill rank cap is 20, or 40 on Legend. Facts from the user go in overrides with `source: user`, and committing extracted facts is OK.

## Not yet specified

- **Building the extraction pipeline:** turning the saved guide pages into the checked-in data files. It needs [Save the guide pages](issues/08-save-guide-pages.md), [Game data schema and seeding](issues/05-game-data-schema.md) and [Filling the guide's gaps](issues/07-guide-gaps.md). Known parts:
  - a grammar for prerequisite lines (watch for the misspelt "Prerequsites" labels)
  - deriving what each archetype replaces by diffing its table against the base class
  - a hand-maintained override file
- **Building the rules crate:** applying the progression model and prerequisite checks with TDD.
- **Summary screen:** layout and contents of the level 1–20/40 summary, fitting both Fold 7 screens.
- **Saved builds:** the table schema and the URL/slug scheme.
- **Class-specific and path-specific picks:** how far feature selections (rogue talents, rage powers, hexes and so on) and mythic path picks (Aeon gazes, Trickster bonus feats and so on) go, and in what order to add them.
- **Checking the data:** how we confirm the extracted data matches the guide, by spot checks or golden builds.
- **End-to-end smoke tests:** tooling and which flows to cover.

## Out of scope

- Spells known or prepared: not part of the class/feat/mythic summary.
- Gear and companions: not part of a character's build choices.
- Deployment: this map ends with the app running locally.
- User accounts: builds are anonymous snapshots that can't be changed.
- Reproducing the guide's full text: copyright; store a summary and a link instead.
