-- Ticket 125: three more kinds of value a pick names (`parameter_kind`: `rules::ParameterKind`), and a mythic
-- ability that takes one.
--
-- - `metamagic`: a metamagic feat (Favorite Metamagic). Which feats are metamagic is a rule on the key in
--   `rules` (`Feat::is_metamagic`), not a column.
-- - `stat`: `rules::Stat`, a stat Stat Focus lists.
-- - `element`: `rules::Element`, an element Ascendant Element lists.
--
-- Only picks name these values: no prerequisite, option value or granted feat does (`rules::GameData::check`:
-- `PickOnlyValue`), so the value columns stay as they are. `feat` takes the new kinds (Stat Focus); the database
-- is built from scratch, so it is recreated with the wider CHECK. `mythic_feat.parameter` and
-- `selection_option.parameter` keep the first six kinds until a mythic feat or an option takes one of these.

CREATE TABLE feat_new (
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
        'skill', 'weapon', 'exotic_weapon', 'spell_school', 'energy', 'armour_type', 'metamagic', 'stat',
        'element'
    )),
    repeat TEXT NOT NULL CONSTRAINT repeat_kind CHECK (repeat IN ('once', 'times', 'unlimited', 'per_value')),
    repeat_times INTEGER CONSTRAINT u8 CHECK (repeat_times BETWEEN 0 AND 255) CHECK (repeat_times >= 2),
    CONSTRAINT feat_repeat_times_with_times CHECK ((repeat = 'times') = (repeat_times IS NOT NULL)),
    CONSTRAINT feat_per_value_with_parameter CHECK (repeat <> 'per_value' OR parameter IS NOT NULL)
) STRICT;
INSERT INTO feat_new SELECT * FROM feat;
DROP TABLE feat;
ALTER TABLE feat_new RENAME TO feat;

-- What a mythic ability is taken for (NULL: nothing; each pick of it names a value of this kind, or none yet). An
-- ability is held once, whatever its value, so it has no `repeat`.
ALTER TABLE mythic_ability ADD COLUMN parameter TEXT CONSTRAINT parameter_kind CHECK (parameter IN (
    'skill', 'weapon', 'exotic_weapon', 'spell_school', 'energy', 'armour_type', 'metamagic', 'stat', 'element'
));
