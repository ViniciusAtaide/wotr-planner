-- Ticket 165: the gear items (`rules::GearItem`, `gear.ron`), after ticket 148 dropped the first, assumed-gear
-- tables. No Build choice names one yet.
--
-- `gear_item`: `slot` is `rules::GearSlot`. `guide` is `page#section`, left empty (NULL) for an item the guide
-- doesn't list (the generic stat belts and headbands, from the user).
CREATE TABLE gear_item (
    key TEXT NOT NULL PRIMARY KEY CONSTRAINT text_key CHECK (key GLOB '[a-z0-9]*' AND key GLOB '*[a-z0-9]'
        AND key NOT GLOB '*[^a-z0-9/-]*' AND key NOT GLOB '*[-/][-/]*'),
    name TEXT NOT NULL UNIQUE CHECK (name <> ''),
    slot TEXT NOT NULL CHECK (slot IN (
        'belt', 'boots', 'bracers', 'cloak', 'glasses', 'gloves', 'headwear', 'necklace', 'ring', 'robe', 'armour',
        'weapon', 'usable'
    )),
    guide TEXT CHECK (guide <> '')
) STRICT;

-- `rules::GearBonus`: one always-on bonus of an item. `target` is `rules::GearTarget`: `ability` with its `ability`,
-- `all_saves`, `save` with its `save`, `initiative`, `skill` with its `skill_key`, `spell_dc` with a `scope`
-- (`rules::SpellScope` without `value`, its school, subschool or energy in their own column), `caster_level`,
-- `spell_penetration` or `concentration`. `bonus_type` is `rules::BonusType`; a penalty is `untyped`.
CREATE TABLE gear_bonus (
    id INTEGER PRIMARY KEY,
    gear_item_key TEXT NOT NULL REFERENCES gear_item (key) DEFERRABLE INITIALLY DEFERRED,
    target TEXT NOT NULL CHECK (target IN (
        'ability', 'all_saves', 'save', 'initiative', 'skill', 'spell_dc', 'caster_level', 'spell_penetration',
        'concentration'
    )),
    ability TEXT CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    )),
    save TEXT CONSTRAINT save CHECK (save IN ('fortitude', 'reflex', 'will')),
    skill_key TEXT REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
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
    amount INTEGER NOT NULL CHECK (amount BETWEEN -128 AND 127 AND amount <> 0),
    bonus_type TEXT NOT NULL CHECK (bonus_type IN (
        'alchemical', 'armour', 'circumstance', 'competence', 'deflection', 'dodge', 'enhancement', 'inherent',
        'insight', 'luck', 'morale', 'natural_armour', 'profane', 'racial', 'resistance', 'sacred', 'shield', 'size',
        'untyped'
    )),
    CONSTRAINT gear_bonus_ability_with_target CHECK ((target = 'ability') = (ability IS NOT NULL)),
    CONSTRAINT gear_bonus_save_with_target CHECK ((target = 'save') = (save IS NOT NULL)),
    CONSTRAINT gear_bonus_skill_with_target CHECK ((target = 'skill') = (skill_key IS NOT NULL)),
    CONSTRAINT gear_bonus_scope_with_spell_dc CHECK ((target = 'spell_dc') = (scope IS NOT NULL)),
    CONSTRAINT gear_bonus_school_with_scope CHECK ((scope IS 'school') = (spell_school IS NOT NULL)),
    CONSTRAINT gear_bonus_subschool_with_scope CHECK ((scope IS 'subschool') = (subschool IS NOT NULL)),
    CONSTRAINT gear_bonus_energy_with_scope CHECK ((scope IS 'energy') = (energy IS NOT NULL)),
    CONSTRAINT gear_bonus_penalty_untyped CHECK (amount > 0 OR bonus_type = 'untyped')
) STRICT;

CREATE INDEX gear_bonus_item ON gear_bonus (gear_item_key);

-- `rules::GearItem::other_effects`: one short line each, in the planner's own words.
CREATE TABLE gear_other_effect (
    id INTEGER PRIMARY KEY,
    gear_item_key TEXT NOT NULL REFERENCES gear_item (key) DEFERRABLE INITIALLY DEFERRED,
    text TEXT NOT NULL CHECK (text <> ''),
    UNIQUE (gear_item_key, text)
) STRICT;
