# Compact MCP responses

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The Max DC run (2026-11-09, see [119](119-caster-line.md)) spent much of its context on text it didn't use:
- `search {kinds: ["mythic_ability"]}` returned all 63 abilities with full prerequisites. Witch Wandering Hex's prerequisite alone names about 55 hexes; Mythical Beast's names 17 animal companions, two of them twice ("Animal Companion or Animal Companion").
- `options` on `slot.mythic.1` listed 41 open abilities with summaries, and every `options` call repeats the envelope (`build`, `link`, `todo`), a code and a link of about 300 characters each, twice per call.
- Every `choose` returns the link beside the code; the agent needs the link only at the end.

## Scope

1. **A `compact` argument** on `search` and `options`: each hit or option as key, name, kind or category, and status, with no summary, prerequisite or guide link. The default stays as today.
2. **Long prerequisites shortened** in `search` and `describe`: past a length, a prerequisite of many alternatives reads "one of 55 hexes (Air Barrier, Battle Master, …)" with the full tree under `describe`. Duplicate alternatives in the text are merged (the Mythical Beast case is the text's, or the data's; find which).
3. **The envelope on request:** `options` returns `build` and `todo` only when asked (`envelope: true`), since it changes nothing; `choose`, `withdraw` and `new_build` keep it.

## Open questions (with recommendations)

1. **Drop `link` from the envelope?** *Recommend:* keep it on `choose`; it's what the user gets, and the agent may stop at any step.
2. **Mythical Beast's "Animal Companion or Animal Companion":** *Recommend:* if two features share a name, the text names their owner ("Animal Companion (Hunter)") rather than merging them, as the other alternatives already do.

## Done when

Server tests cover `compact` on both tools, a shortened prerequisite and `options` without the envelope; the run's `search {kinds: ["mythic_ability"]}` is under half its size in compact form (measured, the numbers in this ticket); `scripts/check.sh` passes.

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (user: "where a ticket has open questions, take the recommended answers").

