# Monk rows and lists

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: ki powers and rows read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 1 `scaling_features.rs`, part 2 item 10; half A `archetype_losses.ron`) differ on `monk.ron`:
- **Ki powers:** the planner grants `monk/ki-power` at levels the game gives a ki-power selection of the archetype's own (the Drunken Master's `DrunkenKiSelection` 97da13a43026460f8d4d54e1c69af202 at 4, 6, …, 20, with `KiPowerDrunkenFeature` 814ddce615b84e488fb7a196acebd408 once at 3; the Scaled Fist's `ScaledFistKiPowerSelection` 4694f6ac27eaed34abb7d09ab67b4541). The Drunken Master's list holds 12 powers, no Diamond Body or Diamond Soul.
- **The Sensei** (`SenseiArchetype` f8767821ec805bf479706392fcc3394c), **the Student of Stone** (`StudentOfStoneArchetype` 3b81e57a75299b74ab6b144e830864e9) and **the Zen Archer** (`ZenArcherArchetype` 2b1a58a7917084f49b097e86271df21c) keep the Ki Power at 3 (`KiPowerFeature` e9590244effb4be4f830b1e3fffced13); the Zen Archer keeps the monk's weapon proficiency and adds its own (bows); its Way of the Bow offers the shortbow focus.
- **The Scaled Fist** (`ScaledFistArchetype` 5868fc82eb11a4244926363983897279): its AC bonus unlock, power feature and stunning-fist variants replace the planner's monk rows (AC bonus, ki pool, stunning fist fatigue and sicken, the ki strikes).
- **Bonus feat lists:** the Scaled Fist's, the Student of Stone's and the Zen Archer's lists hold fewer feats than the planner's (Deflect Arrows, Crane Style, Improved Blind-Fight, Improved Disarm and Trip out where the game lacks them).
- **Steps:** the Sohei's Ki Weapon (`SoheiKiWeaponFeature` 52b62805b9925ca498ae886cd1a41ffd) at 4, 5, 9, 13, 17; Drunken Ki once at 3 with the selection at the even levels; the Sensei's inspire courage and competence ranks; Drunken Resilience, Bones and Soul of Stone ([254](254-a-features-steps-follow-the-class-level.md)'s pass); the Quarterstaff Master's damage bonuses at 12 and 20.

## Findings (the game check, `known.txt` section 279)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / in game only | 38 |
| progression rows / in planner only | 27 |
| progression rows / a further rank in planner only | 22 |
| progression rows / a further rank in game only | 11 |
| option lists / in planner only | 9 |
| option lists / in game only | 1 |
| prerequisites / not in game | 1 |
| **all** | **109** |

The 16 entries: `archetype monk/drunken-master`, `archetype monk/quarterstaff-master`, `archetype monk/scaled-fist`, `archetype monk/sensei`, `archetype monk/sohei`, `archetype monk/student-of-stone`, `archetype monk/zen-archer`, `feat list monk/scaled-fist/bonus-feats-1`, `feat list monk/scaled-fist/bonus-feats-10`, `feat list monk/scaled-fist/bonus-feats-6`, `feat list monk/student-of-stone/bonus-feats-6`, `feat list monk/zen-archer/bonus-feats-1`, `feat list monk/zen-archer/bonus-feats-10`, `feat list monk/zen-archer/bonus-feats-6`, `selection monk/zen-archer/way-of-the-bow`, `selection option monk/ki-power/sudden-speed`.

The report lists each finding under its group, marked `[known: 279]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and lists become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; `archetype_losses.ron`'s sensei, student-of-stone and zen-archer entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `scaling_features.rs`: `the_monk_archetypes_step_the_features_the_guide_gives_steps_to` (Drunken Ki, Ki Weapon).
- `ki_powers.rs`: `every_monk_archetype_picks_a_ki_power_at_its_tables_levels` (the Drunken Master picks from its own list of 12).
- `archetype_losses.rs` (sensei, student-of-stone, zen-archer).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 279 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
