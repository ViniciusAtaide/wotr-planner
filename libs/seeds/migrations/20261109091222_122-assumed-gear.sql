-- Ticket 122: the guide's caster level and spell DC items, which an agent may assume on top of a Build's numbers.
--
-- `gear_item` (`rules::GearItem`, `gear.ron`): an item of the guide's two "Caster Level and Spell DC Boosting Items"
-- tables on its `items` page, with its group heading (`slot`, `rules::GearSlot`) and the guide page it's on (`guide`).
-- No Build choice names one, and no prerequisite reads one.
CREATE TABLE gear_item (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL CHECK (name <> ''),
    slot TEXT NOT NULL CHECK (slot IN (
        'dagger', 'quarterstaff', 'scimitar', 'estoc', 'bracers', 'cloak', 'glasses', 'gloves', 'headwear',
        'necklace', 'ring', 'robe', 'rod'
    )),
    guide TEXT NOT NULL CHECK (guide <> '')
) STRICT;

-- `rules::GearBonus`: one bonus of an item. `target` is `rules::GearTarget`. `circumstance` is
-- `rules::GearCircumstance`: `spells` with a `scope` (`rules::SpellScope` without `value`, its school, subschool or
-- energy in their own column), `save` with its `save`, `mind_affecting`, or `class` with its `class_key`.
CREATE TABLE gear_bonus (
    id INTEGER PRIMARY KEY,
    gear_item_key TEXT NOT NULL REFERENCES gear_item (key) DEFERRABLE INITIALLY DEFERRED,
    target TEXT NOT NULL CHECK (target IN ('spell_dc', 'caster_level', 'spell_penetration', 'concentration')),
    amount INTEGER NOT NULL CHECK (amount BETWEEN 1 AND 127),
    circumstance TEXT NOT NULL CHECK (circumstance IN ('spells', 'save', 'mind_affecting', 'class')),
    scope TEXT CHECK (scope IN ('all', 'school', 'subschool', 'energy', 'metamagic')),
    spell_school TEXT CONSTRAINT spell_school CHECK (spell_school IN (
        'abjuration', 'conjuration', 'divination', 'enchantment', 'evocation', 'illusion', 'necromancy',
        'transmutation'
    )),
    subschool TEXT CHECK (subschool IN (
        'calling', 'creation', 'healing', 'summoning', 'teleportation', 'scrying', 'charm', 'compulsion',
        'figment', 'glamer', 'pattern', 'phantasm', 'shadow', 'polymorph'
    )),
    energy TEXT CONSTRAINT energy CHECK (energy IN ('acid', 'cold', 'electricity', 'fire')),
    save TEXT CONSTRAINT save CHECK (save IN ('fortitude', 'reflex', 'will')),
    class_key TEXT REFERENCES class (key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT gear_bonus_scope_with_spells CHECK ((circumstance = 'spells') = (scope IS NOT NULL)),
    CONSTRAINT gear_bonus_school_with_scope CHECK ((scope IS 'school') = (spell_school IS NOT NULL)),
    CONSTRAINT gear_bonus_subschool_with_scope CHECK ((scope IS 'subschool') = (subschool IS NOT NULL)),
    CONSTRAINT gear_bonus_energy_with_scope CHECK ((scope IS 'energy') = (energy IS NOT NULL)),
    CONSTRAINT gear_bonus_save_with_circumstance CHECK ((circumstance = 'save') = (save IS NOT NULL)),
    CONSTRAINT gear_bonus_class_with_circumstance CHECK ((circumstance = 'class') = (class_key IS NOT NULL))
) STRICT;

CREATE INDEX gear_bonus_item ON gear_bonus (gear_item_key);
