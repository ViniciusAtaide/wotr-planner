---
name: ticket-queue
description: Run a queue of wotr-planner tickets with subagents, from research to merge, proof and QA. Use when asked to dispatch agents for several tickets, or to finish the open tickets.
---

# Running a queue of tickets

You are the main session: you answer questions, order the work, review, merge and run the heavy checks. Subagents research, build and review. `CLAUDE.md`'s "Parallel agents" holds the limits; this is the order of the work.

## Steps

1. **Look at who else is here.** `git status` and `git log` in the main checkout, `git worktree list`, and `ps` for `cargo`. Another session's uncommitted edits to `map.md`, or its builder, change what you may touch and how many lanes you get: two builders on the machine, theirs included. Done when you can name every other writer and builder.
2. **Read the tickets** and each one's `Model:` line. Commit ticket files that are still uncommitted, so worktrees see them.
3. **Research, all at once.** One read-only agent per ticket, on the ticket's build model, with no cargo command: the facts from the guide and the code, the edits planned, the tests planned, the files touched, and every open question with its evidence, options and a recommendation. Reading costs no CPU, so these run side by side.
4. **Answer.** A question the guide, a map decision or the data's own convention settles, you answer and record as yours. The rest go to the user in one round, each with a recommendation; work that doesn't hang on them carries on.
5. **Sort into lanes** by the files each ticket touches. Tickets that share a file go in one lane, in order. Create one worktree per lane from local `main` (`git worktree add -b ticket/<n>-<slug> .claude/worktrees/lane-<name> main`, then check the base). A lane keeps its worktree for the whole queue, so its `target/` stays warm.
6. **Build.** Brief a `ticket-builder` with the worktree, branch, base, the files it may touch, the decisions, and the research report. Resume the research agent for a small ticket; for a fix round on an agent whose context has grown past about 300k tokens, start a fresh builder with a pointed brief.
7. **Review before the merge,** on the model the ticket names, with `ticket-reviewer`. Read the diff yourself too. A builder's report is a claim.
8. **Merge** into local `main` with `git -C <main> merge --no-ff`, then move the lane to the next ticket's branch from `main`. A lane that worked beside another merges `main` into its branch and reruns `scripts/check.sh` first. When another session has staged files in the main checkout, a true merge there refuses: make the merge commit in the lane (`git switch -c integrate/<n> main`, `git merge --no-ff ticket/<n>-…`), then `git -C <main> merge --ff-only integrate/<n>`, which leaves their index and working tree alone. Commit your own files there with `git commit -m … -- <paths>`.
9. **Close the queue,** with every lane idle: a code review of the merged range; `scripts/check.sh --full`; `scripts/proof.sh <mode> <out> <the commit before the queue> -- --explain`, each group of `explain.txt` attributed to a ticket (delete the dumps after: about 1.5 MB a Build); a QA session (the `qa-session` skill). Fix what the queue caused; write a ticket for what is older.
10. **Record.** One entry per ticket and one for the proof in the map; follow-up tickets; then offer to remove the lane worktrees and merged branches.

## Hazards met before

- **`cd` in your own Bash** moves the session's directory under running subagents. Use `git -C` and absolute paths.
- **A file created through the IDE is staged at once,** and a staged new file makes `git merge` refuse. `git restore --staged <file>` first, or commit it.
- **The dev server reads the game data at startup.** After a data-only merge it serves the old data until it restarts; check before reproducing through the MCP tools.
- **A design you steer a builder to is a claim too.** When you override a research agent's recommended shape, ask the reviewer to attack that choice by name.
