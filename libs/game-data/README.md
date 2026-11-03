# Game data

The planner's game data, as RON files that deserialise straight into the `rules` crate types
(`libs/rules/src/game_data.rs`). These files are the source of truth: fix data by editing them, then reseed.

The files were extracted once from the saved guide pages (`cargo run -p extract -- --force`, which never
overwrites `.ron` files). `EXTRACTION_REPORT.md` lists, by category, what the extractor couldn't turn into data;
fix an item in the `.ron` files and delete its line. `libs/seeds/tests/fixtures/game-data/` is a small hand-written
example of the layout.

## Layout

| File | Holds |
| --- | --- |
| `progression.ron` | `Progression`: the mythic rank table (the Character level of each Mythic rank) |
| `races.ron` | `Vec<Race>`, with subraces and the race's own feat lists (the half-elf's Adaptability) |
| `skills.ron` | `Vec<Skill>` |
| `weapons.ron` | `Vec<Weapon>`: the weapons a feat can be taken for, from the guide's simple, martial and exotic lists (`proficiency`; the Sawtooth Saber is exotic, from Weapon Finesse's list), plus what Weapon Focus names besides weapons (unarmed strike, touch attacks, rays), each natural attack the guide names (bite, claw, gore, hoof, slam, spike, tail, talon, wing), Kinetic Blast and Bomb |
| `feats.ron` | `Vec<Feat>` |
| `backgrounds.ron` | `Vec<Background>`: the user's list (the guide doesn't list backgrounds, so they have no guide link), carried by `extract::backgrounds` |
| `classes/<class>.ron` | one base `Class`, with its features, feature selections, level rows and archetypes |
| `prestige/<class>.ron` | one prestige `Class` |
| `mythic/abilities.ron` | `Vec<MythicAbility>` |
| `mythic/feats.ron` | `Vec<MythicFeat>` |
| `mythic/paths/<path>.ron` | one `MythicPath`: its rank table, picks and switch rule (what it keeps, drops and grants, the paths it's open from, the character level cap it sets) |

Any other `.ron` file is an error; other files (like this README) are ignored. Order means nothing anywhere
in the data: catalogs are sorted by key, class levels by level, mythic ranks by rank, and the children of
`All`/`Any`/`AtLeast` are a set, so write entries in whatever order reads best.

What follows from a rule is code in the `rules` crate, not data: each mythic rank's general pick (odd ranks a
mythic ability, even ranks a mythic feat), the path choice at MR 3 and the switch at MR 8, which feat categories
are Combat feats (`FeatCategory::is_combat`), the skill rank cap (the Character level), choosing the path at MR 3
granting its MR 1–2 rows (held from MR 3 on), the rank after a path switch (one above `kept_through_rank`), each
class's level cap (its number of level rows; an archetype has its
class's), and BAB and saves (from the class's progressions).

- A class's level rows run `level: 1`, `2`, … with none missing; what taking a class grants goes in row 1. Rows
  hold no BAB or saves: the class names its progressions (`bab: Full`/`ThreeQuarters`/`Half`, and `fort`, `reflex`,
  `will`: `Good`/`Poor`) and the rules compute the numbers, with the prestige save formula for prestige classes. A
  save cell the guide prints differently goes in `save_exceptions: [(level: 20, save: Will, value: 20)]`, exactly as
  printed; the check refuses one the progression gives anyway.
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
  slot is checked after the other selections'. `extract::option_levels` reads all this from the guide.
