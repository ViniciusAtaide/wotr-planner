# Uncanny Dodge as the Enforcer's level-8 slayer talent

Type: design
Status: open
Blocked by: 160
Model: Sonnet to build (one entry in `gold.rs`).
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists "Slayer Talents - Uncanny Dodge" at the Enforcer's level 8 as a known gap. The guide's slayer talents (the rogue talents plus Trapfinding and Slayer Camouflage) don't include Uncanny Dodge, and the guide is the planner's source of truth. Ticket [88](88-qa-full-builds.md) called it the write-up's mistake; the user's rule for the gold builds says the write-ups are right.

The user, 2026-11-11: "let's deal with it in a subsequent ticket".

What to find out before the user decides:

- What the saved pages say: `guide-cache/slayer.html` and `rogue.html` on slayer talents and on Uncanny Dodge (a rogue class feature at level 4; is it offered as a talent anywhere?), and the Arcane Enforcer archetype's own text.
- Whether the write-up could mean something the data already has under another name.
- What the Enforcer's level 8 slot offers in the planner today, and what the build takes instead if the slot stays empty.

Options:
- **A.** Add Uncanny Dodge to the slayer talents, against the guide, on the user's word that the game offers it.
- **B.** Keep the guide's list. The line stays a known gap for good, marked as the one place the gold standard and the guide disagree.

## Answer

**Decided** (user, 2026-11-14, in a grilling session): B. "Uncanny dodge is only available through the imitator slayer archetype." It is not a slayer talent, so the guide's list stands and the write-up is mistaken here.

Evidence gathered for the decision:
- The guide's slayer talents (`slayer#slayer-talents`): the rogue talents, a ranger's combat style feats, Trapfinding and Slayer Camouflage.
- Uncanny Dodge is a rogue class feature. Only the Eldritch Scoundrel takes (Improved) Uncanny Dodge as a talent (`rogue.txt`, built by ticket 190).
- The Arcane Enforcer's level 8 is a plain Slayer Talents slot.
- In the data, slayers gain it only through an archetype: the Imitator's Barbarian and Rogue stances, and the Vanguard at 7th (`slayer#uncanny-dodge`, which the guide gives the Vanguard).

The build: the gold test's entry (`gold.rs`, `Why::AgainstTheGuide(173, …)`) stays, and becomes permanent. Its reason names the write-up's mistake and the user's word. `gold/source.txt` keeps the write-up as it was written.

## Done when

The user has chosen, the Answer records the evidence and the choice, and either the data and the gold test's known gaps are changed (A) or the entry is marked permanent (B).
