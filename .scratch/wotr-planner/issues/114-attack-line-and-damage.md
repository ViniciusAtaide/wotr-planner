# An attack line and a damage estimate

Type: design
Status: resolved
Blocked by: none (option C waits on [113](113-effects-as-data.md) option C and a scope decision)
Map: [WotR Build Planner](../map.md)

## Question

The user asked an agent for "maximum attack damage" (2026-11-08, the Demon Greatsword run, see [112](112-effect-summaries.md)). The sheet gives BAB, saves and ability scores, but nothing about an attack:
- no attack bonus per attack and no attacks per round;
- no damage per hit, no crit range or multiplier.

So the agent couldn't measure "maximum", and `compare` can't show which of two branches hits harder. Any damage claim was the agent's own arithmetic.

What could be computed today without gear:
- iterative attacks from BAB;
- Str to attack, and to damage (×1.5 two-handed);
- Power Attack's trade;
- the flat bonuses of Weapon Focus and Weapon Specialization;
- crit range and multiplier from the weapon and Improved Critical.

`libs/game-data/weapons.ron` holds only `key`, `name` and `proficiency`. Damage dice, crit range, crit multiplier and handedness are missing.

This touches the map's **Out of scope**: gear and buffs. A greatsword's base stats are game data, not gear; its enhancement, the character's items and the party's buffs are gear and buffs.

## Options

- **A. Nothing:** damage stays the agent's own knowledge, labelled as such.
- **B. An attack line in `sheet`:** for one base weapon the agent names, at each level: the attack bonus of each iterative attack and the damage per hit (weapon dice + Str + flat feat bonuses + Power Attack), plus crit range and multiplier. Unconditional numbers only: no rage, no stances, no gear, no buffs. `compare` lists it.
- **C. A damage estimate:** expected damage per round against an AC, with toggles (rage, stances, Power Attack, mythic feats). It needs 113's option C and a decision on gear and buffs.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It is facts the engine can own, it lets `compare` rank two branches by an honest number, and it stays inside the current scope.
2. **Weapon stats in the data?** *Recommend:* yes, for option B: dice, crit range, crit multiplier and handedness in `weapons.ron`, from the saved pages, with a real-data test.
3. **In the web app too?** The attack line would be an engine computation in `rules`, so the web app could render it as well. Then `scripts/proof.sh` should show that nothing else changes. *Recommend:* yes, in the Build summary under the grid, with a weapon picker. The MCP and the web app then show the same numbers.
4. **The honesty boundary:** with B, the instructions add the attack line to what an agent may claim as fact, and say that rage, stances, gear and buffs are not in it.

## Choices settled

The user took every recommendation (2026-11-08): option B, weapon stats in the data, the line in the web app too, and the honesty boundary updated. Settled while building:

