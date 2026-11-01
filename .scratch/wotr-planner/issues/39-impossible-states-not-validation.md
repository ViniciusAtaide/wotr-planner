# Impossible states instead of validation

Type: task
Status: open
Blocked by: 38
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-01:** the backend serves only this frontend, so drop any validation of the payload that the UI can make an impossible state. Example: with Power Attack picked, the UI offers Cleave and what Power Attack enables, so nothing needs to expect a Build holding Cleaving Finish without Cleave. Remove that validation from the service layer.

## What exists today (survey, 2026-11-01)

- **No server-side validation.** The server has one server fn, `get_game_data`. A Build never reaches the backend: the URL fragment is the Build (ticket 13), and `rules` runs in the browser.
- **One check, used twice.** `rules::engine::check_pick` decides whether a pick fits a slot at a point of the Build. It runs:
  1. in `Engine::options` (and `class_options`, `switch_options`): which options the picker enables, with the reason a disabled one shows ("needs Power Attack");
  2. in the walk, over every pick already in the Build, producing `Sheet.problems`. Those feed `Engine::validate` (the stale-link banner, ticket 13 Q9) and `Engine::would_break` (an edit to an earlier choice that would break a later pick is disabled, ticket 12), and a broken pick doesn't count as held.
- `Issue` has 18 variants (`rules/src/problem.rs`). Some are reasons a picker shows (`Unmet`, `AlreadyHeld`, `MaxTimes`, `ClassLevelCap`, `NotFromPath`). Others only arise from a Build the UI can't produce: `NotAllowed`, `NoSuchSlot`, `ParameterMissing`, `WrongParameter`, `LevelCap`, `SkillRankCap`, `SkillPointsOverspent`, `PointBuy`, `ScoreOutOfRange`, `NotChoosable`, `NoAbilityIncrease`, `UnknownKey`.
- A Build the UI can't produce enters only by a link: a stale one (the game data changed since) or a hand-edited one.

## Decisions

**User, 2026-11-01:**

1. **Changing an earlier pick: remove from the leaves inward.** A pick shows a remove ("×") control only when no later pick depends on it. With Power Attack → Cleave → Cleaving Finish, only Cleaving Finish shows ×; once it's removed, Cleave shows ×, and so on. No edit is offered that would break a later pick, so there's no "Would break: …" message and the Build the UI holds is always possible. Read generally (my reading, not the user's words): every other choice a later pick depends on (the class at a level, an archetype, a point-buy score, a skill rank, the path) offers no change or remove control until its dependants are gone. The engine answers "may this choice be removed/changed" per choice; it doesn't list what would break.
2. **Stale links keep the banner** (ticket 13 Q9 stands). A link whose game data changed opens as is, with broken picks marked with their reason and removed keys shown as "removed from game data". A broken pick always shows ×, whatever depends on it, since removing it is how the player fixes the link.
3. **Error reasons only an impossible Build can produce go.** Where it's cheap, make the bad state unrepresentable in the types (for example a point-buy score, or a parameterised pick that must name a value of the right kind), with the link decoder refusing a link that can't be read into those types. Keep the reasons a disabled option shows (unmet prerequisites, already held, max times, class maxed out, path switch not open from the Build's path).

**How 2 and 3 fit together** (mine, so the agent should check it): since stale links still load, the reasons a *game-data change* can cause stay too, because the banner shows them (a key gone, a prerequisite changed, a feat leaving a slot's list, a class table shortened). What goes is what only a *hand-edited* link or a buggy UI could produce: a score outside 7..=18, an overspent point buy, overspent skill points or ranks above the cap, a level above the cap, an ability increase at a level without one, a value of the wrong kind or a missing one. Those become unrepresentable types, a decode error ("this link can't be read"), or simply aren't checked. List each `Issue` variant in the Answer with what happened to it and why.

## Build

AFK, test-first (`tdd`), in `rules` (and the codec); no schema or game-data change is expected. Only agent in the repo.

- **Removable choices.** Replace `would_break` (a list of broken later picks for an edit) with what the UI needs: whether each choice in a Build may be removed or changed now. A pick is removable when no later pick depends on it (it is a leaf), or when it is itself broken. Prefer a dependency answer that doesn't walk the whole Build once per choice, if that stays simple; otherwise correctness first, and note the cost.
- **The walk** keeps checking picks only for what a stale link can contain (decision 3), so the banner still works; a broken pick still doesn't count as held.
- **Issues and types** per decision 3.
- **Tickets 12 and 13:** update their text to match (ticket 12's "disabled with Would break: …" line becomes the × rule; ticket 13's stale-link paragraph stays, plus "a broken pick always shows ×").
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: in the Power Attack → Cleave → Cleaving Finish chain only Cleaving Finish is removable, then Cleave, then Power Attack; a class level, archetype or point-buy score a later pick depends on isn't changeable; a stale link's broken pick is removable even with dependants; a hand-edited link with an out-of-range value doesn't decode into a Build; every removed `Issue` variant is gone from the code and tests.

## Progress log

## Answer