1. **`link` stays in the envelope** of `choose`, `withdraw`, `new_build` and `sheet`: it's what the user gets, and the agent may stop at any step. Recommended answer, taken while the user slept, 2026-11-09.
2. **Mythical Beast's "Animal Companion or Animal Companion" was the text renderer's**, not the data's: the data names two different options, `druid/nature-bond/animal-companion` and `oracle/revelations/nature/animal-companion`, both called "Animal Companion", and `rules`' prerequisite text named options by name alone (features already read with their owner when they share a name). Now, in a group (`Any`, `All`, `AtLeast`, and the `Any` of a refusal's reason), an option whose text another member shares reads with its Feature selection: "Animal Companion (Nature Bond) or Animal Companion (Nature Revelations)". When the selections share a name too, the selection's owner comes first: Disruptive's "Fighter level 6 or Arcane (Bloodline) or Arcane (Bloodline)" now reads "… or Arcane (Bloodrager Bloodline) or Arcane (Sorcerer Bloodline)". They are named apart, not merged, as recommended. Options alone, or whose name no other member shares, read as before. A scan of every feat's, mythic pick's, class's and archetype's prerequisite found three texts this changes: Mythical Beast, Boon Companion and Disruptive. Recommended answer, taken while the user slept, 2026-11-09.
3. **`compact`** (default false) on `search`: each match as `kind`, `key`, `name`, `category` and `matched` (ticket 123) only, with no summary, parent, prerequisite or guide link. On `options`: each option without its summary; `value`, `name`, `status` (with a closed option's issue and reason), `category`, `takes` and `current` stay, since `choose` and a refusal need them. The smallest reading of scope 1.
4. **Shortened prerequisites in `search` only:** a prerequisite whose text is over 200 characters has each group of more than 6 alternatives (`Any`, or `AtLeast`) read as its count, its noun and the first three: "one of 49 hexes (Air Barrier, Battle Master, Battle Ward, …)", "one of 17 animal companions (…)", "2 of 27 alternatives (…)". The noun is the alternatives' one name in the plural (a bracketed ending aside), else their Feature selections' one name, else "feats", else "alternatives". `describe` gives the whole text, which is where scope 2 puts "the full tree". The shortening is `Engine::short_prerequisite_text`, beside `prerequisite_text` in `rules`, so it shares the renderer and its naming apart.
5. **`options` without the envelope** unless `envelope: true` (then `build`, `link` and `todo`, as before). The other Build tools keep it.
6. The tool descriptions of `search` and `options`, the `options` argument docs, and two lines of the server instructions (workflow step 2: `options` returns the envelope only when asked; the Values paragraph: `compact`) say so.

## Progress log

- 2026-11-09: Found the duplicate's source: both "Animal Companion" alternatives are options (Nature Bond's and the Nature revelations'), and `Engine::prerequisite_text` named options without telling them apart. Fixed it in `libs/rules/src/engine/labels.rs` (`told_apart`), for prerequisite texts and refusal reasons, and added `short_prerequisite_text` there. Two `rules` tests in `tests/engine/labels.rs` (options of one name in a group, in a text and a reason; a short prerequisite's count, noun and first three, for `Any` and `AtLeast`).
- Five server tests in the new `apps/mcp/tests/server/compact.rs`: compact `search` of every mythic ability (only kind, key, name and category; under half the size), Witch Wandering Hex shortened in `search` and whole in `describe` (and a short prerequisite unchanged), Mythical Beast's options told apart in `describe` and shortened by their shared name in `search` (and Disruptive's two Arcane bloodlines told apart by their classes), compact `options` without summaries, and `options` with the envelope only when asked. One existing test (`a_feat_slot_lists_its_open_feats_and_counts_the_rest`) now wants no envelope by default.
- Measured on real data, as compact JSON of the structured result:
  - The run's `search {kinds: ["mythic_ability"], limit: 100}` (63 abilities): 24,573 bytes before this ticket (after 123), 22,603 bytes with the long prerequisites shortened, **6,250 bytes with `compact: true`**, about a quarter.
  - `options` on a Human Fighter 1's general feat slot (59 open feats): 11,606 bytes as before (with the envelope), 11,002 without it, 5,611 with `compact: true`. The envelope's share grows with the Build: this one's code is 30 characters, the run's about 300, so the run saved about 600 characters a call twice over.
  - Search hits whose prerequisite is shortened (feats, mythic picks and classes, `limit: 100` per kind and feat category): Witch Wandering Hex and Extra Hex (49 hexes), Mythical Beast and Boon Companion (17 animal companions), Domain Zealot (37 domains), Accomplished Sneak Attacker and Mythic Sneak Attack (14 sneak attacks), Extra Rage Power (38), Extra Rogue Talent (36), Extra Arcanist Exploit (16), Extra Discovery and Extra Arcana, the wild-shape feats, Shifter's Edge, Loremaster and Dragon Disciple; Arcane Trickster's 923-character "2 of: …" reads "2 of 27 alternatives (Accomplished Sneak Attacker, …)". Hellknight Signifer (292) and Extra Channel (231) stay whole: no group of theirs has more than 6 alternatives.
- `scripts/proof.sh rules` (with its own `CARGO_TARGET_DIR` in the session's scratchpad, not shared with other worktrees): 2,162 Builds, every difference a slot's lock-reason texts (`reasons`, 23,390 lines in 2,160 Builds: feat slots and Mythic rank slots, where Boon Companion, Disruptive and Mythical Beast are closed); no option set, count, status, class, path or switch picker differs. That is the intended change: those three prerequisites' texts.
- `scripts/check.sh`: ALL-OK. 99 server tests pass (5 new), and the `rules` engine tests (2 new).

## Answer

Done 2026-11-09. `search` and `options` take `compact: true`: a search match is then its kind, key, name and category, and an option loses its summary; the run's every-mythic-ability search drops from 24.6 kB to 6.3 kB. In `search`, a prerequisite over 200 characters reads each group of more than 6 alternatives as "one of 49 hexes (Air Barrier, Battle Master, Battle Ward, …)"; `describe` gives it whole. `options` returns the envelope only with `envelope: true`. "Animal Companion or Animal Companion" was the renderer naming two options (Nature Bond's and the Nature revelations') by name alone: options sharing a name in a group now read with their Feature selection, in prerequisite texts and refusal reasons alike.
