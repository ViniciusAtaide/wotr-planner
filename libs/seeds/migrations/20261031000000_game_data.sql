-- Game data: the relational shape of `rules::GameData`.
--
-- An SQLite database (one file). `seeds::replace_game_data` (libs/seeds/src/seed.rs), which the `seed` binary
-- reaches through `seeds::seed`, builds a new file from this migration, loads every table in one
-- transaction and renames the file over the database. Rows are never edited in place; fix the
-- `libs/game-data/*.ron` files and reseed.
--
-- Conventions:
-- - Game entities use readable text keys from the guide's anchors (`power-attack`), with no type
--   prefix: the table says what a key names, so two tables may hold the same key. Keys of things
--   owned by a class, archetype or path start with the owner (`rogue/eldritch-scoundrel`), and
--   `feature`, `feature_selection`, `feat_list` and `selection_option` check it.
-- - No derived values. Order comes from natural keys and `rules::GameData` is made canonical in
--   Rust, so nothing depends on insertion order. Rule-based facts (odd mythic ranks give an
--   ability, the path is chosen at MR 3, which feat categories are combat feats, the rank after a
--   path switch, class level caps, BAB and saves from a class's progressions) live in the `rules`
--   crate, not in columns. Archetypes store only the levels where they differ from their class.
-- - True sets are unique on their natural key. Granted features and selections are multisets stored
--   as counts: one row per (grantor, key) with `count >= 1`, which the loader expands. Feat slots and
--   ability modifiers have a surrogate `id` and no other key, so duplicates can be stored.
--   `class_level` and `mythic_rank` also have an `id`, as the target of their grants' foreign keys,
--   and keep their natural key UNIQUE. `prerequisite_node.id` is what `parent_id` and the owners'
--   `prerequisite_root_id` point at.
-- - A row owned by a class or one of its archetypes has `class_key NOT NULL` and a nullable
--   `archetype_key`, with a composite foreign key to `archetype (key, class_key)`, so an
--   archetype row always names its own class. A row that can belong to one of several unrelated
--   owners has one nullable column per owner and a CHECK that exactly one is set.
-- - Every foreign key is DEFERRABLE INITIALLY DEFERRED, so the seed can insert in any order and
--   the references are checked at commit (sqlx turns SQLite's `foreign_keys` on). Before it commits, the
--   seed runs `PRAGMA foreign_key_check` to report every missing reference, not just that one exists.
-- - Every table is STRICT, so a column holds only its declared type: INTEGER or TEXT. A boolean is an
--   INTEGER, 0 or 1 (CONSTRAINT `boolean`).
-- - Enum values are snake_case text. The ones used in several tables are named after their type: SQLite
--   has no domains, so each column of the type repeats its CHECK under the type's name (below), and a
--   value outside it fails with `CHECK constraint failed: <name>`. Every table-level CHECK is named
--   `<table>_<rule>` for the same reason.
-- - `rules::GameData::check` runs before every seed and covers what constraints can't express
--   (contiguous level rows, archetypes only on base classes, prerequisite group sizes, ...).

-- Named column types ----------------------------------------------------------------------------
--
-- Each is a `CONSTRAINT <name> CHECK (...)` on every column of the type:
-- - `boolean`: 0 or 1.
-- - `u8`: an unsigned game number (`u8` in `rules`), 0 to 255.
-- - `text_key`: a lowercase slug, or slugs joined by `/` after an owner (`rogue/rogue-talents/fast-stealth`):
--   only `a-z`, `0-9`, `-` and `/`, not empty, never starting or ending with `-` or `/`, and no two of them
--   in a row.
-- - `ability`: `rules::Ability`.
-- - `bab_progression`: `rules::BabProgression`: full, ¾ or ½ base attack bonus.
-- - `save_progression`: `rules::SaveProgression`: a good or a poor save.
-- - `mythic_ability_group`: `rules::MythicAbilityGroup`: the guide's headings under "Mythic Abilities".
-- - `mythic_feat_group`: `rules::MythicFeatGroup`: the guide's headings under "Mythic Feats".
-- - `feat_category`: `rules::FeatCategory`.
-- - `spell_school`: `rules::SpellSchool`: the eight schools of magic.
-- - `energy`: `rules::Energy`: the energy types Elemental Focus lists.
-- - `armour_type`: `rules::ArmourType`: the armour types Armor Focus lists.
-- - `parameter_kind`: `rules::ParameterKind`: what a parameterised feat is taken for.
-- - `repeat_kind`: `rules::Repeat` without its number: how many times a feat or an option can be held.
--   `times` carries its number in a `repeat_times` column.
-- - `option_group`: `rules::OptionGroup`: a family of options of which a character holds only one when an
--   option's prerequisite says so (the sorcerer's Draconic and Elemental bloodlines, and the bloodrager's
--   apart).
-- - `level_from`: `rules::LevelFrom`: where an option held through a selection naming no classes gets its
--   option level (the other options held, the granting option, or the Mythic ranks since the pick).
-- - `mythic_extra_pick`: `rules::MythicExtraPick`: what the slot a mythic feat gives offers (Extra Feat: a
--   feat; Extra Mythic Ability: a mythic ability).

-- One row: the hash of the data files the tables were last seeded from.
CREATE TABLE game_data_seed (
    singleton INTEGER NOT NULL PRIMARY KEY DEFAULT 1 CHECK (singleton = 1),
    hash TEXT NOT NULL CHECK (length(hash) = 64 AND hash NOT GLOB '*[^0-9a-f]*'),
    seeded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;

-- progression.ron ------------------------------------------------------------------------------

-- `rules::MythicAscension`: the Character level at which each Mythic rank is reached.
CREATE TABLE mythic_ascension (
    rank INTEGER NOT NULL PRIMARY KEY CHECK (rank BETWEEN 1 AND 10),
    character_level INTEGER NOT NULL CHECK (character_level BETWEEN 1 AND 40)
) STRICT;

-- Prerequisite trees ---------------------------------------------------------------------------

-- One node of a `rules::Prerequisite` tree. A node is a group (`all`, `any`, `at_least`, or `not`
-- with exactly one child) whose children point back at it through `parent_id`, or a single
-- condition that fills exactly its own typed columns. Owning entities point at the root through
-- `prerequisite_root_id`. Only groups have children, a `not` has one, a root has no parent, and
-- every node is in exactly one owner's tree; the loader rejects data that breaks these
-- (`Trees::finish` in libs/seeds/src/prerequisites.rs).
CREATE TABLE prerequisite_node (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN (
        'all', 'any', 'at_least', 'not',
        'feat', 'mythic_feat', 'mythic_ability', 'class_level', 'archetype', 'feature',
        'selection_option', 'skill_ranks', 'ability_score', 'base_attack_bonus', 'spell_level',
        'caster_level', 'character_level', 'mythic_rank', 'alignment', 'race', 'feat_for',
        'feat_for_same_value', 'story', 'no_option_from', 'spontaneous_spell_level', 'feat_if_value',
        'no_other_option_of_group', 'picked_through', 'deity', 'atheist', 'worships', 'deity_offers'
    )),
    -- References to other game entities.
    feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_feat_key TEXT REFERENCES mythic_feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_ability_key TEXT REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT REFERENCES archetype (key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    selection_option_key TEXT REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    skill_key TEXT REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    weapon_key TEXT REFERENCES weapon (key) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    deity_key TEXT REFERENCES deity (key) DEFERRABLE INITIALLY DEFERRED,
    -- Values.
    ability TEXT CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    )),
    spell_type TEXT CHECK (spell_type IN ('arcane', 'divine')),
    alignment TEXT CHECK (alignment IN (
        'lawful_good', 'neutral_good', 'chaotic_good', 'lawful_neutral', 'true_neutral',
        'chaotic_neutral', 'lawful_evil', 'neutral_evil', 'chaotic_evil'
    )),
    amount INTEGER CONSTRAINT u8 CHECK (amount BETWEEN 0 AND 255),
    spell_school TEXT CONSTRAINT spell_school CHECK (spell_school IN (
        'abjuration', 'conjuration', 'divination', 'enchantment', 'evocation', 'illusion', 'necromancy',
        'transmutation'
    )),
    energy TEXT CONSTRAINT energy CHECK (energy IN ('acid', 'cold', 'electricity', 'fire')),
    armour_type TEXT CONSTRAINT armour_type CHECK (armour_type IN ('light', 'medium', 'heavy')),
    -- `rules::Story`: a story condition, out of scope, so it always holds and the sheet shows it as a note.
    story TEXT CHECK (story IN ('vow_of_dahak', 'uncorrupted')),
    option_group TEXT CONSTRAINT option_group CHECK (option_group IN (
        'draconic', 'elemental', 'bloodrager_draconic', 'bloodrager_elemental'
    )),
    -- Each kind fills exactly the columns listed for it, and every other column is NULL. `feat_for`
    -- (`rules::Prerequisite::FeatFor`: a parameterised feat held for one value) and `feat_if_value`
    -- (`rules::Prerequisite::FeatIfValue`: `feat_key` needed when the pick is taken for the value) fill
    -- `feat_key` and exactly one value column: `skill_key`, `weapon_key`, `spell_school`, `energy` or
    -- `armour_type`. `amount` is
    -- `at_least`'s n, the level of `class_level`, `spell_level`, `caster_level` and `character_level`,
    -- `mythic_rank`'s rank, `spontaneous_spell_level`'s level,
    -- `feature`'s count (granted at least that many times), `skill_ranks`' ranks, `ability_score`'s
    -- score and `base_attack_bonus`'s BAB.
    -- `IS`, not `=`: a NULL (a kind the CASE lacks) would pass a CHECK.
    CONSTRAINT prerequisite_node_columns_of_kind CHECK (
        concat_ws(',',
            CASE WHEN feat_key IS NOT NULL THEN 'feat_key' END,
            CASE WHEN mythic_feat_key IS NOT NULL THEN 'mythic_feat_key' END,
            CASE WHEN mythic_ability_key IS NOT NULL THEN 'mythic_ability_key' END,
            CASE WHEN class_key IS NOT NULL THEN 'class_key' END,
            CASE WHEN archetype_key IS NOT NULL THEN 'archetype_key' END,
            CASE WHEN feature_key IS NOT NULL THEN 'feature_key' END,
            CASE WHEN selection_option_key IS NOT NULL THEN 'selection_option_key' END,
            CASE WHEN skill_key IS NOT NULL THEN 'skill_key' END,
            CASE WHEN race_key IS NOT NULL THEN 'race_key' END,
            CASE WHEN weapon_key IS NOT NULL THEN 'weapon_key' END,
            CASE WHEN selection_key IS NOT NULL THEN 'selection_key' END,
            CASE WHEN deity_key IS NOT NULL THEN 'deity_key' END,
            CASE WHEN ability IS NOT NULL THEN 'ability' END,
            CASE WHEN spell_type IS NOT NULL THEN 'spell_type' END,
            CASE WHEN alignment IS NOT NULL THEN 'alignment' END,
            CASE WHEN amount IS NOT NULL THEN 'amount' END,
            CASE WHEN spell_school IS NOT NULL THEN 'spell_school' END,
            CASE WHEN energy IS NOT NULL THEN 'energy' END,
            CASE WHEN armour_type IS NOT NULL THEN 'armour_type' END,
            CASE WHEN story IS NOT NULL THEN 'story' END,
            CASE WHEN option_group IS NOT NULL THEN 'option_group' END
        ) IS CASE kind
            WHEN 'all' THEN ''
            WHEN 'any' THEN ''
            WHEN 'at_least' THEN 'amount'
            WHEN 'not' THEN ''
            WHEN 'feat' THEN 'feat_key'
            WHEN 'mythic_feat' THEN 'mythic_feat_key'
            WHEN 'mythic_ability' THEN 'mythic_ability_key'
            WHEN 'class_level' THEN 'class_key,amount'
            WHEN 'archetype' THEN 'archetype_key'
            WHEN 'feature' THEN 'feature_key,amount'
            WHEN 'selection_option' THEN 'selection_option_key'
            WHEN 'skill_ranks' THEN 'skill_key,amount'
            WHEN 'ability_score' THEN 'ability,amount'
            WHEN 'base_attack_bonus' THEN 'amount'
            WHEN 'spell_level' THEN 'spell_type,amount'
            -- `rules::Prerequisite::SpontaneousSpellLevel`: spell-level access through a spontaneous caster.
            WHEN 'spontaneous_spell_level' THEN 'spell_type,amount'
            -- `rules::Prerequisite::NoOptionFrom`: no option of this selection held.
            WHEN 'no_option_from' THEN 'selection_key'
            -- `rules::Prerequisite::NoOtherOptionOfGroup`: no option of the group held but the one picked.
            WHEN 'no_other_option_of_group' THEN 'option_group'
            -- `rules::Prerequisite::PickedThrough`: the pick fills a slot of this selection.
            WHEN 'picked_through' THEN 'selection_key'
            WHEN 'caster_level' THEN 'amount'
            WHEN 'character_level' THEN 'amount'
            WHEN 'mythic_rank' THEN 'amount'
            WHEN 'alignment' THEN 'alignment'
            WHEN 'race' THEN 'race_key'
            WHEN 'feat_for_same_value' THEN 'feat_key'
            WHEN 'story' THEN 'story'
            -- `rules::Prerequisite::Deity`: the Build follows this deity.
            WHEN 'deity' THEN 'deity_key'
            -- `rules::Prerequisite::Atheist` and `Worships`: atheism; any deity.
            WHEN 'atheist' THEN ''
            WHEN 'worships' THEN ''
            -- `rules::Prerequisite::DeityOffers`: the Build's deity offers this domain.
            WHEN 'deity_offers' THEN 'selection_option_key'
            -- `feat_key` and the one value column that is set; none or two set can't match.
            WHEN 'feat_for' THEN 'feat_key,' || COALESCE(
                CASE WHEN skill_key IS NOT NULL THEN 'skill_key' END,
                CASE WHEN weapon_key IS NOT NULL THEN 'weapon_key' END,
                CASE WHEN spell_school IS NOT NULL THEN 'spell_school' END,
                CASE WHEN energy IS NOT NULL THEN 'energy' END,
                CASE WHEN armour_type IS NOT NULL THEN 'armour_type' END,
                ''
            )
            WHEN 'feat_if_value' THEN 'feat_key,' || COALESCE(
                CASE WHEN skill_key IS NOT NULL THEN 'skill_key' END,
                CASE WHEN weapon_key IS NOT NULL THEN 'weapon_key' END,
                CASE WHEN spell_school IS NOT NULL THEN 'spell_school' END,
                CASE WHEN energy IS NOT NULL THEN 'energy' END,
                CASE WHEN armour_type IS NOT NULL THEN 'armour_type' END,
                ''
            )
        END
    )
) STRICT;

