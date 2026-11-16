# 297: numbers as components

**What the data says now.** Every number the planner counts comes from a component (`rules::Component`, ticket 295's
vocabulary) on the entry that gives it; no hand-shaped number field is left.

- `bonuses.ron` is gone. Its 179 rows are components on the 123 entries they named (races, subraces, backgrounds,
  feats, mythic feats and abilities, features, options):
  - a save, initiative or skill bonus is a `StatBonus` (`AllSaves` as one per save, `AllSkills` as one per skill,
    `Held` as `stat: Parameter`);
  - its amount a `Value`: `Flat`, a rank on `MythicLevel` (`Div2` for half), on `StatBonus(ability)` (an ability's
    modifier), on a skill's ranks with `Custom([(9, a), (100, b)])` (the ten-ranks bonuses), and the Duelist's Improved
    Reaction on duelist levels (`Custom([(7, 2), (100, 4)])`, its two rows with a class-level `requires`);
  - a spell DC bonus is `SpellDc(scope, value)`, spell penetration `SpellPenetration(value, while)` (Spell Penetration
    (Mythic)'s two rows with `HasNot`/`Has` Greater Spell Penetration), concentration `Concentration(value)`.
- Ticket 258's rules (superseded into 297): Spell Focus and Greater Spell Focus are `SpellFocus(bonus: 1, mythic_focus:
  MythicFeat("spell-focus-mythic"))`; Spell Focus (Mythic) holds none (its summary now says it doubles the two);
  Expanded Arsenal is `ExpandedArsenal(focuses: [Feat("greater-spell-focus"), Feat("spell-focus")])`.
- A feature's `ability_bonuses` are components: `Fixed`/`Every` a `StatBonus(stat: Ability(…), value: PerRank(n),
  descriptor: Racial)` per score (one per grant, as before); `Highest` `HighestAbilityBonus(n)`; `AtLeast`
  `AbilityAtLeast(n)`. Perfect Mind's `bab_at_least_mythic_rank_plus` is `BaseAttackAtLeast(15)` and its
  `skill_ranks_at_least_mythic_rank` `GoldenDragonSkills`. A feature's `steps` is `Scales`, a rank on the same level
  rising one step at each step level (`Custom` bands), so Infernal Resistance's `Scales((base: ClassLevel(classes:
  ["bloodrager"]), progression: Custom([(4, 0), (7, 1), (100, 2)])))`.
- A gear item's `bonuses` are its `components`, one `GearStat(stat, value, descriptor)` each (92 items, the planner's
  robe included).
- The lines' named feats hold kinds: `WeaponFocus` (Weapon Focus, Greater), `WeaponSpecialization` (both),
  `ImprovedCritical`, `CriticalEdge` (the Trickster's two threat feats), `CriticalMultiplier` (the third),
  `Flag(WeaponFinesse)`, `PowerAttack`, `Metamagic(Heighten)`.
- Files: 239 under `libs/game-data/` (feats 40, features 33, gear 92, mythic feats 5, options 15, races 11, subraces
  35, backgrounds 8) and `bonuses.ron` deleted; the seeds fixture's four features and three feats, and its
  `bonuses.ron` (deleted: nothing read it).

**How.** A rewrite outside the repo (`<scratchpad>/game/297/rewrite/`, `cargo run --release -- <game-data dir>`), from
the data at `f5409f88`, not the game: it reads the old fields with its own copies of the old types, writes each entry's
components after those it holds, takes the field out, reads the file back, and deletes `bonuses.ron`. Run again, it
moves nothing. The Spell Focus kinds (258) and the Duelist's and Spell Penetration (Mythic)'s forms are its explicit
cases; Spell Focus (Mythic)'s summary was edited by hand.

**Decisions.**
- The main session (295's review): every source's stat bonuses to one stat stack together by the game's rule
  (`ModifiableValue.ModifierList`, `ApplyModifiersFiltered`, `DefaultStackingDescriptors`). With the data's types as
  they are no corpus number changed: every former `bonuses.ron` row is untyped (as the planner held it), the former
  ability bonuses are `Racial` (both stack with everything), and no two gear items of the data share a type the
  planner's old gear rule and the game's treat apart (no racial, dodge or natural-armour gear; every penalty untyped).
- Mine: the descriptors are today's rule in the game's terms, not the game's descriptors, where they differ. The
  former ability bonuses count toward ability prerequisites (the planner's rule since ticket 166), which in the game is
  `Racial` (`Modifier.IsPermanent`). The game's: `DragonDiscipleConstitution` 95de611796547be489e527fae84abbd6 is
  `Racial` (agrees); `LegendFeature` 7f99961610390044391f300c7ef5e0c8, `TheSwarm` 704d7f26933c3784ea28949e0c3511c0
  and `EyeOfTheSwarmFeature` 73c9302686b957d4dae298061eb9ffe1 are `None` (untyped: the game's Legend +4 meets no
  ability prerequisite, the 252 follow-up). The importer (293) writes the game's, which changes those prerequisites.
- Mine: the Gold Dragon kinds keep the planner's reading. `AbilityAtLeast` reads the score after every stat bonus;
  the game's overrides (`DragonLevel2StrengthOverride` 9ed55389d2079714eab8495957964af2) read the base value.
  `GoldenDragonSkills` raises the ranks a total counts (with the class-skill bonus from 1 rank in total, and a
  ten-ranks bonus reading them); the game's `AddGoldenDragonSkillBonus` is a modifier set at `OnTurnOn`, which leaves
  `BaseValue` alone. Each is a later ticket's change of number.
- Mine: Improved Reaction's +4 from duelist 8 is a rank on duelist levels; the game's `ImprovedReaction`
  d1b7dd3736ca41446aaff9f253768d1d is `AddStatBonus` Initiative +2 granted twice (its `Value × rank`). Same numbers.

**Not modelled.** `WeaponFocusParametrized`'s doubling while Weapon Focus (Mythic) is held (the attack line leaves
mythic feats out); Expanded Arsenal's `m_SchoolMastery` caster level (left out with School Mastery); descriptors from
the game (293).

**Tests.** `libs/seeds/tests/game_data/numbers_as_components.rs` (the line feats' kinds, Perfect Mind's floors, the
Dragon Disciple's racial bonus, the bloodrager features' `Scales`, a gear item's `GearStat`s, `bonuses.ron` gone);
`caster_line.rs`'s 258 tests, no longer ignored (Spell Focus (Mythic) doubling, the Max DC run's DCs from level 7,
Expanded Arsenal's +8) and `the_components_cover_the_dc_runs_sources`.
