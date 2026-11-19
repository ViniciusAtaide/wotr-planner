# L: Ledgers, the detailed design

(Explorer, read-only, main at 790eff76, 2026-11-18. Fits `overall.md` §1 L and §5 T7; keys on N's Narrowing from T6. Paths under `tools/game-check/` unless said.)

One module, `src/ledger/`, two ledgers split by direction on one line reader and one two-sided reconcile. Owners live only here: N records evidence, never a ticket.

## 1. Interface

### The line reader (`ledger/lines.rs`)

```rust
/// A list's text and the name every error and stale line carries (`known.txt`, `user-facts.txt`, …).
#[derive(Clone, Copy)]
pub struct Source<'a> { pub file: &'static str, pub text: &'a str }

/// One line that holds something: its number (1-based), the text before ` -> `, the text after, and the line as
/// written (comment cut, trimmed), for stale reports.
pub struct Line<'a> { pub file: &'static str, pub number: usize, pub key: &'a str, pub decision: &'a str, pub text: &'a str }

/// Each non-blank, non-comment line split at its first ` -> `. `#` anywhere starts a comment (today's rule for three
/// of the four lists; no carrier hit holds `#`, pinned by a round-trip test).
pub fn lines(source: Source<'_>) -> Result<Vec<Line<'_>>, LedgerError>;

#[derive(Debug, PartialEq, Eq)]
pub struct LedgerError { pub file: &'static str, pub line: usize, pub why: String }
// Display: "{file} line {line}: {why}". `Error::Known(String)` (lib.rs:110, :127, which prefixes `known.txt:` to
// every list's error) becomes `Error::Ledger(LedgerError)`, printed as is.
```

- **Invariants:** `key` and `decision` are both trimmed and non-empty. Line numbers count every physical line.
- **Error modes:** "no ` -> <decision>`", and "an empty decision" for a trailing ` ->`. Each error names the file and the line.
- **Determinism:** the output follows file order.

### The reconcile (`ledger/reconcile.rs`)

```rust
/// Items against lines, both sides: each item with the line that decides it (`best`: the index of the winning line,
/// or none), the items no line decides, and the lines that decide no item.
pub struct Reconciled<I> { pub decided: Vec<(I, usize)>, pub unlisted: Vec<I>, pub stale: Vec<usize> }

pub fn reconcile<I>(items: impl IntoIterator<Item = I>, lines: usize, best: impl Fn(&I) -> Option<usize>) -> Reconciled<I>;
```

- **Invariants:** `decided` and `unlisted` keep the items' input order. `stale` is ascending, i.e. file order. Every item is in exactly one of `decided` and `unlisted`.
- It is pure, and it has no error modes.

### The known-differences ledger (`ledger/known.rs`)

*Is this finding decided, and which ticket removes it?*

```rust
pub struct KnownDifferences { source: &'static str, lines: Vec<KnownLine> }
struct KnownLine { number: usize, text: String, pattern: Pattern, ticket: Ticket }
/// `area[ / kind[ / entry[ / value]]]`, kind `*` any. Rank: (value, entry, named kind), the earlier line on a tie.
struct Pattern { area: Area, kind: Option<String>, entry: Option<String>, value: Option<String> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ticket { Assigned(String), Unassigned } // `-> ?`: written by regeneration for the main session to set

pub struct KnownOutcome<'f> {
    pub known: Vec<(&'f Finding, &'f str)>,  // finding, its ticket
    pub unknown: Vec<&'f Finding>,           // no line, or an Unassigned one
    pub unassigned: Vec<String>,             // `line <n>: <text>` of `?` lines that decide a finding
    pub stale: Vec<String>,                  // `line <n>: <text>` deciding none, file order
}

impl KnownDifferences {
    pub fn embedded() -> Result<KnownDifferences, LedgerError>;   // include_str!("../../known.txt")
    pub fn parse(source: Source<'_>) -> Result<KnownDifferences, LedgerError>;
    pub fn ticket_for(&self, finding: &Finding) -> Option<&Ticket>;  // for a writer that leaves listed items out (307c)
    pub fn reconcile<'f>(&'f self, findings: &'f [Finding]) -> KnownOutcome<'f>;
    /// `text` with the block of `area` (between `# BEGIN <area>` and `# END <area>`) written again from `findings`:
    /// each line's ticket kept where its key still decides a finding, stale lines dropped, a new finding `-> ?`.
    /// Every byte outside the block is kept.
    pub fn regenerate(text: &str, area: Area, findings: &[Finding]) -> Result<String, LedgerError>;
}
```

- **Invariants:**
  - Every line names a known `Area`, has at most four parts, and has a non-empty ticket (`?` is Unassigned).
  - No two lines have the same pattern. `items` and `items / *` count as the same pattern, as Facts already rules (`facts.rs:83-91`).
- **Errors** (all `known.txt line N: …`):
  - no area `x`
  - more than area, kind, entry and value
  - no ticket
  - the same as line M
  - a block with no end, and a block in an area it doesn't name (from `regenerate`)
- **Determinism:** `known` and `unknown` keep the findings' sorted order (`Out`'s findings are sorted by `Report::build` today). `stale` is in file order. `regenerate` writes a block's lines in the order the findings are sorted.

### The write registry (`ledger/registry.rs`)

*May the writer make this change?*

```rust
pub enum Write<'a> {
    Change { area: Area, kind: &'a str, entry: &'a str },
    Narrowing(&'a qualification::Narrowing),   // N's type (T6): entry, kind, fact, evidence; `said()` prints it
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision { Free, Held, Applied(String), Allowed(String) /* the line's reason, its evidence */ }

pub struct WriteRegistry { facts: Vec<FactLine>, allowed: BTreeMap<(String, String), AllowLine> }

impl WriteRegistry {
    pub fn embedded() -> Result<WriteRegistry, LedgerError>;   // user-facts.txt and prerequisite-narrowing.txt
    pub fn parse(facts: Source<'_>, allowlist: Source<'_>) -> Result<WriteRegistry, LedgerError>;
    pub fn decide(&self, write: Write<'_>) -> Decision;
    /// Under (a): the user-fact lines naming an entry the data lacks (today's `Facts::stale`): `--write` exits 2.
    pub fn stale_facts(&self, data: &GameData) -> Vec<String>;
    /// The allowlist lines no narrowing of the plan's record makes: the plain check exits 1.
    pub fn stale_allowed(&self, made: &[qualification::Narrowing]) -> Vec<String>;
    pub fn kinds(&self) -> Vec<(Area, String)>;   // facts.rs:104, for every_kind_in_the_registry_is_one_the_check_emits
}
```

- **Invariants:**
  - A `Change` decides only `Free`, `Held` or `Applied`.
  - A `Narrowing` decides only `Allowed` (a line names it) or `Held` (no line: the entry stays whole). A narrowing is never Free.
  - `user-facts.txt` is read, never written: no function returns its text.
  - The allowlist keeps today's grammar. Its narrowing part is parsed by N's `NarrowingKind::parse_said`, so N's new kinds (adds requirement, raises value, Any to All) are accepted, not only `drops` and `adds Not`.
- **Errors:** `user-facts.txt line N: …` keeps today's five refusals (`write.rs:889`). `prerequisite-narrowing.txt line N: …` keeps today's six (`prerequisites_write.rs:957`).
- **Determinism:** stale lines come out in file order. `decide` is a pure lookup; on a tie the earlier line wins.

### Under (b)

If the user picks (b), the registry gains `stale_held(&self, findings: &[Finding]) -> Vec<String>`. It returns the `held` lines whose pattern decides no finding, using the reconcile with `Pattern`'s rank. Applied lines are never stale here: once written, they have no finding.

`check_all` attaches those lines under "user-facts.txt held lines that match no difference:", and the plain check exits 1 on them. Clearing one then needs a `User approved:` line naming its fact. That makes a third test, `a_held_line_no_difference_matches_fails_the_check`, and `stale_facts` stays for `--write`.

## 2. Behind the seam

### What moves, function by function

| Today | Becomes |
|---|---|
| `known.rs:1-127` (`Known`, `parse` 40-75, `ticket_for` 86-114, `lines` 117-126) | `ledger/known.rs`; `parse` on `lines()`; ranking in `Pattern`; deleted file |
| `report.rs:201-262` (the `used` vector, the known counts, the stale lines) | `KnownDifferences::reconcile`; `Report::build` takes `KnownOutcome` and only prints |
| `report.rs:283-324` `with_carriers` | deleted |
| `report.rs:269-281` `with_stale_narrowings` | kept, fed by `stale_allowed`; text unchanged |
| `carriers_known.rs:1-130` (`KnownCarriers`, `CarrierCheck`) | deleted, with `lib.rs:14,23` |
| `write/facts.rs:1-169` (`Facts`, `Decision`, `decide` 126-143, `stale` 146-168, `kinds` 104-114) | `ledger/registry.rs`; `decide` shares `Pattern`'s rank; `facts.rs` deleted |
| `write/prerequisites.rs:162-253` (`Narrowings`, `parse`, `allows`) | `registry.rs`; `allows` is `decide(Write::Narrowing)` |
| `prerequisites.rs:259-277` `stale` | shrinks to `stale_allowed(&record.made)`: no second `plan` |
| `prerequisites.rs:283-312` `audit` | reads N's record (T6); its strings go |
| `prerequisites.rs:1013-1063` `owner` | deleted in T6 if N drops owners, else here (§5) |
| `lib.rs:110,127` `Error::Known` | `Error::Ledger(LedgerError)` |
| `write/mod.rs:675-711, 782-790`; `tables.rs:125` | take `&WriteRegistry`; `run` passes an empty allowlist |
| `main.rs:46-48, 159-166, 182, 217-238, 302-303` | rewired as below |

### Carrier hits become findings

- `report.rs:15-60` gains `Area::Carriers`, named `"carriers"` and placed last in `ALL`, so no other area's line moves.
- A new `areas/carriers.rs::compare(record, &mut out)` reads N's record of every entry's carrier narrowings: the absolute audit, `overall.md` §4. For each audited entry it calls `out.compared`. For each hit it adds:
  ```
  Finding { area: Carriers, kind: "holds" | "left whole", entry: "<entry>", guid: <grantor GUID>,
            blueprint: <grantor name>, game: "<fact> granted by <grantor> <GUID>", planner: "<planner entry> doesn't grant" }
  ```
- `check_all(pins, dump, data, written, known) -> Result<Report, Error>` is `check` plus this area. Here `written` is the data as the files hold it, which the writer and N need.
- The fixtures keep calling `check`, which has no carriers area. The Areas block then prints `carriers: nothing compared`, which the fixtures' expected texts must accept (§4).
- The plain check calls `check_all`. The writer's before/after (`main.rs:295,375`) keeps `check`, so the log's "findings removed" is unchanged.

### The 176 lines move in the same commit (T7b)

`carriers-known.txt` is deleted. `known.txt` ends with:

```
# BEGIN carriers
# The carrier audit's hits (ticket 322; 308h): … [the old header's prose, minus its grammar]
# Regenerated, never edited inside: game-check --carrier-audit --known-format > known.txt.new && mv …
carriers / holds / feat ability-focus-bombs / feature bomb granted by ExplosiveExpert 8a551fcc… -> 307
…
# END carriers
```

The migration is mechanical, with no hand-rewritten line:
1. Add an empty block.
2. Run `--carrier-audit --known-format`. Every hit comes out as `-> ?`.
3. Run one Edit `replace_all` of ` -> ?` to ` -> 307` inside the block. That is valid because all 176 tickets are 307 today: `uniq -c` gives `176 307`.
4. The check exits 0, and the carrier groups total 176.

### Regeneration keeps the prose

- `regenerate` replaces only the lines between the area's markers and keeps every byte outside them. The prose that matters, `known.txt:15-60` and its like, sits outside every block.
- Comments inside a block are dropped. The block's first comment line, which is kept, says so.
- `--known-format` reads `known.txt` from disk, not the embedded copy, and prints the whole file.
- The `.new` and `mv` steps stay: redirecting straight onto `known.txt` would empty it before cargo embeds it.
- The check's failure message prints the command, as `with_carriers` does today (`report.rs:315-322`).
- 307c's `carriers-left.txt` becomes one more `Area` and its own block. Its writer asks `ticket_for` to leave listed candidates out. The finding's shape is 307c's call.

### The guard still sees `held` lines byte-identical

`user-facts.txt` keeps its path, grammar and bytes. Nothing in L writes it, and the parser reads it as today: `#` anywhere, ` -> held`, and the name is the last ` / ` field, as `guarded-files.sh:177-185` reads it.

A new test pins that coupling: every line the registry decides `Held` must be one the guard's rule names, by that same last field. `scripts/guarded-files.sh` does not change.

## 3. Tickets

### T7a: the line reader, the reconcile, both ledgers on today's four files

- **Problem:** four parsers, four ideas of a stale line, and errors that all say `known.txt` (`lib.rs:127`). This also makes the second `plan` per check go.
- **Done when:**
  - `cargo test -p game-check` passes, with each new test from §4 in `--list`.
  - The plain check's stdout is byte-identical to the base (`cmp` on both runs).
  - `--write <every area> --dry-run` output is byte-identical.
  - The exit codes are 0, 1 and 2 with today's meanings.
  - A deliberately broken line in each of the four files prints that file's own name.
- **Files:**
  - new `src/ledger/{mod,lines,reconcile,known,registry}.rs`
  - `src/{lib,report,main}.rs`, `src/write/{mod,tables,prerequisites}.rs`
  - deleted `src/known.rs` and `src/write/facts.rs`
  - `tests/fixture/{known,write,prerequisites_write,support,pins,determinism}.rs`
- **Checks:** `cargo test -p game-check`, `cargo fmt --all --check`, `scripts/test-modules.sh`, and the game check (main checkout's dump) on the base and the tip.
- **After:** T6. **Model:** Sonnet to build.

### T7b: carrier hits as the carriers area; `carriers-known.txt` folded in

- **Problem:** a third comparison in the check (322), a list with its own grammar, and a fifth one coming with 307c.
- **Done when:**
  - The report is the base's except for: one more Areas line (`carriers: N compared, …`); the carrier section, now `carriers / holds [known: 307]: N` groups; and the Differences line, whose counts now include the hits. The builder shows this as a `diff` of the two reports, and lists every changed line.
  - The check exits 0.
  - A planted unlisted hit gives 1, a stale block line gives 1, and a `?` line gives 1.
  - `--known-format` round-trips the tip's `known.txt` byte for byte.
  - `carriers-known.txt` is gone.
- **Files:** `src/areas/{mod,carriers}.rs`, `src/{report,main,lib}.rs`, `src/ledger/known.rs`, `known.txt`, deleted `carriers-known.txt` and `src/carriers_known.rs`, `tests/fixture/{prerequisites_write,command,known}.rs`.
- **The main session's edits:**
  - CLAUDE.md's game-check paragraph: it now names `known.txt`'s carriers block in place of `carriers-known.txt`.
  - 307c's ticket, which then targets `known.txt`.
- **Checks:** the same as T7a, plus `--carrier-audit --known-format` diffed against the file.
- **After:** T7a. **Model:** Opus to build.

## 4. Tests

### New, at the ledger's interface (`tests/fixture/ledger.rs`)

| Test | What it pins |
|---|---|
| `each_lists_errors_name_its_own_file` | one malformed line per Source prints `known.txt line 2:`, `user-facts.txt line 2:`, `prerequisite-narrowing.txt line 2:` |
| `a_comment_anywhere_ends_the_line` | the `#` rule |
| `a_trailing_arrow_is_an_empty_decision` | the reader's refusal |
| `the_reconcile_keeps_item_order_and_lists_stale_lines_in_file_order` | `Reconciled`'s order |
| `a_known_line_without_a_ticket_is_refused`, `an_unknown_area_is_refused`, `five_parts_are_refused`, `two_lines_of_one_pattern_are_refused` | today's untested Known refusals, plus the new duplicate rule |
| `a_value_matches_the_game_or_the_planner_value_whole` | value matching |
| `a_star_kind_ranks_below_a_named_kind_and_above_the_area` | the rank |
| `of_two_equally_specific_lines_the_earlier_wins_and_the_later_is_stale` | the tie-break: a value line matching `game` and one matching `planner` |
| `an_unassigned_line_decides_its_finding_but_fails_the_check` | `?` |
| `regenerating_a_block_keeps_every_byte_outside_it_and_each_listed_ticket` | prose kept, stale lines dropped, new findings `-> ?` |
| `a_block_without_its_end_is_refused` | regenerate's error |
| `every_carrier_line_round_trips` | the parse then print of each embedded block line is identical (no ` / `, ` -> ` or `#` inside a hit) |
| `a_change_is_never_allowed_and_a_narrowing_never_free` | the registry's ranges |
| `every_held_line_is_one_the_guard_names` | the registry and `guarded-files.sh`'s rule agree on the embedded file |
| `the_report_is_the_same_bytes_twice_with_known_lines_and_carriers` | `determinism.rs` with a non-empty ledger (today it has `Known::default()`, `:46`) |

### Survive

- `known.rs:6,22,31` survive. `:22` adds `carriers -> a` only if it moves to `check_all`.
- `pins.rs:17` survives.
- The facts tests at `write.rs:821-953` and `:1279` survive, with `Facts::parse(x)` becoming `WriteRegistry::parse(facts(x), allow(""))` through a support helper.
- The narrowing tests at `prerequisites_write.rs:870,902,947,957,989,995` survive the same way. `:995`'s report text is unchanged.
- `command.rs:62` survives.

### Move

`prerequisites_write.rs:1192` and `:1242` are rewritten against the carriers area and `regenerate`, and asserted on the new report text.

### Deleted

`:1267`, whose refusals are covered by the known-ledger refusals, and `:1283`, whose file is gone.

## 5. Risks and unknowns

- **N's Narrowing and its said text are T6's.** How the builder settles it: read T6's merged `qualification` before the first edit. If T6 slips, T7a keys the allowlist on today's `(entry, said)` strings, and T6 swaps the type in.
- **Who deletes `owner()`.** If T6 keeps it, T7a deletes it, and the writer's blocked narrowings print `-> ?`. That would change `a_blocked_narrowing_names_its_facts_owner` (`:902`) and the dry-run output. To settle: the main session decides in T6's brief, and T7a's byte-identical claim then excludes those lines.
- **A duplicate in `known.txt` becomes a parse error** (exit 2, where today it was a stale line and exit 1). Main's check exits 0, so no duplicate decides a finding today. To settle: the builder parses the embedded file at the base, before the first edit.
- **A hit text breaking the grammar.** `every_carrier_line_round_trips` settles it. If a hit holds ` / `, `Finding.game` drops the grantor's name and keeps the GUID.
- **The report's count line moves in T7b,** and so does group order, since the 176-hit group is sorted by size. To settle: the done-criterion lists every changed line, and the main session approves the form.
- **`?` is a new decision.** It realises "owners live only in L", but it lets a ticketless line be committed. To settle: the check fails on it, and the main session assigns the ticket before merging.
- **`check` vs `check_all`.** Fixtures without a written-data copy keep `check`. The builder confirms with `analyze_calls` that only `main.rs` needs `check_all`.
- **Under (b),** a stale `held` line fails every plain check until the user approves its removal. That is the reason for recommending (a).
