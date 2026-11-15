# The mythic paths' ranks and options

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md) (the Aeon's repeated grants)
Model: Opus to build: each path's rows are read from its progression.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's mythic ranks, and the prerequisites and options of the paths' own entries, differ from the blueprints; 242's research (half B part 3 `mythic.rs`, `life_bonding_friendship.rs`) adds:
- **The Aeon:** rank 2 holds Rectify State (`AeonSecondAbilityFeature` 56201ba48e08def4a9180075485c5bac, in `AeonFirstLevelsProgression` 46df0532714a9454eb5fbad64ce6c14f), not Force Reality again. Its gaze, immunities and demythication upgrades are the game's features at their ranks (`AeonGazeFeature` at 3, `ImmunityTo…` at 5 and 7, `AeonTenthLevelImmunities` and the second extra gaze at 10, `AeonDemythicationUpgrade1`/`2` at 7 and 9), where the planner repeats its own entries. The enforcing gazes need a rank the game doesn't ask (`mythic rank 5`, 8).
- **The Azata:** Life-Bonding Friendship's follow-up teamwork pick (`LifeBondingFriendshipSelection1` 69a33d6ced23446e819667149d088898) is a rank-4-to-10 row the planner doesn't hold the game's way; both its selections (`LifeBondingFriendshipSelection` cfad18f581584ac4ba066df067956477 and the `1`) ignore prerequisites and offer 12 teamwork feats, not Spell Chain; the superpower needs not holding Life-Bonding Friendship already.
- **The Gold Dragon:** rank 10 holds two breath selections (`GoldDragonBreathFeatureSelection` 81fe63677a4f4f7fb64e9be7cf075948), four in all, against the planner's five; the breaths need Golden Dragon 2 and the corruption feature (or its absence), not Mythic rank 9; the dragon feats (`DragonLevel2FeatSelection` a21acdafc0169f5488a9bd3256e2e65b ×3, ignoring prerequisites) offer the basic feats less eight (the three Advanced Weapon Training ranks, Completely Normal Spell, the three Improved² Critical feats, Stat Focus).
- **The Lich:** the deity feature at 5; the skeletal upgrade selection (`LichSkeletalUpgradeSelection` a434dddab8026e947bc16eb36d18a783) offers the game's tiers, each excluding the others.
- **The Angel:** the incorporated spellbook at 3; the greater sword's improvements (`AngelSwordGreaterImprovementsSelection` e0ce40968bf0007408b11089a10f36cf) offer nine the planner lacks.
- **The Demon** rank 8's forced rage; **the Swarm-That-Walks'** clones at 8, 9 and 10 (three features where the planner grants one); **the Devil's** Hell's Decree and Hell's Seal are features, not choices (`HellsDecreeFeature` 442e199eca0ab2f4dbdcf7147fa2c399, `HellsSealFeature` b6798b29d36982b4786a32dfd81a914f).

## Findings (the game check, `known.txt` section 266)

| The check's group (area / kind) | Findings |
| --- | --- |
| mythic ranks / in game only | 23 |
| option lists / in game only | 15 |
| prerequisites / missing in planner, a blueprint it doesn't map | 14 |
| prerequisites / not in game | 13 |
| mythic ranks / a further rank in planner only | 7 |
| prerequisites / missing in planner | 6 |
| mythic ranks / in planner only | 3 |
| option lists / a choice the game doesn't offer | 2 |
| **all** | **83** |

The 27 entries: `mythic path aeon`, `mythic path angel`, `mythic path azata`, `mythic path demon`, `mythic path gold-dragon`, `mythic path lich`, `mythic path swarm-that-walks`, `selection angel/greater-sword-of-heaven`, `selection devil/hells-decree`, `selection devil/hells-seal`, `selection lich/skeleton-upgrade`, `selection option aeon/enforcing-gaze/enforcing-gaze-area-effects`, `selection option aeon/enforcing-gaze/enforcing-gaze-concentration`, `selection option aeon/enforcing-gaze/enforcing-gaze-hit-points`, `selection option aeon/enforcing-gaze/enforcing-gaze-normalize-size`, `selection option aeon/enforcing-gaze/enforcing-gaze-power-of-law`, `selection option aeon/enforcing-gaze/enforcing-gaze-primordial-magic`, `selection option aeon/enforcing-gaze/enforcing-gaze-relativity-of-space`, `selection option aeon/enforcing-gaze/enforcing-gaze-summoners-gaze`, `selection option aeon/enforcing-gaze/enforcing-gaze-temporal-jump`, `selection option azata/superpowers/life-bonding-friendship`, `selection option gold-dragon/improved-gold-dragon-breath/annihilating-breath`, `selection option gold-dragon/improved-gold-dragon-breath/decaying-breath`, `selection option gold-dragon/improved-gold-dragon-breath/enlightening-breath`, `selection option gold-dragon/improved-gold-dragon-breath/rejuvenating-breath`, `selection option lich/skeleton-upgrade/skeletal-barbarian-i`, `selection option lich/skeleton-upgrade/skeletal-fighter-i`.

The report lists each finding under its group, marked `[known: 266]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each path's rows, options and prerequisites become the game's; repeated grants follow [254](254-a-features-steps-follow-the-class-level.md)'s model.

## Tests 242 marks as waiting on this ticket

- `mythic.rs`: `the_mr_3_path_gives_its_mr_1_and_2_rows_without_a_first_ascension_twelve_rows_and_thirty_features` (the Aeon's MR 2: Force Reality once and Rectify State once), `a_gold_dragon_takes_three_dragon_feats_ignoring_prerequisites_and_rises_in_every_score` (Stat Focus isn't offered).
- `life_bonding_friendship.rs`: `the_slots_offer_the_teamwork_feats_and_no_other_feat` (12, no Spell Chain), and the sorcerer's Shield Wall case (open: the slots ignore prerequisites).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 266 (exit 0, run by the main session).
- The tests above hold the game's answers, cited; a data note lists the changes; the scoped checks pass.
