# The form proof: a ticket that changes only form proves nothing moved

Type: build (tools)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the queue's retro (user, 2026-11-16, "let's do 3 and 4"). Ticket 235 removed the two-sided proof. Since then, four tickets that claimed "no behaviour moves" each wrote a throwaway proof, ran it, and deleted it:
- 235 hashed the corpus;
- 243 compared link decodes;
- 244's `proof244.rs` recorded names, `Debug` and sheet hashes, links and reopened links;
- 248–253 compared slot ids.

Tickets 296, 297 and 298 are form-only changes of every data file. Each depends on exactly this proof.

## Change

**A committed, ignored corpus test**, `tools/corpus/tests/form_proof.rs`, with two modes:
- **`record`** (`FORM_PROOF_OUT=<dir>`) writes, for every corpus Build:
  - its name and link, and whether the link opens;
  - its decoded picks;
  - every walk slot (id, kind, pick);
  - its `build_sheet` (the link line left out);
  - the hashes of all of these.
- **`compare`** (`FORM_PROOF_BEFORE=<dir>`) records again and compares against `<dir>`. It prints the first differences grouped by kind, and fails on any.

**The records are text, so they can be diffed.** Start from 244's and 248's throwaway proofs in the main session's scratchpad, `game/244/proof244.rs.kept` and `game/slots248.rs`.

**`CLAUDE.md` "Behaviour checks"** gains one line: a ticket that changes only the data's or the model's form runs `record` on its base (in a temporary worktree of the base commit) and `compare` on its tip, and puts the counts in its Answer.

This doesn't bring back the snapshot check or a per-ticket proof. It runs only when a ticket claims no behaviour moves.

## Answer

Built `tools/corpus/tests/form_proof.rs`: two ignored tests, `record` and `compare`, over the corpus (`corpus::builds::all`, 2,162 Builds, 2,001 of which open).

- **`record`** (`FORM_PROOF_OUT=<dir>`) writes text files: `builds.tsv` (name, link, whether it opens, picks as `Debug`, the picks the link decodes to), `slots.txt` (every walk slot: `L<level>`/`R<rank>`, id, kind, pick), `sheets.txt` (`build_sheet` without its `#b=` link line) and `summary.txt` (counts and a hash of each file; `DefaultHasher`, so within one toolchain, and `compare` never reads them). It builds the Builds in parallel (`FORM_PROOF_THREADS`) and takes about 6 s in release.
- **`compare`** (`FORM_PROOF_BEFORE=<dir>`) records again and compares kind by kind (name, link, opens, picks, decoded picks, slots, sheet), then decodes every old link with this tree's `LinkKeys` and compares the picks, slots and sheet of what it opens to. It prints the counts, then for each kind that differs the number of Builds and the first five with the first differing line, both sides. It fails on any difference. One run prints, for example, `2162 Builds compared (2001 open), 2162 old links reopened, 0 differences`.
- **Decision (mine, tooling):** the old-link reopen is part of `compare` rather than a third mode, since a data ticket that renames a link key would otherwise pass on a record that re-encodes. It starts from 244's and 248's throwaway proofs.
- **`CLAUDE.md` "Behaviour checks"** gains the one line (base in a temporary worktree, `record`; tip, `compare`; counts in the Answer). The corpus crate docs list the test.

Verified on this tree: `record` then `compare` printed `2162 Builds compared (2001 open), 2162 old links reopened, 0 differences` and passed. Planted throwaway change (renaming the class's `name` in `libs/game-data/classes/alchemist.ron` to `Alchemistx`, reverted, not committed): `compare` failed with `140 differences`, e.g. `sheet: 70 Builds differ`, `alchemist alchemist/chirurgeon: line 7 before: Level 1: Alchemist (Chirurgeon) now: Level 1: Alchemistx (Chirurgeon)`. No game fact changes, so no citation. `cargo test -p corpus -- --list` lists `compare` and `record`, both `ignored` in the normal run.

## Done when

- `record` then `compare` on one tree passes.
- A planted change fails `compare`, naming the Build and the slot or sheet line. Show it with a throwaway edit, not committed.
- The test is listed by `cargo test -p corpus -- --list`, and is ignored in the normal run.
- The scoped checks pass.
