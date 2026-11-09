-- Ticket 119: what the caster line needs from the game data.
--
-- `casting_ability` (`rules::Class::casting_ability`): the ability a class's spells use. NULL for a class that casts
-- no spells of its own (a fighter; every prestige class, which advances another's). On an archetype, NULL casts with
-- its class's ability and a value replaces it (Sage Sorcerer: Intelligence).
ALTER TABLE class ADD COLUMN casting_ability TEXT CONSTRAINT ability CHECK (casting_ability IN (
    'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
));

ALTER TABLE archetype ADD COLUMN casting_ability TEXT CONSTRAINT ability CHECK (casting_ability IN (
    'strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma'
));

-- `rules::ItemBonus` (`bonuses.ron`): a flat bonus an item gives while held. Exactly one item column names the item
-- (`rules::BonusItem`). `target` is `rules::BonusTarget`: `spell_dc` with a `scope` (`rules::SpellScope`, its
-- school, subschool or energy in their own column), `spell_penetration` or `concentration`; a later target (ticket
-- 120: initiative, a skill, a save) widens the CHECK and adds its column. `amount_kind` is `rules::BonusAmount`, with
-- `amount` for a flat one. `prerequisite_root_id`: what the character must hold besides the item (`requires`).
CREATE TABLE item_bonus (
    id INTEGER PRIMARY KEY,
    race_key TEXT REFERENCES race (key) DEFERRABLE INITIALLY DEFERRED,
    subrace_key TEXT REFERENCES subrace (key) DEFERRABLE INITIALLY DEFERRED,
    feat_key TEXT REFERENCES feat (key) DEFERRABLE INITIALLY DEFERRED,
    mythic_feat_key TEXT REFERENCES mythic_feat (key) DEFERRABLE INITIALLY DEFERRED,
    feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED,
    selection_option_key TEXT REFERENCES selection_option (key) DEFERRABLE INITIALLY DEFERRED,
    target TEXT NOT NULL CHECK (target IN ('spell_dc', 'spell_penetration', 'concentration')),
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
    amount_kind TEXT NOT NULL CHECK (amount_kind IN (
        'flat', 'half_mythic_rank', 'mythic_rank', 'spell_focus_of_another_school'
    )),
    amount INTEGER CHECK (amount BETWEEN -128 AND 127),
    prerequisite_root_id INTEGER UNIQUE REFERENCES prerequisite_node (id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT item_bonus_one_item CHECK (((race_key IS NOT NULL) + (subrace_key IS NOT NULL)
        + (feat_key IS NOT NULL) + (mythic_feat_key IS NOT NULL) + (feature_key IS NOT NULL)
        + (selection_option_key IS NOT NULL)) = 1),
    CONSTRAINT item_bonus_scope_with_spell_dc CHECK ((target = 'spell_dc') = (scope IS NOT NULL)),
    CONSTRAINT item_bonus_school_with_scope CHECK ((scope IS 'school') = (spell_school IS NOT NULL)),
    CONSTRAINT item_bonus_subschool_with_scope CHECK ((scope IS 'subschool') = (subschool IS NOT NULL)),
    CONSTRAINT item_bonus_energy_with_scope CHECK ((scope IS 'energy') = (energy IS NOT NULL)),
    CONSTRAINT item_bonus_amount_with_flat CHECK ((amount_kind = 'flat') = (amount IS NOT NULL))
) STRICT;
