# Alignment after faith

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-07:** "make the alignment depend on the deity. you cannot assign an alignment if you didn't selected you deity. and depending on the deity selected some alignments will be restricted"

The restriction half is [ticket 86](86-deities.md)'s: a deity allows only the alignments within one step of its own (`Engine::faith_fits`, `Issue::DeityAlignment`), the alignment wheel greys out the others with the deity's reason, and the deity picker locks deities that don't allow the chosen alignment. What's new is the order: **the faith first, then the alignment.** A Build start with no faith chosen can't have an alignment. Atheism counts as a faith and allows all nine.

## Build

AFK, test-first (`tdd`), in `rules` and `ui`; no schema, game-data or link-format change.

- **The rule lives in the engine.** The start check (`Walker::faith`) fails `Choice::Alignment` when the start has an alignment and no faith, so such a Build isn't possible. `Engine::faith_fits` (what the wheel and the deity picker ask) gives the wheel the reason: a new `Issue::NeedsFaith`, "Choose a deity or atheism first".
- **Choices come off from the leaves inward** (tickets 39–42): the faith can't be removed while an alignment is set; the alignment comes off first. Changing the deity stays as today: a deity that doesn't allow the chosen alignment is locked with its reason.
- **UI:** the start panel lists the Deity row before Alignment; with no faith the wheel locks all nine with the one reason under it.
- **Old links:** decide how a link with an alignment and no faith opens (ticket 40's convention), and keep the link format.
- **Docs:** `GLOSSARY.md` (Faith, Build start) and a Decisions line on the map.

**Done when:** the rules tests are red first, then green; `scripts/check.sh` passes; in the browser pane the wheel is locked before a faith, opens after Atheism or a deity, and a deity restricts it, with no console errors.

## Choices settled

1. **Where the failure lands:** at `Choice::Alignment` (the alignment is the choice that needs the faith), in the start check beside the deity's own.
2. **The reason's shape:** `Issue::NeedsFaith`, from `faith_fits`, a start-picker reason like `DeityAlignment`. Not a prerequisite leaf: no game-data row asks for it; it's a rule of the Build start.
3. **Removing the faith:** taking back one Build start choice no longer excuses another Build start choice failing. `Engine::removable` walked a start withdrawal with every start choice allowed to fail (they are all `Edit::Start`'s choices); a withdrawal now allows only the choice it takes back (the race also its subrace). Nothing failed at the start on a withdrawal before this ticket (a race, alignment, background or score taken back never fails another start choice), so only the faith's removal changes. The remove control is hidden while the alignment needs the faith, as for any choice something depends on; no remove control shows a reason anywhere, so none is added here. The wheel says why an alignment needs a faith.
4. **Old links:** a link with an alignment and no faith still decodes (the codec is unchanged), but the Build isn't possible, so `Engine::open_link` gives the generic stale-link error, as ticket 40 does for any link whose Build the rules no longer hold ("no partial repair": the alignment isn't dropped to save the rest). Only links made since ticket 86 (2026-11-06) can hold an alignment without a faith; ticket 86 already made every older link stale.

## Progress log

- 2026-11-07: started. Baseline HEAD 20ac16e, clean tree. Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md` and ticket 86.
- 2026-11-07: rules tests first, in `libs/rules/tests/engine/deities.rs` (a section of its own, 5 new). Red with `Issue::NeedsFaith` stubbed (`t94/red.txt`, `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t94/`): 4 failing, each at its assertion (an alignment without a faith was possible, `faith_fits` said `Ok`, the faith was removable under an alignment, the link opened); `changing_the_deity_keeps_the_alignment_check` passed from the start, as it pins today's behaviour. Then green: `Walker::faith` fails `Choice::Alignment`, `faith_fits` gives `NeedsFaith`, start withdrawals excuse only themselves (choice 3). `cargo test -p rules`: 427 passed.
- 2026-11-07: the tests' Builds that set an alignment without a faith got one: the rules world's `build` helper is an atheist; the real-data tests set alignments through a new `support::align` (atheism unless the Build has a faith), `alignment.rs`'s `aligned` falls back to atheism, and three deity tests' "no deity" Builds are atheists. Ticket 91's old-link test: the `bench fighter20` link (and any other with an alignment and no faith) now opens as a stale link; the first-ascension checks run on the same Build as an atheist. `cargo test --workspace`: all pass. The proof harness's `generate::faithful` already gives such a Build atheism first, so no change there; the proof wasn't run (no game-data change, and the rules change is the one intended).
- 2026-11-07: UI: the Deity row above Alignment; the wheel needed no code change (its locks and reason come from `faith_fits`). `GLOSSARY.md` (Build start, Faith) and the map's Decisions line.

- 2026-11-07: browser pane (127.0.0.1:3911, scratch site root, server stopped after). `scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-07. The faith comes first, then the alignment, and the rule is in the engine:

- **Rules.** `Walker::faith` fails `Choice::Alignment` for an alignment with no faith; `Engine::faith_fits` returns the new `Issue::NeedsFaith` ("Choose a deity or atheism first") before the deity check (`DeityAlignment`, unchanged). `Engine::removable`'s Build start withdrawals now excuse only the choice they take back (choice 3), so the faith can't come off while an alignment is set; changing the deity is locked by its alignment reason as before.
- **UI.** The Deity row is above Alignment. The wheel needed no code change: with no faith all nine are locked with "Choose a deity or atheism first" under it.
- **Old links** (choice 4): they still decode and re-encode unchanged; one with an alignment and no faith opens as the generic stale link. Of ticket 91's recorded old links, `bench fighter20` is one.
- **Tests** (`libs/rules/tests/engine/deities.rs`): `an_alignment_needs_a_faith_first`, `the_alignment_wheel_is_locked_until_a_faith_is_chosen`, `the_faith_comes_off_only_after_the_alignment`, `changing_the_deity_keeps_the_alignment_check`, `a_link_with_an_alignment_but_no_faith_is_stale`; `the_start_pickers_lock_a_misfit_with_the_deitys_alignments` and `the_deity_comes_off_once_nothing_needs_it` adjusted. Older tests' Builds got a faith (atheism) where they set an alignment (progress log).
- **Browser:** with no faith the wheel locks all nine with the one reason; after Atheism all nine open; Lawful Neutral chosen, the deity picker shows no remove control; Erastil chosen, the wheel opens LG, NG, LN and locks the other six with "Erastil allows Lawful Good, Neutral Good or Lawful Neutral"; after removing the alignment the deity's remove control is back; at level 1 (Erastil, no alignment) the paladin, monk and Angelfire Apostle are locked with their alignment reasons as before. No console errors.
- **Not done:** the behaviour proof (`scripts/proof.sh rules`), not asked for; its generator already gives such Builds atheism.
