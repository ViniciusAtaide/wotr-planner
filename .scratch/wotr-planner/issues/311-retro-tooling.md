# The queue's retro: tooling that stops the mistakes it found

Type: build (tools and docs)
Status: resolved
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

## Answer

Built in the lane `lane-game` on `ticket/311-retro-tooling`; item 3 (research in a scratchpad) and item 6 are the skill change below. Item 4 was not part of the ticket.

**What was built**
1. `game-check` (`tools/game-check/src/lookup.rs`, `main.rs`): `show` and `refs` print a banner first when the name is shared, naming each GUID and what grants it (progressions, archetypes, classes and races before units, five at most, then "and N more"). New `granted-by <name|guid>` follows `refs` upward, breadth first, to the first `BlueprintProgression`, `BlueprintArchetype`, `BlueprintCharacterClass`, `BlueprintRace` or `BlueprintUnit` and prints each chain (30 at most). On the real dump, `show BardTalentSelection` prints the banner and `granted-by 94e2cd84bf3a8e04f8609fe502892f4f` reaches `BardProgression`. The check report is byte-identical before and after (`cmp` on the real dump). CLAUDE.md's "Reading a blueprint" line names `granted-by` and says to pick by what grants it.
2. `tools/corpus/tests/form_proof.rs`: an offers record (`offers.bin`: per Build, 9 bytes an option, a FNV-1a hash of slot and pick and the index of the status kind; `offers-kinds.txt`), written and read a Build at a time. `compare` reports each changed option as (pick key, kind before -> after) with counts and Builds, plus options that appeared or disappeared, and fails on any. An old record without `offers.bin` compares as before and says offers were not recorded. `FORM_PROOF_KIND_MAP=<file>` (old kind TAB new kind) is applied to `slots.txt` and the reopened slots, and `compare` prints the mapped lines apart from the differences. The existing four files and their output are unchanged. The test's header and CLAUDE.md's form-proof line document both.
3. `ticket-queue` skill: research reports and decisions go to `.scratch/wotr-planner/research/<ticket>/`, committed with the ticket files; step 3 says no builder starts on a ticket without its report; step 6 hands the builder its path.
5. CLAUDE.md: "Rust tests" is a short paragraph over the new `scripts/test-modules.sh` (in `queue-checks.sh`, the pre-commit hook and CI); "Formatting" is three sentences; "Dev server" moved to `docs/dev-server.md` behind a one-line pointer; "One-off tools" now says no Python in any form (user, 2026-11-16, relayed by the main session).

**Decisions** (mine, within the ticket): record every option's kind in 9 bytes instead of ok-in-full plus a hash per slot, so every changed option is named, not only the ok ones. A disappeared option is counted and not named (the record holds hashes). The offers are judged on the Builds the old links open to.

**Tests:** `lookup::a_shared_name_gets_a_banner_naming_each_guid_and_what_grants_it`, `lookup::a_name_one_blueprint_holds_gets_no_banner`, `lookup::granted_by_follows_referrers_up_to_the_first_granting_blueprint`, `lookup::granted_by_of_a_name_prints_a_block_per_guid_and_says_when_nothing_grants` (game-check fixture); `pieces::*` in `form_proof` (fnv, pick_key, status_kind, record round trip, changed-status report, disappearing option, kind map, mapped lines counted apart). No expected value is a game fact: the tools' behaviour is pinned on invented blueprints.

**Measured:** the corpus's offers record is 162,808,457 bytes for 18,079,504 options (7 status kinds). `record` took 47 s and `compare` 48 s in release; `compare`'s peak (`/usr/bin/time -l`, maximum resident set) was 588 MB, of which the recorded `numbers.txt`/`sheets.txt` text is most; the offers are never held whole. `record` then `compare` on one tree: 0 differences, 0 offer differences. Planted `power-attack` Strength 13 -> 17 (reverted): `power-attack: ok -> Unmet` 543 options and `BreaksLater -> Unmet` 15, in 392 Builds, nothing else moved. An old record without offers: "offers: not recorded", 0 differences. `scripts/test-modules.sh` passes on the tree and fails on a planted undeclared `tests/fixture/planted.rs`.

**Offers are opt-in (main session, after review of the first commit):** `record` writes `offers.bin` only with `FORM_PROOF_OFFERS=1` (`offers_wanted`, tested by `pieces::the_offers_are_recorded_only_when_asked_for`), for a ticket that changes offers or repeats (as 303 did); `compare` checks offers when the recorded folder has them and otherwise prints "offers: not recorded". Without the variable, `record` took 17.4 s and `compare` 14.7 s in release (the machine was loaded; before the offers existed `compare` took about 10 s on a quiet one); with it, 47 s and 48 s as measured above. The test header and CLAUDE.md's form-proof line say when to use it.
