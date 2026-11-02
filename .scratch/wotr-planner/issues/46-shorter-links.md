# Shorter links

Type: task
Status: open
Blocked by: 45
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** the link grows linearly as options are chosen; make it grow much less. Measured on 2026-11-02 (`scratchpad/linksize/`: every base class to 20, every slot filled with its first open option, two skill ranks a level, a background and a path): today's links are **630–850 characters**. The body is postcard with every key as text, raw DEFLATE, base64url. Unique keys (feats, options) barely repeat, so DEFLATE can't shrink them: they're most of the link. Estimates from the same Builds:

| Encoding of each key | Link |
|---|---|
| Text (today) | 630–850 |
| A 3-byte hash of the key, per kind, then DEFLATE | 330–430 |
| A 2-byte index into the game data's list | ~250–330, but any data change shifts indices and breaks or silently changes old links |
| A preset DEFLATE dictionary of the keys | ~10% shorter only |

**Decision (user, 2026-11-02): hash each key.** The link stores a short hash of each key instead of its text; opening a link looks each hash up in the current game data. Links still survive game-data changes that keep the keys, as today. A hash of the whole Build is not an option: it can't be reversed without storing Builds on a server, which the map rules out.

## Build

AFK, test-first (`tdd`), in `rules` (`codec.rs` and whatever `open_link` needs); no schema or game-data change. No link has shipped, so replace version 1's body rather than adding a version 2, and regenerate the frozen `V1_LINK` with a comment saying why.

- **Key hashes.** Each key is written as a fixed-width hash of its text, per kind (feats, options, classes, …): a specified, stable hash (for example FNV-1a 32-bit, truncated), documented in the module docs. Choose the width so that no two keys of one kind collide in the real data, with a real-data test that fails if a future data change makes two collide (then widen it or change the seed). 24 bits is the starting estimate.
- **Decoding needs the game data** to turn hashes back into keys, so the link boundary changes: `Engine::open_link` decodes against its data. A hash no key of its kind has is a stale link (the one generic error). Keep a pure round-trip (encode → decode with the same data) for the tests and proptests.
- **Structural trims, measured:** try each and keep those that shorten links without making the codec fragile:
  - a run of levels of one class stored once (Fighter × 20) rather than the class at every level;
  - picks without their full `SlotId` (for example level-relative or in the Sheet's slot order), only if decoding stays simple and exact;
  - anything else the measurement shows is large.
- **Measure** with the `scratchpad/linksize/` Builds (copy and extend it to `t46/`), before and after, and put the table in the Answer.
- **Docs:** the codec module docs, ticket 13's encoding paragraph, `GLOSSARY.md` if it describes the link, a map "Decisions so far" line.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; the measured links are at most half today's length; a real-data test shows no hash collisions within any kind; round-trip tests and the proptest pass; a link naming a key the data no longer has opens as a stale link; `V1_LINK` is regenerated.

## Progress log

## Answer
