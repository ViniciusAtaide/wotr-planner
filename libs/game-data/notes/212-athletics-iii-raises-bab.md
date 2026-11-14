# Ticket 212: Athletics III raises the base attack bonus

## What the data says now

`libs/game-data/mythic/paths/trickster.ron`, the option `trickster/greater-mythic-trick/athletics-iii`: `bab_from_ranks: Some((skill: "athletics", extra_at_full_bab: 5))`. No other option sets it (`libs/seeds/tests/game_data/athletics_iii.rs`). The rule is in Rust (`Walker::refresh_bab`, `libs/rules/src/engine/walk/hold.rs`): while the option is held, the BAB is at least the ranks spent in the skill (`CharacterState::skill_ranks`, not the skill's total), or those plus `extra_at_full_bab` while the class tables' BAB equals the character level.

## What it rests on

`trickster#athletics-iii`: "Your BAB is at least your Athletics rank. If your BAB is equal to your character level, it is at least your Athletics rank + 5." A fifth and later attack: `other-mythic-paths#playing-a-legend` ("an additional main hand attack each additional +5 BAB, up to a maximum 8 attacks at 36+ BAB"); `iteratives` in `libs/rules/src/engine/attack.rs` now stops at eight, not four.

## Decisions

- The user, 2026-11-14 (ticket 212's Answer): the raise counts everywhere a BAB does (sheet, attack line, Power Attack steps, feat and prestige prerequisites); "Athletics rank" is the ranks spent; ranks first.
- The main session, 2026-11-14: the guide gives no cap on iterative attacks but the Legend note, so the engine's cap of four is lifted to eight. The raise counts for later picks inside MR 7's own rank (it applies the moment the option is held). The class and general slots the walk fills before the rank at that level don't see it (ticket 32's order). A prestige entry at the level after MR 7 is checked against the character before that level, as every number is.
- Inference (not stated by the guide): "your BAB is equal to your character level" compares the class tables' BAB (Perfect Mind's prerequisite floor ignored), so the +5 doesn't feed on itself.
- The effect is a field on the option row plus the rule in Rust, like `Feature.bab_floor`; a held option's raise is recomputed on hold, on release (a Legend switch drops MR 3 to 8's picks) and at each level after its ranks.

## Not modelled

Nothing else of the trick; the BAB raise is not a `bab_floor` (that one only prerequisites see).

## Tests

`libs/seeds/tests/game_data/athletics_iii.rs`; `libs/rules/tests/engine/path_rules.rs` (Athletics III block), `attack_line.rs`, `check.rs`.
