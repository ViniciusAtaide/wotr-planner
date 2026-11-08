# The engine says which choices a Build holds, and the withdrawal of each

Type: task
Status: open
Blocked by: [94](94-alignment-after-faith.md) (it changes the start choices this ticket lists)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 2) found the list of "which choices a Build holds, and the edit that takes each back" written four times:

- `Edit::choices` (`libs/rules/src/build.rs:474–520`);
- the engine's `withdrawals` and `first_level` (`libs/rules/src/engine/removable.rs:84–246`), deep but private;
- nine hand-built `(Choice, Edit)` remove pairs in `libs/ui/src/picker.rs` (around 399, 767, 811, 817, 940–949, 960–967, 1083–1086, 1129–1136, 1167–1174);
- the proof's `battery` and `every_choice` (`tools/proof/src/fingerprint.rs:196–312`).

The race withdrawal (race, subrace and racial +2 all cleared) is written three times. The proof's copy has fallen behind: `every_choice` has no `Choice::Deity`, `Choice::FirstAscension` or `Choice::CharacterLevel`, and `battery` has no deity starts and no `Edit::FirstAscension`. So the proof's `is_removable`-vs-`removable` and `allows` checks never covered tickets 86 and 91.

The user settled the design in a grilling round (2026-11-07, all recommendations taken).

## Build

- **The word:** a **Withdrawal** is the edit that takes a held choice back (a race's also clears its subrace and racial +2). Add it to `GLOSSARY.md` beside **Removable**, and use it in the interface.
- **For a remove control:** one engine call per choice that answers both questions a picker's remove control asks: `None` when the Build doesn't hold that choice, otherwise its withdrawal and whether it's removable now. Its cost is no more than `is_removable`'s today.
- **For the proof:** a public list of every choice the Build holds, removable or not, each with its withdrawal. `removable`, `is_removable` and `removable_classes` stay as they are.
- **The UI:** every picker's remove control (slot Clear, class, archetype, path, first ascension, switch and the start choices) goes through the engine call; the picker header takes only a `Choice`. The nine hand-built pairs go.
- **The proof:** `every_choice` reads the engine's held-choices list, and `battery` adds each held choice's withdrawal plus deity starts and `Edit::FirstAscension`, so a new `Choice` is covered without editing the proof. (Ticket 96 later builds the battery from the engine's start options.)
- Test-first in `rules` (`tdd`): the held-choices list on the small world covers every `Choice` variant a Build can hold, including `Deity`, `FirstAscension` and `CharacterLevel`; the race withdrawal clears all three fields.

**Done when:**
- The rules tests above pass, and `Edit::choices`, `withdrawals` and the UI agree because the UI and proof have no copy left.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference, with the proof's added coverage reported as its own breakdown.
- The browser check (its own port): each remove control still removes, and is locked where it was.
- `scripts/check.sh` passes.
