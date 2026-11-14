# A slayer's second Combat Style talent leaves a choice that can't be filled

Type: bugfix
Status: open
Blocked by: none
Model: Opus to find the cause (the engine's option-selection slots); Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by the second MCP QA session (2026-11-14, on `main` at `8a457a0`). Older than the queue of tickets 188 to 193: the slayer's shared Combat Style selection predates it (the session's inference; check with `git log`).

A slayer takes the talents `slayer/slayer-talents/combat-style-feat-2` at 2, `-6` at 6 and `-10` at 10. `todo` then lists `slot.option-selection.2.0`, `.6.0` and `.10.0`, each "Combat Style (Slayer)". The code with all three open:

`AaVOvwsBYRj-3jcZPrrrSkwGG7npDEQGKSU_SlapKxml_BjOdsMVm9ENZt0iFv8AuVHJaLAbleUG350MSrl4et7n7Xl6e3oLilHJwJ3Xhj9T5eV_yBq6GGxZgcF15k-Uyov4oS7Koqlwx1quyWx1WbxMzdBcU_Oe9Tg26kjpCdn3pV7jpq_a-ma7E05WJJmVw0bqzKoIEPQCcAJHKY0SB0jc7e9wLgGQ-ODlEe3wXT5lTwGK9ocuBh4`

- **Level 2 first:** `choose` of a style at `slot.option-selection.2.0` applies, and the next step, the same style at `.6.0`, is refused with `no_such_choice`: "The Build has no choice `slot.option-selection.6.0`". `todo` had printed that address one call earlier. That is consistent (one style serves all three talents), and only surprising.
- **Level 6 first:** `choose` of `ranger/combat-style-feat/archery-feats` at `slot.option-selection.6.0` applies and `.10.0` disappears, but `.2.0` stays in `todo`. `options` on it shows every style refused with `breaks_later` "Would break Archery Feats", Archery Feats included, and `choose` of any style there says the same. The Build can never empty its `todo`, and the reason is untrue: taking Archery would not break Archery. The code after this order:

`AaVOvwsBYRj-3g83HN11JSwGG7npDEQGGSQ_SlapKxml_BjOdsMVZbjRDWbdIhb_ALlRyWiwG5XlBt-dDEq5eHre5-15ent6C5JeycCdVYY_U2bFf0gaujjQMv2D68yXKJUX8UOdF3lDYo61XJPY6rJ4UY3gXJHz7vU4NuoI6Qna94Ve46at2tpmu-NOZiSZFcN66qxOQ6QNAcIUAMMxNE1HkQ2MnO3vsC8BXMgLL4-xFb7Lp-wpQGPrQwfjAYooPAA`

## Build

- Find why the style chosen at a later talent's slot doesn't serve the earlier talent's, when the style chosen at the earliest serves the later ones: where the walk decides an option-selection slot is already answered (`only_without_option` or its like), and in what order.
- One style, held once, however the slots are filled: either the earlier slot takes the style already held (and shows it as held there), or it isn't offered once a later slot holds the style, or the later slots are never offered while an earlier one is open. Pick the one the engine's model of an option's own selection supports; the web app's picker shows the same slots, so check it there too.
- Whatever is chosen, no order of `choose` steps may leave a `todo` entry that nothing can fill; pin both orders.

## Done when

Both orders end with an empty `todo` and the same Build; a rules test and an MCP test pin it; the snapshot check moves nothing but what follows from the fix; `scripts/check.sh` passes.
