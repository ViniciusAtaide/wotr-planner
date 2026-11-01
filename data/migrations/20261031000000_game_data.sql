-- Game data: the relational shape of `rules::GameData`.
--
-- `data::replace_game_data` (data/src/seed.rs), which the `seed` binary reaches through
-- `data::seed`, truncates and reloads every table here in one transaction. Rows are never edited in
-- place; fix the `game-data/*.ron` files and reseed.
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
--   the references are checked at commit.
-- - Enum values are snake_case text. The ones used in several tables are domains.
-- - `rules::GameData::check` runs before every seed and covers what constraints can't express
--   (contiguous level rows, archetypes only on base classes, prerequisite group sizes, ...).

-- Domains ---------------------------------------------------------------------------------------

-- An unsigned game number (`u8` in `rules`).
CREATE DOMAIN u8 AS SMALLINT CHECK (VALUE BETWEEN 0 AND 255);

-- A text key: a lowercase slug, or slugs joined by `/` after an owner (`rogue/rogue-talents/fast-stealth`).
CREATE DOMAIN text_key AS TEXT CHECK (VALUE ~ '^[a-z0-9]+(-[a-z0-9]+)*(/[a-z0-9]+(-[a-z0-9]+)*)*$');

-- `rules::Ability`.
CREATE DOMAIN ability AS TEXT CHECK (VALUE IN (
    'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
));

-- `rules::BabProgression`: full, ¾ or ½ base attack bonus.
CREATE DOMAIN bab_progression AS TEXT CHECK (VALUE IN ('full', 'three_quarters', 'half'));

-- `rules::SaveProgression`: a good or a poor save.
CREATE DOMAIN save_progression AS TEXT CHECK (VALUE IN ('good', 'poor'));

-- `rules::MythicAbilityGroup`: the guide's headings under "Mythic Abilities".
CREATE DOMAIN mythic_ability_group AS TEXT CHECK (VALUE IN (
    'melee', 'ranged', 'magic', 'healing', 'miscellaneous', 'class'
));

-- `rules::MythicFeatGroup`: the guide's headings under "Mythic Feats".
CREATE DOMAIN mythic_feat_group AS TEXT CHECK (VALUE IN (
    'melee', 'ranged', 'armour_and_shield', 'magic', 'miscellaneous', 'combat_manoeuvre'
));

-- `rules::FeatCategory`.
CREATE DOMAIN feat_category AS TEXT CHECK (VALUE IN (
    'weapon', 'ranged', 'unarmed_combat', 'armour_and_shield', 'magic', 'miscellaneous',
    'mounted_combat', 'combat_manoeuvres', 'teamwork', 'skill', 'class', 'racial'
));

