# Drop the type prefix from keys

Type: task
Status: open
Blocked by: 17, 18, 19, 20
Map: [WotR Build Planner](../map.md)

## Question

**User decision (2026-11-01):** the `feat:`, `skill:`, `class:`, … prefix on every text key is a derived value, because the table, or the Rust key type, already says what kind of thing it is. Drop it everywhere.

For example, `feat:power-attack` becomes `power-attack`, and `archetype:rogue/eldritch-scoundrel` becomes `rogue/eldritch-scoundrel`.

**Out of scope:** owner prefixes stay (`rogue/eldritch-scoundrel`, `rogue/rogue-talents/fast-stealth`). They keep names unique where the guide reuses them, for example Paladin's and Inquisitor's "Divine Hunter". Revisit them with [Answer the schema questions from the real data](21-schema-questions-real-data.md).

**Build** (AFK, test-first `tdd`; run it only when no other agent is working, per `CLAUDE.md`):
- **Schema:** edit the one migration, replacing every `CHECK (key LIKE '<type>:%')` with a slug-format check if useful. Reset and reseed the dev database per `CLAUDE.md`.
- **Rust:**
  - the `rules` key newtypes and any parsing that relies on prefixes
  - `GameData::check()`, including the selection-option-extends-selection rule, now without the type prefix
  - the URL codec. No link has shipped, so regenerate the frozen v1 vector; don't add a v2.
- **Data:** rewrite every key and reference in `game-data/**/*.ron`, the test fixture, `game-data/README.md` and `GLOSSARY.md`. Use a one-off script and diff the counts; the files are hand-edited now, so don't re-run the extractor over them.
- **The extractor:** stop emitting the prefix, so a future run matches.
- **Done when:**
  - `cargo test --workspace`, clippy, `cargo fmt --check`, the wasm build of `rules` and the offline build all pass
  - `--check-only` passes and the real data seeds
  - `grep -rE '"(feat|skill|class|archetype|race|subrace|feature|selection|path|mythic-ability|mythic-feat):' game-data data/tests/fixtures rules` finds nothing
