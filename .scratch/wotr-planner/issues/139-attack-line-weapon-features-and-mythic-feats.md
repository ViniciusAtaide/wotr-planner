# The attack line counts weapon class features and mythic weapon feats

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [131](131-gaps-from-the-second-martial-run.md)'s option B, which wasn't built. Most of what decides a martial Build's damage is unconditional, yet `ATTACK_LINE_LEAVES_OUT` (`libs/rules/src/engine/attack.rs`) leaves it out:

- **Class features:** Weapon Training, Two-Handed Weapon Training, Backswing, Greater Power Attack, Weapon Mastery.
- **Mythic feats:** Power Attack, Improved Critical, Weapon Focus and Weapon Specialization (Mythic).
- **Mythic abilities** that raise attack or damage without being switched on.

So `compare` and `improve` rank martial Builds on part of their numbers. The second martial run's two branches still read alike, apart from the Trickster critical chain.

Options:
- **A.** Ticket [119](119-caster-line.md) and [120](120-initiative-and-skill-totals.md)'s per-item bonus shape (`bonuses.ron`, `rules::ItemBonus`), with attack-line targets: attack, damage, threat range and multiplier, Power Attack's trade, each limited to the weapon the feat is held for or to a grip (two-handed). Amounts may scale with class level, as Weapon Training's do. Each source is its own `counts` row.
- **B.** Hard-code each source in `attack.rs`, as the Trickster critical chain is.

*Recommend:* A. The items are data, so the next feature to count is a data edit, and the caster line's sources already have this shape.

Questions for the build: which of the class features' numbers the guide states (Weapon Training's bonus per group and level, Greater Power Attack's trade, Weapon Mastery's critical); and which weapon group Weapon Training counts, since its group is a label (ticket [54](54-mercies-and-enforcer-exploits.md)). One answer is to count the best group for the line's weapon and say so in the row.

## Done when

The Human Two-Handed Fighter 20 with a falchion from ticket 131 shows a `counts` row for each source it holds, and its numbers move by the guide's amounts. Real-data tests cover each source. `ATTACK_LINE_LEAVES_OUT` no longer names what is now counted. The snapshot check passes with the snapshot files untouched, since they hold no attack line (`CLAUDE.md`, "Behaviour checks"). `scripts/check.sh --schema` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A, the sources as `ItemBonus` data, each its own `counts` row. Weapon Training counts the best group for the line's weapon, and its row says so.
