# 304 decisions (main session, 2026-11-16), on the research report
(Research files beside this: fields.tsv, compare.txt, scripts/; the *.show/*.json dumps hold game data and stay out of the repo.)

- Q1: apply the header-derived levels for the 19 selections, including the five differences (non-draconic bloodlines drop Dragon Disciple and add Eldritch Scion magus; draconic sorcerer/eldritch-scion/dragon-disciple bloodlines count all three; draconic bloodrager bloodlines count Dragon Disciple; Frost spirit counts Winter Witch; Weapon Mastery adds oracle and Magic Deceiver arcanist). Game wins (234); each cited by blueprint name+GUID and CalcLevel, each with a real-data test. Research found 0 corpus Builds affected: the form proof must confirm.
- Q2: honour GiveFeaturesForPreviousLevels (LevelUpHelper.UpdateProgression); the Primalist and expanded element keep the planner form.
- Q3: rename the ranger/slayer style stage keys to the game's selections (GUIDs); only the slots.txt kind column may change. Every SlotId and pick stays byte-identical (the link stores only the feat).
- Q4: the requisite-feature gating quirks (bloodline feat lists via any bloodrager bloodline or Tiefling Classic heritage; RangerStyleSelection2 refusing a slayer with a ranger style; RangerStyleSelection10 offering SkillFocusSelection) go to known.txt under 293, not 304.
- Q5: write headers on the domain options now; the selection's own fields override until 294.
- option_lists: nest as the game does where ids stay identical (sorcerer per-bloodline lists; ranger/slayer stage selections); bloodrager and kineticist keep a planner nest with a `why`. `option_lists`, `at_level`, `feat_list`, `option_feat_lists` go.
- Fields: `half_other_classes` and the curses' `option_level_archetypes` go; `option_levels` stays on the 49 listed selections until 293/294; `level_from`, `skipped_levels`, `replaced_by`, `later_rows_behind` stay.
