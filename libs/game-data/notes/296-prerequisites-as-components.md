# 296: prerequisites as components

**What the data says now.** 608 blueprint entries (197 feats, 309 options, 44 mythic feats, 24 mythic abilities,
18 archetypes, 5 base classes, 11 prestige classes) hold their prerequisite as `components: [Requires(…), …]`
(ticket 295's vocabulary, `rules::Component`) in place of the `prerequisite` field, with the same values: the
conditions compile back into the very tree the field held (both canonical). The checked data holds that tree in the
field again and no `Requires` (`GameData::checked`), so the engine, the MCP server, the game check and the browser's
JSON read what they read before; the JSON the server sends is the same document.

**The form.** One `Requires` per condition, in the place the field had in the file (a one-line file keeps one line, a
multi-line file one component a line). From the tree:

- a top-level `All`'s conditions each in the `All` group, but when exactly one of them is an `Any` (not of
  alignments only), that `Any`'s conditions in the `Any` group, as `BlueprintFeature.MeetsPrerequisites` reads the
  game's groups (124 entries); a top-level `Any` in the `Any` group; any other tree one condition;
- `Feat`, `Feature` (count 1), `SelectionOption`, `Selection`, `MythicAbility`, `MythicFeat`, `Archetype`, `Race`,
  `Deity` are `Has(<fact>)`, and `Not` of one `HasNot`; `Not(Archetype)` is `NoArchetype`, `Not(ClassLevel(c, 1))`
  `NoClassLevel`; `Feature` of a count above 1 `FactRank`; `FeatFor` `ParametrizedFeature`; `ClassLevel`
  `ClassLevel`; `AbilityScore`, `FullAbilityScore`, `SkillRanks`, `BaseAttackBonus` a `Stat`; `SpellLevel` and
  `SpontaneousSpellLevel` `CasterTypeSpellLevel`; `CharacterLevel`, `MythicRank` (`MythicLevel`); an `Any` of
  alignments only (or one alignment) one `Alignment` set, as the game's `PrerequisiteAlignment` mask; an
  `AtLeast(n, …)` over facts `AtLeast`; a nested `All`, `Any` or `Not` the condition's own.

The rewrite is a script kept outside the repo (ticket 296's Answer names it), run on the data as it stood at
`e1022ba2`, not on the game. Re-run, it finds nothing left to move.

**What keeps the field.** The planner's entries (`planner/`), and the 62 blueprint entries whose tree holds a node
with no condition yet (each is a whole tree: the check refuses a field beside a `Requires`):

| Node | Entries | Why |
| --- | --- | --- |
| `Story` | the 4 Gold Dragon breaths | The planner's own: story is out of scope (`GoldDragonBreathAnnihilating` e8b9aad5295c49539f1bbf901d314964 needs `CorruptedGoldenDragonFeature` 1e66cfb4f351466fb05636082bce50d6, a story fact). Stays a field. |
| `FeatForSameValue` | 8 feats, 3 mythic feats (Weapon Specialization, Greater Weapon Focus, …) | The parameterised feature's own prerequisite (`BlueprintParametrizedFeature.m_Prerequisite`): the header's `values_of`, ticket 298. |
| `FeatIfValue` | Armor Focus | The game's is a selection of one feature per armour type (`ArmorFocusSelection` 76d4885a395976547a13c5d6bf95b482), ticket 298. |
| `Worships`, `Atheist` | cleric, inquisitor, warpriest, Divine Hunter, Deliverer; Living Deity | The faith as facts: `ClericClass` 67819271767a9dd4fbfd4ae700befea0 is `PrerequisiteNoFeature` `AtheismFeature` 92c0d2da0a836ce418a267093c09ca54; the planner holds no atheism fact yet. Ticket 293. |
| `DeityOffers`, `PickedThrough` | Angelfire Apostle, Priest of Balance (`DeityOffers`); the 4 alignment domains (`PickedThrough`); the 4 alignment blessings (both) | The deity's `…DomainAllowed` facts (`LawBlessingFeature` 9c49504e2e4c66d4aa341348356b47a8 is `PrerequisiteFeature` `LawDomainAllowed` 092714336606cfc45a37d2ab39fabfa8): the deity's grant and the options' `Has`, tickets 293 and 298. |
| `NoOtherOptionOfGroup` | the 28 draconic and elemental bloodlines (sorcerer, bloodrager) | The game's is a `PrerequisiteCondition` tree over the other bloodlines (`BloodlineDraconicRedProgression` 8c6e5b3cf12f71e43949f52c41ae70a8): the importer writes it, ticket 293. |
| `NoOptionFrom` | Dragon Disciple | A `PrerequisiteCondition` tree over the bloodlines (`DragonDiscipleClass` 72051275b1dbb2d42ba9118237794f7c), ticket 293. |
| `AtLeast` over a rank | Arcane Trickster | The game's is `PrerequisiteFullStatValue` `SneakAttack` 2 (`ArcaneTricksterClass` 9c935a076d4fe4d4999fd48d853e3cf3), a stat the planner doesn't count: ticket 297's numbers, then 293. |

Moving any of these now would change a value (another tree, another reason's text), which this ticket doesn't.

**Pinned by** `libs/seeds/tests/game_data/prerequisites_as_components.rs`: every field left has one of those nodes
(with the count of entries holding each), 608 entries moved, no planner entry holds a `Requires`; Power Attack (`PowerAttackFeature`
9972f33f977fc724c838e59641b2fca5, `PrerequisiteStatValue` Strength 13) and Crane Style (`CraneStyleFeat`
0c17102f650d9044290922b0fad9132f, Dodge and Improved Unarmed Strike in `All`, monk 1 or BAB 2 in `Any`) hold the
game's groups. Crane Style's `PrerequisiteStatValue` Intelligence 3 isn't in the data: a score of 3 always holds.
