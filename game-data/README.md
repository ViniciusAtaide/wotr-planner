# Game data

The planner's game data, as RON files that deserialise straight into the `rules` crate types
(`rules/src/game_data.rs`). These files are the source of truth: fix data by editing them, then reseed.

The files were extracted once from the saved guide pages (`cargo run -p extract -- --force`, which never
overwrites `.ron` files). `EXTRACTION_REPORT.md` lists, by category, what the extractor couldn't turn into data;
fix an item in the `.ron` files and delete its line. `data/tests/fixtures/game-data/` is a small hand-written
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
cargo run -p data --bin seed -- --check-only   # read and check the files only, no database
cargo run -p data --bin seed            # this folder (or $GAME_DATA_DIR)
cargo run -p data --bin seed -- <dir>   # another folder, e.g. data/tests/fixtures/game-data
```

The seed first runs `GameData::check` and refuses data with problems, listing them. `--check-only` stops
there, never touching Postgres, so several editors can check at once; keys that don't exist are only caught by a
real seed (the database's foreign keys). Then it replaces every game
table in one transaction and stores a hash of the `.ron` files. The server loads the game data once at startup,
so restart it after a reseed. It logs a warning at startup when the stored hash doesn't match the files in
`$GAME_DATA_DIR` (default: this folder); it never reseeds on its own.