CREATE INDEX prerequisite_node_parent_id ON prerequisite_node (parent_id);

-- races.ron, skills.ron, feats.ron -------------------------------------------------------------

CREATE TABLE skill (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    ability TEXT NOT NULL CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    ))
) STRICT;

CREATE TABLE race (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
) STRICT;

CREATE TABLE subrace (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    race_key TEXT NOT NULL REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
) STRICT;

-- `rules::AbilityModifier` of a race or a subrace. A NULL `ability` is the chosen kind: the player
-- picks the ability at Build start.
CREATE TABLE ability_modifier (
    id INTEGER PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    subrace_key TEXT REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    ability TEXT CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    )),
    amount INTEGER NOT NULL CHECK (amount BETWEEN -128 AND 127),
    CONSTRAINT ability_modifier_one_owner CHECK (((race_key IS NOT NULL) + (subrace_key IS NOT NULL)) = 1)
) STRICT;

-- `rules::Weapon`: a weapon a parameterised feat can be taken for, from the guide's weapon lists.
-- `proficiency` is the proficiency feat whose list names it; NULL for what no list names (unarmed strike,
-- rays).
CREATE TABLE weapon (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    proficiency TEXT CHECK (proficiency IN ('simple', 'martial', 'exotic'))
) STRICT;

