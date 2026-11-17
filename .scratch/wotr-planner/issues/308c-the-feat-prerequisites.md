# Importer S4, 308c: the feats' prerequisites, as the game's

Type: build (the importer, stage S4)
Status: resolved
Blocked by: 308a, 308b, 306aa, 306q, 306r, 306n, 306s
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

Built on base `73bafe1d`. The note is `libs/game-data/notes/308c-feat-prerequisites.md`.

**What was built**

- **Written from the game.** `game-check --write prerequisites --only 'feat *' --log` wrote 70 feats, the 64 of the slice plus those 306q/r/s and 308a added. The extra six include Arcane Strike, Natural Charmer, Perfect Strike, the armour and shield proficiencies, Extra Mercy and Extra Arcane Pool. A second `--dry-run` writes 0.
- **Hand-written** (decision 2, the main session): Extra Arcana and Extra Hex (Shaman) are the game's `Requires`, less the Eldritch Scion's and the Winter Witch's selections. Neither has an entry or a Build that holds it.
- **Game check.** It went from 3,789 differences (175 not known) to 3,614, all known. Prerequisites now: 960 agree and 487 differ (898 and 549 before).
- **`known.txt`.** The 263 and 264 sections are now one section, with the slice's header ("superseded by 293"). Each line left names its owner:
  - 307: 17 feats;
  - 313: Fencing Grace, Slashing Grace, Point-Blank Master;
  - 314: Accomplished Sneak Attacker;
  - 315: the Trickster critical feats;
  - 174: barding ×3, Boon Companion;
  - 306x: Disruptive;
  - 308: Extended Aspects;
  - Extra Channel stays 267's and Favored Enemy Spellcasting 275's, as 308a homed them.
- **Narrowed `known.txt` lines.** Extra Arcana's, Extra Hex's and Extra Hex (Shaman)'s lines narrow to their missing member, `-> 306o`. The six `-> 308` proficiency lines are deleted.
- **Allowlist.** All 91 `feat` lines of `prerequisite-narrowing.txt` are removed: Manyshot, Rapid Shot, Advanced Weapon Training, Critical Mastery, Pummeling Style, Extra Arcanist Exploit, Extra Discovery (Vivisectionist), Extra Rage Power, Extra Wild Talent and Extra Mercy. The header and the other entries are kept.
- **Generator.** `talents.rs`'s "t57 exploiter extra exploit with no exploit" opened now: the game admits the feat once the Exploiter's selection is held. Its `full` copy would have made 2,157 Builds. It is now "t57 wizard extra exploit with no exploit selection", a plain wizard the game refuses. **2,156 Builds before, 2,156 after (1,990 open).**

**Decisions**

- The `a_hexcrafter_5_meets_extra_arcana` test is ignored again, now `-> 283` (mine). The prerequisite is the game's, but the Hexcrafter's row 3 lacks `magus/magus-arcana`, which is 283's (`a_hexcrafter_keeps_its_magus_arcana_at_3`).
- `option_lists`: all 11 Extra feats now name selections, not options. `NO_OPTIONS_NAMED` lists them, and the stale greater-exploits exception goes.
- The four wild shape feats and Spell Specialization keep their §1.2 owner, 307. Their other gap is named in a comment.

**Proof** (`compare`: `FORM_PROOF_OFFERS=1`, `FORM_PROOF_CENSUS` = the log plus the two hand rows)

- **Census.** 4,020 lines in 175 Builds: builds.tsv 174, numbers 3,333, sheets 276, slots 235, reopened slots 2. **0 unattributed.** The only other differences are the renamed t57 Build, lost and new.
- **Links.** 1,897 the same, 93 refused. 306-base gains two blocks: 74 lines refuse Spell Chain, which needs Spell Penetration (SpellChain d8d641edc9a74a64809f56feff9495b6). One line, `full t61 urban hunter 12 captor`, refuses Greater Disarm, which now needs Intelligence 13.
- **Offers.** 200,273 statuses changed in 875 groups. By group, each toward the game:
  - **Refusal fallout:** `ok -> BreaksLater` in about 93 Builds, the refused links' Builds. Inference, as in 306aa.
  - **Spell Chain:** `-> Unmet` in 2,153 Builds.
  - **Loosened:** Rapid Shot `Unmet -> ok` in 2,154 Builds (only Intelligence 3, omitted), Manyshot in 11, Ironguts in 80 (dwarf), Arcane Strike in 60 (`CasterType`), Agile Maneuvers in 21 (Disarm added), and Feint in 4 (Kinetic Warrior).
  - **The Extra feats `Unmet -> ok`:** Rage Power 55, Arcanist Exploit 19, Hex (Shaman) 14, Mercy 24, Arcana 10, Lay On Hands 10, Bombs 4. Each selection is held whatever is picked in it.
  - **Extra Rage Power `ok -> Unmet`** in 55 Builds, characters with none of the three selections.
  - **Tightened:** Combat Mobility 35 (Dex 13); Greater Trip 24, Greater Disarm 20, Disarming Strike 17 and Greater Dirty Trick 15 (Int 13); Greater Bull Rush 4 (Str 13); Pummeling Style 34 (the monk's flurry only); Improved Snap Shot 3 (Point-Blank Shot); Dragon Ferocity and Dragon Roar 3 (their added leaves).
  - **22 options only now:** the slots that exist where an Extra feat is now held.
- **306aa's open finding** does not move. `full t60 bard 20 / filled` reads the same Weapon Focus statuses on base and tip, compared with a scratch probe: BreaksLater for short-bow and bomb at 3 and 5.
- **Gold builds:** unchanged, all pass.

**Tests**

- Un-ignored: `class_skills::advanced_weapon_training_needs_the_fighters_weapon_training`, `skald_rage_powers::extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection` and `skald_rage_powers::a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot`.
- New, in `feat_prerequisites.rs`: one test per pattern, each citing its blueprint and GUID (Cleave's chain, the manoeuvre grouping, the races, BAB, the Extra feats' selections, the Weapon Focus widening, Arcane Strike, the full score, Critical Mastery).
- Pinned values updated, each cited to the game: `wild_talents::extra_wild_talent_needs_the_wild_talent_selection` (renamed), `class_skills` ×2, `repeating_talents` (Cleave's Str 13), `extra_picks` (magus 3 meets Extra Arcana), and `prerequisites_as_components` (650 → 652).

**Checks**

- `cargo test -p seeds -p corpus`: ok. The last run of seeds after its last edit gives 599 passed, 105 ignored. Corpus: links 6 passed, form_proof 22.
- `cargo test -p game-check`: 158 passed.
- `check-data`: "checked … 276 feats …".
- `cargo fmt --all --check`: clean.
- `scripts/test-modules.sh`: "test-modules OK".
- Game check: exit 0, "3614 known", no stale line.
- Dry run: "0 fields to write".
