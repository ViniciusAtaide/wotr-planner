-- Ticket 121: the spell catalogue, `rules::Spell` and `rules::SpellList`. Reference data: no Build choice names a
-- spell, and a spell has no level of its own (it differs by list); a list's entries give it.
--
-- Named column types (as in the first migration):
-- - `spell_descriptor`: `rules::SpellDescriptor`: the descriptors the guide's spell blocks name, the enchantment
--   subschools (compulsion, charmed) and summoning among them.
-- - `casting_time`: `rules::CastingTime`.
-- - `spell_level`: a level on a spell list, 0 to 10 (`rules::GameData::check` keeps a class's or an archetype's
--   list to 9; a Mythic spellbook runs to 10).

-- `rules::Spell`, from its block in the guide. `school` NULL: Universalist (Dismiss Spell), the guide's school for
-- a spell of no school. `components`, `duration`: as printed, NULL where the block has no such row.
-- `spell_resistance`: the block's ✓ (its N/A is 0).
CREATE TABLE spell (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    school TEXT CONSTRAINT spell_school CHECK (school IN (
        'abjuration', 'conjuration', 'divination', 'enchantment', 'evocation', 'illusion', 'necromancy',
        'transmutation'
    )),
    guide TEXT NOT NULL,
    summary TEXT NOT NULL,
    components TEXT,
    target TEXT NOT NULL,
    duration TEXT,
    saving_throw TEXT NOT NULL,
    spell_resistance INTEGER NOT NULL CONSTRAINT boolean CHECK (spell_resistance IN (0, 1)),
    casting_time TEXT NOT NULL CONSTRAINT casting_time CHECK (casting_time IN (
        'standard', 'full_round', 'move', 'swift', 'free'
    ))
) STRICT;

-- A descriptor of the spell (`rules::Spell::descriptors`, a set).
CREATE TABLE spell_descriptor (
    spell_key TEXT NOT NULL REFERENCES spell (key) DEFERRABLE INITIALLY DEFERRED,
    descriptor TEXT NOT NULL CONSTRAINT spell_descriptor CHECK (descriptor IN (
        'acid', 'channel_negative_energy_harm', 'chaos', 'charmed', 'cold', 'compulsion', 'cure', 'curse',
        'death', 'disease', 'divine', 'electricity', 'emotion', 'evil', 'fear', 'fire', 'force', 'good', 'law',
        'mind_affecting', 'movement_impairing', 'negative_emotion', 'negative_level', 'poison', 'polymorph',
        'sight', 'sleep', 'sonic', 'stunned', 'summoning'
    )),
    PRIMARY KEY (spell_key, descriptor)
) STRICT;

-- `rules::SpellList`: owned by a class, an archetype or a Mythic path (exactly one), and keyed by it, or under it
-- for the owner's second list (`angel/merged`).
CREATE TABLE spell_list (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL,
    guide TEXT NOT NULL,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT REFERENCES archetype (key) DEFERRABLE INITIALLY DEFERRED,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT spell_list_one_owner CHECK (
        ((class_key IS NOT NULL) + (archetype_key IS NOT NULL) + (path_key IS NOT NULL)) = 1),
    CONSTRAINT spell_list_key_under_owner CHECK (
        key = COALESCE(class_key, archetype_key, path_key)
        OR substr(key, 1, length(COALESCE(class_key, archetype_key, path_key) || '/'))
            = COALESCE(class_key, archetype_key, path_key) || '/')
) STRICT;

-- Another class, archetype or path that casts from the list (`rules::SpellList::used_by`): exactly one of them.
CREATE TABLE spell_list_user (
    list_key TEXT NOT NULL REFERENCES spell_list (key) DEFERRABLE INITIALLY DEFERRED,
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    archetype_key TEXT REFERENCES archetype (key) DEFERRABLE INITIALLY DEFERRED,
    path_key TEXT REFERENCES mythic_path (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT spell_list_user_one_caster CHECK (
        ((class_key IS NOT NULL) + (archetype_key IS NOT NULL) + (path_key IS NOT NULL)) = 1)
) STRICT;
CREATE UNIQUE INDEX spell_list_user_unique ON spell_list_user (list_key,
    COALESCE(class_key, ''), COALESCE(archetype_key, ''), COALESCE(path_key, ''));

-- A spell on a list at a level (`rules::SpellListEntry`, a set). `only_archetype_key` or `only_story` (at most
-- one): the table's footnote limiting the entry to an archetype ("Flame Dancer only") or a path's story branch.
CREATE TABLE spell_list_entry (
    list_key TEXT NOT NULL REFERENCES spell_list (key) DEFERRABLE INITIALLY DEFERRED,
    spell_key TEXT NOT NULL REFERENCES spell (key) DEFERRABLE INITIALLY DEFERRED,
    level INTEGER NOT NULL CONSTRAINT spell_level CHECK (level BETWEEN 0 AND 10),
    only_archetype_key TEXT REFERENCES archetype (key) DEFERRABLE INITIALLY DEFERRED,
    only_story TEXT CHECK (only_story IN ('vow_of_dahak', 'uncorrupted')),
    CONSTRAINT spell_list_entry_one_limit CHECK (
        ((only_archetype_key IS NOT NULL) + (only_story IS NOT NULL)) <= 1)
) STRICT;
CREATE UNIQUE INDEX spell_list_entry_unique ON spell_list_entry (list_key, spell_key, level,
    COALESCE(only_archetype_key, ''), COALESCE(only_story, ''));
