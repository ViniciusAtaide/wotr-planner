# The weapon picker shows only the weapons the character can use

Type: feature
Status: resolved
Blocked by: none
Model: Opus.
Map: [WotR Build Planner](../map.md)

## What to build

User, 2026-11-14: "filter the weapon selection by the only weapons you can wield." The attack line's weapon picker (the summary's `AttackBlock`) lists every weapon with stats; a Hunter is offered the Bastard Sword and the Dwarven Urgrosh, which it isn't proficient with.

Today the engine knows proficiency only through feats: Simple Weapon Proficiency, Martial Weapons Proficiency and Exotic Weapon Proficiency for one weapon. Many lists are text only, and a filter on the feats alone would leave a wizard nothing and take the rapier from a rogue:

- the class and archetype proficiency features (`wizard/wizard-proficiencies`: "Club, dagger, heavy and light crossbow and quarterstaff"; druid, monk, bard, cleric, magus, sorcerer, rogue's extras, …), about 25;
- the races' and subraces' weapons (the elf's longbows, shortbows, longsword and rapier; the half-orc's greataxe and falchion; …);
- the backgrounds' "Proficient with …" notes (22);
- a deity's simple and martial favoured weapons under Cleric Proficiencies (only exotic ones are held today, as Exotic Weapon Proficiency).

So:

1. **Data:** a `weapons` list (weapon keys) on `Feature`, `Race`, `Subrace` and `Background`: the weapons each makes the character proficient with, read from the summary or note beside it. What the weapon list doesn't hold (sap, whip, shields) stays text. The data check refuses an unknown key. A note under the README's Layout.
2. **Engine:** `Engine::proficient_weapons(start, state)`, the set: a weapon on no proficiency list (unarmed strike, rays, natural attacks) always; a simple or martial one with that list's feat; an exotic one with Exotic Weapon Proficiency for it; any one a held feature, the race, the subrace or the background lists; the deity's favoured weapons while a held feature grants them.
3. **Web app:** the picker lists the weapons the character is proficient with at the Build's last level, plus the one chosen if it isn't (so the select never shows a weapon it doesn't hold). No proficiency penalty in the attack line: it still leaves proficiency out.

Not in scope: the MCP's attack line and `options`, and prerequisites (Weapon Focus doesn't check proficiency today).

## Done when

- A wizard 1 is offered the club, dagger, both crossbows and the quarterstaff (and the unarmed strike); a rogue 1 the simple weapons, rapier, short sword and short bow; a Hunter 1 the simple and martial weapons and no exotic one; an elf wizard also the elven weapons. Real-data tests in `libs/seeds/tests/game_data/` pin these.
- The snapshot check moves only for the two data fixes below (the lists themselves are read by nothing it records).
- `scripts/check.sh` passes; checked in the app.

## Outcome

Built 2026-11-14 in the main session. The real-data sweep (`proficiency::every_class_and_archetype_wields_some_weapon_at_level_1`) found two gaps besides the text-only lists: the base wizard row lacked Wizard Proficiencies (only its archetype rows had it), and no sorcerer was proficient with anything. The base wizard row now holds the feature, and every sorcerer level 1 row holds Simple Weapon Proficiency (the guide names only "Sorcerer Proficiencies"; the simple weapons are the game's). The snapshot check's changes: `level.1.feat/simple-weapon-proficiency` in class-sorcerer and gold-jack, `level.1.feature/wizard/wizard-proficiencies` in class-wizard, and Simple Weapon Proficiency gone from class-sorcerer's ten general feat slots (it's held). Inquisitor Proficiencies gained a summary; the empty class-feature summaries went from 113 to 104.
