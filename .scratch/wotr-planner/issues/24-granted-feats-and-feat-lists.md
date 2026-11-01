# Granted feats and named feat lists

Type: task
Status: open
Blocked by: 23
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) S3 and R2).** Ticket 21 sections 4, 6 and "owner prefixes" have the numbers and the shapes.

- **Granted feats:** level rows (class, archetype, and mythic rank if the data needs it) gain `feats: Vec<FeatKey>`, table `granted_feat`. The 91 single-feat `one_of` slots (27 lists: Scribe Scrolls, Brew Potions, Improved Unarmed Strike, …) become grants, so they're no longer slots and no longer in the Build or URL. A granted feat is held without a prerequisite check, like a bonus-list pick. A grant of a feat already held does nothing extra.
- **Proficiency feats:** classes grant the real Armor and Shield Proficiency feats (from each class page's proficiency line in `guide-cache/`), so Armor Focus, Shield Focus, Shield Bash, Tower Shield and the like see them. Keep a proficiency feature only if it still holds something no feat covers.
- **Named feat lists:** `FeatList { key, name, guide, feats, ignores_prerequisites }` and `FeatSlot::OneOf { list: FeatListKey }`; tables `feat_list` + `feat_list_member` replace `feat_slot_option`, and `ignores_prerequisites` moves from the slot onto the list (no list appears with both values). Lists live in the owning class's file, plus one shared file for the 10 cross-class lists. A list that grows (ranger Archery at 2/6/10) is one list per stage.
- **Grant counts:** `granted_feature` and `granted_selection` become `(owner, key, count ≥ 1)` with a natural unique key in place of the surrogate `id`. The Rust `Vec`s keep their multiset meaning: the seed counts, the loader expands.
- **Mythic groups:** `MythicAbilityGroup` and `MythicFeatGroup` enums with `name()`, and domains on `group_name`.
- **Data fixes:** `starts_with(key, owner || '/')` CHECKs on `feature` and `feature_selection`; make `mythic/hard-to-kill` and the 6 Bypass Epic DR keys consistent with that rule (choose and justify the simplest shape); drop the 14 redundant rogue-talent `Any` alternatives.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB, per `CLAUDE.md`):
- Data via a one-off script, not the extractor; prove nothing else changed (same feats held at each level for every class/archetype, except slots that became grants). The extractor emits the new shape.
- The URL codec loses the picks for slots that became grants. No link has shipped, so regenerate the frozen v1 vector; no v2.
- Update `game-data/README.md` and `GLOSSARY.md` (**Feat slot**, **Bonus-list slot**, a **Granted feat** / **Feat list** entry if they earn one).
- **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests that a fighter can take Shield Focus and that a wizard's Scribe Scrolls is held with no pick.