-- A feat. `parameter` is what a parameterised feat is taken for (NULL: nothing; each pick of it names a
-- value of this kind). `repeat` is how many times it can be taken: `times` up to `repeat_times`, `per_value`
-- once for each value of its parameter.
CREATE TABLE feat (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    category TEXT NOT NULL CONSTRAINT feat_category CHECK (category IN (
        'weapon', 'ranged', 'unarmed_combat', 'armour_and_shield', 'magic', 'miscellaneous',
        'mounted_combat', 'combat_manoeuvres', 'teamwork', 'skill', 'class', 'racial'
    )),
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    parameter TEXT CONSTRAINT parameter_kind CHECK (parameter IN (
        'skill', 'weapon', 'exotic_weapon', 'spell_school', 'energy', 'armour_type'
    )),
    repeat TEXT NOT NULL CONSTRAINT repeat_kind CHECK (repeat IN ('once', 'times', 'unlimited', 'per_value')),
    repeat_times INTEGER CONSTRAINT u8 CHECK (repeat_times BETWEEN 0 AND 255) CHECK (repeat_times >= 2),
    CONSTRAINT feat_repeat_times_with_times CHECK ((repeat = 'times') = (repeat_times IS NOT NULL)),
    CONSTRAINT feat_per_value_with_parameter CHECK (repeat <> 'per_value' OR parameter IS NOT NULL)
) STRICT;

-- classes/, prestige/ --------------------------------------------------------------------------

-- A base class or a prestige class. BAB and saves are progressions; `rules` computes the numbers
-- (base and prestige classes have different save formulas). `advances` (`rules::Advancement`) is how
-- many classes the rows' spellcasting advancement serves: `each`, one per spell type the rows advance
-- (Mystic Theurge), or `either`, one class, arcane or divine (Loremaster). `rules::GameData::check`
-- gives `either` only to a class whose rows advance both types.
CREATE TABLE class (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    is_prestige INTEGER NOT NULL CONSTRAINT boolean CHECK (is_prestige IN (0, 1)),
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    skill_points INTEGER NOT NULL CONSTRAINT u8 CHECK (skill_points BETWEEN 0 AND 255),
    bab TEXT NOT NULL CONSTRAINT bab_progression CHECK (bab IN ('full', 'three_quarters', 'half')),
    fort TEXT NOT NULL CONSTRAINT save_progression CHECK (fort IN ('good', 'poor')),
    reflex TEXT NOT NULL CONSTRAINT save_progression CHECK (reflex IN ('good', 'poor')),
    will TEXT NOT NULL CONSTRAINT save_progression CHECK (will IN ('good', 'poor')),
    advances TEXT NOT NULL CHECK (advances IN ('each', 'either')),
    -- Its spells are cast spontaneously rather than prepared (`rules::Class::spontaneous`).
    spontaneous INTEGER NOT NULL CONSTRAINT boolean CHECK (spontaneous IN (0, 1)),
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
) STRICT;

-- `rules::CountsAs`: a class's levels count as `as_class_key` levels, toward one thing only:
-- `feat_prerequisites` (Diverse Training: Eldritch Knight levels as fighter levels for a feat's
-- prerequisites) or `feature`, the other class's rows granting `feature_key` (Frost Power: Winter
-- Witch levels stack with witch levels for Major and Grand Hexes). `rules::GameData::check` keeps
-- `feature_key` to one the other class's rows grant.
CREATE TABLE class_counts_as (
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    as_class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    toward TEXT NOT NULL CHECK (toward IN ('feat_prerequisites', 'feature')),
    feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT class_counts_as_feature_with_toward_feature CHECK (
        (toward = 'feature') = (feature_key IS NOT NULL)),
    CONSTRAINT class_counts_as_not_itself CHECK (as_class_key <> class_key)
) STRICT;
CREATE UNIQUE INDEX class_counts_as_unique ON class_counts_as (class_key, as_class_key, toward,
    COALESCE(feature_key, ''));