- Ticket 33 (`extract::option_follow_ups`, the user's decisions on tickets 30 and 31):
  - An option may belong to a group, `group: Draconic` (the sorcerer's ten dragons; `Elemental`, its four
    elements), and `NoOtherOptionOfGroup(Draconic)` in its prerequisite needs no option of the group held but the
    one picked (a Crossblooded or Second Bloodline sorcerer can't hold two dragons).
  - A selection naming no classes may take its option level from elsewhere: `level_from: OtherOptions` (the
    highest level of the other options held that it offers: Impossible Domain's `cleric/impossible-domain`, Second
    Mystery's `oracle/second-mystery`, so a Living Deity's run on arcanist levels) or `level_from:
    GrantingOption` (the level of the option whose row granted the slot: Maneuver Mastery's maneuver follows its
    revelation).
  - `once_per_option: true`: an option the selection gave once can't be picked through it again
    (`kineticist/expanded-element`: the 15th-level element isn't the 7th-level one).
  - `PickedThrough("warpriest/champion-of-the-faith/chosen-alignment")` in an option's prerequisite holds when the
    pick fills that selection's slot: the four alignment blessings are `Any([Not(PickedThrough(...)),
    Alignment(...), ...])`, the Champion's own alignment only when he picks them as his chosen alignment.
  - Daily choices also mark the Unsworn Shaman's Minor Spirit and Wandering Spirit 2, and the hunter's Animal
    Focus and Second Animal Focus (features, not a pick of the eight aspects). Elemental Movement is one feature
    per element (none for Earth). The Divine Hunter skips no domain level. The Signifer's order is
    `hellknight-signifer/order`, sharing the Hellknight's seven.
- Ticket 35 (`extract::second_follow_ups`, the user's decisions on tickets 32 and 33):
  - A subrace's `feat_slots` replace its race's, as its ability modifiers do (the guide lists each subrace's own
    traits): none written means none, so the half-elf's Dual Heritage and Kindred Raised get no Adaptability. They
    may name the race's lists.
  - `level_also_from: "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity"` beside `level_from:
    OtherOptions` (`cleric/impossible-domain`): that option's level counts as one of the other options', so a Living
    Deity's Impossible Domain runs on arcanist levels from the pick. Impossible Domain's prerequisite also accepts
    `SelectionOption("hellknight/disciplines/pentamic-faith")`, whose domain is levelled by Hellknight levels.
    It names no druid (ticket 36): the druid's page says a druid who chooses a domain can't take Impossible Domain,
    so neither the Nature Bond domain nor the Blight Druid's or Winter Child's qualifies; the module asserts this.
  - `hellknight-signifer/order` is `only_without_option`, as `hellknight/order`.
  - The Chaos, Evil, Good and Law domains: `Any([Not(PickedThrough("cleric/separatist/domain")), Alignment(...),
    ...])`, the six alignments that don't conflict, when taken as the Separatist's second domain.
  - `group: BloodragerDraconic` and `BloodragerElemental` with `NoOtherOptionOfGroup(...)` on the bloodrager's ten
    dragons and four elements: one of each at most (Second Bloodrager Bloodline), apart from the sorcerer's
    groups.
- The second group of options with rows (ticket 31, `extract::option_families`, read from the guide's prose):
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
- Options' class skills (ticket 37, `extract::option_class_skills`, the user's decision on ticket 34's first open
  question): the "Class Skill: …" features of the bloodlines, mysteries, domains and elements, and the cavalier
  orders' "Skills", became their rows' `class_skills`; the Empyreal and Sylvan Sorcerers' went (their own lists
  hold the skill). Ticket 38 (`option_class_skills::cleanup`, the user's decisions on ticket 37's open questions)
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
- Ticket 43 (`extract::report_gaps`, the report items the saved pages settle): the totem rage powers are exclusive
  ('Selecting one "Totem" power closes off the other categories'), so each of the twelve (Beast, Celestial, Daemon,
  Fiend: Lesser, plain, Greater) has a `Not(SelectionOption(...))` for each of the other lines' nine in its `All`, one
  `Not` per power so that a lock names the power held ("Needs not Beast Totem, Lesser"); the shaman's Wandering Hex (6,
  14), "temporarily filled" from his main spirit or his current Wandering Spirit, is a `daily_choice`, as the
  Wandering Spirit is; the Eldritch Scoundrel's sneak attack dice are at 1, 3, 7, 11, 15 and 19, as its section lists
  them.
- Ticket 44 (`extract::report_gaps::decisions`, the user's decisions on ticket 43's questions): every sorcerer's
  level-1 "Bonus Feat" is an `OptionList(selection: "sorcerer/bloodline")` slot, as at 7, 13 and 19: the base
  sorcerer's (new), Crossblooded's (either bloodline's list), the Empyreal, Sage and Sylvan Sorcerers' (their fixed
  bloodline's), the Nine Tailed Heir's and the Seeker's (the bloodline chosen "for the purpose of bonus feats"; the
  Seeker's magic feats stay at 5, 10 and 15). The Geomancer ("does not gain bloodline spells or bloodline feats")
  keeps its `General` slot; the Overwhelming Mage ("Does not receive bonus feats") has none. Extra Slayer Talent
  needs `ClassLevel(class: "slayer", level: 2)`, the level the slayer's table first gives Slayer Talents ("Talented
  Slayer" is on no saved page). Deities stay out of scope.
- Ticket 45 (`extract::wild_talents`, ticket 43's decision 2): the kineticist's wild talents are a selection,
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
- Ticket 47 (`extract::talent_follow_ups`, the user's decisions on ticket 45's questions, 2026-11-02): the Skill Focus
  wild talent and rogue talent are `repeat: PerValue`, once for each value of their chosen feat (an option is
  `PerValue` only with a `chosen_feat`): Skill Focus in another skill each time. The rogue talent Weapon Focus stays
  once. Combat Trick ("Any combat feat that the character qualifies for") is `repeat: Unlimited` with a row 1 of
  `feat_slots: [CombatFeat]`, a bonus Combat feat slot each time it's taken, its feat's prerequisites checked; the
  selections offering it are levelled by their class (`option_levels: ["rogue"]` on the rogue talents, Fey Tricks
  and Discovery; the slayer's talents by the slayer already). An `OptionList` slot may carry its own `label`: the
  bonus-feat wild talent's reads "Wild talent feat" (`OptionList(selection: "kineticist/element", label: "Wild
  talent feat")`).
- Ticket 49 (`extract::extra_picks`, ticket 45's decision 2, user, 2026-11-02): a feat whose guide entry says "gain
  one additional …" names the selections its "Prerequisites" names in `extra_pick_of` (`extra-hex`: `["shaman/hex",
  "witch/hex"]`), and each pick of it gives one more slot of the one the character had a slot of first, at the
  level it's taken (a `FeatSelection` slot); a character who had a slot of none gets none. Eight feats: Extra
  Arcana, Extra Arcanist Exploit, Extra Discovery (the alchemist's and the Underground Chemist's), Extra Hex, Extra
  Rage Power, Extra Rogue Talent (the rogue's and the Sylvan Trickster's Fey Tricks), Extra Slayer Talent and Extra
  Wild Talent. Extra Mercy says it too, but the paladin's mercies were a feature, not a selection, so it named none
  until ticket 54.
- Ticket 54 (`extract::mercies_and_exploits`, ticket 49's decisions 1 and 5, user, 2026-11-02): the paladin's
  "Mercy" feature is the selection `paladin/mercy` on the rows that held it (each archetype's where its table has
  "Mercy": the Divine Hunter's 3, 9, 12, 15, the Stonelord's 6, 12, 18), with the guide's 15 mercies, each needing
  `ClassLevel(paladin, its level)` and the mercy its table names (Exhausted: Fatigued). Extra Mercy needs a mercy held
  (an `Any` of the 15) and has `extra_pick_of: ["paladin/mercy"]`. The Arcane Enforcer's "Arcanist Exploits" feature
  is a slot of the arcanist's own `arcanist/arcanist-exploits` at its rows 1, 4, 7, 10, 13, 16 and 19 (the guide gives
  it no list of its own; a row may hold another class's selection, as a bloodrager's rage powers), so an exploit is
  taken once across both classes and Extra Arcanist Exploit's pick reaches it; Extra Arcanist Exploit's `Any` no
  longer names the feature. The Enforcer's "Greater Exploits" stayed a feature until ticket 57.
- Ticket 57 (`extract::exploits`, ticket 54's decisions 1 and 2, user, 2026-11-02): the Arcane Enforcer's "Greater
  Exploits" feature at 13 is a slot of the arcanist's own `arcanist/greater-exploits`, read as the arcanist's own
  "Greater Exploits" cell at 11 is (one pick; each greater exploit needs the exploit it extends). The Exploiter
  Wizard's "Exploiter Exploits" feature is a slot of `arcanist/arcanist-exploits` at its rows 1, 5, 9, 13 and 17 (the
  guide gives it no list of its own), and its "Greater Exploits" a slot of `arcanist/greater-exploits` at 13; both
  features are gone. An exploit is taken once across the arcanist, the Enforcer and the Exploiter, and Extra Arcanist
  Exploit's pick reaches the Exploiter too.
- Ticket 58 (`extract::ki_powers`, ticket 54's decision 2, user, 2026-11-02): the monk's "Ki Power" feature is the
  selection `monk/ki-power` on the rows that held it (each archetype's where its table has "Ki Power": the Scaled
  Fist not at 12, the Sohei from 6, the Student of Stone not at 6 or 12, the Zen Archer not at 10), with the guide's
  14 ki powers, each needing `ClassLevel(monk, its level)`; a power with no section of its own links to the page. The
  Traditional Monk picks none: its fixed powers ("Ki Power: Wholeness of Body" at 6, Abundant Step at 8, Diamond Body
  at 10, Diamond Soul at 12, Quivering Palm at 14) are features of its own on rows 6 to 14, in place of the base
  class's pick (those rows were the base's before). No feat gives a ki power: Extra Ki and Abundant Ki Pool grow the
  pool.
- Ticket 59 (`extract::infusions`, ticket 54's decision 2, user, 2026-11-02): the kineticist's "Infusion" feature
  after 1st level is the selection `kineticist/infusions` ("Infusion") on the rows that held it ("She gains new
  Infusions at 3rd, 5th, 9th, 11th, 13th, 17th and 19th levels"; each archetype's where its table has "Infusion":
  the Blood Kineticist's 3, 13, 17, 19, the Kinetic Knight's 5, 11, 13, 17, 19), with the guide's 32 infusions. Each
  needs `ClassLevel(kineticist, twice its level)`, one of its elements (`SelectionOption`, an `Any` for several; none
  for Universal) and one of the options its "Prerequisites" names. The Kinetic Sharpshooter's four own infusions
  need `Archetype("kineticist/kinetic-sharpshooter")`, the six it "cannot use" `Not(Archetype(...))`; its
  "Infusion (Kinetic Sharpshooter)" feature is gone (its rows 3 to 19 hold the slot; 17 and 19 are the base's now).
  The 1st-level "Infusion" stays a feature: no infusion is open at kineticist 1. The guide's "Dazziing Infusion" is
  named "Dazzling Infusion", linking to the guide's anchor. No feat gives an infusion.
- Ticket 60 (`extract::borrowed_selections`, ticket 54's decision 2, user, 2026-11-02): picks from another class's
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
  Hexcrafter through `witch/hex`. Extra Discovery and Extra Arcana give the Vivisectionist and the Hexcrafter none.
- Ticket 61 (`extract::bonus_feat_choices`, ticket 54's decision 2, user, 2026-11-02): a bonus feat chosen from a
  fixed list is a `OneOf` slot of that list where the table has the cell, as the Divine Guardian's bonus feats are,
  with no feature standing for it:
  - the Titan Fighter's "Massive Attacks" (2, 6, 10, 14, 18, in place of the fighter's bonus combat feat) picks from
    its own `fighter/titan-fighter/bonus-feats-2`, `-6` and `-10`: the ranger's Two-Weapon Combat Style feats by
    then (Double Slice, Shield Bash, Two-Weapon Fighting; Improved Two-Weapon Fighting from 6; Greater Two-Weapon
    Fighting from 10), without the extras every ranger style adds, ignoring prerequisites as the ranger's do;
  - the hunter's Precise Companion (2, all but the Forester), the Urban Hunter's Captor (6, 9, 12, 15, 18) and the
    Stormwalker's Archery (2, 6, 10, 14, 18) were slots of their lists already (ticket 18); the features beside them
    went.
- Ticket 55 (`extract::mythic_extra_picks`, ticket 49's decision 4, user, 2026-11-02): a mythic feat's `extra_pick`
  is the slot a pick of it gives at the Mythic rank it's taken, after the rank's general pick (a `MythicExtra`
  slot): `Feat`, a feat slot offering what the general slot does (`extra-feat`: "You can select any normal feat"),
  or `MythicAbility`, a mythic ability slot offering what a rank's general pick does (`extra-mythic-ability`). Both
  are taken once: their entries don't say otherwise. A mythic ability in that slot grants its `selections` as the
  general pick's would (Second Bloodline), and a feat with an `extra_pick_of` gives its slot at the rank's level.
- Ticket 62 (`extract::mythic_ability_picks`, ticket 55's decisions 1 and 2, user, 2026-11-02): two more mythic
  abilities grant a selection slot through `selections`, as Second Spirit and Second Mystery do. Witch Wandering Hex
  ("You can select any hex for which you meet the prerequisites") gives one slot of `witch/hex`, each hex's own
  prerequisites checked, and needs a hex held, the witch's or the shaman's: its "Prequisites: Hex" (so spelt in the
  guide, which the extraction passed over) is Extra Hex's `Any`. Beneficial Curse ("You gain an additional Curse")
  gives one slot of `oracle/oracles-curse`; a curse held is "Already taken" there.
- Ticket 63 (`extract::archetype_alignments`, ticket 58's question 1, user, 2026-11-02): an archetype's alignment
  requirement replaces its class's, and the class's other conditions still hold (`rules` does this; the Drunken
  Master's "Cannot be Lawful" replaces the monk's "Lawful"). So a class's or an archetype's alignment requirement is a
  condition of its own: the whole `prerequisite`, or one of a top-level `All`'s conditions, never mixed into a group
  with others (the check refuses that). Seven archetypes have one: the Drunken Master, Bloodseeker, Angelfire
  Apostle, Reformed Fiend and Prophet of Pestilence from their guide lines; the Priest of Balance's `Any([LN, TN,
  CN])` from its "Alignement restricted to …" line (so spelt in the guide, which the extraction passed over); and the
  **Mantis Zealot's `Any([LN, LE, NE])`, the user's fact, not the guide's** (2026-11-02: one step from Achaekek's
  Lawful Evil; the guide says only that it worships Achaekek, and deities are out of scope).
- A path's switch rule: `switch: (from: ["aeon", "azata"], kept_through_rank: 7, path_kept_through_rank: 7,
  dropped_features: [...], dropped_selections: [...], features: [...], further_ascensions: true, level_cap:
  Some(40))`. The general picks (mythic abilities and feats) of MR 1 to `kept_through_rank` stay, and the old
  path's own grants (its rows' features, the picks in its slots) of MR 1 to `path_kept_through_rank`; the
  `dropped_*` lists take named grants of the old path away whatever their rank; `features` are granted on
  switching (Legend's +4). `from` left out: open from any path.
- A feature that changes the character's own ability scores lists `ability_bonuses`: `Fixed(ability: Strength,
  amount: 4)`, `Every(amount: 4)` (Legend), `Highest(amount: 4)` (the highest score) and `AtLeast(score: 14)`
  (every score at least). They count toward ability-score prerequisites while the feature is held; gear never
  does. A feature can also set floors that prerequisites see while it's held: `bab_floor: Some(15)` and
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
