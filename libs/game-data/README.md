# Game data

The planner's game data, as RON files that deserialise straight into the `rules` crate types
(`libs/rules/src/game_data.rs`). These files are the source of truth (ticket 75): fix data by editing them, then
reseed.

The files were extracted once from the guide pages saved in `guide-cache/` and have been corrected by hand since;
the extractor is gone (ticket 82). A data change edits the `.ron` files by hand, adds a note under Layout below (the
ticket, the decision behind it and what the data now says, as the earlier tickets' notes do) and adds a real-data
test in `libs/seeds/tests/game_data/`; then check the files (`--check-only`, under Seeding) and reseed.
`EXTRACTION_REPORT.md` is the hand-kept list of open data gaps, by category (it began as what the extractor couldn't
turn into data): fix an item in the `.ron` files and delete its line. `libs/seeds/tests/fixtures/game-data/` is a
small hand-written example of the layout.

## Layout

| File | Holds |
| --- | --- |
| `progression.ron` | `Progression`: the mythic rank table (the Character level of each Mythic rank) |
| `bonuses.ron` | `Vec<ItemBonus>`: the flat bonuses items give while held, each naming its item (tickets 119 and 120): spell DCs, caster level against spell resistance, concentration, initiative, saves, skills |
| `interactions.ron` | `Vec<ItemInteractions>`: what the guide says about how an item combines with others, at most one note per item (ticket 124); optional |
| `gear.ron` | `Vec<GearItem>`: the named items a character wears or wields, with their always-on bonuses and a few words on what else they do (ticket 165); optional |
| `races.ron` | `Vec<Race>`, with subraces and the race's own feat lists (the half-elf's Adaptability) |
| `skills.ron` | `Vec<Skill>` |
| `weapons.ron` | `Vec<Weapon>`: the weapons a feat can be taken for, from the guide's simple, martial and exotic lists (`proficiency`; the Sawtooth Saber is exotic, from Weapon Finesse's list), plus what Weapon Focus names besides weapons (unarmed strike, touch attacks, rays), each natural attack the guide names (bite, claw, gore, hoof, slam, spike, tail, talon, wing), Kinetic Blast and Bomb; each weapon's stats for the attack line (ticket 114) |
| `feats.ron` | `Vec<Feat>` |
| `backgrounds.ron` | `Vec<Background>`: the user's list (the guide doesn't list backgrounds, so they have no guide link; ticket 34) |
| `deities.ron` | `Vec<Deity>`: the guide's deity table (`character-creation#deities`, 26 rows; ticket 86) |
| `classes/<class>.ron` | one base `Class`, with its features, feature selections, level rows and archetypes |
| `prestige/<class>.ron` | one prestige `Class` |
| `mythic/abilities.ron` | `Vec<MythicAbility>` |
| `mythic/feats.ron` | `Vec<MythicFeat>` |
| `mythic/paths/<path>.ron` | one `MythicPath`: its rank table, picks and switch rule (what it keeps, drops and grants, the paths it's open from, the character level cap it sets) |
| `spells/<page>/level-<n>.ron` | `Vec<Spell>`: the spells whose blocks sit under that guide page's "Level <n>" heading (`wizard`, `cleric` and `druid` for the three spell pages, else the class or path page), all read into one catalogue (ticket 121) |
| `spells/lists.ron` | `Vec<SpellList>`: every spell list table, its owner, the classes that also use it and its entries by level (ticket 121) |

Any other `.ron` file is an error; other files (like this README) are ignored. Order means nothing anywhere
in the data: catalogs are sorted by key, class levels by level, mythic ranks by rank, and the children of
`All`/`Any`/`AtLeast` are a set, so write entries in whatever order reads best.

What follows from a rule is code in the `rules` crate, not data: each mythic rank's general pick (odd ranks a
mythic ability, even ranks a mythic feat), the path choice at MR 3 and the switch at MR 8, which feat categories
are Combat feats (`FeatCategory::is_combat`), the skill rank cap (the Character level), the first ascension at MR 1
taking one path's MR 1 row there and its MR 2 row at MR 2 (the MR 3 path's when the Build names none; ticket 91), the
first ascension's options (the paths without a switch rule that have an MR 1 row), the rank after a path switch (one above `kept_through_rank`), each
class's level cap (its number of level rows; an archetype has its
class's), and BAB and saves (from the class's progressions, or an archetype's for a save it replaces).

