-- Ticket 176: an item's always-on effects, a list beside `gear_other_effect`. `rules::GearItem::always_on`: one
-- short line each, in the planner's own words; the summary appends them to the bonuses in `id` order, so the
-- order is part of the data. They count in no number.
CREATE TABLE gear_always_on_effect (
    id INTEGER PRIMARY KEY,
    gear_item_key TEXT NOT NULL REFERENCES gear_item (key) DEFERRABLE INITIALLY DEFERRED,
    text TEXT NOT NULL CHECK (text <> ''),
    UNIQUE (gear_item_key, text)
) STRICT;
