# Martial builds: damage per round

The attack line gives one hit's damage and the regular attacks. A martial goal ("maximum damage") is usually damage per round, which the planner doesn't compute. Work it out with this worksheet and [dpr.py](dpr.py), the same way for every branch, and report it as your estimate.

## The worksheet

1. **The line.** `compare` the branches: each Build's attack line is for its own Weapon Focus weapon (give `weapon` to put every branch on one weapon). Take each attack's bonus, the damage per hit, Power Attack's trade and the critical range: these are the planner's facts, naked, with no gear.
2. **What it leaves out.** Go through `attack_leaves_out` item by item. For each class feature, mythic feat, mythic ability and path feature the Build holds that adds to attack or damage, read its summary with `describe` and add its number at level 20, labelled as your own game knowledge. These are the usual large ones:
   - **Damage per hit:** Two-Handed Weapon Training, Backswing and Overhand Chop (double Strength bonus), Greater Power Attack (Power Attack ×2), Ki Weapon, a Sohei's Weapon Training, rage, Demonic Rage, the Demonic aspects (Incubus), Unrelenting Assault, the weapon mythic feats.
   - **Critical hits:** Weapon Mastery (+1 multiplier and automatic confirmation), the Trickster's critical chain, Critical Focus (+4 to confirm).
   - **Attack bonus:** Weapon Training, Ki Weapon, rage, Aspect of Schir, Athletics III.
   - **Extra attacks**, made at the highest bonus: Haste, Flurry of Blows, Ki Power – Extra Attack, Aspect of Pazuzu.
3. **The split.** Separate damage a crit multiplies (weapon dice, Strength, flat bonuses) from damage it doesn't (sneak attack dice, energy dice): `damage` and `extra` in the plan.
4. **The round.** Write the plan's JSON, one entry per branch, and run `python3 -I .claude/skills/wotr-build/dpr.py plan.json` from the repo root. Price the round at AC 30, 45 and 60 unless the user names a foe.
5. **The report.** Give each branch's damage per round at each AC, with the inputs from steps 2 and 3 beside it, as your estimate. Say which inputs are least certain, and how the order of the branches changes across the ACs.

## Gear

Compare the branches naked, as the planner's numbers are. Add gear only if the user asks for it, the same gear on every branch, as your own judgement, labelled as such beside the naked numbers.

## Traps that cost a rebuild

Check these with `options` (`only_open: false`) at the slot before the plan depends on them:

- **Proficiency.** `options {values: true}` on Exotic Weapon Proficiency lists every exotic weapon (the fauchard is one). A class without proficiency with the weapon needs that feat, or takes the non-proficiency penalty.
- **Fighter-only feats.** Weapon Specialization, its Greater form, Critical Mastery and Advanced Weapon Training need Fighter levels, so a monk or Sohei can't take them, nor Weapon Specialization (Mythic), which needs the ordinary feat. Critical Mastery also needs two critical feats held.
- **Trickster's critical chain.** Improved Improved Critical and the two after it need Perception II (an Improved Mythic Trick, from MR 4), so the first goes at a level after MR 4's, each in the same weapon as Improved Critical.
- **Skill-gated tricks.** Athletics III makes BAB no lower than the Athletics ranks: spend a rank in Athletics every level.
- **Archetype feat lists.** An archetype's bonus feat slots take only their own list; Power Attack and Weapon Focus go in race or general slots.

## In the game, not the Build

Weapon Mastery's weapon, a Sohei's Weapon Training groups, which minor Demonic aspects are active, and Ki spent each round (Ki Weapon and the extra attack cost 1 each): list the ones the plan relies on, with the Ki budget per rest when the round spends Ki.
