# Granted feats and named feat lists

Type: task
Status: resolved
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

## Progress log

- 2026-11-01: baseline in the session scratchpad `t24/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t24/`): `old-src/` is `git archive HEAD` (fc09aad); `extract-old/` is the HEAD extractor's output (identical to ticket 23's `t23/extract-new/`); `proof-old/` walks a one-class Build per class and archetype through the HEAD engine with every one-feat slot picked and dumps each level (`old-levels.jsonl`, 4570 lines; `old-fixture-levels.jsonl`, 25). `pages/*.txt` are the saved guide pages as text. `rewrite.py` (+ `names.py`, the list names) dry-runs clean on a copy (`dry/`): 27 one-feat lists on 76 slots become grants; 43 lists (29 owned by an archetype, 535 members); 28 proficiency features removed (and `mythic/hard-to-kill`); 11 redundant `Any` alternatives (not 14: the other 3 are in Extra Rogue Talent's "any rogue talent" list, where they're needed); 119 mythic groups; Hard to Kill ×6 and Bypass Epic DR ×6 become `<path>/…`. No list is shared by two classes once the one-feat lists are grants, so there is no shared lists file (see the Answer). Next: rules types and engine, test-first.
- 2026-11-01: `rules` done. Types: `FeatListKey`; `FeatList { key, name, guide, ignores_prerequisites, feats }` on `Class`, `Archetype` and `MythicPath` (`feat_lists`), with `GameData::feat_lists()`; `FeatSlot::OneOf { list }` (`FeatSlot::ignores_prerequisites` is gone: `Engine::slot_ignores_prerequisites` asks the list); `feats: BTreeSet<FeatKey>` on `ClassLevelRow`/`ArchetypeLevelRow` and in `EffectiveRow`; `MythicAbilityGroup`/`MythicFeatGroup` with `name()`. Engine: indexes the lists (`Engine::feat_list`), holds a row's granted feats in `take_class_level` before the level's slots. Check: `DataAt::FeatList`, `KeyOutsideItsOwner` (features, selections, lists), `FeatListTooShort`, `UnusedFeatList`, bonus lists by the list's flag, granted feats count in "row same as base". Tests first (new `granted_feats.rs`, `feat_lists.rs`, check tests); the world gains Scribe Scrolls (wizard 1), Improved Unarmed Strike (monk 1, Dex 13 invented), Shield/Tower Shield Proficiency (fighter 1), Shield Focus, and named lists. `cargo test -p rules` 125 + 1 pass, clippy clean, wasm builds; the v1 link vector still decodes unchanged (the Build's shape didn't change). Next: `data` crate.
- 2026-11-01: `data` done and the data rewritten. Migration: domains `mythic_ability_group`/`mythic_feat_group`; owner-prefix CHECKs on `feature`, `feature_selection` and the new `feat_list` (+ `feat_list_member`); `feat_slot.feat_list_key` (CHECK: set iff `one_of`) replaces `ignores_prerequisites` and `feat_slot_option`; `granted_feature`/`granted_selection` are `(grantor, key, count >= 1)` with `UNIQUE NULLS NOT DISTINCT` and no `id`; new `granted_feat (class_level_id, feat_key)`. 28 tables. Seed counts, loader expands; the `feat_slot_option` loader test is gone with its table. Tests: files (lists, granted feats, groups), postgres (counts round trip, new CHECKs and domains), new `data/tests/game_data.rs` on the real files (check passes, fighter takes Shield Focus, wizard holds Scribe Scrolls, 43 lists). `rewrite.py` ran over `game-data/` (same as the dry run) and the fixture (lists, groups; then by hand: list names, monk 1 grants a new fixture feat Improved Unarmed Strike, Armiger 1 grants Armiger Drill). Dev DB reset (only `wotr-planner-postgres`, 5434), migrated, seeded (same counts), `cargo sqlx prepare`. **Proof passes**: `proof-new/` walks the same Builds with no picks; `compare.py` finds all 4570 real lines (and 25 fixture lines) equal to `old-levels.jsonl` except the logged proficiency feats added and proficiency features dropped (and the fixture's two hand-added grants). Next: extractor.
- 2026-11-01: extractor done: new `extract::feat_lists` (`PendingLists` while building; `grant_lone_feats` after `drop_dangling`; `name_lists` after the rows are sparse, with the same owner/key rule as `rewrite.py` and mechanical names "Bonus Feats (from level N)" linked to the owner's guide), new `extract::proficiencies` (the same tables as `rewrite.py`, cross-checked equal), `references::drop_redundant_alternatives`, a cell granting a feat outright is `Granted::Feat`, `sparse_archetype_rows` compares granted feats, the `mythic-paths` page's headings are each path's own (`FeatureOwner::Mythic` is gone; `prose::path_cell` reads "Bypass Epic DR" as Bypass Epic Damage Reduction), groups are the enums, `ron_out` writes `feat_lists`, `feats` and bare groups. Tests first: `feat_lists.rs` (3), `proficiencies.rs` (3), references (+1), progressions (+1 case); `cargo test -p extract` 50 pass. A fresh extraction (`t24/extract-new/`) holds the same data as `rewrite.py` applied to the HEAD extraction (`t24/extract-old-rewritten/`), checked as canonical `GameData` by `t24/same-data/`; `EXTRACTION_REPORT.md` is byte-identical (82 items, 0 check problems); the files differ only in the order of 8 lines (renamed path features and one collapsed `Any` sort into place in the extractor's canonical output). Next: docs, full verification.
- 2026-11-01: docs (`game-data/README.md`, `GLOSSARY.md`: Feat slot, Bonus-list slot, new Feat list and Granted feat), final DB reset and reseed, `cargo sqlx prepare`, every Verification check passes. Done.

## Answer

Done 2026-11-01, uncommitted. A feat a class gives outright is now a granted feat on its level row (no slot, no pick, not in the Build or link); classes grant the real Armor, Shield and Weapon Proficiency feats from their pages' proficiency text; fixed lists are named `FeatList`s that hold the bonus-list flag; grants are counted; mythic groups are enums; owner prefixes are checked, with Hard to Kill and Bypass Epic DR made each path's own; the 11 redundant `Any` alternatives are gone.

### Rust (`rules`)

- `FeatListKey`, and `FeatList { key, name, guide, ignores_prerequisites, feats: BTreeSet<FeatKey> }` in `feat_lists` on `Class`, `Archetype` and `MythicPath` (the owner whose file defines it; any row may name any list, as with features). `GameData::feat_lists()` lists them all.
- `FeatSlot::OneOf { list: FeatListKey }`. `FeatSlot::ignores_prerequisites()` is gone: `Engine::slot_ignores_prerequisites(slot)` asks the list, and `Engine::feat_list(key)` finds one. A slot naming an unknown list accepts nothing.
- `ClassLevelRow.feats` and `ArchetypeLevelRow.feats` (`BTreeSet<FeatKey>`, not the ticket's `Vec`: a feat is held once, so it's a set), carried by `EffectiveRow.feats`. An archetype row's feats replace the base row's like its other grants. The engine holds a row's granted feats when the class level is taken, before the level's slots are filled: no prerequisite check, a feat already held adds nothing, and a later pick of it is "already held". Mythic rank rows don't grant feats: the data has no one-feat list on a path.
- `MythicAbilityGroup { Melee, Ranged, Magic, Healing, Miscellaneous, Class }` and `MythicFeatGroup { Melee, Ranged, ArmourAndShield, Magic, Miscellaneous, CombatManoeuvre }`, each with `ALL` and `name()` (the guide's heading).
- `GameData::check` adds `DataAt::FeatList`, `KeyOutsideItsOwner` (a feature, selection or feat list not keyed `<owner>/…`; an archetype's under the archetype's key), `FeatListTooShort` (fewer than two feats: grant a lone feat) and `UnusedFeatList`. `BonusListOutsideAClass` now reads the named list's flag, and a granted feat counts toward "row same as base".
- Tests first: new `granted_feats.rs` (6: a wizard's Scribe Scrolls is held with no pick, a granted feat skips its prerequisites, a fighter can take Shield Focus, picking a granted feat is already held, granting a held feat adds and breaks nothing, an archetype row replaces the granted feats), new `feat_lists.rs` (5) and 3 new check tests. The test world gains Scribe Scrolls (wizard 1), Improved Unarmed Strike (monk 1), Shield and Tower Shield Proficiency (fighter 1), Shield Focus and three named lists.

### Schema (the one migration)

- Domains `mythic_ability_group` and `mythic_feat_group` on `group_name`.
- `feature` and `feature_selection` get `CHECK (starts_with(key, COALESCE(archetype_key, class_key, path_key) || '/'))`.
- `feat_list (key, class_key/archetype_key/path_key, name, guide, ignores_prerequisites)` with the same owner columns, composite archetype FK and prefix CHECK, and `feat_list_member (feat_list_key, feat_key)`. `feat_slot.feat_list_key` (set iff `kind = 'one_of'`) replaces `feat_slot.ignores_prerequisites` and the `feat_slot_option` table.
- `granted_feature` and `granted_selection` are `(class_level_id, mythic_rank_id, key, count u8 >= 1)` with `UNIQUE NULLS NOT DISTINCT` and no `id`; the seed counts, the loader expands. New `granted_feat (class_level_id, feat_key)`, primary key both. `feat_slot` keeps its `id` (still a multiset, not asked to change). 28 tables.
- Tests: fixture round trip (lists on a class, an archetype and a path; granted feats on a class and an archetype row; groups), a new counts test (a feature ×2 and a selection ×3 stored as counts and read back in full), the new CHECKs and domains, and a new `data/tests/game_data.rs` on the real files (the check passes, a fighter 1 takes Shield Focus in its bonus slot, a wizard 1 holds Scribe Scrolls and has no list slot, 43 lists of two or more). The loader test that planted a `feat_slot_option` row went with the table; the class-skills test still covers rows no owner claims.

### Data rewrite

Script: `t24/rewrite.py` (+ `t24/names.py`, the list names) in the session scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t24/`). Line based, like ticket 23's: each archetype's table is materialised in full, every row rewritten, then made sparse again; everything else stays byte for byte. The extractor was never run over `game-data/`.

| | Real data | Fixture |
| --- | --- | --- |
| one-feat lists → granted feats | 27 lists on 76 slots | 0 (two grants added by hand) |
| named lists (owned by an archetype) | 43 (29), 535 members | 3 (1, plus one on a path) |
| proficiency features removed | 28 | 0 |
| redundant `Any` alternatives dropped | 11 | 0 |
| mythic groups | 119 | 4 |
| archetype rows | 1622 → 1622 (none added or dropped) | 6 → 6 |

- **Lists.** Every distinct (feats, flag) of a class file is one list, owned by the archetype that alone uses it in its stored rows, else by the class; key `<owner>/bonus-feats`, or `<owner>/bonus-feats-<first level>` when the owner has several (the ranger's archery lists share level 2 with its combat style list, so they are `-2b`, `-6b`, `-10b`). The names and links come from the guide's heading for each list (`Monk Bonus Feats (from level 6)`, `monk#monk-bonus-feats`). **No list is used by two classes** once the one-feat lists are grants (ticket 21's 10 cross-class lists were all one-feat lists: Scribe Scrolls, Brew Potions, …), so there is no shared lists file; were one ever shared, a class would own it and the others name it, as prestige classes grant `rogue/sneak-attack`. `ignores_prerequisites` was a function of the list everywhere, so it moved onto the list without a conflict.
- **Proficiency feats.** From each class page's "… Proficiencies" section (all quoted in the script and in `extract/src/proficiencies.rs`). The weapon feats are granted too: Eldritch Knight requires Martial Weapons Proficiency (the "real proficiency feats" of decision R2). Level 1 unless noted; S simple, M martial, L/Md/H light/medium/heavy armour, Sh shield, T tower shield:

  | Class | Feats | Archetypes that differ |
  | --- | --- | --- |
  | fighter | S M L Md H Sh T | |
  | cavalier, paladin, warpriest | S M L Md H Sh | Beast Rider, Divine Hunter: no H; Cult Leader: S L; Mantis Zealot: S M L |
  | barbarian, bloodrager, hunter, ranger, skald, slayer | S M L Md Sh | Instinctual Warrior: S M Sh; Stygian Slayer: S M L |
  | cleric, inquisitor, oracle, shifter | S L Md Sh | Angelfire Apostle: S L Sh; Demonbane Priest: + H; Herald Caller: S L; Purifier: + H at 11 |
  | magus | S M L, Md at 7, H at 13 | Armored Battlemage: Md at 1, H at 7; Arcane Rider, Spell Dancer: no Md/H; Sword Saint: S |
  | shaman | S L Md | |
  | druid | L Md Sh | |
  | alchemist, rogue, kineticist, bard | S L (bard + Sh) | Eldritch Scoundrel: S; Kinetic Knight: + Md H Sh |
  | arcanist, witch | S | |
  | monk, wizard, sorcerer | none | Sohei: S M L |

  Heavy armour that a table already granted as a one-feat list (Armored Hulk, Steelblood, Hellknight Signifer) is now simply a granted feat. A proficiency feature stays where it says more than the feats (named weapons, bombs, casting in armour, a deity's weapon, Slayer's Feint naming Slayer Proficiencies, or no text at all for the sorcerer's): 28 were removed (arcanist, barbarian, Instinctual Warrior, bloodrager and its 8 archetype copies, cavalier, Beast Rider, Ghost Rider, Angelfire Apostle, Demonbane Priest, fighter, hunter, oracle, paladin, Divine Hunter, ranger, shaman, shifter, Stygian Slayer, warpriest, witch) and 27 `…-proficiencies` features stay.
- **Owner prefixes.** Hard to Kill becomes `<path>/hard-to-kill` on each of the six paths that grant it, and the six Bypass Epic DR keys (five `<path>/bypass-epic-dr`, the Trickster's `mythic/bypass-epic-damage-reduction`) become `<path>/bypass-epic-damage-reduction`, all named and linked as the `mythic-paths` heading that describes them. Why this shape: both describe a general ability that each path's own table grants; a single key would need an owner that isn't a path (or would pretend one path owns it), while per-path keys follow the rule with no exception and match what five of the six Bypass keys already did. It costs ten more feature rows. Every feature and selection now passes the prefix CHECK.
- **Redundant alternatives.** 11, not 14: 9 `Any([Feat("weapon-focus"), SelectionOption("rogue/rogue-talents/weapon-focus")])` and 2 Skill Focus. The other 3 that name the Weapon Focus, Skill Focus and Intimidating Prowess talents sit in Extra Rogue Talent's "any rogue talent" list, where the talent is wanted as a talent, so they stay.

### Proof

`t24/proof-old/` (built against `git archive HEAD`) walks one Build per class and archetype, every level of the class's table, through the HEAD engine with each one-feat slot picked as a player would, and dumps each level: BAB, saves, spell levels, caster level, feats held, features held with counts, and the slots offered (lists as their feats and flag, the one-feat slots left out). `t24/proof-new/` walks the same Builds through the new engine with **no picks** and dumps the same lines. `t24/compare.py` checks every field equal, allowing only what `rewrite.py --log` recorded per row: proficiency feats added and proficiency features removed (and, for the fixture, its two hand-added grants). **Result: all 4570 real lines and 25 fixture lines match**; 3900 lines hold more feats (the proficiencies), 2260 fewer features (the removed proficiency features), nothing else differs, and neither walk has a slot problem.

### Extractor

- New `extract::feat_lists`: fixed lists are pending (`PendingLists`) while the data is built; after `drop_dangling` prunes their feats, `grant_lone_feats` makes each one-feat list a granted feat; after the rows are sparse, `name_lists` gives owners, keys, the mechanical name `Bonus Feats` / `Bonus Feats (from level N)` and the owner's guide link (the headings' names in `game-data/` are a hand fix).
- New `extract::proficiencies` (the same tables as the script, checked equal by a script) runs on the full rows; `references::drop_redundant_alternatives`; a cell that grants a feat outright is `Granted::Feat`; `sparse_archetype_rows` compares granted feats; the `mythic-paths` page's headings are each path's own (`FeatureOwner::Mythic` is gone) and `prose::path_cell` reads "Bypass Epic DR" as Bypass Epic Damage Reduction; groups are the enums; `ron_out` writes `feat_lists`, `feats` and bare groups.
- Tests: `feat_lists.rs` (3), `proficiencies.rs` (3), references (+1), progressions (a row differing only in granted feats is kept).
- A fresh extraction (`t24/extract-new/`) holds **the same data** as `rewrite.py` applied to the HEAD extractor's output (`t24/extract-old-rewritten/`), compared as canonical `GameData` (`t24/same-data/`), and `EXTRACTION_REPORT.md` is byte-identical (82 items, 0 check problems). The files differ only in the order of 8 lines: the renamed path features and one collapsed `Any` sort into place in the extractor's canonical output, while the script leaves them where they were.

### Docs

- `game-data/README.md`: named lists and where they live, granted feats and proficiency feats, owner prefixes, mythic groups.
- `GLOSSARY.md`: Feat slot and Bonus-list slot updated; new Feat list and Granted feat.

### Verification

- `cargo test --workspace`: pass (rules 125 + unit 1, extract 50, data files 15 + game_data 4 + postgres 15).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths (unchanged). Rows: `class_level` 2272, `granted_feature` 2894 (2896 grants), `granted_selection` 325 (334), `granted_feat` 752, `feat_slot` 289 (142 `one_of`), `feat_list` 43, `feat_list_member` 535, `feature` 1747.
- The v1 link vector needed no regeneration: the `Build` type didn't change (slots that became grants just have no pick), so the frozen link decodes to the same Build and encodes to the same code.
- Proof: matches (above).

### Open questions (for the user; the data follows the class pages, as the ticket says)

1. **The feats page and the class pages disagree on shields.** The feats page's Shield Proficiency entry gives it to "Clerics (not Ecclesitheurge)", "Slayers (all)" and "Warpriests (not Cult Leader)", and doesn't list kineticists; the class pages say the Herald Caller "doesn't gain proficiency with medium armour or shields", the Stygian Slayer and Mantis Zealot lines name no shields, and the Kinetic Knight's Elemental Bastion gives shields. The data follows the class pages (no Shield Proficiency for Herald Caller, Stygian Slayer, Mantis Zealot; yes for Kinetic Knight). Say if the feats page should win.
2. **Kinetic Sharpshooter** lists only bows and crossbows; the data keeps the kineticist's simple weapons and light armour (the feats page doesn't list it among those without light armour).
3. **Azata's All-skilled** ("proficient with all types of weapons and armour", a superpower selection option) isn't modelled: an option grants at most one feat. It would need options that grant several feats.


## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. Shields: the class pages win over the feats page's summary list. No data change.
2. Kinetic Sharpshooter keeps the kineticist's simple weapons and light armour. No data change.
3. Azata's All-skilled: a selection option may grant several feats; All-skilled grants every weapon, armour and shield proficiency feat. Built in [ticket 26](26-mythic-path-rules.md).
