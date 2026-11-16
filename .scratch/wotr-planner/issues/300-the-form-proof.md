# The form proof: a ticket that changes only form proves nothing moved

Type: build (tools)
Status: open
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

## Done when

- `record` then `compare` on one tree passes.
- A planted change fails `compare`, naming the Build and the slot or sheet line. Show it with a throwaway edit, not committed.
- The test is listed by `cargo test -p corpus -- --list`, and is ignored in the normal run.
- The scoped checks pass.
