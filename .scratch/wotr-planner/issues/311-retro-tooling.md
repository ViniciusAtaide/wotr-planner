# The queue's retro: tooling that stops the mistakes it found

Type: build (tools and docs)
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retro of queue 235–310 (user, 2026-11-16, "fix 1 2 3 5 6"):
1. **A shared name led to the wrong blueprint.** `game-check show` prints every blueprint sharing a name, with nothing to say so. A research agent took the first `BardTalentSelection` (40f85fbe…, used only by units) for the bard's (94e2cd84…, granted by `BardProgression`). Its counts reached the user as fact.
2. **The form proof records no offers and allows no renamed kind,** so builders made their own tools. 303 dumped every offer into 16 GB of TSV, and its Python diff used tens of GB of RAM. 301, 302 and 304 each wrote a classifier for "only the slot-kind column differs". The working `offers-diff` tool lives only in a session scratchpad.
3. **Research and decisions live in a scratchpad** that a reboot can clear, yet builders depend on them.
5. **CLAUDE.md carries sections that a check or a pointer could carry:** "Rust tests" (a mechanical rule), "Formatting" (mostly what the hook enforces) and "Dev server" (needed only when starting it).
6. **298 went to a builder without its research**, hit a scope decision, and waited about 7 hours.

## Change

1. **`game-check`:**
   - `show` and `refs` print a banner line first when a name is shared. It names each GUID and what grants it.
   - A new `granted-by <name|guid>` follows `refs` up to the first `BlueprintProgression`, `BlueprintArchetype`, `BlueprintCharacterClass`, `BlueprintRace` or `BlueprintUnit` and prints each chain.
   - Fixture tests cover both.
   - CLAUDE.md's "Reading a blueprint" line names `granted-by` and says to pick a blueprint by what grants it.
2. **`tools/corpus/tests/form_proof.rs`:**
   - **An offers record:** every option's status at every slot of every Build, held to bounded disk (well under 1 GB for the corpus) and streamed, never loaded whole. `compare` then reports each changed option as (pick key, status kind before → after) with counts and Builds, as the scratchpad's `offers-diff` does (its source: `/private/tmp/claude-501/-Users-thaynamarcela--claude-spares-spare-nijyzu/f65a1f87-6cde-4710-9c11-116f63b89469/scratchpad/game/303/offers-diff/src/main.rs`). A design that stays within bounds: record only the `ok` options in full, plus a hash per slot of every (key, status kind).
   - **`FORM_PROOF_KIND_MAP=<file>`:** a two-column map (old slot kind, new slot kind) that `compare` accepts as equal. It counts the mapped lines apart from the differences.
   - The test's header documents both. CLAUDE.md's form-proof line names them.
3. **The `ticket-queue` skill:**
   - Research reports and decisions are written to `.scratch/wotr-planner/research/<ticket>/` and committed with the ticket files. Only bulky dumps stay in the scratchpad.
   - Step 3 says: no builder starts on a ticket without its research report (6).
5. **CLAUDE.md:**
   - "Rust tests" becomes `scripts/test-modules.sh`, a shell check that every `tests/<dir>/*.rs` other than `main.rs` is declared in that folder's `main.rs`. It runs in `scripts/queue-checks.sh` and the pre-commit hook. The section shrinks to a line or two.
   - "Formatting" shrinks to what the hook doesn't do.
   - "Dev server" moves to a doc, behind a one-line pointer.

## Done when

- `cargo test -p game-check` and `cargo test -p corpus` pass, with the new tests listed. The real dump's `game-check show BardTalentSelection` prints the banner, and `granted-by 94e2cd84bf3a8e04f8609fe502892f4f` reaches `BardProgression`.
- The form proof's `record` then `compare` on one tree passes with 0 offer differences. A planted throwaway change to one option's prerequisite shows that option's change, and is not committed.
- `scripts/test-modules.sh` passes on the tree and fails when a planted test file is undeclared. `bash -n` passes on each changed script.
- `cargo fmt --all --check` passes.