1. **The source of the stats.** No saved guide page lists weapon dice, threat ranges or multipliers, so they are the PF1e core tables' values, said so in `libs/game-data/README.md`. The guide wins where it speaks: its Weapon Finesse list is the `finesse` flag, the Elven Curve Blade is two-handed and the Estoc one-handed (Fencing Grace's "finessable, one-handed piercing" weapons). A double weapon has its first head's stats, as one two-handed weapon.
2. **No stats for what has no dice of its own.** The natural attacks (their dice depend on what grants them: a mutagen, a shape, a race), Rays, Touch Attacks, Kinetic Blast and Bomb (a spell's or a class feature's damage) have none and get no line. The unarmed strike has 1d3 (a monk's damage is a class feature).
3. **Ranged weapons and Weapon Finesse,** beyond the brief, because without them the line would be wrong as a fact: a ranged weapon attacks with Dexterity, a thrown one adds Strength to damage and one that shoots none; Weapon Finesse with a finessable weapon attacks with Dexterity when it is higher (the guide: Dex "for attack rolls rather than your Strength modifier", not damage). Power Attack is for melee weapons only.
4. **Power Attack at BAB 0** is −1/+2, as the guide's formula gives (prerequisite Str 13 only, a step more at BAB 4 and every four after).
5. **The line's shape:** each attack's bonus, the dice and a flat bonus, the threat range and multiplier, Power Attack's trade beside the numbers (not applied to them), the parts it counts (Strength, Dexterity, each weapon feat, with what each adds) and a fixed list of what it leaves out (`ATTACK_LINE_LEAVES_OUT`): rage and stances, mythic feats and abilities (Improved Critical (Mythic) among them), class features, gear, buffs, Deadly Aim and Piranha Strike, a second weapon, size, proficiency and situational bonuses. Feats count however they are held (picked or granted: a warpriest's Weapon Focus, Weapon Mastery's Improved Critical).
6. **The default weapon** is the first Weapon Focus taken that has stats, by level, in key order within a level (`Engine::default_attack_weapon`).
7. **The web app** uses a native `<select>` (the phone's own picker, thumb-sized) in the summary, as local UI state on the planner (`Planner::attack_weapon`), not in the Build or link. "Not counted" folds the leaves-out list into a `<details>`.
8. **MCP:** `sheet` and `compare` take `weapon` (a weapon key); `sheet` adds `attack` to each level and `compare` to each summary, with `attack_leaves_out` once beside them. A key with no line is `invalid_arguments`. The shapes live in `apps/mcp/src/attack.rs`, to keep the edits to the shared files small while another agent works there.

## Progress log

- 2026-11-08: Read the map's Notes, this ticket and the glossary. Searched every saved guide page for weapon dice, threat ranges and multipliers: none (the feats page has the Finesse list and Fencing Grace's one-handed weapons only).
- Test-first in `rules` (`tests/engine/attack_line.rs`, 13 tests; `check.rs` 1 more): iteratives, two-handed Strength, the weapon feats, Power Attack, Improved Critical, ranged weapons, Weapon Finesse, no line without stats, the parts and leaves-out list, the text labels, the picker's list and the default weapon. `GameData::check` refuses stats no weapon has and a ranged finessable weapon.
- Schema: `weapon` gains the stats columns (all set or all NULL). Rebuilt a worktree-local database per `CLAUDE.md`, reseeded and re-prepared the sqlx data. Real-data test `libs/seeds/tests/game_data/weapons.rs` (4 tests).
- Web app: the attack block in `grid.rs`'s summary. MCP: `attack.rs`, the `weapon` arguments, the instructions' honesty boundary; `tests/server/attack_line.rs` (5 tests). Glossary: **Attack line**.
- Checked the web app on the worktree's own server at 360×840 and 840×760 with a real fighter 2 (Str 19, Power Attack, Weapon Focus in Greatsword and Longsword): it defaults to the Greatsword and shows +7 · 2d6+6 · 19–20/×2, Power Attack −1/+3; the Long Bow gives +2 · 1d8 · 20/×3 and no Power Attack. "BAB, Strength +4/+6" read ambiguously, so the counts now read "Counts: BAB; Strength +4 attack, +6 damage; …"; the flex summary had lost its disclosure marker, now drawn.
- `scripts/proof.sh both <out> e13b371`: 2,162 Builds on each side, none only in one, none differing (empty `compare.txt`). `SQLX_OFFLINE=true scripts/check.sh --schema`: ALL-OK, 916 workspace tests.

- 2026-11-08, after the merge: the user gave the game's weapon table, which replaces the PF1e values (choice 1). It changed six weapons: the Gnome Hooked Hammer's first head 1d8, the Tongi's 19–20 threat, the Sawtooth Saber light, the Nunchaku and the Light Shield's bash finessable (the table wins over the guide's Finesse list), and the Sling Staff's Strength trait, a new `Ranged::ProjectileWithStrength` (it shoots and adds Strength to damage; the migration's `ranged` CHECK gains `projectile_with_strength`). The values the first draft flagged for confirmation (Estoc, Tongi's ×3, Fauchard, Throwing Axe thrown, Starknife melee) match the table. Test-first: `tests/game_data/weapons.rs` now holds the whole table and a Sling Staff line; both failed on the old data, then passed.

## Answer

Done 2026-11-08 (option B). `libs/game-data/weapons.ron` gives 60 weapons their stats (PF1e core values; the guide's Finesse list and hands where it speaks); `Engine::attack_line` computes one base weapon's line at a character state, the web app shows it at the Build's last level for a weapon picked beside the summary (default: the first Weapon Focus), and `sheet`/`compare` take `weapon`. The instructions now let an agent state the line as fact and name what it leaves out.

The weapon stats are the game's table as the user gave it (2026-11-08), which replaced the first draft's PF1e values; see the progress log's last entry.
