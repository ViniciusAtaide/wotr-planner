# Backgrounds

Type: task
Status: open
Blocked by: 36
Map: [WotR Build Planner](../map.md)

## Question

The user decided (2026-11-01) that a background is a level-1 character-creation choice that makes its skills class skills.

The user then gave the full list (2026-11-01): 38 backgrounds in 8 groups plus None, each with its class skills, some with weapon or armour proficiencies, three with a feat, and other bonuses. It's in [research/backgrounds-from-user.md](../research/backgrounds-from-user.md), and it is the source: the saved guide pages don't list backgrounds.

**Decisions** (user, 2026-11-01, "all recommended"):
1. **Weapon proficiencies:** the Aldori Swordsman grants Exotic Weapon Proficiency (Dueling Sword) as a granted feat for that value. The other weapon proficiencies (no feat in the data checks a single simple or martial weapon) and the bucklers are notes on the background.
2. **Light armour:** Bounty Hunter, Gladiator, Guard and Mercenary grant Armor Proficiency (Light Armor).
3. **Feats:** Martial Disciple grants Improved Unarmed Strike, the Arcane Scholar Scribe Scrolls, the Divine Scholar Brew Potions. Granted feats count for prerequisites like any other.
4. **Everything else** (save, AC, damage and skill bonuses, Acolyte's and Healer's ability swaps, the Nature Scholar's orison, the Nomad's companion bonus) is a note on the background, shown in the summary. Nothing else is modelled.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): a `backgrounds.ron` from the user's list, the Build start's background choice (regenerate the frozen v1 link vector; no v2), class skills from it, and the schema, seed and loader. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a Shackles Corsair holds Athletics and Mobility as class skills; an Aldori Swordsman holds Exotic Weapon Proficiency (Dueling Sword); a Guard can take Armor Proficiency (Medium Armor) at level 1; a Martial Disciple's Improved Unarmed Strike meets Stunning Fist's Improved Unarmed Strike prerequisite; None grants nothing.
