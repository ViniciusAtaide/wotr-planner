# Mythic path rules

Type: task
Status: open
Blocked by: 25
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) R1, R3 and S3).** Background: [Hand-fix the extracted mythic data](20-handfix-mythic.md) follow-ups and ticket 21's follow-up 5 and 7.

- **MR 1–2 on choosing the path:** choosing a path at MR 3 grants its MR 1–2 rows (12 rows, 30 features today). They show at their ranks and are held from the MR 3 ascension on; a pick made before MR 3 can't rely on them.
- **Kept ranks:** `PathSwitch.kept_through_rank: u8` replaces `kept_ranks`; drop `path_switch_kept_rank` for a column on `path_switch`.
- **Partial keeps:** from the guide's prose, Gold Dragon, Swarm That Walks and Devil keep the general picks (mythic abilities and feats) of MR 1–7 but drop the old path's MR 3–7 path powers. Model what the guide states; decide the shape with `kept_through_rank`.
- **Previous path:** `PathSwitch.from: BTreeSet<MythicPathKey>` (empty = any). Devil only from Aeon or Azata.
- **Ability bonuses from features:** a new fact, a feature that grants ability-score bonuses (Legend's +4 to every score; Gold Dragon's Perfect Body, Mind and Soul; check the guide for others). They count toward ability prerequisites from the point they're held: they're the character's own scores, and only gear stays excluded.
- **Legend keeps Trickster feats:** a Trickster who switches to Legend keeps the feats Perception II unlocked, and picks made with them stay valid.
- **`MythicRank(n)` prerequisite:** a `prerequisite_node` kind `mythic_rank` (`amount` = rank). Use it for the Gold Dragon breaths and replace the Aeon gazes' `CharacterLevel` stand-ins.
- **Gold Dragon's Dragon Feats** ignore prerequisites: allow a bonus list on that path rank's slot (narrow `BonusListOutsideAClass` to exactly what the guide says).

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): data by one-off script, docs and `GLOSSARY.md` (**Path switch**, **Mythic track**) updated, the map's Out of scope line on gear stays true. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests for each bullet.
