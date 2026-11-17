---
name: subagent-rescue
description: Rescue a wotr-planner subagent that has run past 30 minutes: diagnose read-only why it's slow or stuck, hand the main session an unblock plan, then write a retro of what would have prevented it. Use when the ticket-queue watchdog fires, or when asked to help a running agent reach its goal.
---

# Rescuing a slow subagent

A subagent has been running past 30 minutes. You find out why, tell the main session how to get it to its goal, and record what would have prevented the slowdown. You're a diagnostician, not a second builder: the subagent is still live in its worktree, so you **read only**. You don't edit, commit, switch branches or run cargo there, because two writers in one worktree collide. No Python in any form. The main session acts on your report.

## Steps

1. **Establish the goal.** Read the subagent's brief (the main session gives it to you), its ticket and the ticket's research: for a slice, `.scratch/wotr-planner/research/<ticket>/slices.md` under its heading, and `decisions.md`. Done when you can list the done-when items.
2. **Establish its state** from the worktree, never from the agent's transcript:
   - `git -C <lane> status`, `diff --stat` and `log -1` show what's done;
   - `ls -lt` on the worktree and its scratchpad folder shows when it last wrote;
   - `ps` with `lsof -a -d cwd` shows what runs under the worktree: a test, a hung shell, a Python process;
   - look for locks: `target/.cargo-lock`, and `index.lock` in the worktree's git dir;
   - logs in its scratchpad folder (`*.log`, the form proof's `compare.log`) hold its last results;
   - the machine itself: before calling a process hung, compare its elapsed time with the test's own "finished in", and read `pmset -g log` for `Sleep` lines since the builder started. If the machine slept, the verdict is "wait", not a rescue (306j);
   - the load: `ps -axo pcpu,comm -r | head` shows what else uses the CPU (Spotlight indexing the `target/` folders drove the load past 60 during 306j's fix).

   Done when you can say, for each done-when item: met, in progress, or not started.
3. **Judge it:**
   - **Progressing:** it's writing or compiling now, and its recent results move toward done.
   - **Stalled:** no writes and no process for over 10 minutes. API stalls of about 3 minutes are normal; several in a row aren't.
   - **Stuck:** it repeats a failing step, waits on something that will never finish, or works on the wrong thing.

   Also read the diff for mistakes it will have to undo: a key that breaks the slot-order rule, a list that can't match the game (check with `<lane>/target/debug/game-check show|granted-by`), a half-edited file, or a change away from the game.
4. **Report the unblock plan** to the main session, under 400 words:
   - the verdict and its evidence;
   - each done-when item's state;
   - anything wrong in the work so far, with file, key and count;
   - one next action: wait; message the subagent with an exact list (the failing tests by name, the filtered test command that iterates faster, the fix with its citation); or stop it and resume with a fresh builder from its committed and uncommitted state, with the brief to use.
5. **Write the retro.** Append one entry to `.scratch/wotr-planner/research/rescue-retros.md` (create it if missing). Give the date, the ticket, minutes spent, and what made it slow. Then name the one change that would have prevented it: a missing line in the brief or the research's edit list; a check that should be a tool (a guard in `game-check`, a test); a slower command it should have filtered; an instruction in `.claude/agents/ticket-builder.md` it didn't follow or that's missing. Mark each change "applied" or "proposed". The main session applies proposed ones to the builder, the skill or the tools. When the same cause shows up in two entries, say so: that cause gets a mechanical fix.

## What counts as evidence

- A file's modification time, a process's elapsed time, a log's last result: yes.
- The subagent's own report or a guess about its reasoning: no. A report is a claim.
- Check a game fact with `game-check show` against the blueprint before calling a list wrong.
