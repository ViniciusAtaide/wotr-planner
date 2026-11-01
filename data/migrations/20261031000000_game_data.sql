-- Game data: the relational shape of `rules::GameData`.
--
-- `data::replace_game_data` (data/src/seed.rs), which the `seed` binary reaches through
-- `data::seed`, truncates and reloads every table here in one transaction. Rows are never edited in
-- place; fix the `game-data/*.ron` files and reseed.
--
-- Conventions:
-- - Game entities use readable text keys from the guide's anchors (`feat:power-attack`).
-- - No derived values. Order comes from natural keys and `rules::GameData` is made canonical in
--   Rust, so nothing depends on insertion order. Rule-based facts (odd mythic ranks give an
--   ability, the path is chosen at MR 3, which feat categories are combat feats, the rank after a
--   path switch, class level caps) live in the `rules` crate, not in columns.
-- - True sets are unique on their natural key. Multisets (granted features and selections, feat
--   slots, ability modifiers) have a surrogate `id` and no other key, so duplicates can be stored.
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

-- `rules::Ability`.
CREATE DOMAIN ability AS TEXT CHECK (VALUE IN (
    'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
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
    key TEXT PRIMARY KEY CHECK (key LIKE 'skill:%'),
    name TEXT NOT NULL,
    ability ability NOT NULL
);

CREATE TABLE race (
    key TEXT PRIMARY KEY CHECK (key LIKE 'race:%'),
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL
);

CREATE TABLE subrace (
    key TEXT PRIMARY KEY CHECK (key LIKE 'subrace:%'),
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
    key TEXT PRIMARY KEY CHECK (key LIKE 'feat:%'),
    name TEXT NOT NULL,
    category feat_category NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

-- classes/, prestige/ --------------------------------------------------------------------------

-- A base class or a prestige class.
CREATE TABLE class (
    key TEXT PRIMARY KEY CHECK (key LIKE 'class:%'),
    is_prestige BOOLEAN NOT NULL,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    skill_points u8 NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE archetype (
    key TEXT PRIMARY KEY CHECK (key LIKE 'archetype:%'),
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
    key TEXT PRIMARY KEY CHECK (key LIKE 'mythic-ability:%'),
    name TEXT NOT NULL,
    group_name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE mythic_feat (
    key TEXT PRIMARY KEY CHECK (key LIKE 'mythic-feat:%'),
    name TEXT NOT NULL,
    group_name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE mythic_path (
    key TEXT PRIMARY KEY CHECK (key LIKE 'path:%'),
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

-- A named ability of a class, archetype or mythic path, such as Sneak Attack.
CREATE TABLE feature (
    key TEXT PRIMARY KEY CHECK (key LIKE 'feature:%'),
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
    CHECK (archetype_key IS NULL OR class_key IS NOT NULL)
);

-- The fixed list a Feature selection picks from (rogue talents, a path's picks).
CREATE TABLE feature_selection (
    key TEXT PRIMARY KEY CHECK (key LIKE 'selection:%'),
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(class_key, path_key) = 1),
    CHECK (archetype_key IS NULL OR class_key IS NOT NULL)
);

-- One option of a Feature selection. Its key extends the selection's:
-- `selection:rogue/rogue-talents/fast-stealth`.
CREATE TABLE selection_option (
    key TEXT PRIMARY KEY,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    guide TEXT NOT NULL,
    feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    prerequisite_root_id BIGINT UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    CHECK (starts_with(key, selection_key || '/'))
);

-- Progression rows -----------------------------------------------------------------------------

-- `rules::ClassLevelRow`: one Class level of a class (`archetype_key` NULL) or of one of its
-- archetypes. An archetype's rows replace its base class's in full. A table's row count is the
-- class level cap.
CREATE TABLE class_level (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    class_key TEXT NOT NULL REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT,
    level SMALLINT NOT NULL CHECK (level BETWEEN 1 AND 40),
    bab u8 NOT NULL,
    fort u8 NOT NULL,
    reflex u8 NOT NULL,
    will u8 NOT NULL,
    arcane_spell_level SMALLINT NOT NULL CHECK (arcane_spell_level BETWEEN 0 AND 9),
    divine_spell_level SMALLINT NOT NULL CHECK (divine_spell_level BETWEEN 0 AND 9),
    advances_arcane BOOLEAN NOT NULL,
    advances_divine BOOLEAN NOT NULL,
    FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key, class_key)
        DEFERRABLE INITIALLY DEFERRED,
    UNIQUE NULLS NOT DISTINCT (class_key, archetype_key, level)
);

-- `rules::MythicRankRow`: what one Mythic rank of a path gives beyond the general picks. The rank is
-- one of `mythic_ascension`'s.
CREATE TABLE mythic_rank (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    path_key TEXT NOT NULL REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    rank SMALLINT NOT NULL REFERENCES mythic_ascension (rank) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE (path_key, rank)
);

-- A feature granted by a class level or a mythic rank. The feature may belong to another class
-- (a prestige class granting Sneak Attack).
CREATE TABLE granted_feature (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT NOT NULL REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(class_level_id, mythic_rank_id) = 1)
);

-- One Feature selection slot granted by a class level or a mythic rank.
CREATE TABLE granted_selection (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    selection_key TEXT NOT NULL REFERENCES feature_selection (key) DEFERRABLE INITIALLY DEFERRED,
    CHECK (num_nonnulls(class_level_id, mythic_rank_id) = 1)
);

-- `rules::FeatSlot` granted by a race, a class level or a mythic rank.
CREATE TABLE feat_slot (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    class_level_id BIGINT REFERENCES class_level (id) DEFERRABLE INITIALLY DEFERRED,
    mythic_rank_id BIGINT REFERENCES mythic_rank (id) DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK (kind IN ('general', 'combat_feat', 'category', 'one_of')),
    category feat_category,
    CHECK (num_nonnulls(race_key, class_level_id, mythic_rank_id) = 1),
    CHECK ((kind = 'category') = (category IS NOT NULL))
);

-- The feats a `one_of` feat slot accepts.
CREATE TABLE feat_slot_option (
    feat_slot_id BIGINT NOT NULL REFERENCES feat_slot (id) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT NOT NULL REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (feat_slot_id, feat_key)
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
