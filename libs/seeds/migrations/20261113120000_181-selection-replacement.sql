-- The feature a selection gives at a skipped level in place of the option's row (the Empyreal Sorcerer's
-- Apotheosis at 20 for the Celestial bloodline's Ascension): `rules::FeatureSelection::replaced_by`.
-- `rules::GameData::check` keeps it to the selection's own skipped levels.
ALTER TABLE selection_skipped_level
    ADD COLUMN replaced_by_feature_key TEXT REFERENCES feature (key) DEFERRABLE INITIALLY DEFERRED;
