-- Ticket 172: an option repeats once per value of its own too (Finesse Training: once per weapon), not only of a
-- chosen feat (the Skill Focus talents: once per skill), as `rules::GameData::check` has it
-- (`PerValueOptionWithoutValue`). `selection_option`'s `per_value` CHECK widens to an option with a `parameter`; the
-- database is built from scratch, so the table is recreated with it, as ticket 125 recreated `feat`.

CREATE TABLE selection_option_new (
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
    -- Once per value counts a value: its chosen feat's, or its own.
    CONSTRAINT selection_option_per_value_with_a_value CHECK (
        repeat <> 'per_value' OR chosen_feat_key IS NOT NULL OR parameter IS NOT NULL),
    CONSTRAINT selection_option_parameter_or_chosen_feat CHECK (parameter IS NULL OR chosen_feat_key IS NULL),
    CONSTRAINT selection_option_key_under_selection CHECK (
        substr(key, 1, length(selection_key || '/')) = selection_key || '/')
) STRICT;
INSERT INTO selection_option_new SELECT * FROM selection_option;
DROP TABLE selection_option;
ALTER TABLE selection_option_new RENAME TO selection_option;
