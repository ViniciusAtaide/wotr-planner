# R: Refusal — detailed design

(Explorer 2, read-only, `main` at 790eff76, 2026-11-18. Frame: `overall.md`. Designed to the architect's recommendations: R between 199 and 200; `validate` keeps asking the walk; wording item 4 accepted. **[b]** marks what changes if the user picks "R after 202", **[c]** "R first, 199 rewritten on it".)

The question R answers: *why can't this Pick or value be taken at this Address in this Build, and what would let it in?* One module in `rules` (`libs/rules/src/engine/refusal.rs`, re-exported), with every tool-naming clause left to four adapters.

## 1. Interface

```rust
/// Where an `Issue` came from. The two sources differ in three facts R must know, and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A picker's status (`try_pick`, a per-picker `*_options`, a stepper). Its `Unmet` is already trimmed by
    /// `reason_for` (pickers.rs:133); it may hold `MetLater` and `BreaksLater`; the Build doesn't hold the value asked.
    Picker,
    /// A walk `Failure` of a Declared Build (`Engine::failures`). Untrimmed; never `MetLater` or `BreaksLater`
    /// (overall.md, verified); the Build holds the failing pick.
    Walk,
}

/// What is refused, at one Address: the pick in question (a slot's, or the declared one) and the edit that takes it
/// (a picker's; `None` for the walk). An entry listed without its values passes its shown value's pick and edit
/// (`SlotEntry::shown`, today's `Offer.like`).
#[derive(Clone, Copy, Debug)]
pub struct Asked<'p> {
    pub address: &'p Address,
    pub pick: Option<&'p Pick>,
    pub edit: Option<&'p Edit>,
}

/// One call's refusals over one Build: holds the caches every refusal of the call shares.
pub struct Refusals<'a> { /* engine, build, sheet, source; private:
    sheet: OnceCell<Sheet> when built without one; failing: OnceCell<BTreeSet<Choice>> (Breaks);
    later: RefCell<BTreeMap<u8, LaterRankLevels>> (192, keyed by rank);
    without: RefCell<BTreeMap<SlotId, Option<(Build, Sheet)>>> (Walk: the declared Build without the failing pick) */ }

impl<'a> Refusals<'a> {
    /// `sheet` is `build`'s when the caller has it; otherwise walked once, when the first refusal needs it.
    pub fn new(engine: &'a Engine, build: &'a Build, sheet: Option<&'a Sheet>, source: Source) -> Self;

    /// The refusal of `asked` for `issue`. **No walk** beyond the call's one sheet: the text, `needs` and the
    /// later rank's *place* are worked out here; the level that lets the pick in is not (see `opens`).
    /// Panics never; an issue with nothing to say beyond `issue_text` gives that text.
    pub fn of(&self, asked: Asked<'_>, issue: &Issue) -> Refusal;

    /// What a refused edit would break, naming the choices: the call's first such question walks the Build's
    /// failing choices once, and each edit costs one walk (today's `Breaks`, options.rs:131). `None` when the edit
    /// refuses nothing the walk can name.
    pub fn breaks(&self, edit: &Edit) -> Option<Breaks>;

    /// The level of a Mythic rank that would let the refused pick in, with the ranks the move takes along.
    /// Lazy: asked only for a reason that is shown. Costs, for a rank's own slot (ticket 171), one walk to the slot and
    /// one per level tried (`rank_level_that_opens`); for a later rank's pick (192), two walks per level tried, once
    /// per rank per call (`later_rank_levels`, cached here), then one per pick (`level_that_lets_in`). For `Walk`,
    /// it is tried on the declared Build without the failing pick, and only when that Build is possible (one more
    /// walk, cached per slot), as `validate` does now (wording.rs:178-185). `None` when nothing opens it.
    pub fn opens(&self, refusal: &Refusal) -> Option<RankMove>;

    /// For a locked entry of values: its first value that a later level opens, with that value's name (199's
    /// entry rule: "for Longsword"). Walks as `opens`, per value, until one answers.
    pub fn entry_opens(&self, entry: &SlotEntry) -> Option<(RankMove, String)>;
}

/// The engine's answer for one refused pick or value. Owns no borrow, so an `Offer` (200) or the web app's `Arc`'d
/// entries can keep it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// As worded: a `Walk` issue trimmed to the character's classes at the slot's level (today's `Wording::trimmed`).
    /// The *status* keeps the picker's own issue: R never feeds back into a status (CODING_STANDARDS:16).
    pub issue: Issue,
    /// The engine's words, no tool and no address (CODING_STANDARDS:9): `issue_text`, with held-where for
    /// `AlreadyHeld`, the `MetLater` clause, and the 192 clause ("…: dodge is in Mythic rank 4's … slot, taken at
    /// level 14, after this slot's level 13"). `None` only for `BreaksLater`, whose words come from `breaks`.
    pub text: Option<String>,
    /// Each part of an unmet "and", once, without "Needs " (today's `tree.rs::needs`); empty for other issues.
    pub needs: Vec<String>,
    /// For `BreaksLater` with an edit: the choices and the cause, from `Refusals::breaks`.
    pub breaks: Option<Breaks>,
    /// What `opens` works the level out from. Private: callers ask `Refusals::opens`.
    later: Option<Later>,
}

enum Later { OwnRank { slot: SlotId, pick: Pick }, LaterRankPick { later: LaterRankPick, slot: SlotId, pick: Pick } }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankMove { pub rank: u8, pub level: u8, pub moves: Vec<(u8, u8)> }
impl RankMove {
    /// "MR 4 taken at level 12 opens it, which moves MR 5 to level 12" (199's one wording).
    pub fn text(&self, engine: &Engine) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breaks { pub refused: Vec<Choice>, pub names: String /* first 3, "and 4 more" */, pub cause: BreakCause }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BreakCause {
    /// "Dodge needs Dexterity above 10": a score the edit moves.
    Score { ability: Ability, side: Side, value: u8 },
    /// "…needs more than 2 Stealth ranks at level 3".
    Skill { skill: SkillKey, side: Side, ranks: u8, level: u8 },
    /// "Toughness is taken in level 3's Feat slot".
    Held(String),
    /// They need what the edit takes away. The adapter words it: its sentence names the address
    /// ("depends on what `slot.feat.1.class.0` holds now"; withdraw's "depends on `…`").
    Needs { slot_holds: bool },
    Other,
}
impl Breaks { /// The engine's words for every cause but `Needs`: "Would break …", "… needs Dexterity above 10".
    pub fn text(&self) -> Option<String>; }
```

**Invariants.**
- Reasons stay off the status path. `of` reads an `Issue` and never produces one that a status or `allows` sees. The self-checks (`tools/corpus/tests/self_checks.rs`) and the agreement sweeps don't call R.
- Walk per call, not per option (CODING_STANDARDS:17). Every walk sits behind a cache in `Refusals`: the sheet, the failing choices, the per-rank levels and each `without` Build. `of` is cheap and runs per option; `opens` and `breaks` are lazy and run per shown reason.
- `text` and `needs` name no tool and no address. A test sweeps the test world for this (§4).
- The two sources read alike for a pick and an issue they share. The picker's `MetLater` is the only difference, and stays (overall §6).

**Error modes.** None are surfaced:
- an address with no slot gives a `Refusal` with no `later`;
- `opens` returns `None` for a pick outside a rank's slot, and for a `Walk` Build with another failure.

## 2. Behind the seam

**Moves into `rules` (`engine/refusal.rs`)**

| From | Lines | Becomes |
|---|---|---|
| `slot_entries_in`'s per-option reason (`engine/pickers.rs`) | 180–195, plus the `walked` sheet at 170 | `Refusals::of(Source::Picker)`; `EntryOption.reason` and `SlotEntry.reason` become `refusal: Option<Refusal>`, with a `reason()` accessor so `slot_entries` callers (seeds `finesse_training.rs:124`, `class_rules.rs:141`) still read text |
| 199's later-rank fields and cache on the entries | wherever 199 puts them | `Refusal::later` and `Refusals::{opens, entry_opens}` |
| MCP `Breaks` (`apps/mcp/src/options.rs`) | 131–243 | `Refusals::breaks`, `Breaks`, `BreakCause::text`; `failing` and `sheet` become `Refusals`' |
| MCP `Breaks` helpers | `NAMED` (122), `Cause` (246–253) | `BreakCause` (`Needs`/`Other` keep withdraw's split) |
| `issue_reason` | `options.rs:277–281` | deleted; the `BREAKS_LATER` fallback (118) stays MCP and names `sheet` |
| `Wording::trimmed` (`validate/wording.rs`) | 141–152 | `Source::Walk` in `of` |
| `Wording::unmet` | 158–186 | `of` (the 192 clause) plus `opens` (192 *and 171*: closes 330.1) |
| `Wording::already_held` | 189–199 | `of` (held-where) |
| `needs` (`validate/tree.rs`) | 271–287 | `Refusal::needs` (reads P's normal `Unmet` after T1: `All`'s children) |

Kept in `rules`, unchanged and public:
- `issue_text`, the bare words. The seeds' ~26 calls keep it.
- `reason_for`, which the picker's status still uses.
- `rank_level_that_opens`, `later_rank_pick`, `later_rank_text`, `later_rank_levels`, `level_that_lets_in` (`ranks.rs:387–600`) and `held_where`. R calls them. Making them private is a later ticket with 200's.

**The four adapters, each with what it keeps**

1. **Web app** (`libs/ui/src/picker.rs`)
   - The slot picker shows `refusal.text`, then `" · "` and `opens(...).text()`. The entry line uses `entry_opens`, as 199 left it.
   - The older pickers' `reason` (449–454) becomes `Refusals::of(...).text`, and `BreaksLater` stays reasonless (glossary **Removable**).
   - Keeps: the separator only. No address or tool anywhere.
   - **[b]** By then 201 has replaced the older pickers with the Offer→`Item` conversion, so only that conversion reads `refusal`.
2. **MCP `options` and `choose`**
   - One function, `fn refusal_reason(refusals, refusal, address, fix: Fix) -> String`, in `options.rs` beside `Fix`.
   - It joins `text`, or for `BreaksLater` `breaks.text()`, or `Needs { slot_holds: true }` worded as "{names} depends on what `{address}` holds now", or `BREAKS_LATER`.
   - It then adds ". " plus `opens.text()` plus `opens_text`'s action clause ": `choose` `mythic.R.level` L first".
   - It replaces the duplicated block at `options.rs:984–998` and `choose.rs:231–251`.
   - `Offer`'s `reason`, `like` and `opens` (`options.rs:29–41`) become `refusal: Option<Refusal>`.
   - `slot_offers`' remaining reason code (652–669, 692–716, after 199) goes. The entry line gains the 171 hint through `entry_opens` (accepted item 4).
   - Keeps: `Fix`, the action clause of `opens_text` (766–777, now only the clause after the engine's text), `BREAKS_LATER`, `issue_code` and the `Needs` sentence.
3. **MCP `validate`** (`validate/wording.rs`, `tree.rs`)
   - `Wording::node` (33–55) builds one `Refusals` per call, with `Source::Walk` and the declared Build and its sheet, and one `Refusal` per failure.
   - `reason` (127–136) keeps only the branches that are diagnostics of a Declared Build, not refusals of a pick: `unknown` (201–320), `not_allowed` (323–469) and `not_reached` (486–558). Their closest keys, `options` pointers and addresses are tool clauses. Their fallbacks (306, 317, 428, 441, 467, 553, 556) read `refusal.text`.
   - The unmet and held-already reasons come from `refusal_reason(..., Fix::Declare)`.
   - `needs` comes from `refusal.needs`.
   - Keeps: `echoes` (tree.rs:341), the tree, `Given`/`pick_gives` (G's, T4) and `Fix::Declare`'s "declare `mythic.R.level` L".
4. **MCP `withdraw`** (`withdraw.rs:89–113`)
   - It asks `Refusals::breaks(&edit)` and keeps its sentence frames ("…, so `addr` can't be withdrawn now: withdraw X first") and `verb`.
   - Out of R: `gear::needed_reason` (66–73) and the rank-level case (76–87), which are already engine-worded.

The deletion test passes. Without R, the trim, held-where, 171/192 assembly, Breaks and `needs` come back in each of the four adapters, which is the state before 199.

## 3. Tickets

**What 199 must have landed for T5 to start**
1. `later_rank_level`, `later_rank_clause`, `Opens` and `LaterLevels` are gone from `options.rs`, and `later_rank_levels`/`joined` are gone from `picker.rs`.
2. The engine words the 171 and 192 texts on `EntryOption` and `SlotEntry`, and `RankMove`'s wording ("which moves") is the engine's.
3. A public engine type works the 192 level out lazily with a per-rank cache.
4. `validate`'s `unmet` calls that type, not the deleted MCP helpers. 199 doesn't say what `validate` uses; check on its merge.

If 199's cache lives inside a slot-entries result, T5a lifts it into `Refusals`.

**[c]** None of this applies: T5a writes `Refusals` and the lazy level itself, deletes `options.rs:741–839` and `picker.rs:605–695`, and owns the change from "moving" to "which moves". 199 shrinks to "the web app's slot picker reads `Refusals`".

### T5a: the engine words a refusal (picker side)

- **Problem.** The reason for a refused option is built three ways. The block in `options.rs:984–998` and `choose.rs:231–251` is duplicated; MCP's `Breaks` names choices in the tool crate; the MCP entry line in a rank's slot lacks the 171 hint the web app gives.
- **Change.** Add `Refusals`, `Refusal`, `Breaks` and `RankMove` in `rules`, plus `slot_entries_in` on `Source::Picker`. The MCP `options`/`choose` adapter becomes `refusal_reason`; the `withdraw` adapter calls `breaks`; the web app's older pickers read `refusal.text`.
- **Done when:**
  - the duplicated block is one adapter call;
  - `issue_reason`, MCP `Breaks` and `Cause` are gone;
  - `Offer` carries `refusal`;
  - the entry line in a rank's slot gains the 171 hint (item 4, exact-text test);
  - the reasons record shows no other change;
  - `FORM_PROOF_OFFERS=1` shows no status moved.
- **Files:** `libs/rules/src/engine/{refusal.rs (new), pickers.rs}`, `libs/rules/src/{engine.rs, lib.rs}`, `apps/mcp/src/{options.rs, choose.rs, withdraw.rs, gear.rs (Offer construction)}`, `libs/ui/src/picker.rs`, `libs/rules/tests/engine/{refusals.rs (new), main.rs}`, MCP tests.
- **Model:** Opus.

### T5b: `validate` asks R (walk side)

- **Problem.** 330 item 1: `validate` drops the 171 hint. It also re-derives the trim, held-where, the 192 clause and `needs` from the walk on its own (192's fix rounds d0097f67 and 9ce1f809).
- **Change.** The `validate` adapter (§2.3). `trimmed`, `unmet`, `already_held` and `tree.rs::needs` go.
- **Done when:**
  - 330.1 reads "… MR 9 taken at level 20 opens it, which moves MR 10 to level 20: declare `mythic.9.level` 20", tested exactly through `validate`;
  - with a second failure beside it, the hint is absent (as 192's);
  - the reasons record shows only the 330.1 lines changed.
- **Files:** `apps/mcp/src/validate/{wording.rs, tree.rs, mod.rs}`, `libs/rules/src/engine/refusal.rs`, tests.
- **Model:** Opus.
- **Ordering.** T5b after T5a, and after T4 (which shares `labels.rs` and `validate/`). One builder each, in sequence.

**Checks (both), from CLAUDE.md's table**
- `cargo test -p rules -p mcp -p ui -p frontend`
- `cargo test -p seeds -p corpus`
- `cargo build -p rules --target wasm32-unknown-unknown`
- `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`
- `scripts/css-classes.sh`
- `cargo fmt --all --check`
- `scripts/test-modules.sh`
- The form proof:
  - on the base, before the first edit: `FORM_PROOF_OUT=<dir> FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1 cargo test --release -p corpus --test form_proof record -- --ignored --nocapture`;
  - on the tip: `FORM_PROOF_BEFORE=<dir> … compare`, with the grouped counts in the Answer.
  - If T0 records only engine-level texts, the MCP-assembled 330.1 tail is invisible to it, and the exact-text test carries it (risk 6).

**What 200's Offer then takes from `Refusal`**
- `Offer.refusal: Option<Refusal>` replaces its `reason`, `like` and the 199 later-rank reason.
- Its `status: Result<(), Issue>` stays: a status is behaviour.
- 200's decision "Would break: the engine names the choices" is already done by T5a, so 200 drops it.
- `Engine::offers` returns its Offers with the call's `Refusals` (an `Offers<'a>` owning one), so `opens` stays lazy. The signature is 200's call.
- **[b]** 200 has already moved the duplicated block into `rules` with the `Offer`. T5a then replaces `Offer`'s `reason`/`like`/`opens` inside `rules` (`engine/offers.rs`, not `options.rs`/`choose.rs`), and its done-criterion "one block" becomes "`Offer` carries a `Refusal`". The web adapter is 201's conversion. T5b is unchanged.

## 4. Tests

**New at the interface** (`libs/rules/tests/engine/refusals.rs`, `test_world`, written first):

| Test | Pins |
|---|---|
| `a_picker_refusal_and_a_walk_refusal_of_one_pick_read_alike` | the same `text` and `needs` for Toughness (Mythic), an unmet feat, a held pick and an "or" trimmed, from `Source::Picker` on the possible Build and `Source::Walk` on the declared one |
| `a_walk_refusal_keeps_an_or_to_the_characters_classes` | the trim (test-world version of `validate.rs:501`) |
| `a_held_pick_says_where_it_is_held` | held-where from both sources |
| `a_ranks_pick_names_the_later_level_of_its_rank_that_opens_it` | 171 `opens`, both sources; Walk on the Build without the pick |
| `the_walk_names_no_level_when_another_choice_fails_too` | the `without` guard |
| `a_later_ranks_pick_says_where_it_is_and_the_level_that_lets_the_pick_in` | 192 clause in `text`, the level in `opens`, the moves along |
| `an_entry_of_values_takes_its_first_opened_values_level_for_that_value` | `entry_opens` ("for Longsword") |
| `breaks_names_the_choices_and_why` | each `BreakCause` |
| `needs_lists_each_part_of_an_and_once` | moved from 329's MCP `needs` |
| `no_refusal_names_a_tool_or_an_address` | sweep of every option's and failure's `text`/`needs` over the test world's Builds: no backtick, no `choose`/`validate`/`options`/`withdraw`/`declare`, no `slot.`/`mythic.`/`level.` |

**Moved into rules tests**
- From `rules/tests/engine/rank_levels.rs`: `a_ranks_slot_says_which_later_level_…` (614), `the_level_that_opens_a_ranks_pick_comes_with_…` (820), `a_pick_whose_prerequisite_is_a_later_ranks_…` (878) and `a_later_ranks_pick_is_named_only_for_the_value…` (926). After 199 they assert through entries; they move to `refusals.rs` through `Refusals`.
- From `slot_entries.rs`: the three held-where tests (194, 359, 395) and `…would_break_a_later_choice_shows_no_reason` (208, now the web adapter's rule: `text: None` unless `breaks`) move. The entry-grouping tests stay.

**MCP: one adapter test per tool form survives**
- `rank_levels.rs:83` (171, `choose`) gains `options` and `validate`, the latter being 330.1.
- 743 (`Fix::Choose`) and 784 (`Fix::Declare`, with the two-failure case) stay.
- `choose.rs:589/627/724` stay, as the `Needs` sentence with an address. 694 ("Would break combat-trick") moves to `breaks_names_the_choices_and_why`.
- `withdraw.rs:177` stays.

**`wording.rs` (329) shrinks to one adapter test each**
- Selections named alike: keep `validate`'s `needs`. The `options` and `choose` asserts are deleted, and the text moves to a real-data seeds test, `libs/seeds/tests/game_data/refusals.rs::extra_rage_power_names_each_selection_by_its_owner`, through `Refusals` with the GUIDs cited.
- Archetype level once: keep the `options` row. `choose` is deleted. `validate`'s `needs` moves to seeds `kinetic_knights_infusions_read_their_class_level_once`.
- Pentamic Faith and Impossible Domain test `describe`, which is not R. They stay.

**Seeds:** the ~26 `issue_text` calls and the `lock_reason` family (`support.rs:724, 771`) survive unchanged. `issue_text` keeps its meaning. X (T8) may later absorb the helpers.

## 5. Risks and unknowns

1. **The trim on two sources.** If `reason_for` isn't idempotent on an already-trimmed `Unmet`, `Source::Picker` must not re-trim; the design already passes it through. *Settle:* the read-alike test, plus a property loop over the test world's locked options asserting that `reason_for(reason_for(x)) == reason_for(x)`.
2. **The walk's issue and the picker's differ beyond `MetLater`.** For example, the picker trims at the state before the slot, the walk at the level's state. *Settle:* the read-alike test over every failing slot of the corpus's short run (seeds), listing each pair that differs. Each difference is fixed toward the picker, or named in the Answer as out (`MetLater`).
3. **The shape of 199's lazy cache.** *Settle:* read 199's merge first; if the cache is tied to a slot-entries result, lift it (T5a). **[c]** is moot.
4. **Where "for Longsword" sits beside the moves clause** (item 4): "…opens it for Longsword, which moves …" or 199's choice. *Settle:* take 199's engine text unchanged, and show the MCP entry line's new text in the Answer and the reasons record.
5. **Laziness changes the cost profile.** 171 becomes lazy in MCP (`options` with `only_open` stops computing it for hidden rows; `choose` computes it only for the refused value). `entry_opens` adds walks to `options` for locked entries of values in a rank's slot. *Settle:* time `options slot.mythic.N` with `only_open: false` on the corpus's heaviest mythic Build before and after (medians on a quiet machine), and put the numbers in T5a's Answer.
6. **What T0's reasons record covers.** If it records engine texts only (`issue_text` per option and failure), adapter-assembled tails (the 171/192 level, the action clauses) aren't proven by it. *Settle:* the builder reads T0's header. If the tails are uncovered, the MCP adapter tests above are the proof, and the Answer says so.
7. **The `Walk` `without` Build costs** one `is_possible` and one sheet per unmet rank-slot failure whose reason is shown. That is today's cost plus the 171 attempt. *Settle:* cached per slot in `Refusals`, counted in the Answer on 330.1's Build.
8. **The public `SlotEntry` API changes** (`reason` → `refusal` with an accessor). *Settle:* keep `reason()` as a method so seeds and `rules` tests compile unchanged; `cargo test -p seeds` is the check.
9. **Ordering with T1 and T4.** `needs` assumes P's normal `Unmet` (one level of `All`). Before T1 it would need `and_parts`. T4 shares `labels.rs`/`validate/`. *Settle:* start T5a only with T1, T4 and 199 merged (`git merge-base --is-ancestor`), per overall §5.
10. **`withdraw`'s two causes.** Today it splits only Needs from everything else. `BreakCause::Needs { slot_holds }` must keep withdraw's text byte-identical. *Settle:* `withdraw.rs:177` plus a `Needs` case test, unchanged.
