-- Ticket 148: the numbers are naked (user, 2026-11-10, reversing ticket 122): no agent assumes gear on top of a
-- Build, so the guide's caster level and spell DC items (`gear.ron`) and their tables are gone.
DROP INDEX gear_bonus_item;
DROP TABLE gear_bonus;
DROP TABLE gear_item;
