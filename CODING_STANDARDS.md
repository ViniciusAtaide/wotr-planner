# Coding standards

Read at review, by the reviewer (`.claude/agents/ticket-reviewer.md`). These are the judgement calls no check makes yet. What a check can hold belongs in a test or the pre-commit hook, not here.

## Texts a person or an agent reads

- **A reason is true for this Build.** It names the pick that is really held (by its value too, not only its key), its level and its rank. It carries no hint that doesn't apply to the address or the value at hand.
- **One wording per refusal.** A condition that `options`, `choose`, `validate` and `withdraw` can each refuse is worded by one function they all call (`apps/mcp/src/unreached.rs` is the model). A second wording of the same condition is a finding.
- **The rules crate names no tool.** A text built in `libs/rules` is shown in the web app too: no MCP tool name, no address. The MCP server appends those.
- **A pronoun has one possible referent.** Name the choice ("withdraw Cleave first") where "it" could be the address.
- **Counts agree in number.** "1 level", "13 levels": through `apps/mcp/src/words.rs`, never "level(s)".

## Behaviour

- **A status is behaviour.** `Issue`, `Unmet` and `issue_text` are hashed by the proof (`tools/proof/src/fingerprint.rs`). A wording change lives in a picker entry's reason or in the MCP server's text, unless the ticket means to change the status.
- **Walk per call, not per option.** Work that doesn't depend on the option (a rank's allowed levels, the Build's failing choices) is done once per call. A walk per listed option needs a stated reason and a measured cost.

## Game data

- **An archetype row replaces the base row in full.** A new or changed row carries everything the base row gives at that level (features, feats, feat slots, selections), less what the archetype loses there.
- **A list that names a selection's options moves with the selection.** A feat's `Any` of a selection's options, a sharer's `shared_options`: an option added to the selection is added there or left out for a stated reason.
- **The guide's two places.** An archetype's overview and the note under its table can say different things about one feature. Read both before deciding what a count stands for.
- **An inference is recorded as one,** in the ticket's Answer and the note file, with the sentence it rests on.
- **Summaries are in our own words;** the guide's text is never copied at length.

## Tests

- **A test asserts what its comment says.** A comment that a row "keeps" something is backed by an assertion on it.
- **Every archetype is pinned,** the ones that lose a feature included (`Some(&[])`), so a leak fails.
