# The game's blueprints and decompiled code in the repo, git-ignored

Type: build
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14): the game is the source of truth for rules, at patch 2.7.0x. Its two sources live in git-ignored folders in the repo.
- **`decompiled/`** is already cloned (user: "put the clone inside our repo in a gitignored folder"): xADDBx/WrathDecompiled, shallow, at `6ce4a6b` ("2.7.0x"). `.gitignore` already names it.
- **`blueprints/`** doesn't exist yet. It holds BubblePrintsData's release `2.7.0x_0` (https://github.com/factubsio/BubblePrintsData/releases): a `.binz` file of about 131 MB. That is BubblePrints' own container, LZ4-compressed chunks of the JSON the project already extracted.

The only copy today sits in another session's temp scratchpad, which a reboot clears.

## Change

- **`/blueprints/` in `.gitignore`.**
- **A script, `scripts/game-sources.sh`,** that fetches both sources at their pinned versions into an empty or missing folder and leaves a filled one alone (or checks it against the pin and says so):
  - the BubblePrintsData `2.7.0x_0` release asset, through `gh release download` or its URL;
  - WrathDecompiled at `6ce4a6b`, a shallow clone.

  The pins live in one place in the script. The game's last update is 2.7.0x (Steam, 2025-05-15), and the pin moves only when both sources have a newer patch.
- **The dump is pinned by its hash, not only its release name.** The asset is `blueprints_raw_2.7.0x_0.binz`, 136,905,792 bytes, SHA-256 `03ba6e29782c0c14e0c0383cd739560358641f7b5842f1dcfb690246e73c9c6a`. That is GitHub's own digest for it, and the copy used by 234's research matched it on 2026-11-14.
  - The script checks the hash after a download, and on a filled folder too. It refuses a file that differs and says so, rather than going on with it.
  - The game check ([238](238-the-game-check.md)) reads the hash from the same place and refuses a different file too. A changed dump can't silently change a report.
  - Moving the pin means changing the release name and the hash together.
- **`CLAUDE.md`**, a short "Game sources" section beside "Game data":
  - what each folder holds;
  - how a fact or a rule is cited: blueprint name and GUID, or the code's file and method with the WrathDecompiled commit;
  - that both live only in the main checkout, so an agent in a worktree reads them there by absolute path;
  - that the repo never holds the game's text strings, the dump or its code, not even quoted in a ticket.
- **`libs/game-data/README.md`** says the same in a line, where it describes the data's sources.

The script commits nothing of the game's: it fetches the community's published files into ignored folders.

## Done when

- `scripts/game-sources.sh` on a clone with neither folder leaves both filled at the pins. On a filled one it changes nothing and says it's current.
- A `.binz` whose SHA-256 isn't the pinned one is refused, with a message naming both hashes. Try it once with a truncated copy.
- `git status` shows neither folder.
- `CLAUDE.md` and the game-data README say where the sources are and how to cite them.
- `scripts/check.sh` passes.