- A class's level rows run `level: 1`, `2`, … with none missing; what taking a class grants goes in row 1. Rows
  hold no BAB or saves: the class names its progressions (`bab: Full`/`ThreeQuarters`/`Half`, and `fort`, `reflex`,
  `will`: `Good`/`Poor`) and the rules compute the numbers, with the prestige save formula for prestige classes. A
  save cell the guide prints differently goes in `save_exceptions: [(level: 20, save: Will, value: 20)]`, exactly as
  printed; the check refuses one the progression gives anyway. An archetype whose saves differ from its class's
  names the progressions it replaces (`fort: Some(Poor)`, `reflex: ...`, `will: Some(Good)`; left out, the class's).
  A replaced save takes the archetype's formula at every level, without the class's `save_exceptions` for that save;
  the check refuses one equal to the class's.
- An archetype has a row only at the levels where it differs from its base class. A row's `features`, `feats`,
  `feat_slots` and `selections` replace the base row's in full (a row with none takes them away), and
  `spell_levels: (arcane: 1, divine: 0)` replaces the base row's spell levels when set. A level without a row is
  the base class's. The check refuses a row that changes nothing and spell levels equal to the base row's.
- Only prestige classes set `advances_arcane`/`advances_divine` (the spell types a level advances), and only base
  classes have archetypes. A Build names the advanced classes on entry: one per spell type the rows advance
  (Mystic Theurge: an arcane and a divine class), or, with `advances: Either` on the class, one class of either
  type ("advances one class, arcane or divine": Hellknight Signifer, Loremaster, Winter Witch). The check refuses
  `Either` on a class whose rows don't advance both types.
- A class whose levels count as another class's says so, and toward what: `counts_as: [(class: "fighter", toward:
  FeatPrerequisites)]` (Diverse Training: a feat's `ClassLevel(class: "fighter", ...)` sees fighter + Eldritch
  Knight levels; nothing else does), or `toward: Feature("witch/major-hexes")` (Frost Power: the witch's rows that
  grant it are reached at witch + Winter Witch levels, while the character has witch levels). The check refuses a
  class counting as itself, and a feature the other class's rows never grant.
- A background (`backgrounds.ron`) makes its `class_skills` class skills and grants its `feats` and `feats_for`
  as a class level row does (the Aldori Swordsman: `feats_for: [(feat: "exotic-weapon-proficiency", value:
  Weapon("dueling-sword"))]`; Bounty Hunter, Gladiator, Guard, Mercenary: Armor Proficiency (Light Armor)). What
  the planner doesn't model is its `note` ("Proficient with scimitar. +2 CMD against trip."). `group` is the game's
  heading (`Craftsman`, `Noble`, `Oblate`, `Regional`, `Scholar`, `StreetUrchin`, `Wanderer`, `Warrior`). There's
  no "None" background: a Build start without one simply has none (user, 2026-11-02).
- A selection option's key extends its selection's: `rogue/rogue-talents/fast-stealth`. A feature's, a
  selection's and a feat list's key starts with its owner's (the archetype's when an archetype defines it), and
  the schema checks it: Hard to Kill is each path's own `aeon/hard-to-kill`, not one shared key.
- An option is defined once, in the selection whose list it is, and another selection that offers it lists its
  key in `shared_options: [...]` after its own `options` (one key per line): the slayer's talents share the 36
  rogue talents, the shaman's hexes the witch's 11 shared hexes, the Sylvan Trickster's Fey Tricks the rogue
  talents and every witch hex, and the Underground Chemist's Discovery (her rogue-talent slots from 10th level)
  the rogue talents and six alchemist discoveries. Never copy an option: a copy is a second key, so a character
  could take it twice. A shared option's prerequisite is the same in every selection, so where it names the
  defining class's feature or level it accepts each sharing class's with an `Any`
  (`Any([Feature(feature: "rogue/advanced-talents", count: 1), Feature(feature: "slayer/advanced-talents", count: 1)])`).
  A selection may share no option of its own (the schema checks it), and one made only of shared options has
  `options: []`.
- The druid's Nature Bond is a selection, `druid/nature-bond` (Animal Companion or Domain), at level 1 of the druid
  and of the archetypes whose table keeps both columns (Defender of the True World, Feyspeaker); a prerequisite
  that needs a companion accepts its option (`SelectionOption("druid/nature-bond/domain")`). The domain option's
  first row grants the druid's own domain pick, `druid/domain` (ticket 31).
- A selection option that grants feats names them: `feats: ["weapon-focus"]` (a rogue talent that is the feat of
  the same name), or several (the Azata's All-skilled: every weapon, armour and shield proficiency). Picking the
  option holds them.
- An option whose pick names a value without granting a feat for it says what kind: `parameter: Weapon` (the
  shaman's Battle Master, whose rows grant `chosen_feats` for that weapon at 8 and 16); never beside a
  `chosen_feat`, whose kind it is then. An option may limit its values: `values: [Weapon("long-bow"),
  Weapon("short-bow")]` (the Zen Archer's Way of the Bow). A selection with one option that takes a value is still
  a pick.
- An option that grants over class levels (a bloodline, school, order, spirit, way or combat style) is a block
  with `levels: [...]`, one row per line, by **option level**: `(level: 9, features: [...], feats: ["wings"],
  selections: ["sorcerer/bloodline/arcane/school-power"], feat_list: "sorcerer/bloodline/abyssal/feats",
  class_skills: ["athletics"])`. A row's `feat_list` is the option's list from that level on (one per stage for a
  ranger style); its `class_skills` are class skills while the row is held (ticket 37), so an archetype that skips
  the row (Divine Herbalist) or a lists-only holding (Crossblooded's second bloodline) gives none. Its selection says
  which classes' levels make the option level, `option_levels: ["dragon-disciple", "sorcerer"]` (an option taken
  late gets every row up to its level at once), and an archetype that changes the grant has its own selection
  that shares the options it allows: `skipped_levels: [3, 9, 15]` (Seeker; its own rows replace them),
  `feat_lists_only: true` (Crossblooded's second bloodline: its lists and its `arcana`, the option's named
  first-row feature, `arcana: "sorcerer/bloodline/abyssal/demonic-arcana"`), `levels_behind: 2` (the Separatist's
  second domain, `cleric/separatist/domain`), only some options (Hag-Riven), or one option, which
  is then granted with no pick (a Sage Sorcerer's Arcane bloodline). `only_without_option: true` grants the
  selection only to a character holding none of its options (Dragon Disciple's Blood of Dragons). A bonus slot
  that picks from the held option's list is `OptionList(selection: "sorcerer/bloodline")`. The option features
  are the class's (`sorcerer/bloodline/abyssal/grow-claws`; the ten dragons share `sorcerer/bloodline/draconic/...`).
  Spells stay out. A Primalist's choice replaces the bloodline's row with `replaces_row_of:
  "bloodrager/bloodline"`; a feature can grant one row of a held option, `option_row: (selection:
  "sorcerer/bloodline", level: 20)` (Bloodline Ascendance); a mythic ability grants `features` and `selections`
  (Second Bloodline). A choice the game asks for every day is `daily_choice: true` on its feature (Mixed
  Bloodline). A class (or archetype, `spontaneous: Some(true)`) that casts spontaneously says `spontaneous: true`,
  which `SpontaneousSpellLevel(spell_type: Arcane, level: 1)` checks; `NoOptionFrom("sorcerer/bloodline")` needs
  none of that selection's options held. A selection with `pool_of` offers the pools (`pool`) of the options held
  in another (an oracle's revelations, `oracle/revelation`, from the mysteries held), beside any options of its own
  (the Dual-Cursed Oracle's `oracle/dual-cursed-oracle/revelation`: Misfortune and Fortune); within a level its
  slot is checked after the other selections'. All this was read from the guide.
- Ticket 33 (the user's decisions on tickets 30 and 31):
  - An option may belong to a group, `group: Draconic` (the sorcerer's ten dragons; `Elemental`, its four
    elements), and `NoOtherOptionOfGroup(Draconic)` in its prerequisite needs no option of the group held but the
    one picked (a Crossblooded or Second Bloodline sorcerer can't hold two dragons).
  - A selection naming no classes may take its option level from elsewhere: `level_from: OtherOptions` (the
    highest level of the other options held that it offers: Impossible Domain's `cleric/impossible-domain`, Second
    Mystery's `oracle/second-mystery`, so a Living Deity's run on arcanist levels) or `level_from:
    GrantingOption` (the level of the option whose row granted the slot: Maneuver Mastery's maneuver follows its
    revelation); since ticket 89 also `level_from: MythicRanks` on a path's selection (Mythic ranks since the
    pick: the Azata's superpowers).
  - `once_per_option: true`: an option the selection gave once can't be picked through it again
    (`kineticist/expanded-element`: the 15th-level element isn't the 7th-level one).
  - `PickedThrough("warpriest/champion-of-the-faith/chosen-alignment")` in an option's prerequisite holds when the
    pick fills that selection's slot: the four alignment blessings are `Any([Not(PickedThrough(...)),
    Alignment(...), ...])`, the Champion's own alignment only when he picks them as his chosen alignment.
  - Daily choices also mark the Unsworn Shaman's Minor Spirit and Wandering Spirit 2, and the hunter's Animal
    Focus and Second Animal Focus (features, not a pick of the eight aspects). Elemental Movement is one feature
    per element (none for Earth). The Divine Hunter skips no domain level. The Signifer's order is
    `hellknight-signifer/order`, sharing the Hellknight's seven.
- Ticket 35 (the user's decisions on tickets 32 and 33):
  - A subrace's `feat_slots` replace its race's, as its ability modifiers do (the guide lists each subrace's own
    traits): none written means none, so the half-elf's Dual Heritage and Kindred Raised get no Adaptability. They
    may name the race's lists.
  - `level_also_from: "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity"` beside `level_from:
    OtherOptions` (`cleric/impossible-domain`): that option's level counts as one of the other options', so a Living
    Deity's Impossible Domain runs on arcanist levels from the pick. Impossible Domain's prerequisite also accepts
    `SelectionOption("hellknight/disciplines/pentamic-faith")`, whose domain is levelled by Hellknight levels.
    It names no druid (ticket 36): the druid's page says a druid who chooses a domain can't take Impossible Domain,
    so neither the Nature Bond domain nor the Blight Druid's or Winter Child's qualifies; a real-data test asserts
    this.
  - `hellknight-signifer/order` is `only_without_option`, as `hellknight/order`.
  - The Chaos, Evil, Good and Law domains: `Any([Not(PickedThrough("cleric/separatist/domain")), Alignment(...),
    ...])`, the six alignments that don't conflict, when taken as the Separatist's second domain.
  - `group: BloodragerDraconic` and `BloodragerElemental` with `NoOtherOptionOfGroup(...)` on the bloodrager's ten
    dragons and four elements: one of each at most (Second Bloodrager Bloodline), apart from the sorcerer's
    groups.
- The second group of options with rows (ticket 31, read from the guide's prose):
  - **Domains** are one selection, `cleric/domain` (37 options, rows by cleric levels: the power at 1, the one
    at 6 or 8, the Animal and Scalykind companion at 4, class skills on row 1 as for the bloodlines, but the Animal
    domain's Lore (Nature) on row 4 with its companion). The cleric's level 1 grants it twice (once for an archetype that "May only select a single
    domain"). Every other class takes domains through a selection of its own that shares them and is levelled by
    its own class: `inquisitor/domain`, `druid/domain` (through Nature Bond), `hunter/divine-hunter/domain`,
    `hellknight/pentamic-faith` (through the Pentamic Faith discipline's row), the Living Deity's
    `arcanist/magic-deceiver/domain` (two at 20), and the archetypes' limited ones. The War domain's bonus Combat
    feat is a slot on its row 8, `feat_slots: [CombatFeat]` (slot id `OptionFeat`).
  - **Mysteries**, `oracle/mystery`: class skills at 1, the Final Revelation at 20, and `pool:
    "oracle/revelations/battle"`, whose options are the revelations; the oracle's revelation cells are
    `oracle/revelation` slots. A revelation that grants later has rows: `chosen_feats: ["improved-critical"]` is
    held for the weapon the pick named for its `chosen_feat` (Weapon Mastery). "Prerequisites: Level 7" is
    oracle or arcanist (Living Deity) level 7.
  - **Blessings**, `warpriest/blessings`: the minor power at 1, the major at 10.
  - **Elements**, `kineticist/element`: class skills and the wild-talent bonus feat list at 1, the defensive talent
    at 2; `kineticist/expanded-element` at 7 and 15 holds a second element (or the first again, `repeat:
    Times(2)`) for its list only.
- Options' class skills (ticket 37, the user's decision on ticket 34's first open
  question): the "Class Skill: …" features of the bloodlines, mysteries, domains and elements, and the cavalier
  orders' "Skills", became their rows' `class_skills`; the Empyreal and Sylvan Sorcerers' went (their own lists
  hold the skill). Ticket 38 (the user's decisions on ticket 37's open questions)
  then dropped the Overwhelming Mage's and the Seeker's "Class Skill" (each holds its bloodline's 1st row, and so
  its class skill), gave the Sohei its own list (the monk's plus Lore (Nature)) for its "Skills", and moved the
  Animal domain's Lore (Nature) to its row 4 ("At 4th level, you gain an Animal Companion. … Lore (Nature) becomes
  a Class Skill."). Left as features: the Arcane bloodline's and the Sage Sorcerer's "Class Skill", a choice of
  one of three skills, shown on the sheet.
- The slayer's talents (`slayer/slayer-talents`, levelled by slayer) include three combat style feats, one per
  window the guide gives ("2 or 4, 6 or 8 and 10 or greater"): each is taken once and its row 1 grants
  `slayer/combat-style` (one of the ranger's five styles, levelled by slayer, `only_without_option`: the first
  talent picks the style) and an `OptionList(selection: "slayer/combat-style", at_level: 2)` slot (6, 10), so
  each window's feat comes from the style's list of its stage, whoever levels the style (the user's decisions: one
  style, as a ranger picks, shared with a ranger's; the window's stage, not the ranger's levels). `at_level` left
  out: the list in force at the option's own level.
- Ticket 43 (the report items the saved pages settle): the totem rage powers are exclusive
  ('Selecting one "Totem" power closes off the other categories'), so each of the twelve (Beast, Celestial, Daemon,
  Fiend: Lesser, plain, Greater) has a `Not(SelectionOption(...))` for each of the other lines' nine in its `All`, one
  `Not` per power so that a lock names the power held ("Needs not Beast Totem, Lesser"); the shaman's Wandering Hex (6,
  14), "temporarily filled" from his main spirit or his current Wandering Spirit, is a `daily_choice`, as the
  Wandering Spirit is; the Eldritch Scoundrel's sneak attack dice are at 1, 3, 7, 11, 15 and 19, as its section lists
  them.
- Ticket 44 (the user's decisions on ticket 43's questions): every sorcerer's
  level-1 "Bonus Feat" is an `OptionList(selection: "sorcerer/bloodline")` slot, as at 7, 13 and 19: the base
  sorcerer's (new), Crossblooded's (either bloodline's list), the Empyreal, Sage and Sylvan Sorcerers' (their fixed
  bloodline's), the Nine Tailed Heir's and the Seeker's (the bloodline chosen "for the purpose of bonus feats"; the
  Seeker's magic feats stay at 5, 10 and 15). The Geomancer ("does not gain bloodline spells or bloodline feats")
  keeps its `General` slot; the Overwhelming Mage ("Does not receive bonus feats") has none. Ticket 92 replaced
  this level-1 slot with a Magic feat (the Geomancer's too); the slots at 7, 13 and 19 stay. Extra Slayer Talent
  needs `ClassLevel(class: "slayer", level: 2)`, the level the slayer's table first gives Slayer Talents ("Talented
  Slayer" is on no saved page). Deities stay out of scope.
- Ticket 45 (ticket 43's decision 2): the kineticist's wild talents are a selection,
  `kineticist/wild-talents` (levelled by kineticist), picked at every even level (the base's rows 2 to 20, the
  Kinetic Knight's 4 and the Blood Kineticist's 20 in place of the "Wild Talents" feature). Its 22 options are the
  guide's 26 talents but the four defensive ones (each element's 2nd-level row still). Each needs
  `ClassLevel(class: "kineticist", level: …)` at twice its talent level, one of its elements (`SelectionOption`,
  through any selection: Expanded Element's too; an `Any` for "Fire, Water"; none for Universal and All) and what
  its "Prerequisites" names (a defensive talent's feature, Kinetic Healer's option; "Fire blast" is the Fire element
  already). Kineticist Bonus Feat's row 1 is an `OptionList(selection: "kineticist/element")` slot (the held
  elements' lists, which ignore prerequisites); the Skill Focus talent is `chosen_feat: "skill-focus"`; Expanded
  Defense's row 1 holds a feature with `option_row: (selection: "kineticist/expanded-element", level: 2)`, and an
  option row of the very selection a `feat_lists_only` option is held through is given in full (an expanded
  element's defensive talent), now or when the element comes later. Extra Wild Talent needs kineticist level 2.
- Ticket 47 (the user's decisions on ticket 45's questions, 2026-11-02): the Skill Focus
  wild talent and rogue talent are `repeat: PerValue`, once for each value of their chosen feat (an option is
  `PerValue` only with a `chosen_feat`): Skill Focus in another skill each time. The rogue talent Weapon Focus stays
  once. Combat Trick ("Any combat feat that the character qualifies for") is `repeat: Unlimited` with a row 1 of
  `feat_slots: [CombatFeat]`, a bonus Combat feat slot each time it's taken, its feat's prerequisites checked; the
  selections offering it are levelled by their class (`option_levels: ["rogue"]` on the rogue talents, Fey Tricks
  and Discovery; the slayer's talents by the slayer already). An `OptionList` slot may carry its own `label`: the
  bonus-feat wild talent's reads "Wild talent feat" (`OptionList(selection: "kineticist/element", label: "Wild
  talent feat")`).
- Ticket 49 (ticket 45's decision 2, user, 2026-11-02): a feat whose guide entry says "gain
  one additional …" names the selections its "Prerequisites" names in `extra_pick_of` (`extra-hex`: `["shaman/hex",
  "witch/hex"]`), and each pick of it gives one more slot of the one the character had a slot of first, at the
  level it's taken (a `FeatSelection` slot); a character who had a slot of none gets none. Eight feats: Extra
  Arcana, Extra Arcanist Exploit, Extra Discovery (the alchemist's and the Underground Chemist's), Extra Hex, Extra
  Rage Power, Extra Rogue Talent (the rogue's and the Sylvan Trickster's Fey Tricks), Extra Slayer Talent and Extra
  Wild Talent. Extra Mercy says it too, but the paladin's mercies were a feature, not a selection, so it named none
  until ticket 54.
- Ticket 54 (ticket 49's decisions 1 and 5, user, 2026-11-02): the paladin's
  "Mercy" feature is the selection `paladin/mercy` on the rows that held it (each archetype's where its table has
  "Mercy": the Divine Hunter's 3, 9, 12, 15, the Stonelord's 6, 12, 18), with the guide's 15 mercies, each needing
  `ClassLevel(paladin, its level)` and the mercy its table names (Exhausted: Fatigued). Extra Mercy needs a mercy held
  (an `Any` of the 15) and has `extra_pick_of: ["paladin/mercy"]`. The Arcane Enforcer's "Arcanist Exploits" feature
  is a slot of the arcanist's own `arcanist/arcanist-exploits` at its rows 1, 4, 7, 10, 13, 16 and 19 (the guide gives
  it no list of its own; a row may hold another class's selection, as a bloodrager's rage powers), so an exploit is
  taken once across both classes and Extra Arcanist Exploit's pick reaches it; Extra Arcanist Exploit's `Any` no
  longer names the feature. The Enforcer's "Greater Exploits" stayed a feature until ticket 57.
- Ticket 57 (ticket 54's decisions 1 and 2, user, 2026-11-02): the Arcane Enforcer's "Greater
  Exploits" feature at 13 is a slot of the arcanist's own `arcanist/greater-exploits`, read as the arcanist's own
  "Greater Exploits" cell at 11 is (one pick; each greater exploit needs the exploit it extends). The Exploiter
  Wizard's "Exploiter Exploits" feature is a slot of `arcanist/arcanist-exploits` at its rows 1, 5, 9, 13 and 17 (the
  guide gives it no list of its own), and its "Greater Exploits" a slot of `arcanist/greater-exploits` at 13; both
  features are gone. An exploit is taken once across the arcanist, the Enforcer and the Exploiter, and Extra Arcanist
  Exploit's pick reaches the Exploiter too.
- Ticket 58 (ticket 54's decision 2, user, 2026-11-02): the monk's "Ki Power" feature is the
  selection `monk/ki-power` on the rows that held it (each archetype's where its table has "Ki Power": the Scaled
  Fist not at 12, the Sohei from 6, the Student of Stone not at 6 or 12, the Zen Archer not at 10), with the guide's
  14 ki powers, each needing `ClassLevel(monk, its level)`; a power with no section of its own links to the page. The
  Traditional Monk picks none: its fixed powers ("Ki Power: Wholeness of Body" at 6, Abundant Step at 8, Diamond Body
  at 10, Diamond Soul at 12, Quivering Palm at 14) are features of its own on rows 6 to 14, in place of the base
  class's pick (those rows were the base's before). No feat gives a ki power: Extra Ki and Abundant Ki Pool grow the
  pool.
- Ticket 59 (ticket 54's decision 2, user, 2026-11-02): the kineticist's "Infusion" feature
  after 1st level is the selection `kineticist/infusions` ("Infusion") on the rows that held it ("She gains new
  Infusions at 3rd, 5th, 9th, 11th, 13th, 17th and 19th levels"; each archetype's where its table has "Infusion":
  the Blood Kineticist's 3, 13, 17, 19, the Kinetic Knight's 5, 11, 13, 17, 19), with the guide's 32 infusions. Each
  needs `ClassLevel(kineticist, twice its level)`, one of its elements (`SelectionOption`, an `Any` for several; none
  for Universal) and one of the options its "Prerequisites" names. The Kinetic Sharpshooter's four own infusions
  need `Archetype("kineticist/kinetic-sharpshooter")`, the six it "cannot use" `Not(Archetype(...))`; its
  "Infusion (Kinetic Sharpshooter)" feature is gone (its rows 3 to 19 hold the slot; 17 and 19 are the base's now).
  The 1st-level "Infusion" stays a feature: no infusion is open at kineticist 1. The guide's "Dazziing Infusion" is
  named "Dazzling Infusion", linking to the guide's anchor. No feat gives an infusion.
- Ticket 60 (ticket 54's decision 2, user, 2026-11-02): picks from another class's
  list are slots, of that class's selection where the guide's list is one, or of a selection sharing the lists it
  mixes (as ticket 25's slayer talents), in place of the feature that stood for them:
  - the bard's "Bard Talents" and the skald's "Skald Talents" ("can select from the Rogue Talent list") are slots of
    `rogue/rogue-talents` where each table has "Bard Talent" or "Skald Talent" (the Beast Tamer at 6, 14, 18; the
    Inciter at 4, 9, 14, 19); the Archaeologist's Advanced Talents (10) opens the 16 advanced rogue talents beside the
    rogue's and the slayer's;
  - the Feyform Shifter's "Rogue Talent" (8, 14; "you cannot choose Advanced Talents") is its own selection sharing
    the 20 other rogue talents;
  - the Vivisectionist's "Medical Discovery" (2 to 18) is its own selection sharing the 14 discoveries and 11 rogue
    talents its section lists; its Advanced Talents (10) opens the three advanced ones there;
  - the Sanctified Slayer's "Slayer Talents" (8, 16, 17, 20) are slots of `slayer/slayer-talents`; Extra Slayer
    Talent's "Slayer Talents" is a slayer 2 or a Sanctified Slayer 8, and the 2nd-or-4th combat style talent needs
    slayer level 2, as the other two windows need theirs;
  - the Hexcrafter's Hex Magus (4) is a slot of `witch/hex`, and its Hex Arcana (6, 9, 12, 15, 18) its own selection
    sharing the magus arcana and the witch hexes; the major hexes open to a Hexcrafter of magus 12 and the grand ones
    of magus 18, in each hex's `Any`.

  Combat Trick's row needs an option level, so `rogue/rogue-talents` is levelled by the bard and the skald too and
  `slayer/slayer-talents` by the inquisitor (their options' rows are all at level 1, so only whether the row comes
  changes). Extra Rogue Talent's extra pick may be of the Feyform Shifter's selection too; Extra Hex reaches the
  Hexcrafter through `witch/hex`. Extra Discovery and Extra Arcana gave the Vivisectionist and the Hexcrafter none
  until ticket 64.
- Ticket 61 (ticket 54's decision 2, user, 2026-11-02): a bonus feat chosen from a
  fixed list is a `OneOf` slot of that list where the table has the cell, as the Divine Guardian's bonus feats are,
  with no feature standing for it:
  - the Titan Fighter's "Massive Attacks" (2, 6, 10, 14, 18, in place of the fighter's bonus combat feat) picks from
    its own `fighter/titan-fighter/bonus-feats-2`, `-6` and `-10`: the ranger's Two-Weapon Combat Style feats by
    then (Double Slice, Shield Bash, Two-Weapon Fighting; Improved Two-Weapon Fighting from 6; Greater Two-Weapon
    Fighting from 10), without the extras every ranger style adds, ignoring prerequisites as the ranger's do;
  - the hunter's Precise Companion (2, all but the Forester), the Urban Hunter's Captor (6, 9, 12, 15, 18) and the
    Stormwalker's Archery (2, 6, 10, 14, 18) were slots of their lists already (ticket 18); the features beside them
    went.
- Ticket 55 (ticket 49's decision 4, user, 2026-11-02): a mythic feat's `extra_pick`
  is the slot a pick of it gives at the Mythic rank it's taken, after the rank's general pick (a `MythicExtra`
  slot): `Feat`, a feat slot offering what the general slot does (`extra-feat`: "You can select any normal feat"),
  or `MythicAbility`, a mythic ability slot offering what a rank's general pick does (`extra-mythic-ability`). Both
  are taken once: their entries don't say otherwise. A mythic ability in that slot grants its `selections` as the
  general pick's would (Second Bloodline), and a feat with an `extra_pick_of` gives its slot at the rank's level.
- Ticket 62 (ticket 55's decisions 1 and 2, user, 2026-11-02): two more mythic
  abilities grant a selection slot through `selections`, as Second Spirit and Second Mystery do. Witch Wandering Hex
  ("You can select any hex for which you meet the prerequisites") gives one slot of `witch/hex`, each hex's own
  prerequisites checked, and needs a hex held, the witch's or the shaman's: its "Prequisites: Hex" (so spelt in the
  guide, which the extraction passed over) is Extra Hex's `Any`. Beneficial Curse ("You gain an additional Curse")
  gives one slot of `oracle/oracles-curse`; a curse held is "Already taken" there.
- Ticket 63 (ticket 58's question 1, user, 2026-11-02): an archetype's alignment
  requirement replaces its class's, and the class's other conditions still hold (`rules` does this; the Drunken
  Master's "Cannot be Lawful" replaces the monk's "Lawful"). So a class's or an archetype's alignment requirement is a
  condition of its own: the whole `prerequisite`, or one of a top-level `All`'s conditions, never mixed into a group
  with others (the check refuses that). Seven archetypes have one: the Drunken Master, Bloodseeker, Angelfire
  Apostle, Reformed Fiend and Prophet of Pestilence from their guide lines; the Priest of Balance's `Any([LN, TN,
  CN])` from its "Alignement restricted to …" line (so spelt in the guide, which the extraction passed over); and the
  **Mantis Zealot's `Any([LN, LE, NE])`, the user's fact, not the guide's** (2026-11-02: one step from Achaekek's
  Lawful Evil; the guide says only that it worships Achaekek, and deities are out of scope).
- Ticket 64 (ticket 60's decisions 2 and 3, user, 2026-11-02): Extra Discovery's
  extra pick may be of the Vivisectionist's `alchemist/vivisectionist/medical-discovery` too, and Extra Arcana's of
  the Hexcrafter's `magus/hexcrafter/hex-arcana` (Medical Discovery stands where the alchemist's table has Discovery;
  Hex Arcana where the magus's has Magus Arcana but the first). Their "Prerequisites" name the class feature
  ("Dicovery", so spelt; "Magus Arcana"), so each `Any` gains the archetype from its first slot, whatever it picked
  there: `All([Archetype("alchemist/vivisectionist"), ClassLevel(class: "alchemist", level: 2)])` and
  `All([Archetype("magus/hexcrafter"), ClassLevel(class: "magus", level: 6)])`, as a Sanctified Slayer 8 meets Extra
  Slayer Talent's.
- Ticket 66 (ticket 58's decision 3, user, 2026-11-02): the Instinctual
  Warrior's monk powers are five options of `barbarian/rage-power` that need its archetype
  (`barbarian/rage-power/deflect-arrows-instinctual-warrior`, …, named "… (Instinctual Warrior)" and linking to
  `barbarian#rage-power-instinctual-warrior`, which lists them): Deflect Arrows needs `Feat("improved-unarmed-strike")`,
  Purity of Body, Diamond Body, Diamond Soul and Perfect Self barbarian level 8, 12, 16 and 20 (`ClassLevel`, as
  every rage power's level is). They aren't the monk's ki powers or features, and holding one is holding a rage
  power, so Extra Rage Power's `Any` names them too.
- Ticket 67 (ticket 62's decision, user, 2026-11-02): the Dual-Cursed Oracle's "Second
  Curse" feature is a second slot of the oracle's own `oracle/oracles-curse` on its level-1 row ("The Oracle must
  choose a second curse"), so the row holds that selection twice, as a cleric's holds `cleric/domain` twice. Every
  curse is taken once, so the second slot doesn't offer the curse taken in the first, and Beneficial Curse's slot
  offers neither. "One of these curses doesn't change as the oracle gains levels" needs nothing: no curse has rows.
- Ticket 86 (the user took all fourteen recommendations, 2026-11-06; spells stay out of scope): deities.
  `deities.ron` holds the guide's table (`character-creation#deities`), one entry per row, parsed from the saved page
  and checked by hand: `alignments` are the row's ticks (the alignments a follower may have), `favoured_weapons` are
  `weapons.ron` keys and `domains` are `cleric/domain` options. Asmodeus's "Mace" is `heavy-mace` (question 9: the
  weapons have a light and a heavy mace, both simple, so nothing a Build holds depends on which). The table, not the
  Domains section's summary table, is the source (question 8: Torag has no Glory, Zon-Kuthon no Undead). Atheism is
  no entry: it's a Build start choice of its own. In the classes: each of the 25 warpriest blessings names its
  domain (`domain: "cleric/domain/air"`); `deity_domains: Offered` limits `cleric/domain`, `inquisitor/domain`,
  `inquisitor/sacred-huntsmaster/domain`, `hunter/divine-hunter/domain` and `warpriest/blessings` to the deity's
  domains, and `NotOffered` the Separatist's second domain to the others (question 6; the druids', Impossible
  Domain, Pentamic Faith, the Living Deity's and the Champion's and Feral Champion's have no limit);
  `favoured_weapon_proficiency: true` is on Cleric Proficiencies, Ecclesitheurge Proficiencies, Inquisitor
  Proficiencies and the Deliverer's Deity (questions 10 and 11: not the Angelfire Apostle's or Demonbane Priest's).
  Prerequisites: `Worships` on the cleric, inquisitor and warpriest and on the Divine Hunter and Deliverer
  (question 2); the Mantis Zealot's user-fact alignment rule is now `Deity("achaekek")`, whose alignments are the
  same three (question 1); the Feral Champion `Any([Deity("erastil"), Deity("gozreh"), Deity("green-faith")])`, the
  Hag of Gyronna `Deity("gyronna")`, the Living Deity way `Atheist`; the Angelfire Apostle adds
  `DeityOffers("cleric/domain/good")` and the Priest of Balance `Not(DeityOffers(...))` of Good and Evil (question
  12: a good deity offers the Good domain, a neutral one neither). The four alignment domains need their alignment
  when picked through a deity-limited selection (question 7: `Any([Not(PickedThrough(...)), Alignment(...) ×3])` for
  each of the four domain selections, beside the Separatist's own rule), and so do the four alignment blessings
  through `warpriest/blessings`; as the Champion's chosen alignment they also need a deity offering that domain
  (question 13: `Any([Not(PickedThrough("warpriest/champion-of-the-faith/chosen-alignment")), DeityOffers(...)])`).
- Ticket 89 (found by the ticket 88 QA builds): Life-Bonding Friendship ("You choose a teamwork feat this rank and
  the next rank", `azata#life-bonding-friendship`) gives its two teamwork feats. `azata/superpowers` has
  `level_from: MythicRanks`: an option held through a path's selection with it takes as its option level the Mythic
  ranks since its pick, the pick's rank counting as 1 (only a path's selection, the schema checks). The superpower's
  rows are `(level: 1, feat_slots: [Category(Teamwork)])` and `(level: 2, ...)`: a Teamwork feat slot at the rank
  it's taken and the next, each the rank's path feat slot after the path row's own, prerequisites checked as in
  any category slot (Shield Wall needs Shield Proficiency). Taken at MR 10, the last rank, it gives the first slot
  only (the ticket's decision: the game has no MR 11 to give the second). No other superpower has rows.
- Ticket 90 (found by the ticket 88 QA builds): the Arcane Enforcer's saves. The guide (`slayer#arcane-enforcer`)
  says "High saving throws for Reflex and Will, low saving throws for Fortitude", where the slayer has good Fortitude
  and Reflex and poor Will, so `slayer/arcane-enforcer` has `fort: Some(Poor)` and `will: Some(Good)` (Reflex stays
  the slayer's). A sweep of every "saving throws for" sentence in `guide-cache/` finds no other archetype with saves
  of its own.
- Ticket 92 (found by the ticket 88 QA builds; the user, 2026-11-07): every sorcerer's level-1 "Bonus Feat" is a
  Magic feat, `feat_slots: [Category(Magic)]` on row 1, in place of ticket 44's bloodline-list slot: the base
  sorcerer's and Crossblooded's, the Empyreal, Sage, Sylvan, Nine Tailed Heir's and Seeker's, and in place of the
  Geomancer's `General` slot. The Jack-in-the-Green write-up (a Sylvan Sorcerer) takes Spell Penetration there, off
  the Fey list. The bloodline feats at 7, 13 and 19 and the Seeker's Magic feats at 5, 10 and 15 stay; the
  Overwhelming Mage keeps Overwhelming Penetration at 1 and no slot. The sheet labels the slot "Magic feat".
- Ticket 91 (found by the ticket 88 QA builds; the user, 2026-11-07): a path's MR 1 and 2 rows are its first
  ascension's (`mythic-paths#first-ascension`: at the first rank "an ability that is linked to one of the Mythic
  Paths", which doesn't force that path, and at the second "a second ability related to your earlier choice" with
  Bypass Epic Damage Reduction at 1 and Hard to Kill at 2). No data changed: each of the six paths' MR 1 row is
  Bypass Epic Damage Reduction and its path-linked ability, its MR 2 row the follow-ups and Hard to Kill, and Gold
  Dragon has none, which is what the rules read. The two quirks the ticket named are the guide's: `trickster.ron`'s
  rank 2 does hold `trickster/hard-to-kill` (the guide's Trickster table: Creative Approach, Hard to Kill,
  Will-o'-Wisp at 2), and the Aeon's rank 2 repeats Force Reality because the guide's Aeon table does ("At 2nd mythic
  rank, you can target all allies": it improves at 2, as Aeon Immunities repeats at 3, 5 and 7).
- Ticket 112 (the user, 2026-11-08): every pickable item has a `summary`, one sentence of at most 160 characters in
  our own words, written from the saved guide pages and then reviewed against them in full by a second reader
  (numbers, levels, uses per day, conditions and who benefits; close paraphrases of the guide's own sentence were
  reworded). A summary addresses the player as "you"; an archetype's or class's description may be in the third
  person ("their"). Where the guide gives a feature only in a level table, or says it is another class's (the
  skald's Bardic Knowledge, the bloodrager's Uncanny Dodge), the summary comes from that table or page. Where the
  guide disagrees with itself the summary says so ("18th (the guide's table; its text says 19th)"); where it looks
  wrong but says nothing (the warpriest Fire blessing's acid damage), the summary follows it. 133 items have no
  summary because the guide has nothing usable on them (cantrips and orisons, features named only in a table,
  guide links without an anchor); `tests/game_data/summaries.rs` keeps their count per kind from growing. Feat
  lists have no `summary` field.
- Ticket 114 (option B, the user, 2026-11-08): weapon stats, for the attack line the rules compute. Each weapon on
  a proficiency list, and the unarmed strike, has `stats: (damage: (count: 2, sides: 6), threat: 19, multiplier: 2,
  handedness: TwoHanded)`, plus `ranged: Thrown`, `Projectile` or `ProjectileWithStrength` for a ranged weapon and
  `finesse: true` for a finessable one. `threat` is the lowest roll that threatens (19 for 19–20). **The source is
  the game's weapon table as the user gave it (2026-11-08)**, not the guide, which lists no weapon stats; it
  replaced the first draft's PF1e core values, changing six weapons: the Gnome Hooked Hammer's first head is 1d8
  (was 1d6), the Tongi threatens on 19–20 (was 20), the Sawtooth Saber is light (was one-handed), the Nunchaku and
  the Light Shield's bash are finessable, and the Sling Staff has the game's Strength trait (`ProjectileWithStrength`:
  it shoots but adds Strength to damage). Where the game has a Kingmaker and a Wrath of the Righteous row (Dagger,
  Dwarven Urgrosh, Scythe, Dart, Sawtooth Saber), the Wrath row is used. The table wins over the guide's Weapon
  Finesse list (`feats#weapon-finesse`), which leaves out the Nunchaku and the Light Shield's bash. A double weapon
  (Dwarven Urgrosh, Gnome Hooked Hammer, Orc Double Axe, Two-bladed Sword) has its first head's dice and critical,
  as one two-handed weapon. The table's composite bows and spiked shields aren't weapons of the data (the data has
  the plain bows and the shield bashes). The natural attacks (bite, claw, gore, hoof, slam, spike, tail, talon, wing)
  have no stats, though the table gives Medium base dice for them, since a feature that grants one sets its own
  dice (Demonic Claws' 1d8); nor have Rays, Touch Attacks, Kinetic Blast and Bomb, which deal a spell's or a class
  feature's damage. `tests/game_data/weapons.rs` holds the whole table.
- Ticket 130 (found in the 2026-11-09 martial build run): a race's or subrace's `skill_points` are the skill points
  it adds at every level, on top of the class's and Intelligence's; a subrace's replace its race's, as its ability
  modifiers do. Only the guide's "Skilled" trait sets them (`character-creation#human` and
  `#half-orc-human-raised`: 1); none written means 0. `tests/game_data/races.rs` keeps the two.
- Ticket 131 (the user, 2026-11-09): the Titan Fighter has no Weapon Training. The guide contradicts itself
  (`fighter#titan-fighter`'s intro says it has none, its table lists it at 5); the user settled it: Incredible
  Heft replaces Weapon Training and doesn't count as it, so what needs Weapon Training (Advanced Weapon Training,
  and in the game Gloves of Dueling) doesn't work for it. Its empty `(level: 5)` row takes the base row's away.
  Weapon Mastery at 20 stays, as the table lists it. `class_rules.rs` tests it.
- Ticket 121 (option B, the user, 2026-11-09; stage 1, the foundation): the spell catalogue, reference data no Build
  choice names. Extracted once from the saved pages (`wizard-spells`, `cleric-spells`, `druid-spells` and the class
  and path pages) by a script kept outside the repo, then hand-fixed as below; the files are the master from here.
  - **Spells:** one per block, a heading "Name (School)" and a table, in the file of the page and "Level <n>"
    heading it sits under: 632 spells in 98 files (wizard 280, cleric 104, druid 53, angel 32, azata 26, lich 25,
    aeon 21, demon 20, trickster 17, gold-dragon 11, bard 10, inquisitor 9, paladin 9, arcanist 7, ranger 4,
    alchemist 2, witch 2). Each has its `key` (the name's slug), `name`, `school` (`None` for Dismiss Spell's
    Universalist, which no Spell Focus takes), `guide` (`page#anchor` of the block), `summary: ""` (stage 2 writes
    them), and the block's rows as printed: `components` (the material, when the block has the row), `target`,
    `duration` (left out where the block has none), `saving_throw`, `spell_resistance` (✓ is `true`, N/A
    `false`), `descriptors` (the subschool, Compulsion or Charmed, is among them; "-" is none) and
    `casting_time` (`Standard`, `FullRound`, `Move`, `Swift`, `Free`; the guide's capitals vary). A spell has no
    level of its own.
  - **Lists:** 24, one per table of spells by level (`["Spell", "School"]`, the alchemist's `["Extract",
    "School"]`), keyed by their owner: the wizard's (`wizard-spells`, "for Wizard and Sorcerer"), cleric's and
    druid's pages; the bard, bloodrager, hunter, inquisitor, magus, paladin, ranger, shaman, warpriest and witch
    (`<class>#<class>-spells`) and the alchemist's extracts; the Magic Deceiver's (`arcanist/magic-deceiver`);
    and the Mythic spellbooks, levels 1 to 10 (aeon, azata, demon, gold-dragon, trickster). The Angel and the
    Lich have two each: `angel` and `lich`, the path's own spellbook (the guide's "(Unmerged)" tables, levels
    1–10), and `angel/merged` and `lich/merged`, the spells a merged spellbook adds at levels 3–10 (their pages:
    "If you choose to merge spellbooks, the following spells are added"). `used_by` names the classes and
    archetypes whose pages say they use the list: the sorcerer, the arcanist and the Eldritch Scoundrel the
    wizard's, the oracle the cleric's, the skald the bard's, the Nature Mage the druid's and the Unlettered
    Arcanist the witch's. An entry's `only` is the table's footnote: an archetype of the list's class ("Flame
    Dancer only"; the druid's Aspect of the Stag, "only available through a Drovier's Communal Aspect") or a
    Gold Dragon story branch (uncorrupted: `Story(Uncorrupted)`; corrupted: `Story(VowOfDahak)`; the two
    footnotes sit under its level 7 table and number entries at every level). Left out, as they aren't Spell /
    School tables: the shaman spirits' bonus spells, the oracle's bonus and additional spells and the Beast
    Tamer's bonus spells.
  - **Hand-fixes:** the witch's level 6 table is headed by an `<h5>` "Level 6", not an `<h4>`; its 19 entries are
    level 6. Block headings: "Soothing Mud Conjuration)" is Soothing Mud (Conjuration); five Illusion schools
    printed "(illusion)"; "Reviving Final Finale" is Reviving Finale, as the bard's table names it; "Embodiment of
    Law" is Embodiment of Order, as the Aeon table names it (the guide links keep the blocks' anchors). Heaven's
    Wing Strike has a block on the bard page and one on the wizard page, with the same facts: one spell, linked to
    the wizard page's. Descriptors: Animate Dead, Lesser's "Summonig" is Summoning; Death Clutch's "Evil. Death"
    is Evil and Death; Transformation and Walk through Space have no descriptors row (none). List names that
    misspell or rename the block's spell: "Neutralise Poison" (twelve lists) is Neutralize Poison,
    "Flamestrike" Flame Strike, "Unbeakable Heart" Unbreakable Heart (cleric); by the block each links to, the
    Magic Deceiver's "Dispel Magic (Target)" and "Dispel Magic, Greater (Target)" are Dispel Magic and Dispel
    Magic, Greater, the paladin's "Protection from Chaos / Evil" (and Communal) and the Demonslayer's
    "Protection from Evil" are Protection from Alignment (and Communal), the Flamewarden's "Summon Small / Medium
    / Large Fire Elemental" are Summon Small / Medium / Large Elemental, and the Trickster's "Summon <size> Beer
    Elemental" (one "Lager") are Summon <size> Elemental (its note: beer elementals are water elementals). A
    link that names another block than the table's name (the Sleep rows link to Hypnotism, Baleful Polymorph to
    Vampiric Shadow Shield, Dismissal to Summon Monster III, Burst of Glory to Crusader's Edge, Inflict Serious
    Wounds to its Mass version, the magus's Animal Aspect to Angelic Aspect) is read by the name. Three blocks
    sit under a heading whose table omits them, and are on that list at that level: Rage (wizard 3), Winds of
    Vengeance (wizard 9) and Thirsting Entanglement (druid 4). The tables' School column disagrees with the block
    in places (Web: Conjuration in the magus's and witch's tables, Transmutation in its block); a spell's school
    is its block's. The shaman's table lists Stoneskin at 4 and at 5, both kept as printed.
  - `GameData::check` refuses a duplicate spell or list key, a list keyed outside its owner, a list naming a
    spell, class, archetype or path the data doesn't have, an entry above level 9 (10 in a Mythic spellbook), an
    entry `only` for an archetype of no class on the list (or a story branch off a path's list) and a spell on no
    list. `tests/game_data/spells.rs` keeps the counts per page and list, Hold Monster's facts and levels, each
    spell's file against its list and level, and the hand-fixes.
  - A data folder without `spells/` (data from before this ticket, which `scripts/proof.sh data` reads as its side
    A) has no spells and no lists; once the folder exists, `spells/lists.ron` is required.
  - The spells and lists stay out of the game data JSON the browser gets (`#[serde(skip_serializing)]` on
    `GameData`): the browser has no spell UI and its rules never read them, and the MCP server reads the data
    files.
- A path's switch rule: `switch: (from: ["aeon", "azata"], kept_through_rank: 7, path_kept_through_rank: 7,
  dropped_features: [...], dropped_selections: [...], features: [...], further_ascensions: true, level_cap:
  Some(40))`. The general picks (mythic abilities and feats) of MR 1 to `kept_through_rank` stay, and the old
  path's own grants (its rows' features, the picks in its slots) of MR 1 to `path_kept_through_rank`; the
  `dropped_*` lists take named grants of the old path away whatever their rank; `features` are granted on
  switching (Legend's +4). `from` left out: open from any path.
- A feature that changes the character's own ability scores lists `ability_bonuses`: `Fixed(ability: Strength,
  amount: 4)`, `Every(amount: 4)` (Legend), `Highest(amount: 4)` (the highest score) and `AtLeast(score: 14)`
  (every score at least). They count toward ability-score prerequisites while the feature is held, after the
  bonuses of the gear the Build holds (ticket 166). A feature can also set floors that prerequisites see while it's held: `bab_floor: Some(15)` and
  `skill_ranks_at_least_mythic_rank: true` (Perfect Mind).
- Mythic abilities and feats name their group as the guide heads it: `group: Melee`, `ArmourAndShield`,
  `CombatManoeuvre` and so on.
- Feat slots are `General`, `CombatFeat`, `Category(Teamwork)` or a named list, `OneOf(list: "monk/bonus-feats-1")`.
  The lists are in the file of the class, archetype or path that owns them, in a `feat_lists: [...]` block before
  its `levels`/`ranks`: `(key: "monk/bonus-feats-1", name: "Monk Bonus Feats", guide: "monk#monk-bonus-feats",
  ignores_prerequisites: true, feats: ["crane-style", "dodge"])`. The key starts with the owner's key; any row may
  name any list. A list that grows with level is one list per stage. A list that is the class's own bonus-feat
  list (a monk's bonus feats, a ranger's combat style) sets `ignores_prerequisites: true`: a pick from it skips
  the feat's prerequisites. Only class and archetype rows, and a path's ranks naming the path's own list, may name
  such a list; left out, prerequisites are checked. A list of every feat (Gold Dragon's Dragon Feats: "any three
  feats") sets `every_feat: true` and names no feats. The check refuses a list of fewer than two feats (unless
  it's every feat, or its one feat takes a value) and one no slot names. A bonus feat of the player's choosing
  (the warpriest's "Weapon Focus as a bonus feat in a weapon of his choosing", the Master of All's Skill Focus at
  3, 9 and 15, the Sword Saint's Chosen Weapon) is a bonus list of that one parameterised feat: the pick names the
  value. A race may own lists too, in a `feat_lists` block before its subraces, and its slots may name only its
  own (the half-elf's Adaptability: Skill Focus as a bonus feat).
- A feat the class gives outright is a granted feat, not a slot: `feats: ["scribe-scrolls"]` on the level row
  (class or archetype). It's held with no pick, whatever its prerequisites, and it's in no Build or link. A
  parameterised one granted for one value is `feats_for: [(feat: "weapon-focus", value: Weapon("sawtooth-saber"))]`
  (the Mantis Zealot; the Chirurgeon's and the Enlightened Philosopher's Skill Focus). Classes
  grant their Armor, Shield and Weapon Proficiency feats this way, from each class page's proficiency text; a
  `<class>-proficiencies` feature stays only where it says more than those feats (named weapons, bombs, casting
  in armour, or a prerequisite names it).

- A feat the guide says to take for something names what: `parameter: Weapon` (also `ExoticWeapon`, `Skill`,
  `SpellSchool`, `Energy`, `ArmourType`), and each pick of it names a value. How often a feat can be taken is
  `repeat:`, left out for once: `PerValue` (once per value: every parameterised feat, by the user's decision),
  `Times(3)` ("up to 3 times"), `Unlimited` ("may be taken multiple times"). A mythic feat takes `parameter` and
  `repeat` the same way (Improved Critical (Mythic): a weapon, once per weapon), and `removes_racial_penalty: true`
  (Destiny Beyond Birth) gives back the race's ability penalties while it's held. A selection option may repeat too
  (`repeat: Times(3)`, never `PerValue`), and one that is a
  parameterised feat of the player's choosing names it as `chosen_feat: "weapon-focus"` (its pick names the
  value); a parameterised feat in an option's `feats` or a row's `feats` is held for every value (All-skilled:
  Exotic Weapon Proficiency in every exotic weapon).
- Three kinds of value only a pick names (ticket 125), never a prerequisite, an option's `values` or `feats_for` (the
  check refuses it): `Metamagic` (a metamagic feat: every feat whose key starts `metamagic-`, a rule in `rules`, not a
  field), `Stat` (Stat Focus's list: AC, attack, attacks of opportunity, CMB, CMD, damage, HP, initiative, Fortitude,
  Reflex, Will, speed) and `Element` (Ascendant Element's: acid, cold, electricity, fire, sonic, negative energy). A
  mythic ability takes `parameter` too, and is held once whatever its value (the guide says nothing of taking it
  again): Favorite Metamagic `parameter: Metamagic`, Ascendant Element `parameter: Element`. Stat Focus is
  `parameter: Stat, repeat: PerValue` (every parameterised feat, by the user's decision). A pick of these three kinds
  may leave its value open, so a link made before they took one still opens, with the value to choose.
- Ticket 125's audit: every feat, mythic ability, mythic feat and option whose summary says "chosen", "choose",
  "choice" or "pick" has its pick (a parameter, a selection, a chosen feat, a pool, rows with slots or selections, an
  extra pick), or is one the ticket checked and left without: Advanced Weapon Training, Energized Wild Shape,
  Metamagic (Selective Spell), Spell Specialization, Rogue Secret, Increased Damage Reduction and Lore (Religion) II
  and III (the real-data test `pick_values` pins the list; the ticket says why each).
- The caster line's data (ticket 119). A class that casts spells of its own names the ability they use,
  `casting_ability: Charisma` (the guide's "primary attribute" where it names one, else the core rules: the
  arcanist, magus, witch and wizard Intelligence, the bard, bloodrager, oracle, paladin, skald and sorcerer Charisma,
  the cleric, druid, hunter, inquisitor, ranger, shaman and warpriest Wisdom); an archetype that casts with another
  names its own (Sage Sorcerer and Eldritch Scoundrel Intelligence, Empyreal Sorcerer and Tortured Crusader Wisdom,
  Stigmatized Witch, Ley Line Guardian and Eldritch Scion Charisma, Living Grimoire Intelligence). Prestige classes
  have none: they advance another class's spells. The check refuses a class or archetype whose rows give spell levels
  with no ability, and an ability on a class that casts nothing of its own.
- `bonuses.ron` lists the flat bonuses (`rules::ItemBonus`): `(item: Feat("spell-focus"), target: SpellDc(Value),
  amount: Flat(1))`. The item is a `Race`, `Subrace`, `Feat`, `MythicFeat`, `Feature` or `SelectionOption`; a
  race's bonus counts without a subrace and a subrace's in place of its race's (the guide lists a subrace's traits in
  full: Gnome +1 illusion, Gnome (Pyromaniac) +1 fire only). The target is `SpellDc(scope)` (`All`, `School(…)`,
  `Subschool(…)`, `Energy(…)`, `Metamagic`: with a metamagic feat other than Heighten Spell, or `Value`: the school or
  energy a parameterised feat or mythic feat is held for), `SpellPenetration` or `Concentration`. The amount is
  `Flat(n)`, `HalfMythicRank`, `MythicRank` or `SpellFocusOfAnotherSchool` (Expanded Arsenal); `requires:` a
  prerequisite the character must also meet (Spell Penetration (Mythic): half the rank without Greater Spell
  Penetration, the full rank with it). Only what the guide states without a condition: gear, buffs, abilities
  switched on in a fight (the Aeon's gaze, Demonic Rage) and bonuses that grow with class level (Combat Casting
  Adept, Horn Call, Cold Specialization) stay out, in the caster line's `leaves_out`. Expanded Arsenal copies all the
  Spell Focus feats of one school (the guide: "any bonuses from Spell Focus feats for another school … only the
  bonuses from one school"): Spell Focus, Greater and Mythic, from the school where they add up to most, and nothing
  in a school with Spell Focus feats of its own ("this has no effect"). The real-data test `caster_line` holds the
  Max DC run's Build and its numbers.
- The totals' bonuses (ticket 120), in `bonuses.ron` beside the caster line's. The item may also be a `Background`
  (the user's list: Pickpocket +2 initiative; Farmhand, Lumberjack, Miner and Smith +1 Fortitude, though the guide's
  table shows it for the Farmhand only; Varisian Explorer +1 Reflex; Nexian Scholar and Osirian Historian +2 to their
  Knowledge) or a `MythicAbility`. The target may be `Initiative`, `Save(Fortitude)`, `AllSaves`,
  `Skill("perception")`, `AllSkills`, or `Held`: the skill or stat a parameterised feat is held for (Skill Focus
  (Persuasion), Stat Focus (Initiative); Stat Focus's other stats count nowhere). The amount may also be
  `TenRanks(amount: 3, with_ten_ranks: 6)` (Skill Focus, Alertness, Persuasive and their kin: the higher number with
  10 or more ranks in the skill, on skill targets only) or `AbilityModifier(Charisma)` (Divine Grace; Cunning
  Initiative's Wisdom beside the Dexterity every initiative counts). Every race and subrace has its skill bonuses
  from the guide's racial traits (a subrace's in place of its race's): Keen Senses, Elven Magic, Obsessive,
  Sure-Footed, the heritages' pairs. Bonuses for one use of a skill ("Persuasion when used for diplomacy", for bluff
  or to intimidate, the Andoran Diplomat's diplomacy), against one kind of effect or foe, in one place (underground,
  a favoured terrain), with one weapon, that grow with class level, and the mythic Iron Will, Great Fortitude and
  Lightning Reflexes (rerolls) stay out, in the totals' `leaves_out`. Steps at a class level use `requires`: the
  Duelist's Improved Reaction (+2, +4 at Duelist 8), Powerless Prophecy (+4 from oracle 5). The real-data test
  `totals` holds the Max DC run's initiative, Will and Persuasion, a halfling paladin's saves and Skill Focus at 10
  ranks.
- `interactions.ron` (ticket 124) holds notes on how an item combines with others (`rules::ItemInteractions`):
  `(item: MythicFeat("expanded-arsenal"), interactions: "…")`, the item named as in `bonuses.ron`, at most one note
  per item, in our own words from the saved guide pages, saying "the guide doesn't say" where it doesn't. Only the
  items an agent run showed two summaries weren't enough for (ticket 124's five: Favorable Magic, Persistent Spell,
  Expanded Arsenal, Second Bloodline, Bloodline Ascendance, and the Fey and Infernal bloodlines' arcana against School
  Power); new notes as later runs find them, no sweep. `describe` shows the note. The real-data test `totals` checks
  they're present.
- `gear.ron` (ticket 165; ticket 122's file of that name was removed by ticket 148) holds the gear items
  (`rules::GearItem`, `rules::GearBonus`, `rules::GearTarget`, `rules::BonusType`), one block each, grouped by slot.
  **Which items:** every item on the saved pages `items`, `items-inevitable-excess` and `items-lord-of-nothing` whose
  properties give an always-on bonus to an ability score (70 distinct items: 73 rows, 3 of them repeated on a DLC
  page), plus 22 the user added by hand with no `guide` (the plain stat belts and headbands at +2, +4 and +6, and
  the two three-score ones at +2 and +4; the guide lists the +6 ones itself). Items whose only score change is a
  penalty or holds in a circumstance (Glasses of Undeniable Truth, Bestial Rags, Robe of False Death, Mutilated
  Angel, Nimble Edge), Vellexia's Magnifying Amulet ("+2 to the highest current ability score", which the shape
  cannot hold) and the relics and artifacts (listed in prose with upgrade tiers, not in tables) are out;
  the second stage ([169](../../.scratch/wotr-planner/issues/169-gear-second-stage.md)) takes the items with no score
  bonus. **Key:** the name in lower case, apostrophes dropped, every other run of non-alphanumerics one hyphen.
  **Slot:** the guide's section (`Belt` … `Robe`, `Usable` for the consumables of "Equipable and Consumable": the six
  manuals and tomes, the Tome of Ancient Perfection, the Elixir of Inconceivable Transmutations and the Potion of
  Humanity; the Trusted Friend sits in that section but is a `Necklace`, since the guide's skill table calls it an
  amulet), plus `Armour` and `Weapon` for the six armour and weapon rows that give a score bonus;
  how many of a slot a Build holds, and that a robe and an armour share one body place, is the engine's.
  **Guide:** `page#section`, e.g. `items#belts`, `items#unique-armour`, `items#weapon-properties`,
  `items-lord-of-nothing#weapons`; a row repeated on a DLC page keeps the main page's. **Bonuses:** one per number
  changed, with its type as the guide words it (`kind` left out for none or "untyped"; a penalty is always
  untyped). The targets are an ability score, all saves or one, initiative, one skill, spell DC, caster level, spell
  penetration and concentration. A bonus for one use of a skill ("Persuasion (Bluff)"), against one kind of foe, in
  some circumstance, per day or after an event is an other effect. **Other effects:** short lines in our own words
  (never the guide's), at most about 140 characters; an AC, attack, damage, CMD or speed bonus is one of them until
  the second stage. `GearItem::summary` generates the text under a name ("Str, Dex, Con +6 enhancement"). The
  real-data test `gear` checks the file and three items against their pages.
- Keys are readable text from the guide's anchors: `power-attack` (a feat), `rogue` (a class),
  `rogue/dark-lurker` (an archetype), `rogue/rogue-talents/fast-stealth` (a selection option). A key has no type
  prefix: where it sits says what it names, so keys of two kinds may be equal (the feature and the selection
  `shaman/spirit`). Things a class or path owns start with the owner's key. A key is lowercase letters, digits
  and `-`, in `/`-separated parts (the schema checks this).
- `guide` fields are `page#anchor` links into the guide. Store a short summary in our own words, never the
  guide's text.
- Prerequisites are `All([...])`, `Any([...])` and `AtLeast(n, [...])` over single conditions such as
  `Feat("power-attack")`, `AbilityScore(ability: Strength, score: 13)` or `MythicRank(9)`. A missing
  `prerequisite` means there is none. `Feat("weapon-focus")` is the feat for any value; `FeatFor(feat:
  "weapon-focus", value: Weapon("dueling-sword"))` needs one value (`Skill("mobility")`,
  `SpellSchool(Conjuration)`, `Energy(Fire)`, `ArmourType(Light)`), and `FeatForSameValue("weapon-focus")` the
  value of the pick being checked (Weapon Specialization), only in a parameterised feat's or a chosen-feat option's
  prerequisite. `FeatIfValue(value: ArmourType(Heavy), feat: "armor-proficiency-heavy-armor")` needs the feat only
  when the pick is taken for that value (Armor Focus: the proficiency of its armour type). `Not(...)` holds when
  its one child doesn't (the slayer's combat style talents: `Not(ClassLevel(class: "slayer", level: 6))`). `Story(VowOfDahak)` and `Story(Uncorrupted)` are story conditions: out of scope, they always hold
  and the sheet shows them as a note, so they go in an `All` (`All([MythicRank(9), Story(VowOfDahak)])`), never
  as one choice of an `Any` or `AtLeast` (the check refuses that).
- A `//` comment line is allowed anywhere and is kept for readers only: the Crusader's bonus lists carry one saying
  they skip every prerequisite though the guide skips only BAB and fighter level (the user's decision).

## The snapshot check

The real-data test `snapshots` (`libs/seeds/tests/game_data/snapshots.rs`, ticket 159) is the cheap behaviour check
that runs with every `cargo test`: it says nothing changed in a small corpus of Builds. It runs the current engine
only; the other side is the snapshot files committed in `libs/seeds/tests/game_data/snapshots/`, one per Build, so an
intended change shows up as a snapshot diff in its commit. It has no `allows` battery and no consistency checks:
those stay in the behaviour proof (`scripts/proof.sh`).

- **The corpus:** the six bench Builds (`bench-<name>`, from `tools/proof/bench-links.tsv`), one Build per class of
  `classes/` at level 20 with no picks (`class-<key>`), and one fighter 20 per mythic path with no pick beyond the
  path (`path-<key>`; a path taken only by a switch is switched to from the first path, in key order, it can be).
  Classes and paths come from the data's own lists, so a new one fails the test until it has a snapshot. A
  pick-less Build that's impossible as it is gets the first alignment (with a faith that fits) that makes it
  possible: the paladin is lawful good. Prestige classes have no pick-less Build (their entry needs feats and
  ranks), so none has a Build of its own (the bench's `multi_prestige20` takes the Assassin). Instead the test
  holds the list of prestige classes known to be uncovered, `UNCOVERED_PRESTIGE_CLASSES` (all 13 today), and
  `snapshots_list_every_uncovered_prestige_class` fails when `prestige/` has a class not on it or the list names one
  that's gone: add a Build for the new class to the corpus, or add it to the list.
- **The lines:** each file is `path = value` lines in tree order. The Build start (`start.*`, `mythic.*`); per level
  its class, class level, BAB, saves, scores, skill points, the ranks it adds, each slot (`slot.feat.5.general.0 =
  Feat: power-attack "Power Attack"`, `-` while empty), and the feats and features gained at that level
  (`level.2.feature/fighter/bravery`, with `×2` when the level gives two copies, and again at a later level that
  gives another); per Mythic rank the same (`rank.*`, its status one of `reached`, `partly-dropped`, `dropped`,
  `not-reached`); then the last level's spellcasting,
  totals (`totals.*`) and caster lines (`caster.*`: each school, Evocation for each energy and with metamagic,
  Enchantment for charm and compulsion). The pick-less class and path Builds also list each slot's open options,
  `slot.feat.1.general.0.open = alertness, …`, a parameterised one as `key(value)`.
- **Reading a failure:** the message lists each changed line with the Builds it's in, most Builds first:
  `level.5.saves.will: 4 → 5 in 12 Builds: …`, `+ level.3.feature/x = X`, `- …`. An `.open` list is compared one
  option at a time, so a newly legal feat reads `+ slot.feat.5.general.0.open/power-attack in 36 Builds: …`. A
  second grouping wildcards the level or rank (the path's first number), so one newly legal feat is one line:
  `slot.feat.*.general.0.open/power-attack: + in 36 Builds, 357 slots`. A missing or extra snapshot file is named
  as such, a file that doesn't parse (a merge conflict's markers) by its first bad line, and a file with the same
  lines in another order or with a line twice as "same lines, different order or duplicates".
- **Regenerating:** when every change is intended, `SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`
  rewrites the files (and deletes those of Builds no longer in the corpus) without reading them, so it also
  repairs a file that doesn't parse; commit them with the change.

## Seeding

```sh
cargo run -p seeds --bin seed -- --check-only   # read and check the files only, no database
cargo run -p seeds --bin seed            # this folder (or $GAME_DATA_DIR)
cargo run -p seeds --bin seed -- <dir>   # another folder, e.g. libs/seeds/tests/fixtures/game-data
```

The seed first runs `GameData::check` and refuses data with problems, listing them. `--check-only` stops
there, never touching Postgres, so several editors can check at once; keys that don't exist are only caught by a
real seed (the database's foreign keys). Then it replaces every game
table in one transaction and stores a hash of the `.ron` files. The server loads the game data once at startup,
so restart it after a reseed. It logs a warning at startup when the stored hash doesn't match the files in
`$GAME_DATA_DIR` (default: this folder); it never reseeds on its own.
