# Cavalier rows

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: steps move to the new model; bonus feats take the game's group.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 1 `scaling_features.rs`; half A `archetype_losses.ron`) differ on `cavalier.ron`:
- **Repeated grants** ([254](254-a-features-steps-follow-the-class-level.md)'s shape): Challenge, Tactician and Banner (`CavalierBanner` 2d957edad0adb3d49991cfcd3ac4cbf8) are granted again at each step; the game grants each once (`CavalierProgression` aa70326bdaa7015438df585cf2ab93b9) and the step is its rank on cavalier level. The archetypes' own (Fearmonger, the Disciple of the Pike's, the Ghost Rider's, the Standard Bearer's banner) likewise.
- **The Standard Bearer** (`StandardBearerArchetype` 07db3fec753209546a792a7942288998) holds the class's own `CavalierBanner` from 1, rising at 5, 10, 15, 20, not a copy; it removes Mighty Charge (`CavalierMightyCharge` ded43678aa1fbe241827175b65e9a749) at 11.
- **The Knight of the Wall** (`KnightOfTheWallArchetype` 112dd0e61f95c3a459ac0a565472e685) removes Demanding Challenge (`CavalierDemandingChallenge` 3fca19d3f3886ee47be4b4019908d223) at 12 for its own.
- **Bonus feats:** the cavalier's bonus feats are `CavalierBonusFeatSelection` (dd17090d14958ef48ba601688b611970, group `MountedCombatFeat`) at 6, 12, 18 for the class and every archetype (the Gendarme's own list too); the planner gives a combat-feat slot. The Cavalier of the Paw's bonus feat is at 6, not 8, and its challenge is `CavalierPawChallenge`.

## Findings (the game check, `known.txt` section 273)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in planner only | 108 |
| progression rows / a bonus pick of another group | 31 |
| progression rows / in planner only | 3 |
| progression rows / at another level | 1 |
| progression rows / in game only | 1 |
| **all** | **144** |

The 9 entries: `archetype cavalier/beast-rider`, `archetype cavalier/cavalier-of-the-paw`, `archetype cavalier/disciple-of-the-pike`, `archetype cavalier/fearsome-leader`, `archetype cavalier/gendarme`, `archetype cavalier/ghost-rider`, `archetype cavalier/knight-of-the-wall`, `archetype cavalier/standard-bearer`, `class cavalier`.

The report lists each finding under its group, marked `[known: 273]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; the bonus feats offer the mounted combat list (a slot category or a feat list, as the planner's forms allow); `archetype_losses.ron`'s cavalier entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `scaling_features.rs`: `the_cavaliers_banner_tactician_and_challenge_step` (the Standard Bearer holds the class's banner), `the_standard_bearers_own_banner_steps_at_10_15_and_20` (the class's banner from 1, rising 5, 10, 15, 20), `the_cavalier_archetypes_step_the_features_the_guide_gives_steps_to` (Banner of Solace: [254](254-a-features-steps-follow-the-class-level.md)'s code pass).
- `archetype_losses.rs` (`archetype_losses.ron`'s knight-of-the-wall and standard-bearer entries).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 273 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