-- `rules::SaveException`: a save cell the guide prints differently from the class's progression,
-- kept as printed. `rules::GameData::check` keeps the level in the class's table and the value off
-- the formula's.
CREATE TABLE save_exception (
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 40),
    save TEXT NOT NULL CHECK (save IN ('fortitude', 'reflex', 'will')),
    value INTEGER NOT NULL CONSTRAINT u8 CHECK (value BETWEEN 0 AND 255),
    PRIMARY KEY (class_key, level, save)
) STRICT;

CREATE TABLE archetype (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    -- NULL: same as the base class.
    skill_points INTEGER CONSTRAINT u8 CHECK (skill_points BETWEEN 0 AND 255),
    -- FALSE: same class skills as the base class. TRUE: the archetype's `class_skill` rows
    -- replace the base class's (possibly with none).
    overrides_class_skills INTEGER NOT NULL CONSTRAINT boolean CHECK (overrides_class_skills IN (0, 1)),
    -- NULL: casts like its class. Set: spontaneously or not (Eldritch Scion: a spontaneous magus).
    spontaneous INTEGER CONSTRAINT boolean CHECK (spontaneous IN (0, 1)),
    -- NULL: the class's progression for that save. Set: replaces it, and the class's `save_exception`
    -- cells for that save with it (Arcane Enforcer: poor Fortitude, good Will). `rules` computes the
    -- numbers; `rules::GameData::check` keeps a set one off the class's.
    fort TEXT CONSTRAINT save_progression CHECK (fort IN ('good', 'poor')),
    reflex TEXT CONSTRAINT save_progression CHECK (reflex IN ('good', 'poor')),
    will TEXT CONSTRAINT save_progression CHECK (will IN ('good', 'poor')),
    -- The target of the composite foreign keys below: an archetype row names its own class.
    UNIQUE (key, class_key)
) STRICT;

-- backgrounds.ron: the user's list (the guide doesn't list backgrounds, so there's no guide link). Its
-- class skills are `class_skill` rows and its feats `granted_feat` rows. NULL group: the background "None".
CREATE TABLE background (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    -- `rules::BackgroundGroup`.
    background_group TEXT CHECK (background_group IN (
        'craftsman', 'noble', 'oblate', 'regional', 'scholar', 'street_urchin', 'wanderer', 'warrior'
    )),
    -- What it gives that isn't modelled (`rules::Background::note`).
    note TEXT NOT NULL
) STRICT;

-- deities.ron: the guide's deity table (`rules::Deity`). Its alignments, favoured weapons and domains are
-- rows of their own. A Build start follows one or chooses atheism; atheism isn't a row.
CREATE TABLE deity (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    guide TEXT NOT NULL
) STRICT;

-- An alignment a follower of the deity may have.
CREATE TABLE deity_alignment (
    deity_key TEXT NOT NULL REFERENCES deity (key) DEFERRABLE INITIALLY DEFERRED,
    alignment TEXT NOT NULL CHECK (alignment IN (
        'lawful_good', 'neutral_good', 'chaotic_good', 'lawful_neutral', 'true_neutral',
        'chaotic_neutral', 'lawful_evil', 'neutral_evil', 'chaotic_evil'
    )),
    PRIMARY KEY (deity_key, alignment)
) STRICT;

