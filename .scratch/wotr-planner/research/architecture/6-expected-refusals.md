# Candidate 6: one list of each Link's expected refusals — findings

(Explorer report, read-only, main at 790eff76, 2026-11-18.)

Terms: a "Link" here is an old Build link that a stale-link test opens on the current data; an "expected refusal" is a pick in it the game now refuses.

## 1. Current state

| # | Where | Excuses (entry / slot / reason) | Citation | Used by | Ticket |
|---|---|---|---|---|---|
| A | `support.rs:435-463` `without_unproficient_picks` | Any entry, any slot, `Issue::NotProficient`; plus any slot whose only failure is `Unmet BaseAttackBonus` when the Build holds `druid/feyspeaker`. **Panics on any other failure**, so it must run last. | `BlueprintParametrizedFeature.CanSelect` (WrathDecompiled 6ce4a6b); `BABLow` 0538… | `first_ascension.rs:207`, `ranks_and_pools.rs:588` | 251, 293l |
| B | `support.rs:465-500` `without_picks_the_fighters_list_lacks` + `THE_FIGHTERS_LIST_LACKS` | 11 keys, slot kind `Feat("combat-feats")`, `Issue::NotAllowed`; other failures pass through | FighterFeatSelection 41c8… plus 6 feat GUIDs; "their Improved, Dragon Roar or Stat Focus" without GUID | `first_ascension.rs:203` | 306s |
| C | `support.rs:342-388` `without_mythic_abilities_the_game_refuses_since_308d` | `over-infused-blasts` when Unmet is exactly `ClassLevel kineticist 1`; `impossible-domain` when it is an `Any` of `*/domain` selections. Matched by slot *id* `SlotId::Mythic`, not slot kind. **Also removes the dependent `MythicSelection{rank,0}` pick.** | DoubleKineticistInfusions 7410…, ExtraDomain 213a…, 3 domain-selection GUIDs | `first_ascension.rs:204`, `ranks_and_pools.rs:589` | 308d |
| D | `support.rs:390-433` `without_discoveries_the_game_refuses_since_308e` | `alchemist/discovery/nauseating-flesh` in slot kind `Selection(alchemist/discovery)`, only if the Build takes alchemist and the Unmet names alchemist 12 and nothing outside a hand-written alternatives list; returns the slots it empties | NauseatingFlesh de11… | `first_ascension.rs:205`; exact list asserted `:223-230` | 308e |
| E | `tools/corpus/tests/links/306-refused.txt` (466 lines, ~18 blank-separated blocks, no headers), read by `problems` (`links.rs:119-162`) through `Stage`/`S2` (`:110-173`) | **The Build name alone**: no slot, entry or reason | None; reasons live only in each ticket's Answer (`306aa:45-52`, `306n:36`, `306s:39`, `308d:82-89`, `308e:178`, `308f` note `:89`) | `every_s2_base_link_opens_the_same_or_through_an_heir_or_is_listed` (`links.rs:240`) | 306c (empty), 306e…308f: 18 commits |
| F | `links.rs:250-287, 320-328`: `SKALD_BONUS_FEAT_LACKS`, `BATTLE_PROWESS_LACKS`, `starts_with("rogue/rogue-talents/")` | Skald: 10 keys in `Feat(skald/bonus-feat)` (scoped by 325). **Battle Prowess: 12 keys, any slot.** **Rogue talents: a key prefix, any slot.** | SkaldFeatSelection 0a19…; BattleProwessSelection 29b4… / RagePowerSelection 2871…; rogue talents: none | `a_battle_scion_link_from_the_s2_base_still_opens` (`links.rs:292`) | 306n/325, 306h review, 306e |
| G | `option_lists.rs:10-16` `EXCEPTIONS` (empty since 308c), checked by `every_exception_covers_an_option_the_prerequisite_leaves_out` (`:145-170`) | Options left out of a feat's prerequisite (feat, prefix, reason) | Reason text | Its own test | 306aa/308c |
| H | `ranks_and_pools.rs:573-606` over `links-before-the-splits.tsv` | An implicit count `(count, stripped) == (8, 5)`; which ones only in a comment | Comment: FakeDivineSparkSelection e6d3… | Itself | 308d |

The agreement sweeps' `KNOWN` lists (`seeds/tests/game_data/agreement.rs:18`, `rules/tests/engine/agreement.rs:14`, read by `rules::agreement::known_for`) are empty and hold engine-to-engine disagreements, not refusals. `form_proof.rs:847-926` computes "Builds whose links are refused" from its own link census, never from E.

## 2. The concept

One fact: an older Build holds a pick the game refuses, at a place, for a reason, on a blueprint's word. They differ in:

