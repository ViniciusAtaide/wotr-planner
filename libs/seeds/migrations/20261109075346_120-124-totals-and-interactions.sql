-- Tickets 120 and 124: initiative, save and skill bonuses on `item_bonus`, and interactions notes on items.
--
-- `item_bonus` (`rules::ItemBonus`, `bonuses.ron`) is recreated wider (ticket 119 made it). Exactly one item column
-- names the item (`rules::BonusItem`), now with `background_key` and `mythic_ability_key`. `target` is
-- `rules::BonusTarget`: `spell_dc` with a `scope` (`rules::SpellScope`, its school, subschool or energy in their own
-- column), `spell_penetration`, `concentration`, `initiative`, `save` with its `save`, `all_saves`, `skill` with its
-- `skill_key`, `all_skills`, or `held` (the skill or stat the item is held for). `amount_kind` is
-- `rules::BonusAmount`: `amount` for a flat one, `amount` and `amount_with_ten_ranks` for `ten_ranks`, `ability` for
-- `ability_modifier`. `prerequisite_root_id`: what the character must hold besides the item (`requires`).
DROP TABLE item_bonus;

CREATE TABLE item_bonus (
    id INTEGER PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    subrace_key TEXT REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    background_key TEXT REFERENCES background (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_feat_key TEXT REFERENCES mythic_feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_ability_key TEXT REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    selection_option_key TEXT REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    target TEXT NOT NULL CHECK (target IN (
        'spell_dc', 'spell_penetration', 'concentration', 'initiative', 'save', 'all_saves', 'skill', 'all_skills',
        'held'
    )),
    scope TEXT CHECK (scope IN ('all', 'school', 'subschool', 'energy', 'metamagic', 'value')),
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
    skill_key TEXT REFERENCES skill (key) DEFERRABLE INITIALLY DEFERRED,
    amount_kind TEXT NOT NULL CHECK (amount_kind IN (
        'flat', 'half_mythic_rank', 'mythic_rank', 'spell_focus_of_another_school', 'ten_ranks', 'ability_modifier'
    )),
    amount INTEGER CHECK (amount BETWEEN -128 AND 127),
    amount_with_ten_ranks INTEGER CHECK (amount_with_ten_ranks BETWEEN -128 AND 127),
    ability TEXT CONSTRAINT ability CHECK (ability IN (
        'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
    )),
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT item_bonus_one_item CHECK (((race_key IS NOT NULL) + (subrace_key IS NOT NULL)
        + (background_key IS NOT NULL) + (feat_key IS NOT NULL) + (mythic_feat_key IS NOT NULL)
        + (mythic_ability_key IS NOT NULL) + (feature_key IS NOT NULL) + (selection_option_key IS NOT NULL)) = 1),
    CONSTRAINT item_bonus_scope_with_spell_dc CHECK ((target = 'spell_dc') = (scope IS NOT NULL)),
    CONSTRAINT item_bonus_school_with_scope CHECK ((scope IS 'school') = (spell_school IS NOT NULL)),
    CONSTRAINT item_bonus_subschool_with_scope CHECK ((scope IS 'subschool') = (subschool IS NOT NULL)),
    CONSTRAINT item_bonus_energy_with_scope CHECK ((scope IS 'energy') = (energy IS NOT NULL)),
    CONSTRAINT item_bonus_save_with_target CHECK ((target = 'save') = (save IS NOT NULL)),
    CONSTRAINT item_bonus_skill_with_target CHECK ((target = 'skill') = (skill_key IS NOT NULL)),
    CONSTRAINT item_bonus_amount_with_kind CHECK ((amount_kind IN ('flat', 'ten_ranks')) = (amount IS NOT NULL)),
    CONSTRAINT item_bonus_ten_ranks_with_kind CHECK
        ((amount_kind = 'ten_ranks') = (amount_with_ten_ranks IS NOT NULL)),
    CONSTRAINT item_bonus_ability_with_kind CHECK ((amount_kind = 'ability_modifier') = (ability IS NOT NULL))
) STRICT;

-- `rules::ItemInteractions` (`interactions.ron`, ticket 124): what the guide says about how an item combines with
-- others, in the planner's own words. Exactly one item column names the item, as on `item_bonus`; one note per item.
-- No stacking column: the saved guide pages give no bonus type for the bonuses ticket 124 asked about, so every bonus
-- held still adds.
CREATE TABLE item_interactions (
    id INTEGER PRIMARY KEY,
    race_key TEXT UNIQUE REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    subrace_key TEXT UNIQUE REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    background_key TEXT UNIQUE REFERENCES background (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT UNIQUE REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_feat_key TEXT UNIQUE REFERENCES mythic_feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_ability_key TEXT UNIQUE REFERENCES mythic_ability (key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT UNIQUE REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    selection_option_key TEXT UNIQUE REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    interactions TEXT NOT NULL CHECK (interactions <> ''),
    CONSTRAINT item_interactions_one_item CHECK (((race_key IS NOT NULL) + (subrace_key IS NOT NULL)
        + (background_key IS NOT NULL) + (feat_key IS NOT NULL) + (mythic_feat_key IS NOT NULL)
        + (mythic_ability_key IS NOT NULL) + (feature_key IS NOT NULL) + (selection_option_key IS NOT NULL)) = 1)
) STRICT;
