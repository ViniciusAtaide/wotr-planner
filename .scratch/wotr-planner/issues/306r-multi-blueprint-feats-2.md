# Importer S2, 306r: feats holding several blueprints II

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306q
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built in one batch with [306q](306q-multi-blueprint-feats-1.md) (same commit; the proof, census and notes are there and in `libs/game-data/notes/306q-multi-blueprint-feats.md`).

**What was built:** `extra-channel` keeps the cleric's and Priest of Balance's channels (`ExtraChannel` cd9f1977…); new `extra-channel-oracle` (53da6bf4…), `-shaman` (4cc73e43…), `-hospitaler` (8d4f82fd…), `-hex-channeler` (88f426f1…), `-empyreal-sorcerer` (347e8d79…), each `ranks: 10`, aliased to `extra-channel`, with its own channel as prerequisite; new `bleeding-critical` (`BleedingCriticalFeature` 42be7d46…, Critical Focus and BAB 11) and `spell-bane` (`SpellBane` d2d1b1f2…). Names and summaries are in our own words (Bleeding Critical's and Spell Bane's rest on the blueprints; the guide lacks them, an inference). All are in the pool `feats` and the Dragon Feats list; Bleeding Critical is in the magus's, Arcane Rider's, warpriest's, cavalier's, Gendarme's and `combat-feats` lists. `known.txt`: the five ExtraChannel pool lines and 263's pool line removed; added two `extra-channel-shaman` lines under 307: the game's `ShamanLifeSpiritChannelEnergyFeature` b8ec9dcc… is granted only by `ShamanLifeSpiritBaseFeature` 9ae3ba76… (`game-check refs`), which the planner holds as `shaman/spirit/life/channel`, so the prerequisite names that feature or the witch doctor's channel (`Any`, as the game); the check compares by GUID, so each side shows until the carriers map it.

**Test:** `importer_lists::extra_channel_is_six_feats_one_per_channel` (the six blueprints above, `Ranks` 10, aliases, prerequisites by the `PrerequisiteFeature`s named in its comment); also `extra_rage_and_extra_performance_have_the_games_class_variants` and `bleeding_critical_and_spell_bane_are_feats`. Written after the data. Proof: see 306q (option lists: no `extra-channel` pick opens through an heir in the corpus; `extra-channel` ok to Unmet in 10 Builds).
