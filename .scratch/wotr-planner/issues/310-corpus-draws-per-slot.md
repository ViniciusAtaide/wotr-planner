# The corpus's generated Builds draw each pick per slot, so an unrelated offer change moves nothing

Type: build (tools)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From 303's review (main session, 2026-11-16). `tools/corpus/src/generate.rs` fills the generated `full …` Builds by drawing each pick as `open[rng.below(open.len())]` from one random stream seeded by the Build's name. When a ticket opens or closes a single option anywhere, the stream's later draws shift, and every later pick of that Build changes. 303 made three picks legal again; that re-drew 147 Builds and gave 825 form-proof differences that said nothing about 303.

The importer's stages (293, 306–309) prove themselves by a census that ties every differing line to a change. Re-drawn Builds would bury those lines.

## Change

- **Each slot's pick is drawn on its own:** among the slot's open options, the one with the highest hash of (Build name, slot id, option key) wins (rendezvous hashing). A slot's pick then moves only when its own offered set changes, and then only if the winner left or a new option outranks it. The path pick and any other draw use the same rule. Keep the generator's other behaviour (which slots it fills, its filters, the Witch Wandering Hex exception) as it is.
- **The hash is deterministic across toolchains,** for example FNV-1a over the bytes, as the generator's seed already is. Not `DefaultHasher`.
- **The corpus changes once:** the generated Builds are re-drawn by the new rule. That's this ticket's only behaviour change. The hand-made Builds, the bench Builds and every link of a Build that isn't generated stay byte-identical.

## Done when

- A test plants a change to one slot's offered set in a generated Build and shows that only that slot's pick, or none, moves. The old rule would re-draw the later slots.
- The form proof (record on the base in the lane, `compare` on the tip) shows differences only in `full …` Builds. The Answer gives their counts and confirms 0 elsewhere.
- `cargo test -p corpus` passes, including the links test and the self-checks' short run. `cargo fmt --all --check` passes.

## Answer

Built by the lane-game builder, 2026-11-16.

**What was built** (`tools/corpus/src/generate.rs` only): `fill_full` takes the Build's name and draws each slot's pick, and the mythic path's, with `draw_for`: the open option with the highest hash of (Build name, slot id's `Debug` text, option key plus its parameter), rendezvous style. The path's slot text is `mythic-path`. The hash is FNV-1a over the bytes with a 0xff after each part, finished by a 64-bit mixer (FNV's last bytes barely reach its high bits); no `DefaultHasher`. Kept as they were: which slots are filled, the three rounds, the filters, the Witch Wandering Hex exception, the 9-level path rule, and the ability-increase and skill-point draws, which still come from the name-seeded stream (`fnv(name) | 1`, unchanged). `fnv` is now built on a shared `fnv_extend`.

**Decisions** (builder): the ability and skill draws stay on the stream, since the ticket asks to keep the rest of the generator and they are not draws from an option set; say so if they should be per-slot too.

**Test:** `generate::tests::a_change_to_one_slots_offers_moves_only_its_own_pick` (shown by `cargo test -p corpus -- --list`). Over 30 Build names and 12 slots of 20 options, it closes the slot's winner, another option, or every option, and asserts no other slot's pick moves and the winner stays when it wasn't closed. It also asserts the old stream rule re-drew later slots (it does, when a slot goes empty and skips a draw). Not a game fact, so no citation. Test-first caveat: the new function and the test were written together; the first run failed only on the old-rule contrast (a modelling mistake in the test, fixed).

**Form proof** (base recorded in this worktree before any edit; `compare` on the tip): 2162 Builds before, 2161 now (one `full ... / switch` Build's switch no longer holds). 1184 Builds differ in each of decoded picks, link, picks, sheet, slots and numbers; all 1184 are `full ...` Builds: 418 plain `full <name>` and 766 `full <name> / <switch>`. The other 977 Builds, all hand-made and bench ones among them, and every link of a Build that isn't generated, are unchanged (0 differences outside `full ...`). The one other line is the corpus count above.

**Checks:** `cargo test -p corpus` passes (links test and self-checks' short run included), `cargo clippy -p corpus --all-targets -- -D warnings` clean, `cargo fmt --all --check` passes.