-- One row: the hash of the data files the tables were last seeded from.
CREATE TABLE game_data_seed (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    hash TEXT NOT NULL CHECK (hash ~ '^[0-9a-f]{64}$'),
    seeded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- progression.ron ------------------------------------------------------------------------------

-- `rules::MythicAscension`: the Character level at which each Mythic rank is reached.
CREATE TABLE mythic_ascension (
    rank SMALLINT PRIMARY KEY CHECK (rank BETWEEN 1 AND 10),
    character_level SMALLINT NOT NULL CHECK (character_level BETWEEN 1 AND 40)
);

-- Prerequisite trees ---------------------------------------------------------------------------

-- One node of a `rules::Prerequisite` tree. A node is a group (`all`, `any`, `at_least`) whose
-- children point back at it through `parent_id`, or a single condition that fills exactly its
-- own typed columns. Owning entities point at the root through `prerequisite_root_id`. Only
-- groups have children, a root has no parent, and every node is in exactly one owner's tree; the
-- loader rejects data that breaks these (`Trees::finish` in data/src/prerequisites.rs).
CREATE TABLE prerequisite_node (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    parent_id BIGINT REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN (
        'all', 'any', 'at_least',
        'feat', 'mythic_feat', 'mythic_ability', 'class_level', 'archetype', 'feature',
        'selection_option', 'skill_ranks', 'ability_score', 'base_attack_bonus', 'spell_level',
        'caster_level', 'character_level', 'alignment', 'race'
    )),
    -- References to other game entities (foreign keys added below, once the tables exist).
    feat_key TEXT,
    mythic_feat_key TEXT,
    mythic_ability_key TEXT,
    class_key TEXT,
    archetype_key TEXT,
    feature_key TEXT,
    selection_option_key TEXT,
    skill_key TEXT,
    race_key TEXT,
    -- Values.
    ability ability,
    spell_type TEXT CHECK (spell_type IN ('arcane', 'divine')),
    alignment TEXT CHECK (alignment IN (
        'lawful_good', 'neutral_good', 'chaotic_good', 'lawful_neutral', 'true_neutral',
        'chaotic_neutral', 'lawful_evil', 'neutral_evil', 'chaotic_evil'
    )),
    amount u8,
    -- Each kind fills exactly the columns listed for it, and every other column is NULL. `amount` is
    -- `at_least`'s n, the level of `class_level`, `spell_level`, `caster_level` and `character_level`,
    -- `feature`'s count (granted at least that many times), `skill_ranks`' ranks, `ability_score`'s
    -- score and `base_attack_bonus`'s BAB.
    CHECK (
        array_remove(ARRAY[
            CASE WHEN feat_key IS NOT NULL THEN 'feat_key' END,
            CASE WHEN mythic_feat_key IS NOT NULL THEN 'mythic_feat_key' END,
            CASE WHEN mythic_ability_key IS NOT NULL THEN 'mythic_ability_key' END,
            CASE WHEN class_key IS NOT NULL THEN 'class_key' END,
            CASE WHEN archetype_key IS NOT NULL THEN 'archetype_key' END,
            CASE WHEN feature_key IS NOT NULL THEN 'feature_key' END,
            CASE WHEN selection_option_key IS NOT NULL THEN 'selection_option_key' END,
            CASE WHEN skill_key IS NOT NULL THEN 'skill_key' END,
            CASE WHEN race_key IS NOT NULL THEN 'race_key' END,
            CASE WHEN ability IS NOT NULL THEN 'ability' END,
            CASE WHEN spell_type IS NOT NULL THEN 'spell_type' END,
            CASE WHEN alignment IS NOT NULL THEN 'alignment' END,
            CASE WHEN amount IS NOT NULL THEN 'amount' END
        ], NULL) = CASE kind
            WHEN 'all' THEN '{}'::TEXT[]
            WHEN 'any' THEN '{}'
            WHEN 'at_least' THEN '{amount}'
            WHEN 'feat' THEN '{feat_key}'
            WHEN 'mythic_feat' THEN '{mythic_feat_key}'
            WHEN 'mythic_ability' THEN '{mythic_ability_key}'
            WHEN 'class_level' THEN '{class_key, amount}'
            WHEN 'archetype' THEN '{archetype_key}'
            WHEN 'feature' THEN '{feature_key, amount}'
            WHEN 'selection_option' THEN '{selection_option_key}'
            WHEN 'skill_ranks' THEN '{skill_key, amount}'
            WHEN 'ability_score' THEN '{ability, amount}'
            WHEN 'base_attack_bonus' THEN '{amount}'
            WHEN 'spell_level' THEN '{spell_type, amount}'
            WHEN 'caster_level' THEN '{amount}'
            WHEN 'character_level' THEN '{amount}'
            WHEN 'alignment' THEN '{alignment}'
            WHEN 'race' THEN '{race_key}'
        END
    )
);

CREATE INDEX prerequisite_node_parent_id ON prerequisite_node (parent_id);

-- races.ron, skills.ron, feats.ron -------------------------------------------------------------

CREATE TABLE skill (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    ability ability NOT NULL
);

CREATE TABLE race (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
);

CREATE TABLE subrace (
    key text_key PRIMARY KEY,
    race_key TEXT NOT NULL REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
);

-- `rules::AbilityModifier` of a race or a subrace. A NULL `ability` is the chosen kind: the player
-- picks the ability at Build start.
CREATE TABLE ability_modifier (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    subrace_key TEXT REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    ability ability,
    amount SMALLINT NOT NULL CHECK (amount BETWEEN -128 AND 127),
    CHECK (num_nonnulls(race_key, subrace_key) = 1)
);

