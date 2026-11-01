# Stale links get a generic error

Type: task
Status: open
Blocked by: 39
Map: [WotR Build Planner](../map.md)

## Question

Build the user's answers to [ticket 39](39-impossible-states-not-validation.md)'s open questions (2026-11-01):

1. **A stale link gets a generic error.** "If a stale link happens to appear just give a generic error. Don't try to reason about it." A link that doesn't decode, or that decodes into a Build that isn't possible with the current game data (a key gone, any pick that doesn't fit, any rule broken), opens nothing: the sheet shows one generic message ("This link no longer works with the current game data") and a blank Build. No banner, no broken picks marked, no reasons, no partial repair. This reverses ticket 13's Q9 and ticket 39's decision 2.
2. **Picks the Build no longer reaches are dropped.** "I don't care to store what feats you chose before selecting Legend." Removing a level drops the picks of that level and of the mythic ranks it no longer reaches (reversing ticket 14's keep), and a Path switch that ends ascensions (Legend) drops the picks of the ranks it takes away. The Build only ever holds what the sheet shows.

## Build

AFK, test-first (`tdd`), in `rules` (and the codec); no schema or game-data change expected. Only agent in the repo.

- **One yes/no check for a loaded link.** Whether a decoded Build is possible is a single boolean (or a `Result<Build, StaleLink>` with no detail) at the link boundary. Reuse the walk; don't keep any code whose only job was explaining or marking a stale Build.
- **What goes:** everything that existed only for stale links:
  - the "a broken pick always shows ×" case in `Engine::removable`, and any "choices a stale link already broke don't count" handling in `allows`;
  - `Issue` variants that only a stale Build can produce (ticket 39's table marks which: `UnknownKey`, `NoSuchSlot`, `ParameterMissing`, `WrongParameter`, `NotChoosable`, and any other whose only remaining use is the stale case). Keep what a picker shows on a locked option and what the locks need (ticket 39 says the walk uses `LevelCap` and `SkillPointsOverspent` for locks: keep them if so, or replace them with something smaller if the picker never shows them);
  - `Problem`/`Choice` reporting, `Engine::validate` and `Sheet.problems`, if nothing but the stale-link check uses them after this; otherwise shrink them to what's used. Say which in the Answer.
- **Unreached picks:** `Edit::RemoveLevel` and the Path switch (choosing or changing a switch that ends ascensions) drop the picks of levels and ranks no longer reached. If that needs the ascension table or other game data, make it an engine call rather than `Build::apply`, and have the sheet use it.
- **Docs:** update ticket 12 (the State line: no "Game data changed" banner; a stale link shows the generic error and a blank Build) and ticket 13 (Q9 and the stale-links paragraph, noting the reversal), `GLOSSARY.md` (Problem, stale link), and the map's lines for tickets 13 and 39 where they describe the banner.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: a link with a removed key, one whose pick no longer fits, and one that breaks a fixed rule each come back as the generic stale error; a link that still fits loads unchanged; removing a level drops its picks and the unreached ranks' picks; choosing Legend's switch drops the picks of the ranks it takes away; the removed variants and code are gone.

## Progress log

## Answer
