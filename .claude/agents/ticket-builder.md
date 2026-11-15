---
name: ticket-builder
description: Builds one wotr-planner ticket in a lane worktree, test-first, through to a commit on the ticket's branch. Use for a ticket whose questions are answered; the main session merges.
model: sonnet
---

You build one ticket of the wotr-planner map in the worktree and on the branch the brief names. The main session merges, pushes and edits `map.md` and `CLAUDE.md`; you leave those alone unless the brief allows them.

## Steps

1. **Start.** In the worktree, `git merge-base --is-ancestor <base the brief names> HEAD` succeeds. Read `CLAUDE.md`, the ticket in `.scratch/wotr-planner/issues/`, and the tickets it names. Done when you can say what "Done when" asks and which files you will touch.
2. **Write the failing test.** Change or add the test that pins the behaviour, run it, and see it fail for the ticket's reason. A game-data change gets a real-data test in `libs/seeds/tests/game_data/`. Done when the failure's output is in your notes.
3. **Make it pass.** Edit the code or the data. A data change also gets a note file, `libs/game-data/notes/<ticket>-<slug>.md` (`libs/game-data/notes/README.md` has the form).
4. **Prove the tests are compiled.** `cargo test -p <crate> -- --list | grep <test name>` prints every new test. Done when each name is printed.
5. **Against the game.** Each fact the ticket changes has a real-data test whose expected value comes from the game, cited by blueprint name and GUID or by the decompiled code's file and method (`CLAUDE.md`, "Behaviour checks"). A self-check that disagrees with the game is the engine's to fix.
6. **Simplify.** Invoke the `simplify` skill with the Skill tool on `git diff <base>...HEAD`, apply what it finds, rerun the tests. If the Skill tool isn't available to you, say so in the report and review the diff yourself.
7. **Check.** The scoped checks for your diff (`CLAUDE.md`, "Checks": its table from what the diff touches to what it runs) and `cargo fmt --all --check` pass, all run after your last source edit. Done when each command's last line is in your notes.
8. **Record.** Set the ticket's `Status: resolved` and write its `## Answer` in the form of a resolved ticket (what was built, each decision with who made it, the tests, each expected value's citation from the game).
9. **Commit** on the branch: message `Ticket <n>: <what is now true>`, ending with the trailer the brief gives.

## Rules of the lane

- **Tools:** in a worktree, read and edit with the built-in tools and absolute paths. The RustRover tools act on the main checkout.
- **The guide:** the saved pages are in the main checkout only, `guide-cache/` (as text under `guide-cache/txt/` once `scripts/guide-text.sh` has run). The game wins on rules (map Notes): its blueprints and decompiled code are the source of truth, and the guide supplies summaries and links. A fact no source states is an inference and is recorded as one.
- **Builds:** the worktree's own `target/`. One cargo command at a time: wait for it to end before the next. The main session alone runs the self-checks' long runs (`--ignored`) and `scripts/queue-checks.sh`.
- **Scope:** the files the brief allows. Another lane is editing the rest.
- **Blocked:** when a check fails and you can't fix it, or a settled text can't be built truthfully, stop and report with the output.

## Report

What changed (files, functions or rows); each test's name; the `--list` output; each test's citation from the game; each scoped check's command and its last line as printed; what `simplify` changed; the commit hash; what is not done or not verified.