CREATE TABLE feat (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    category feat_category NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

-- classes/, prestige/ --------------------------------------------------------------------------

-- A base class or a prestige class. BAB and saves are progressions; `rules` computes the numbers
-- (base and prestige classes have different save formulas).
CREATE TABLE class (
    key text_key PRIMARY KEY,
    is_prestige BOOLEAN NOT NULL,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    skill_points u8 NOT NULL,
    bab bab_progression NOT NULL,
    fort save_progression NOT NULL,
    reflex save_progression NOT NULL,
    will save_progression NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

-- `rules::SaveException`: a save cell the guide prints differently from the class's progression,
-- kept as printed. `rules::GameData::check` keeps the level in the class's table and the value off
-- the formula's.
CREATE TABLE save_exception (
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    level SMALLINT NOT NULL CHECK (level BETWEEN 1 AND 40),
    save TEXT NOT NULL CHECK (save IN ('fortitude', 'reflex', 'will')),
    value u8 NOT NULL,
    PRIMARY KEY (class_key, level, save)
);

CREATE TABLE archetype (
    key text_key PRIMARY KEY,
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    -- NULL: same as the base class.
    skill_points u8,
    -- FALSE: same class skills as the base class. TRUE: the archetype's `class_skill` rows
    -- replace the base class's (possibly with none).
    overrides_class_skills BOOLEAN NOT NULL,
    -- The target of the composite foreign keys below: an archetype row names its own class.
    UNIQUE (key, class_key)
);

-- The class skills of a class (`archetype_key` NULL), or of an archetype that overrides them.
CREATE TABLE class_skill (
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    skill_key TEXT NOT NULL REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE NULLS NOT DISTINCT (class_key, archetype_key, skill_key)
);

-- mythic/ --------------------------------------------------------------------------------------

CREATE TABLE mythic_ability (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    group_name mythic_ability_group NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE mythic_feat (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    group_name mythic_feat_group NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE mythic_path (
    key text_key PRIMARY KEY,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
);

-- `rules::PathSwitch`: a path a Build can switch to at MR 8. A path has at most one.
CREATE TABLE path_switch (
    path_key TEXT PRIMARY KEY REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    further_ascensions BOOLEAN NOT NULL,
    -- The Character level cap after the switch, when it raises the usual 20 (Legend: 40).
    level_cap SMALLINT CHECK (level_cap BETWEEN 20 AND 40)
);

-- The earlier Mythic ranks a switch keeps. The rank after the switch is one above the highest. Each
-- rank is one of `mythic_ascension`'s.
CREATE TABLE path_switch_kept_rank (
    path_key TEXT NOT NULL REFERENCES path_switch (path_key) DEFERRABLE INITIALLY DEFERRED,
    rank SMALLINT NOT NULL REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (path_key, rank)
);

-- Features and feature selections --------------------------------------------------------------

-- A named ability of a class, archetype or mythic path, such as Sneak Attack. Another owner's rows
-- may grant it (a prestige class granting Sneak Attack).
CREATE TABLE feature (
    key text_key PRIMARY KEY,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    -- A class (or one of its archetypes) or a path.
    CHECK (num_nonnulls(class_key, path_key) = 1),
    CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    -- Keyed under its owner: the archetype when set, else the class or the path.
    CHECK (starts_with(key, COALESCE(archetype_key, class_key, path_key) || '/'))
);

-- The fixed list a Feature selection picks from (rogue talents, a path's picks).
CREATE TABLE feature_selection (
    key text_key PRIMARY KEY,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(class_key, path_key) = 1),
    CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    CHECK (starts_with(key, COALESCE(archetype_key, class_key, path_key) || '/'))
);

-- One option of a Feature selection. Its key extends the selection's:
-- `rogue/rogue-talents/fast-stealth`.
CREATE TABLE selection_option (
    key text_key PRIMARY KEY,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    CHECK (starts_with(key, selection_key || '/'))
);

-- Another selection's option that a selection offers too (`rules::FeatureSelection::shared_options`): the
-- slayer's talents share the rogue's. The option keeps its one key and prerequisite, so a character holds it
-- once whichever selection took it. A selection's own options are its `selection_option` rows, never shared.
CREATE TABLE selection_shared_option (
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    option_key TEXT NOT NULL REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (selection_key, option_key),
    CHECK (NOT starts_with(option_key, selection_key || '/'))
);

-- `rules::FeatList`: a named list of feats a `one_of` feat slot picks from, owned like a feature.
-- `ignores_prerequisites` is TRUE on a bonus list, a class's own (the monk's bonus feats): a pick from
-- it skips the feat's prerequisites. `rules::GameData::check` keeps those to class level rows, and
-- refuses a list of fewer than two feats (a lone feat is a `granted_feat`) and one no slot names.
CREATE TABLE feat_list (
    key text_key PRIMARY KEY,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    guide TEXT NOT NULL,
    ignores_prerequisites BOOLEAN NOT NULL,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(class_key, path_key) = 1),
    CHECK (archetype_key IS NULL OR class_key IS NOT NULL),
    CHECK (starts_with(key, COALESCE(archetype_key, class_key, path_key) || '/'))
);

CREATE TABLE feat_list_member (
    feat_list_key TEXT NOT NULL REFERENCES feat_list (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (feat_list_key, feat_key)
);

-- Progression rows -----------------------------------------------------------------------------

-- One Class level of a class (`archetype_key` NULL: `rules::ClassLevelRow`) or of one of its
-- archetypes (`rules::ArchetypeLevelRow`). A class's row count is its class level cap. An archetype
-- has rows only at the levels where it differs from its class: its grants replace the class row's
-- in full, and its spell levels replace the class row's when set (NULL: the class row's). Only a
-- class row says whether it advances spellcasting.
CREATE TABLE class_level (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    level SMALLINT NOT NULL CHECK (level BETWEEN 1 AND 40),
    arcane_spell_level SMALLINT CHECK (arcane_spell_level BETWEEN 0 AND 9),
    divine_spell_level SMALLINT CHECK (divine_spell_level BETWEEN 0 AND 9),
    advances_arcane BOOLEAN,
    advances_divine BOOLEAN,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE NULLS NOT DISTINCT (class_key, archetype_key, level),
    -- Spell levels are set together, and always on a class row.
    CHECK ((arcane_spell_level IS NULL) = (divine_spell_level IS NULL)),
    CHECK (archetype_key IS NOT NULL OR arcane_spell_level IS NOT NULL),
    -- The advances flags are on class rows only.
    CHECK ((archetype_key IS NULL) = (advances_arcane IS NOT NULL)),
    CHECK ((archetype_key IS NULL) = (advances_divine IS NOT NULL))
);

-- `rules::MythicRankRow`: what one Mythic rank of a path gives beyond the general picks. The rank is
-- one of `mythic_ascension`'s.
CREATE TABLE mythic_rank (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    path_key TEXT NOT NULL REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    rank SMALLINT NOT NULL REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (path_key, rank)
);

-- A feature granted by a class level or a mythic rank, `count` times (Trickster MR 3: Sneak Attack
-- twice). The feature may belong to another class (a prestige class granting Sneak Attack).
CREATE TABLE granted_feature (
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT NOT NULL REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    count u8 NOT NULL CHECK (count >= 1),
    CHECK (num_nonnulls(class_level_id, mythic_rank_id) = 1),
    UNIQUE NULLS NOT DISTINCT (class_level_id, mythic_rank_id, feature_key)
);

-- `count` Feature selection slots granted by a class level or a mythic rank.
CREATE TABLE granted_selection (
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    count u8 NOT NULL CHECK (count >= 1),
    CHECK (num_nonnulls(class_level_id, mythic_rank_id) = 1),
    UNIQUE NULLS NOT DISTINCT (class_level_id, mythic_rank_id, selection_key)
);

-- A feat a class level grants outright (`rules::ClassLevelRow::feats`): held with no slot and no
-- pick, whatever its prerequisites.
CREATE TABLE granted_feat (
    class_level_id BIGINT NOT NULL REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (class_level_id, feat_key)
);

-- `rules::FeatSlot` granted by a race, a class level or a mythic rank. A `one_of` slot names the
-- list it picks from.
CREATE TABLE feat_slot (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN ('general', 'combat_feat', 'category', 'one_of')),
    category feat_category,
    feat_list_key TEXT REFERENCES feat_list (key) DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(race_key, class_level_id, mythic_rank_id) = 1),
    CHECK ((kind = 'category') = (category IS NOT NULL)),
    CHECK ((kind = 'one_of') = (feat_list_key IS NOT NULL))
);

-- Prerequisite references, now that every referenced table exists. ----------------------------

ALTER TABLE prerequisite_node
    ADD FOREIGN KEY (feat_key) REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (mythic_feat_key) REFERENCES mythic_feat (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (mythic_ability_key) REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (class_key) REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (archetype_key) REFERENCES archetype (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (feature_key) REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (selection_option_key) REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (skill_key) REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    ADD FOREIGN KEY (race_key) REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED;
