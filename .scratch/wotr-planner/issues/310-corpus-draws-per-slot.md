# The corpus's generated Builds draw each pick per slot, so an unrelated offer change moves nothing

Type: build (tools)
Status: open
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
