# Lore (Religion) II and III pick their domains

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [125](125-favorite-metamagic-picks-its-metamagic.md)'s audit found two Trickster tricks that the guide backs but the planner doesn't build. Lore (Religion) II says "You can select two domains", and Lore (Religion) III adds "two additional domains". The planner offers no pick for either. A domain's abilities come from option levels, and no class level or rule gives a domain an option level through a Trickster trick.

The decision is what level the domains count at.

Options:
- **A.** Each trick gives domain slots (two, then two more) from the cleric's domain list, and the domains' rows take their option level from a rule the guide states: Mythic rank, character level, or a fixed level.
- **B.** Each trick gives domain slots, and the domains are held as picks with no option-level rows. Their names show, but none of their level-gated abilities.
- **C.** Leave them unbuilt, and record them as checked in `pick_values`' audit list.

*Recommend:* read the saved `trickster` page for the level the domains work at. Build A if the page states it. If it doesn't, build B and ask the user what the game does.

Questions for the build: does a domain taken here count for a prerequisite that names the domain? Can it repeat a domain the Build already holds from a class? Does a deity limit it, as it limits clerics (ticket [86](86-deities.md))?

## Done when

The user has chosen A, B or C. If A or B: a Trickster with Lore (Religion) II picks two domains, and with III two more. The picks round-trip through the link with an additive tag. Real-data tests cover them. The audit list drops both items. `scripts/check.sh` passes, with `--schema` if the schema changed.

## Answer

**Decided** (user, 2026-11-10): read the saved `trickster` page first. Build A if it states the level the domains work at. If it doesn't, build B: the domains are held as picks, with no option-level rows, so none of their level-gated abilities show. The questions for the build take the recommended reading unless the page says otherwise.
