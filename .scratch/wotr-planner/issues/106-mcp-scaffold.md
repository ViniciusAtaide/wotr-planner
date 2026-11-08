# Scaffold the MCP server: `search` and `describe`

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 106 of the [MCP server plan](104-mcp-server-plan.md): a new `apps/mcp` package that loads the game data, checks it, prints the site's data hash, and offers `search` and `describe` over stdio, with a checked-in `.mcp.json`. *Done when:* the tests pass, the server connects, and "BFT" finds the archetype.

## Choices settled

1. **Package and SDK.** `apps/mcp` is a workspace member: package `mcp`, binary `wotr-mcp`. The SDK is `rmcp` **3.9.0**, still the latest on crates.io (2026-11-05). It's pinned in `[workspace.dependencies]` with `default-features = false`. The package turns on `server`, `macros` and `transport-io`, and its tests add `client`. `schemars` 1.2.3 (rmcp's major) derives the schemas.
2. **The rmcp API used:**
   - `#[tool_router] impl Server { #[tool(description = …, annotations(read_only_hint = true, open_world_hint = false))] async fn search(&self, Parameters(args): Parameters<SearchArgs>) -> Result<Json<SearchResult>, String> }`.
   - `#[tool_handler(router = self.tool_router)] impl ServerHandler for Server { fn get_info(&self) -> ServerConfig { … } }`. `ServerConfig` is rmcp 3's name for `InitializeResult`, and `get_info` sets the instructions and `serverInfo` (`wotr-planner`, titled "WotR Build Planner").
   - `Json<T>` gives `structuredContent`, a text copy and an `outputSchema`. An `Err(String)` becomes a result with `isError: true` and the message as text, so a bad key isn't a protocol error.
   - `main` runs `Server::new(planner).serve(rmcp::transport::stdio()).await?.waiting().await?`.
3. **Module layout** (`apps/mcp/src/`):
   - `planner.rs`: `Planner`, the checked `GameData`, the `Arc<Engine>` and the search index, loaded once and cheap to clone.
   - `server.rs`: `Server`, the tools and `INSTRUCTIONS`. Each tool runs its work on `spawn_blocking` with a clone of the planner.
   - `shapes.rs`: every JSON shape a tool takes or returns.
   - One module per tool's work: `search.rs` and `describe.rs`. Later tools follow the same pattern.
4. **JSON conventions** (in `shapes.rs`'s docs):
   - Fields are `snake_case`.
   - A result is always an object, so a list sits in a named field: `search` returns `{matches: […]}`, not a bare array. Older clients need an object for `structuredContent`.
   - A field that doesn't apply is left out, except `prerequisite`, which is `null` when there is none.
   - A reference to another item is `{key, name}`.
   - `Kind` is `class | archetype | path | feat | mythic_ability | mythic_feat | option | race | deity | background`. Keys are unique only within a kind, so `describe` takes both.
   - `rules` stays serde-free for this.
5. **Game data and the hash.**
   - It's read with `seeds::check_game_data_files`, which reads the RON and runs `GameData::check`. The folder is `--data <dir>` or `seeds::game_data_dir()`.
   - The site's file name is computed as the server does it (`apps/backend/src/game_data.rs`): the first 16 hex digits of the SHA-256 of the data's JSON. The site serves the data loaded from the database, which equals the files (`seeds`' round-trip test), so the hashes match.
   - It goes to stderr: `wotr-mcp: game data from …/libs/game-data, which the site serves as game-data/a7ba1f835d463455.json`. The deployed site names that same file.
6. **`search` matches** names, keys and initials, ranked: exact name, key or last key segment; initials ("bft" = Brown-Fur Transmuter's words' first letters); name starting with the query; every query word starting a word of the name; name or key containing the query; initials starting with the query.
   - Ties go by kind (in `Kind`'s order), then the shorter name.
   - `limit` defaults to 10, at most 50, and `kinds` filters.
   - The names, the prerequisite text (`Engine::prerequisite_text`) and the group names (`FeatCategory::name`, the mythic and background groups) are the web app's.
   - Options come from every Feature selection: classes', archetypes', prestige classes' and paths'.
   - Backgrounds have no guide link (the user's list), so they have no `guide_url`.
7. **`guide_url` is `GuideRef::url()`**, `GUIDE_BASE_URL` + `page#anchor`. The web app shows no guide links today, so that `rules` function is the only builder.
8. **`describe`:**
   - A class gets its rows by level from `rules::effective_row` (an archetype's merged over its class's), with `Class::base_attack_bonus` and `Class::save_bonus`. It also lists its spell levels, the arcane or divine casting a prestige level advances, features, granted feats, feat slots and selections, plus its archetypes and selections with their options (own options, then shared ones).
   - A path gets its rows by rank, with features from `Engine::path_rank_features`, plus its selections and summary (`Engine::path_summary`).
   - An option gets its rows by level, when it has any. A deity gets its alignments and domains, a race its subraces, and a background its note and feats.
   - A feat, option or mythic feat says what it's `taken_for` ("school", "weapon").
   - An unknown key is an error result that names the closest matches: "No archetype has the key `brown-fur-transmuter`. Closest: archetype `arcanist/brown-fur-transmuter` (Brown-Fur Transmuter). `search` finds keys by name."
9. **One accessor added to `rules`**, `Engine::feat_slot_label(&FeatSlot)`. A row's feat slot has no `Slot` yet, so `slot_label` couldn't name it. `slot_label` now delegates to it, except for the general slot's own "Feat". It's tested in `libs/rules/tests/engine/labels.rs`.
10. **`instructions`** is a short first version: the workflow (search, ask about ambiguous names, describe) and the honesty boundary from the plan's section 1. Guide links are for the user, never fetched. Ticket 110 finishes it.
11. **`.mcp.json`** at the repo root, in Claude Code's project format: `mcpServers.wotr-planner = {type: "stdio", command: "cargo", args: ["run", "-q", "-p", "mcp", "--release", "--"], env: {SQLX_OFFLINE: "true"}}`.
    - `SQLX_OFFLINE` builds `seeds`' queries against the checked-in `.sqlx` data, as `build-site.sh` does, so the server builds without the dev database.
    - Claude Code asks once before it starts a project server.
12. **Tests** (`apps/mcp/tests/server/`, a folder crate) start the server as `Server::new(planner).serve(server_end)` on one end of a `tokio::io::duplex` pipe, and connect `().serve(client_end)` as an rmcp client. `support.rs` loads the real data once (a `LazyLock<Planner>`) and wraps the calls (`Client::start`, `ok`, `call`, `search`, `error_text`).

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (56eb3bc). Read rmcp 3.9.0's sources and tests for the macro API.
- Red first: the `rules` label test, then the server tests, which failed to compile until the crate existed.
- Two expectations were wrong at first. The data keeps a path row's features in canonical (key) order. "augment summoning" matches only the feat, so the ordering check moved to "spell focus".
- Ran the release binary by hand: piped `initialize`, `initialized`, `tools/list` and a `search` for "BFT" into it, and got the instructions, both tools' schemas and the archetype back.
- `scripts/check.sh`: ALL-OK. In a worktree it needs `SQLX_OFFLINE=true`, because the main checkout's `.env` names a database path that the worktree lacks.

## Answer

Done 2026-11-08. `wotr-mcp` loads and checks `libs/game-data`, prints the site's hash, and serves `search` and `describe` over stdio. "BFT" finds `arcanist/brown-fur-transmuter` first. The 15 server tests and the new `rules` test pass, and the project's `.mcp.json` launches it.