-- A favoured weapon of the deity.
CREATE TABLE deity_weapon (
    deity_key TEXT NOT NULL REFERENCES deity (key) DEFERRABLE INITIALLY DEFERRED,
    weapon_key TEXT NOT NULL REFERENCES weapon (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (deity_key, weapon_key)
) STRICT;

-- A domain the deity offers: an option of the domain selection (`rules::GameData::check` keeps it one).
CREATE TABLE deity_domain (
    deity_key TEXT NOT NULL REFERENCES deity (key) DEFERRABLE INITIALLY DEFERRED,
    domain_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (deity_key, domain_key)
) STRICT;

-- mythic/ --------------------------------------------------------------------------------------

CREATE TABLE mythic_ability (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    group_name TEXT NOT NULL CONSTRAINT mythic_ability_group CHECK (group_name IN (
        'melee', 'ranged', 'magic', 'healing', 'miscellaneous', 'class'
    )),
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
) STRICT;

-- A mythic feat. `parameter` and `repeat` are as on `feat` (Improved Critical (Mythic): a weapon, once per
-- weapon). `removes_racial_penalty`: while it's held, the race's ability penalties don't apply to the
-- character's own scores (Destiny Beyond Birth).
CREATE TABLE mythic_feat (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    group_name TEXT NOT NULL CONSTRAINT mythic_feat_group CHECK (group_name IN (
        'melee', 'ranged', 'armour_and_shield', 'magic', 'miscellaneous', 'combat_manoeuvre'
    )),
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    parameter TEXT CONSTRAINT parameter_kind CHECK (parameter IN (
        'skill', 'weapon', 'exotic_weapon', 'spell_school', 'energy', 'armour_type'
    )),
    repeat TEXT NOT NULL CONSTRAINT repeat_kind CHECK (repeat IN ('once', 'times', 'unlimited', 'per_value')),
    repeat_times INTEGER CONSTRAINT u8 CHECK (repeat_times BETWEEN 0 AND 255) CHECK (repeat_times >= 2),
    removes_racial_penalty INTEGER NOT NULL CONSTRAINT boolean CHECK (removes_racial_penalty IN (0, 1)),
    -- `rules::MythicFeat::extra_pick`: the slot a pick of it gives at its rank. NULL: none.
    extra_pick TEXT CONSTRAINT mythic_extra_pick CHECK (extra_pick IN ('feat', 'mythic_ability')),
    CONSTRAINT mythic_feat_repeat_times_with_times CHECK ((repeat = 'times') = (repeat_times IS NOT NULL)),
    CONSTRAINT mythic_feat_per_value_with_parameter CHECK (repeat <> 'per_value' OR parameter IS NOT NULL)
) STRICT;

CREATE TABLE mythic_path (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
) STRICT;

-- `rules::PathSwitch`: a path a Build can switch to at MR 8. A path has at most one. The switch keeps
-- the general picks (mythic abilities and feats) of MR 1 to `kept_through_rank` and the old path's
-- own grants (its rows' features, the picks in its slots) of MR 1 to `path_kept_through_rank`; the
-- rank after the switch is one above `kept_through_rank` (a rule in `rules`, not a column).
-- `rules::GameData::check` keeps both below the switch rank.
CREATE TABLE path_switch (
    path_key TEXT NOT NULL PRIMARY KEY REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    kept_through_rank INTEGER NOT NULL
        REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    path_kept_through_rank INTEGER NOT NULL
        REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    further_ascensions INTEGER NOT NULL CONSTRAINT boolean CHECK (further_ascensions IN (0, 1)),
    -- The Character level cap after the switch, when it raises the usual 20 (Legend: 40).
    level_cap INTEGER CHECK (level_cap BETWEEN 20 AND 40)
) STRICT;

-- The paths a switch is open from (`rules::PathSwitch::from`); none: any path (Devil: Aeon or Azata).
CREATE TABLE path_switch_from (
    path_key TEXT NOT NULL REFERENCES path_switch (path_key) DEFERRABLE INITIALLY DEFERRED,
    from_path_key TEXT NOT NULL REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (path_key, from_path_key),
    CONSTRAINT path_switch_from_not_its_own_path CHECK (from_path_key <> path_key)
) STRICT;

-- Features and feature selections --------------------------------------------------------------

-- A named ability of a class, archetype or mythic path, such as Sneak Attack. Another owner's rows
-- may grant it (a prestige class granting Sneak Attack).
CREATE TABLE feature (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    -- `rules::Feature::option_row`: the row at `option_row_level` of the option held in
    -- `option_row_selection_key`, which the feature grants (Bloodline Ascendance: the bloodline's 20th). Both or
    -- neither.
    option_row_selection_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    option_row_level INTEGER CHECK (option_row_level BETWEEN 1 AND 40),
    -- The game asks for this choice every day, so it isn't part of a Build (Mixed Bloodline).
    daily_choice INTEGER NOT NULL CONSTRAINT boolean CHECK (daily_choice IN (0, 1)),
    -- For prerequisites while it's held: the base attack bonus is at least `bab_floor`, and with
    -- `skill_ranks_at_least_mythic_rank` every skill's ranks are at least the Mythic rank (Perfect Mind).
    bab_floor INTEGER CONSTRAINT u8 CHECK (bab_floor BETWEEN 0 AND 255),
    skill_ranks_at_least_mythic_rank INTEGER NOT NULL
        CONSTRAINT boolean CHECK (skill_ranks_at_least_mythic_rank IN (0, 1)),
    -- While it's held, proficiency with the Build's deity's favoured weapons (Cleric Proficiencies).
    favoured_weapon_proficiency INTEGER NOT NULL
        CONSTRAINT boolean CHECK (favoured_weapon_proficiency IN (0, 1)),
    CONSTRAINT feature_option_row_both_or_neither CHECK (
        (option_row_selection_key IS NULL) = (option_row_level IS NULL)),
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    -- A class (or one of its archetypes) or a path.
    CONSTRAINT feature_one_owner CHECK (((class_key IS NOT NULL) + (path_key IS NOT NULL)) = 1),
    CONSTRAINT feature_archetype_with_class CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    -- Keyed under its owner: the archetype when set, else the class or the path.
    CONSTRAINT feature_key_under_owner CHECK (
        substr(key, 1, length(COALESCE(archetype_key, class_key, path_key) || '/'))
            = COALESCE(archetype_key, class_key, path_key) || '/')
) STRICT;

-- `rules::AbilityBonus`: what a feature does to the character's own ability scores while it's held. They
-- count toward ability-score prerequisites; gear never does. `fixed` adds `amount` to `ability`, `every`
-- to every score, `highest` to the highest; `at_least` raises every score to `amount`.
CREATE TABLE feature_ability_bonus (
    feature_key TEXT NOT NULL REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN ('fixed', 'every', 'highest', 'at_least')),
    ability TEXT CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    )),
    amount INTEGER NOT NULL CONSTRAINT u8 CHECK (amount BETWEEN 0 AND 255),
    CONSTRAINT feature_ability_bonus_ability_with_fixed CHECK ((kind = 'fixed') = (ability IS NOT NULL))
) STRICT;
CREATE UNIQUE INDEX feature_ability_bonus_unique ON feature_ability_bonus (feature_key, kind,
    COALESCE(ability, ''));

