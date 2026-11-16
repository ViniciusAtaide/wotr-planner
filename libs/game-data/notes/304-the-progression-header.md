# 304: the progression header

**What the data says now.** An option whose blueprint is a `BlueprintProgression` holds its `header`, the fields
`BlueprintProgression.CalcLevel` and `LevelUpHelper.UpdateProgression` read (`Blueprints/Classes/
BlueprintProgression.cs`, `UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`, WrathDecompiled 6ce4a6b):

- `classes: {"<class>": <offset>}`: `m_Classes`, each `ClassWithLevel.AdditionalLevel`; a class that has one of the
  header's archetypes counts only taken with it.
- `archetypes: {"<archetype>": <offset>}`: `m_Archetypes`, each `ArchetypeWithLevel.AdditionalLevel`. A sum below 0
  adds 0; with no classes and no archetypes the option level is the character level.
- `half_other_classes: true`: `ForAllOtherClasses` with `AlternateProgressionType` `Div2` (the oracle's curses).
- `previous_levels: true`: `GiveFeaturesForPreviousLevels`. Without it an option gets only the row of each level it
  reaches, never those it passes over (the wizard's schools, the kineticist's elements, the cavalier's orders, the
  Hag bloodline, the Magic Deceiver's ways).
- Not written: `m_FeaturesRankIncrease` (the domains' Trickster and Pentamic Faith ranks, 294's; Lore (Religion) II's
  and III's, which `CalcLevel` never reaches with no classes), and no option has
  `m_AlternateProgressionClasses`, `m_ExclusiveProgression` or an undivided `ForAllOtherClasses`.

143 options have one: the 125 with rows (bloodlines, domains, curses, schools, spirits, orders, elements, ways, Weapon
Mastery, Battle Master, the Primalist's rage powers) and 18 without (the witch's patrons, Order of the Nail, Form of
Flame, Wolverine Aspect, Lore (Religion) II and III), whose level nothing reads. The Lich's three skeletal upgrades
have none: they count `LichMythicClass`, which the planner has no class for (known to the game check under 293).

**Who gives the level.** The 23 selections whose options all have a header lost `option_levels`,
`option_level_archetypes` and `half_other_classes` (the sorcerer's, Eldritch Scion's, Dragon Disciple's and the three
bloodragers' bloodlines and both second bloodlines, the spirits and second spirit, the three curse selections and
Beneficial Curse, the elements, the orders, the four school selections, the Magic Deceiver's ways). The 50 others keep
`option_levels` until 293 and 294, which then gives the level of every option they offer, a header or not (decisions
Q5): the domains, shared by selections the game keeps apart, and the selections whose options have no header. The
three revelation selections set `option_headers_first: true`, so Weapon Mastery takes its own header. `half_other_classes`
is gone from selections; `option_level_archetypes` stays only on the Divine Hunter's domain.

**Five levels moved toward the game** (decisions Q1; 0 corpus Builds moved): a sorcerer's non-Draconic bloodline counts
an Eldritch Scion's magus levels and no Dragon Disciple's (`BloodlineAbyssalProgression`
d3a4cb7be97a6694290f0dcfbd147113); a Draconic one counts sorcerer, Dragon Disciple and Eldritch Scion levels through
any of its selections (`BloodlineDraconicRedProgression` 8c6e5b3cf12f71e43949f52c41ae70a8); a bloodrager's Draconic one
counts Dragon Disciple levels (`BloodragerDragonRedBloodline` 34209f220733fc444a039df1b1076b0b); the Frost spirit
counts Winter Witch levels (`ShamanFrostSpiritProgression` 44c130a12d49ebf4a95945feffe5aae7); Weapon Mastery counts
oracle and Magic Deceiver levels (`OracleRevelationWeaponMastery` 0a4c3556355747241b2d3dcc0a88ec10).

**Lists nested as the game nests them.** `option_lists`, `at_level`, the rows' `feat_list` (62 rows on 52 options) and
the state's option feat lists are gone. A selection of feats whose every option is a list (`is_selection: true`, one
level-1 row granting the list) nests them: its slot offers the feats of each list whose option's prerequisite holds.

- `sorcerer/bloodline-feats` is `SorcererFeatSelection` 3a60f0c0442acfb419b0c03b584e1394 with its nine
  `Bloodline*FeatSelection` options (`sorcerer/bloodline-feats/<bloodline>`), each opened by holding the bloodline
  (both Draconic and all four Elemental bloodlines open one list).
- The ranger's style feat is a selection per stage, `ranger/combat-style-feats-6` (`RangerStyleSelection6`
  61f82ba786fe05643beb3cd3910233a8) and `-10` (`RangerStyleSelection10` 78177315fc63b474ea3cbb8df38fafcd), each
  nesting the five styles' lists of its stage, opened by the style held; `-2` is the planner's
  (`planner/selections/ranger/combat-style-feats-2.ron`: the game's `RangerStyleSelection2` is the style pick and its
  first feat in one). Rows at 10, 14 and 18 grant `-10`. The slayer's three stage selections share the ranger's
  stage options. Every SlotId and pick is the same: a link stores only the feat (decisions Q3).
- The bloodrager's bloodline feat and the kineticist's wild talent feat stay planner nests (`planner/`), each with its
  `why`: the game grants each list from the option's progression, with no selection over them. A planner nest
  (these and `ranger/combat-style-feats-2`'s) writes its gate as a `prerequisite`, as planner entries hold no `Requires`.

**Not modelled; known to the game check under 293** (decisions Q4): the game opens a bloodline's list with the
bloodline's requisite feature (`AbyssalBloodlineRequisiteFeature` b09b58c7f8efff244a33269489abeac6 and the others),
which a bloodrager's bloodline, the Sage's, Empyreal's and Sylvan's and a Tiefling heritage grant too; it opens the
10th-level style lists with the 6th-level one; `RangerStyleSelection10` offers `SkillFocusSelection` beside them.

**Tests.** `libs/seeds/tests/game_data/progression_headers.rs` (the five levels, the previous-levels flag),
`feat_slots_as_selections.rs` (`the_bloodline_feat_slot_is_sorcerer_feat_selection`: the nine nested lists); the
engine's rules in `libs/rules/tests/engine/ranks_and_pools.rs`. The game check compares every option's header with its
blueprint (area "progression headers").