- **Key:** Build name (E); entry + slot kind + issue shape (B, D, skald in F); entry + slot-id shape (C); issue alone for any entry (A's NotProficient); a Build trait (A's Feyspeaker); key prefix or key alone (F's rogue talents, Battle Prowess).
- **Effect:** A–D strip the pick and return a Build the caller asserts `possible`; E and F classify a `Fate::Refused` (`tools/corpus/src/links.rs:69-110`) as allowed.
- **Strictness:** A closes the set by panicking; B, C, D pass other failures through, so chain order matters. E has two-way staleness (a listed Build that opens fails; a listed name with no base link fails). B, C, F have none. D's is the asserted list at `:223-230`.
- **Cascade:** only C removes a dependent pick.

## 3. Tests today, and what an over-broad exception hides

- The interface is a chain of `Build -> Build` functions composed in a fixed order (`first_ascension.rs:200-207`, `ranks_and_pools.rs:585-590`); each data ticket edits a helper and every caller's chain; callers must know A panics and so comes last.
- E's hole is the largest: `links.rs:149` `(Fate::Refused(_), true)` excuses the whole Build for *any* refused pick. A later wrong refusal in a listed Build is invisible. F exists because of it (306e, 306h, 306n re-checked the Battle Scion Builds with a second list).
- Over-broad exceptions that hid things: 306s's first build put the fighter-list keys into A as `NotAllowed` in any slot (`b2feebab`); 308e's first helper dropped Nauseating Flesh wherever refused (`308e:117`, fixed `7ca57896`); 306n's skald list matched by key alone (325 finding #3).

## 4. Damage from the scatter

- Fix rounds: `b2feebab` (306s), `7ca57896` (308e), 325 item 3; `CODING_STANDARDS.md:31` names 306aa, 306s, 308c, 308e.
- The standard is still broken: `BATTLE_PROWESS_LACKS` (`links.rs:274`) has no slot kind; the rogue-talent prefix (`links.rs:322`) is neither entry nor slot kind and has no citation (325 left both "outside the ticket"); A's NotProficient arm matches any entry.
- E has no row identity: 7 Build names listed twice.
- 306aa's Answer needed a paragraph per line (`306aa:45-52`).
- 18 data tickets edited one file, so they "touch the same files" under CLAUDE.md's Parallel agents rule, against the one-file-per-ticket rule for data notes.

## 5. Constraints

- `CODING_STANDARDS.md:32`: an exception names its entry and slot kind, never a key alone, and cites the game's reason. B is the model.
- `CODING_STANDARDS.md:33`: an exact reason stays exact; today's matchers hand-match `Issue` shapes (`support.rs:366-379, 414-427, 446-459, 471`).
- Citations: blueprint name and GUID, or WrathDecompiled file and method with the commit.
- `scripts/guarded-files.sh` does not cover E or the helpers. Guarding every row is too heavy (each data ticket adds rows); a mechanical check catches more: every row cited; the cited GUID is the entry's blueprint or its slot kind's selection blueprint (`support::BLUEPRINTS`, `support.rs:17`); every row hit; exact reason matches. The user could guard added rows that excuse a gold Build or a bench link (306s re-encoded `fighter20`'s bench link: `306s:31`).

## 6. Overlap

1. **Qualification diff:** `prerequisite-narrowing.txt` (`<entry> / drops <fact> -> <reason>`) approves a narrowing; these rows are its victims in real Builds. A row could cite its narrowing line, or the diff could propose rows.
2. **Refusal module:** the expected reason should be its answer; today `Engine::failures` → `rules::Failure { choice, issue }` (`libs/rules/src/choice.rs:132`), `Issue`/`Unmet` (`:73`), `support::why_not` (`support.rs:46`), `Engine::issue_text`.
3. **Ledgers:** a fifth ledger of the same shape (row, ticket, reason, stale-row failure), like `known.txt`, `carriers-known.txt`, `prerequisite-narrowing.txt`, `user-facts.txt`, and also `option_lists::EXCEPTIONS`, `agreement::KNOWN`/`known_for`, `gold::KNOWN_GAPS`. Checked by the engine over corpus Builds, not against the dump.
4. **Grant index:** slot-kind facts ("only the skald's level-1 rows grant `skald/bonus-feat`", `sheet.slot(slot).kind`, `support::grants`/`class_rows` `support.rs:948-999`) could check a row's slot kind is granted where its entry can be picked.
5. **Normalised Prerequisite reading:** C's `domain_selections` (`support.rs:355`), D's alternatives (`:401-408`), `support::conditions` (`:31`), `and_not` (`:1094`), `Prerequisite::canonicalize` hand-normalise to match an `Unmet`.

## 7. Shape options (uncommitted)

- **S1. Per-Build rows** (Build, address, entry, slot kind, exact issue, citation, ticket), generated from `compare`'s `FORM_PROOF_LINKS_OUT`. Exact, closes E's hole; large (~460 Builds × picks); couples to generator names; other corpora (`OLD_LINKS`, `links-before-the-splits`, bench) need names.
- **S2. Rule rows** (entry, slot kind, issue shape, citation, ticket), ~60 rows or fewer; one module `excuse(&Engine, &Build) -> Excused { build, hits per rule, unexcused: Vec<Failure> }`. E becomes derived (a Build is an expected refusal iff every failure is excused); every corpus through one seam; a rule with zero hits is stale. Needs an issue-shape language (from 2 and 5); A's issue-keyed and Feyspeaker rows and C's cascade need fields. Deletion test: helpers A–D, F's lists, E, the ordered chains and per-line justifications all come back — it earns its keep. One adapter (the real engine): the seam is between rows (data) and matcher.
- **S3. Hybrid:** keep E's Build lines with a `<TAB>rule-id` into S2's rules. Smallest migration; still one shared file keyed by Build.

Surviving tests: `links.rs`'s two-way staleness (generalised to rows); `first_ascension.rs:223-230`'s exact drop list (as per-rule hits); `ranks_and_pools`'s `(8, 5)`; `option_lists::EXCEPTIONS` is a different concept (an option left out) and could share only the ledger reader. Deleted under S2: `a_battle_scion_link_from_the_s2_base_still_opens` and helpers A–D.