-- The fixed list a Feature selection picks from (rogue talents, a path's picks). An option held through
-- it gets its `option_level` rows as the levels of the classes in `selection_level_class` reach them,
-- `levels_behind` levels late (the Separatist's second domain: 2), except at its
-- `selection_skipped_level`s; with no classes, `level_from` says where its level comes from (the other
-- options held, the option whose row granted the slot, or the Mythic ranks since a path's pick: Life-Bonding
-- Friendship, only on a path's selection; `rules::GameData::check` refuses any of them with classes);
-- `feat_lists_only`: only the option's feat lists and its arcana count (Crossblooded's second bloodline);
-- `only_without_option`: a level grants it only to a character holding none of its options (Dragon
-- Disciple's bloodline); `pool_of_key`: it offers the pools of the options held in that selection too;
-- `once_per_option`: an option it gave once can't be picked through it again (Expanded Element);
-- `deity_domains`: it offers only the options whose domain the Build's deity offers (`offered`: a cleric's
-- domains) or only the others (`not_offered`: the Separatist's second), NULL when the deity doesn't matter.
CREATE TABLE feature_selection (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    feat_lists_only INTEGER NOT NULL CONSTRAINT boolean CHECK (feat_lists_only IN (0, 1)),
    only_without_option INTEGER NOT NULL CONSTRAINT boolean CHECK (only_without_option IN (0, 1)),
    pool_of_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    levels_behind INTEGER NOT NULL CONSTRAINT u8 CHECK (levels_behind BETWEEN 0 AND 255),
    level_from TEXT CONSTRAINT level_from CHECK (
        level_from IN ('other_options', 'granting_option', 'mythic_ranks')),
    -- `rules::FeatureSelection::level_also_from`: an option whose level counts beside the other options
    -- held (Impossible Domain: a Living Deity's way), so only with `other_options`.
    level_also_from_key TEXT REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    once_per_option INTEGER NOT NULL CONSTRAINT boolean CHECK (once_per_option IN (0, 1)),
    deity_domains TEXT CONSTRAINT deity_domains CHECK (deity_domains IN ('offered', 'not_offered')),
    CONSTRAINT feature_selection_pool_of_not_itself CHECK (pool_of_key <> key),
    CONSTRAINT feature_selection_level_also_from_with_other_options CHECK (
        level_also_from_key IS NULL OR level_from = 'other_options'),
    CONSTRAINT feature_selection_mythic_ranks_on_a_path CHECK (
        level_from IS NOT 'mythic_ranks' OR path_key IS NOT NULL),
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT feature_selection_one_owner CHECK (((class_key IS NOT NULL) + (path_key IS NOT NULL)) = 1),
    CONSTRAINT feature_selection_archetype_with_class CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    CONSTRAINT feature_selection_key_under_owner CHECK (
        substr(key, 1, length(COALESCE(archetype_key, class_key, path_key) || '/'))
            = COALESCE(archetype_key, class_key, path_key) || '/')
) STRICT;

-- The classes whose levels add up to the option level of an option held through a selection
-- (`rules::FeatureSelection::option_levels`: a sorcerer's bloodline, sorcerer and Dragon Disciple).
CREATE TABLE selection_level_class (
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (selection_key, class_key)
) STRICT;

-- An option level whose row an option held through the selection doesn't get (Seeker: 3, 9 and 15).
CREATE TABLE selection_skipped_level (
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 40),
    PRIMARY KEY (selection_key, level)
) STRICT;

-- An old path's feature or selection a switch takes away whatever its rank (Devil: the Aeon's gaze
-- picks and Extra Gaze, the Azata's dragon). Exactly one of the two is set.
CREATE TABLE path_switch_drop (
    path_key TEXT NOT NULL REFERENCES path_switch (path_key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT path_switch_drop_feature_or_selection CHECK (
        ((feature_key IS NOT NULL) + (selection_key IS NOT NULL)) = 1)
) STRICT;
CREATE UNIQUE INDEX path_switch_drop_unique ON path_switch_drop (path_key, COALESCE(feature_key, ''),
    COALESCE(selection_key, ''));

-- A feature the switch itself grants, held from the switch on (Legend: its +4 to every score).
CREATE TABLE path_switch_feature (
    path_key TEXT NOT NULL REFERENCES path_switch (path_key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT NOT NULL REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (path_key, feature_key)
) STRICT;

-- One option of a Feature selection. Its key extends the selection's:
-- `rogue/rogue-talents/fast-stealth`. `chosen_feat_key` is a parameterised feat the option grants for the
-- value its pick names (the rogue talent Weapon Focus); `rules::GameData::check` keeps it parameterised.
-- `parameter` is what the pick names when there's no chosen feat (Battle Master: a weapon, for its rows).
-- `repeat` is as on `feat`; `per_value` only with a chosen feat, once for each value of it (the Skill Focus
-- talents, once per skill). `option_group` is the family it belongs to (a Draconic
-- bloodline); `arcana_key` is the one of its rows' features that a lists-only holding still gets
-- (`rules::GameData::check` keeps it among its rows'); `domain_key` is the domain it stands for when it isn't
-- one (a warpriest's blessing), which a deity-limited selection checks.
CREATE TABLE selection_option (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    chosen_feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    parameter TEXT CONSTRAINT parameter_kind CHECK (parameter IN (
        'skill', 'weapon', 'exotic_weapon', 'spell_school', 'energy', 'armour_type'
    )),
    repeat TEXT NOT NULL CONSTRAINT repeat_kind CHECK (repeat IN ('once', 'times', 'unlimited', 'per_value')),
    repeat_times INTEGER CONSTRAINT u8 CHECK (repeat_times BETWEEN 0 AND 255) CHECK (repeat_times >= 2),
    -- The selection whose options a pool selection offers while this option is held (a mystery's revelations).
    pool_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    -- Picking it skips the row, at the level reached, of the option held in this selection (a Primalist's rage
    -- powers instead of the bloodline power).
    replaces_row_of_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    option_group TEXT CONSTRAINT option_group CHECK (option_group IN (
        'draconic', 'elemental', 'bloodrager_draconic', 'bloodrager_elemental'
    )),
    arcana_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    domain_key TEXT REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT selection_option_repeat_times_with_times CHECK (
        (repeat = 'times') = (repeat_times IS NOT NULL)),
    CONSTRAINT selection_option_per_value_with_chosen_feat CHECK (
        repeat <> 'per_value' OR chosen_feat_key IS NOT NULL),
    CONSTRAINT selection_option_parameter_or_chosen_feat CHECK (parameter IS NULL OR chosen_feat_key IS NULL),
    CONSTRAINT selection_option_key_under_selection CHECK (
        substr(key, 1, length(selection_key || '/')) = selection_key || '/')
) STRICT;

-- `rules::SelectionOption::values`: the values an option's pick may name, when not every value of its kind
-- (the Zen Archer's Way of the Bow: a longbow or a shortbow). Exactly one value column is set.
CREATE TABLE selection_option_value (
    option_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    skill_key TEXT REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    weapon_key TEXT REFERENCES weapon (key) DEFERRABLE INITIALLY DEFERRED,
    spell_school TEXT CONSTRAINT spell_school CHECK (spell_school IN (
        'abjuration', 'conjuration', 'divination', 'enchantment', 'evocation', 'illusion', 'necromancy',
        'transmutation'
    )),
    energy TEXT CONSTRAINT energy CHECK (energy IN ('acid', 'cold', 'electricity', 'fire')),
    armour_type TEXT CONSTRAINT armour_type CHECK (armour_type IN ('light', 'medium', 'heavy')),
    CONSTRAINT selection_option_value_one_value CHECK (
        ((skill_key IS NOT NULL) + (weapon_key IS NOT NULL) + (spell_school IS NOT NULL)
            + (energy IS NOT NULL) + (armour_type IS NOT NULL)) = 1)
) STRICT;
CREATE UNIQUE INDEX selection_option_value_unique ON selection_option_value (option_key,
    COALESCE(skill_key, ''), COALESCE(weapon_key, ''), COALESCE(spell_school, ''), COALESCE(energy, ''),
    COALESCE(armour_type, ''));

-- A feat a selection option grants (`rules::SelectionOption::feats`): a rogue talent that is the feat
-- of the same name, or the Azata's All-skilled with every weapon, armour and shield proficiency. A
-- parameterised one is held for every value (All-skilled: Exotic Weapon Proficiency in every exotic weapon).
CREATE TABLE selection_option_feat (
    option_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (option_key, feat_key)
) STRICT;

-- Another selection's option that a selection offers too (`rules::FeatureSelection::shared_options`): the
-- slayer's talents share the rogue's. The option keeps its one key and prerequisite, so a character holds it
-- once whichever selection took it. A selection's own options are its `selection_option` rows, never shared.
CREATE TABLE selection_shared_option (
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    option_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (selection_key, option_key),
    CONSTRAINT selection_shared_option_not_its_own_option CHECK (
        NOT substr(option_key, 1, length(selection_key || '/')) = selection_key || '/')
) STRICT;

-- A selection a pick of the feat gives one more slot of, each time it's taken (`rules::Feat::extra_pick_of`:
-- Extra Rage Power, a rage power; Extra Hex, a witch's or a shaman's hex). The slot is of the one the character
-- had a slot of first, which `rules` works out.
CREATE TABLE feat_extra_pick (
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (feat_key, selection_key)
) STRICT;

-- `rules::FeatList`: a named list of feats a `one_of` feat slot picks from, owned like a feature or by a
-- race (the half-elf's Adaptability).
-- `ignores_prerequisites` is TRUE on a bonus list, its owner's own (the monk's bonus feats): a pick
-- from it skips the feat's prerequisites. `every_feat` is TRUE on a list of every feat (Gold Dragon's
-- Dragon Feats), which has no members of its own: they would copy the `feat` table.
-- `rules::GameData::check` keeps bonus lists to class level rows and their own race's or path's slots, and
-- refuses a list of fewer than two feats (a lone feat is a `granted_feat`) unless it's `every_feat` or
-- its one feat is parameterised (a bonus feat of the player's choosing: the warpriest's Weapon Focus, the
-- pick names the weapon), an `every_feat` list with members, and a list no slot names.
CREATE TABLE feat_list (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    guide TEXT NOT NULL,
    ignores_prerequisites INTEGER NOT NULL CONSTRAINT boolean CHECK (ignores_prerequisites IN (0, 1)),
    every_feat INTEGER NOT NULL CONSTRAINT boolean CHECK (every_feat IN (0, 1)),
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT feat_list_one_owner CHECK (
        ((class_key IS NOT NULL) + (path_key IS NOT NULL) + (race_key IS NOT NULL)) = 1),
    CONSTRAINT feat_list_archetype_with_class CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    CONSTRAINT feat_list_key_under_owner CHECK (
        substr(key, 1, length(COALESCE(archetype_key, class_key, path_key, race_key) || '/'))
            = COALESCE(archetype_key, class_key, path_key, race_key) || '/')
) STRICT;

CREATE TABLE feat_list_member (
    feat_list_key TEXT NOT NULL REFERENCES feat_list (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (feat_list_key, feat_key)
) STRICT;

-- `rules::OptionLevelRow`: what an option gives at one option level. Its features, selections and feats
-- are `granted_*` rows, its feat slots `feat_slot` rows, its chosen feats `option_level_chosen_feat`
-- rows and its class skills `class_skill` rows; `feat_list_key` is the option's feat list from this
-- level on, which an `option_list` feat slot picks from.
CREATE TABLE option_level (
    id INTEGER PRIMARY KEY,
    option_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 40),
    feat_list_key TEXT REFERENCES feat_list (key) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (option_key, level)
) STRICT;

-- The class skills of a class (`archetype_key` NULL), of an archetype that overrides them, of a
-- background, or of an option level (`rules::OptionLevelRow::class_skills`: held with the row).
CREATE TABLE class_skill (
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    background_key TEXT REFERENCES background (key) DEFERRABLE INITIALLY DEFERRED,
    option_level_id INTEGER REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    skill_key TEXT NOT NULL REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT class_skill_one_owner CHECK (
        ((class_key IS NOT NULL) + (background_key IS NOT NULL) + (option_level_id IS NOT NULL)) = 1),
    CONSTRAINT class_skill_archetype_with_class CHECK (archetype_key IS NULL OR class_key IS NOT NULL)
) STRICT;
CREATE UNIQUE INDEX class_skill_unique ON class_skill (COALESCE(class_key, ''), COALESCE(archetype_key, ''),
    COALESCE(background_key, ''), COALESCE(option_level_id, 0), skill_key);

-- Progression rows -----------------------------------------------------------------------------

-- One Class level of a class (`archetype_key` NULL: `rules::ClassLevelRow`) or of one of its
-- archetypes (`rules::ArchetypeLevelRow`). A class's row count is its class level cap. An archetype
-- has rows only at the levels where it differs from its class: its grants replace the class row's
-- in full, and its spell levels replace the class row's when set (NULL: the class row's). Only a
-- class row says whether it advances spellcasting.
CREATE TABLE class_level (
    id INTEGER PRIMARY KEY,
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 40),
    arcane_spell_level INTEGER CHECK (arcane_spell_level BETWEEN 0 AND 9),
    divine_spell_level INTEGER CHECK (divine_spell_level BETWEEN 0 AND 9),
    advances_arcane INTEGER CONSTRAINT boolean CHECK (advances_arcane IN (0, 1)),
    advances_divine INTEGER CONSTRAINT boolean CHECK (advances_divine IN (0, 1)),
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    -- Spell levels are set together, and always on a class row.
    CONSTRAINT class_level_spell_levels_together CHECK (
        (arcane_spell_level IS NULL) = (divine_spell_level IS NULL)),
    CONSTRAINT class_level_spell_levels_on_class_rows CHECK (
        archetype_key IS NOT NULL OR arcane_spell_level IS NOT NULL),
    -- The advances flags are on class rows only.
    CONSTRAINT class_level_advances_arcane_on_class_rows CHECK (
        (archetype_key IS NULL) = (advances_arcane IS NOT NULL)),
    CONSTRAINT class_level_advances_divine_on_class_rows CHECK (
        (archetype_key IS NULL) = (advances_divine IS NOT NULL))
) STRICT;
CREATE UNIQUE INDEX class_level_unique ON class_level (class_key, COALESCE(archetype_key, ''), level);

-- `rules::MythicRankRow`: what one Mythic rank of a path gives beyond the general picks. The rank is
-- one of `mythic_ascension`'s.
CREATE TABLE mythic_rank (
    id INTEGER PRIMARY KEY,
    path_key TEXT NOT NULL REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    rank INTEGER NOT NULL REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (path_key, rank)
) STRICT;

-- A feature granted by a class level, a mythic rank, an option level or a mythic ability, `count` times
-- (Trickster MR 3: Sneak Attack twice). The feature may belong to another class (a prestige class
-- granting Sneak Attack).
CREATE TABLE granted_feature (
    class_level_id INTEGER REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id INTEGER REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    option_level_id INTEGER REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_ability_key TEXT REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT NOT NULL REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    count INTEGER NOT NULL CONSTRAINT u8 CHECK (count BETWEEN 0 AND 255) CHECK (count >= 1),
    CONSTRAINT granted_feature_one_owner CHECK (
        ((class_level_id IS NOT NULL) + (mythic_rank_id IS NOT NULL) + (option_level_id IS NOT NULL)
            + (mythic_ability_key IS NOT NULL)) = 1)
) STRICT;
CREATE UNIQUE INDEX granted_feature_unique ON granted_feature (COALESCE(class_level_id, 0),
    COALESCE(mythic_rank_id, 0), COALESCE(option_level_id, 0), COALESCE(mythic_ability_key, ''), feature_key);

-- `count` Feature selection slots granted by a class level, a mythic rank, an option level or a mythic
-- ability (Second Bloodline).
CREATE TABLE granted_selection (
    class_level_id INTEGER REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id INTEGER REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    option_level_id INTEGER REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_ability_key TEXT REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    count INTEGER NOT NULL CONSTRAINT u8 CHECK (count BETWEEN 0 AND 255) CHECK (count >= 1),
    CONSTRAINT granted_selection_one_owner CHECK (
        ((class_level_id IS NOT NULL) + (mythic_rank_id IS NOT NULL) + (option_level_id IS NOT NULL)
            + (mythic_ability_key IS NOT NULL)) = 1)
) STRICT;
CREATE UNIQUE INDEX granted_selection_unique ON granted_selection (COALESCE(class_level_id, 0),
    COALESCE(mythic_rank_id, 0), COALESCE(option_level_id, 0), COALESCE(mythic_ability_key, ''),
    selection_key);

-- A feat a class level, an option level or a background grants outright (`rules::ClassLevelRow::feats`):
-- held with no slot and no pick, whatever its prerequisites; a parameterised one for every value, or, on a
-- class level or a background, for the one value set (`rules::ClassLevelRow::feats_for`: Weapon Focus
-- (Sawtooth Saber); the Aldori Swordsman's Exotic Weapon Proficiency (Dueling Sword)). At most one value
-- column is set.
CREATE TABLE granted_feat (
    class_level_id INTEGER REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    option_level_id INTEGER REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    background_key TEXT REFERENCES background (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    skill_key TEXT REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    weapon_key TEXT REFERENCES weapon (key) DEFERRABLE INITIALLY DEFERRED,
    spell_school TEXT CONSTRAINT spell_school CHECK (spell_school IN (
        'abjuration', 'conjuration', 'divination', 'enchantment', 'evocation', 'illusion', 'necromancy',
        'transmutation'
    )),
    energy TEXT CONSTRAINT energy CHECK (energy IN ('acid', 'cold', 'electricity', 'fire')),
    armour_type TEXT CONSTRAINT armour_type CHECK (armour_type IN ('light', 'medium', 'heavy')),
    CONSTRAINT granted_feat_one_owner CHECK (
        ((class_level_id IS NOT NULL) + (option_level_id IS NOT NULL) + (background_key IS NOT NULL)) = 1),
    CONSTRAINT granted_feat_at_most_one_value CHECK (
        ((skill_key IS NOT NULL) + (weapon_key IS NOT NULL) + (spell_school IS NOT NULL)
            + (energy IS NOT NULL) + (armour_type IS NOT NULL)) <= 1),
    CONSTRAINT granted_feat_no_value_on_option_levels CHECK (option_level_id IS NULL
        OR ((skill_key IS NOT NULL) + (weapon_key IS NOT NULL) + (spell_school IS NOT NULL)
            + (energy IS NOT NULL) + (armour_type IS NOT NULL)) = 0)
) STRICT;
CREATE UNIQUE INDEX granted_feat_unique ON granted_feat (COALESCE(class_level_id, 0),
    COALESCE(option_level_id, 0), COALESCE(background_key, ''), feat_key, COALESCE(skill_key, ''),
    COALESCE(weapon_key, ''), COALESCE(spell_school, ''), COALESCE(energy, ''), COALESCE(armour_type, ''));

-- `rules::OptionLevelRow::chosen_feats`: a parameterised feat an option level grants for the value
-- the option's pick named (Weapon Mastery: Improved Critical in the weapon of its Weapon Focus).
CREATE TABLE option_level_chosen_feat (
    option_level_id INTEGER NOT NULL REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (option_level_id, feat_key)
) STRICT;

-- `rules::FeatSlot` granted by a race, a class level, a mythic rank or an option level. A `one_of`
-- slot names the list it picks from; an `option_list` slot the selection whose held options' lists it
-- picks from.
CREATE TABLE feat_slot (
    id INTEGER PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    -- A subrace's slots replace its race's (`rules::Subrace::feat_slots`).
    subrace_key TEXT REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    class_level_id INTEGER REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id INTEGER REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    option_level_id INTEGER REFERENCES option_level (id) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN ('general', 'combat_feat', 'category', 'one_of', 'option_list')),
    category TEXT CONSTRAINT feat_category CHECK (category IN (
        'weapon', 'ranged', 'unarmed_combat', 'armour_and_shield', 'magic', 'miscellaneous',
        'mounted_combat', 'combat_manoeuvres', 'teamwork', 'skill', 'class', 'racial'
    )),
    feat_list_key TEXT REFERENCES feat_list (key) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    -- An option list slot's fixed option level (`rules::FeatSlot::OptionList::at_level`): a slayer's combat
    -- style window's stage. NULL: the option's own level.
    at_level INTEGER CONSTRAINT u8 CHECK (at_level BETWEEN 0 AND 255),
    -- An option list slot's own label on the sheet (`rules::FeatSlot::OptionList::label`): the kineticist's
    -- bonus-feat wild talent's "Wild talent feat". NULL: its selection's name and "feat".
    label TEXT CHECK (label <> ''),
    CONSTRAINT feat_slot_one_owner CHECK (
        ((race_key IS NOT NULL) + (subrace_key IS NOT NULL) + (class_level_id IS NOT NULL)
            + (mythic_rank_id IS NOT NULL) + (option_level_id IS NOT NULL)) = 1),
    CONSTRAINT feat_slot_category_with_category_kind CHECK ((kind = 'category') = (category IS NOT NULL)),
    CONSTRAINT feat_slot_feat_list_with_one_of CHECK ((kind = 'one_of') = (feat_list_key IS NOT NULL)),
    CONSTRAINT feat_slot_selection_with_option_list CHECK (
        (kind = 'option_list') = (selection_key IS NOT NULL)),
    CONSTRAINT feat_slot_at_level_with_option_list CHECK (at_level IS NULL OR kind = 'option_list'),
    CONSTRAINT feat_slot_label_with_option_list CHECK (label IS NULL OR kind = 'option_list')
) STRICT;

