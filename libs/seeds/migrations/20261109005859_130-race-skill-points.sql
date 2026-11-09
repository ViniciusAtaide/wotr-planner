-- Ticket 130: the skill points a race adds at every level, on top of the class's and Intelligence's (the guide's
-- "Skilled": the human and the Human-Raised half-orc, 1). A subrace's replace its race's.
ALTER TABLE race ADD COLUMN skill_points INTEGER NOT NULL DEFAULT 0
    CONSTRAINT u8 CHECK (skill_points BETWEEN 0 AND 255);

ALTER TABLE subrace ADD COLUMN skill_points INTEGER NOT NULL DEFAULT 0
    CONSTRAINT u8 CHECK (skill_points BETWEEN 0 AND 255);
