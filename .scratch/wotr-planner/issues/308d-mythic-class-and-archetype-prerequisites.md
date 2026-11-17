# Importer S4, 308d: mythic abilities', classes' and archetypes' prerequisites

Type: build (the importer, stage S4)
Status: resolved
Blocked by: 308c
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

Built on base `a165d7a2`. The note is `libs/game-data/notes/308d-mythic-class-archetype-prerequisites.md`.

**What was built**

- **Written from the game.** `game-check --write prerequisites --only <24 entries> --log` wrote 11 mythic abilities (Abundant Arcane Pool, Any Means Necessary, Beneficial Curse, Defensive Study, Impossible Domain, Kinetic Overcharge, Limitless Rage, Over-Infused Blasts, Second Bloodrager Bloodline, Second Mystery, Witch Wandering Hex), the mythic feat Weapon Focus (mythic), the oracle, eight prestige classes (Assassin, Duelist, Eldritch Knight, Hellknight, Hellknight Signifer, Mystic Theurge, Stalwart Defender, Winter Witch) and three archetypes (Magic Deceiver, Dark Elementalist, Rageshaper). Abundant Arcane Pool and Weapon Focus (mythic) aren't in the slice's list; the wildcard dry run named them (a flattened `Any`, and the rogue talent's shared WeaponFocus), as 308c's run took its extras. A second `--dry-run --only <same>` writes 0.
- **The slice's entries with no diff:**
  - `abundant-smite-smite-chaos` is already equal to the game (written by 306q/r), so the writer leaves it.
  - `mantis-zealot` is blocked: its NoFeature of DeitySelection is a narrowing no allowlist line names, and its line stays 267's (308a's comment).
  - `hag-of-gyronna` is written then reverted (decision mine, below).
  - `greater-enduring-spells` is left out for ticket 208 (decisions item 4). Its `-> 265` line stays.
- **Allowlist** (`prerequisite-narrowing.txt`): 73 lines removed. Each named entry was written: Kinetic Overcharge 2, Witch Wandering Hex 28, Magic Deceiver 3, Mystic Theurge 2, oracle 2, Defensive Study 4, Winter Witch 2, Beneficial Curse 10, Second Bloodrager Bloodline 2, Second Mystery 10, Impossible Domain 6. The Hag of Gyronna's 2 lines were removed unwritten, and so was one emptied comment heading.
- **`known.txt`:**
  - Deleted: the 24 stale lines the check listed, among them 303's `not in game / mythic ability beneficial-curse`, whose comment I reworded, and 259's Witch Wandering Hex section.
  - Re-homed to the owner the writer prints: Abundant Bane, Abundant Ki Pool, Domain Zealot, Everlasting Judgement and Destiny Beyond Birth to 307; Bloodline Ascendance and Second Bloodline to 306x; School Tolerance to 306p; Second Spirit to 308; the nine Mythic Armor Focus feats, Loremaster and Student of War to 315; Mythic Sneak Attack to 314; Mythical Beast to 174; Aldori Swordlord to 313; the magus to 306o.
  - Unchanged: Arcane Trickster and Dragon Disciple (hand prerequisites) stay 287's, and 267's deity entries stay 267's.
- **Game check:** 3,614 → 3,565 differences, all known, no stale line. Prerequisites: 1,447 compared, 983 agree.

**Decisions**

- **Hag of Gyronna, left as it was (mine).** DarkSisterArchetype 411fa458481e44f0855d47a19358874b names only an alignment. But every other deity's feature and AtheismFeature carry PrerequisiteNoArchetype DarkSister (`game-check refs`; AsmodeusFeature a3a5ccc9c670e6f4ca4a686d23b89900). The planner's deities don't hold those exclusions until 267, so writing the archetype would have let a hag follow any deity. `deities::some_archetypes_need_one_deity_or_atheism` showed it. Its `-> 267` line is back, with a comment.
- **An engine change in `Repeat::derive` (`libs/rules/src/parameter.rs`), mine.** SecondMystery 277b0164740b97945a3f8022bd572f48 and SecondBloodragerBloodline b7f62628915bdb14d8888c25da3fac56 have a PrerequisiteNoFeature of themselves. Each blueprint is shared by the mythic ability and a planner selection (`oracle/second-mystery`, `bloodrager/second-bloodline`), so the writer emits `Not(Any(itself, selection))`. `derive` read only `Not(itself)`, so their header turned to WhileOptionLeft while the game takes them once. It now also reads `Not(Any(…itself…))` as Once. Rules test: `repeats::a_selection_not_of_an_any_naming_itself_is_taken_once`.
- **Negation reading (`libs/rules/src/engine/labels.rs`, `held_text`), mine.** A `Not` of an `Any` of held entries read "Not with (A or A)". It now names each held entry once ("Not with Second Mystery"). Rules test: `labels::a_negated_condition_says_what_rules_the_option_out`. In seeds, `negated_prerequisites` gains the `deity` kind (oracle, "Not for a follower of Apsu") and the `any held` kind, which must read without parentheses.
- **Older links (main session's rule).** `support::without_mythic_abilities_the_game_refuses_since_308d` drops exactly two things:
  - Over-Infused Blasts in a Mythic slot failing on kineticist 1;
  - Impossible Domain in a Mythic slot failing only on the domain selections, with its MythicSelection pick.

  `first_ascension::every_older_link_…` and `ranks_and_pools::links_made_before_the_splits_…` use it. The latter's count goes from (8, 4) to (8, 5): `targeted living-deity-8`'s Impossible Domain at arcanist 6.
- **Generators (mine), 2,156 Builds before and after (1,990 open):**
  - The Dark Elementalist Builds (`every_class`, t59) are neutral evil.
  - `targeted living-deity-8` takes Second Mystery (Flame) in place of Impossible Domain, which the game refuses before arcanist 20. A Living Deity 20 would have added a `/ legend` copy.
  - `targeted hellknight-pentamic-impossible`'s first level is a cleric of Abadar, so it holds DomainsSelection.
- **Tests left ignored:**
  - 259's two Witch Wandering Hex tests: its `selection` field, not a prerequisite.
  - `pick_values::favorite_metamagic_offers_ten…`: the options' prerequisites, not the ability's.
  - The two Domain Zealot tests: their reason is renamed 265 → 307, since Domain Zealot is blocked on 307's carriers.
- **Gold:** unchanged, all pass. The Enforcer's Defensive Study at MR 1 holds: the Arcane Enforcer has `studied-target`.

**Proof** (`compare`: `FORM_PROOF_OFFERS=1`, `FORM_PROOF_CENSUS` = the log less the Hag's row)

- **Census:** 918 lines in 62 Builds (builds.tsv 62, numbers 532, reopened sheet 26, reopened slots 8, sheets 195, slots 95), 0 unattributed. 0 other differences.
- **Links:** 1,928 the same and 62 refused, 66 refused pick lines.
- **Offers:** 138,439 statuses changed in 669 groups and 1,378 Builds. Each group moves toward the game:
  - Over-Infused Blasts `ok/BreaksLater/AlreadyHeld → Unmet` in non-kineticist Builds: kineticist 1.
  - Defensive Study `→ Unmet` (25 Builds): Sanctified Slayer, Tandem Executioner and swift study.
  - Second Bloodrager Bloodline `→ Unmet` (12 Builds): Reformed Fiend (its own selection dd62cb5011f64cd38b8b08abb19ba2cc) and Mixed-Blood Rager (MixedBloodRagerArchetype 10d92d54d19e4a59b5bbd4259ec0974f removes BloodragerBloodlineSelection at 1).
  - Second Mystery `→ Unmet`: Enlightened Philosopher and Divine Herbalist, whose mysteries are other blueprints (9d5fdd3b…, c11ff5db…). `Unmet → ok` where the selection is held with nothing picked in it.
  - Beneficial Curse `Unmet/AlreadyHeld → ok`: the selection held, and repeats while a curse is left. `→ Unmet`: the Living Deity's curse, 4e7265c0ae1345db90d3375f4ced94cc.
  - Impossible Domain `→ Unmet` (10 Builds): Pentamic Faith, Faith Hunter and Living Deity before 20.
  - Any Means Necessary `→ Unmet` (7 Builds): below Hellknight 7.
  - Kinetic Overcharge, Limitless Rage and Witch Wandering Hex `Unmet → ok`: kineticist 1, rampage or devastating form, witch 1 with no hex.
  - The rest are `ok → BreaksLater` in at most 62 Builds: the refused links' Builds (inference, as 306aa).
- **`306-refused.txt`:** 47 link lines in 4 new blocks:
  - 34 Over-Infused Blasts on a non-kineticist;
  - 9 Dark Elementalist Builds that are true neutral;
  - 3 Impossible Domain (living-deity-8, its full copy, hellknight-pentamic-impossible);
  - 1 Defensive Study (`full t60 sanctified 9 extra slayer talent`).

**Tests**

- Un-ignored: `option_follow_ups::a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain`.
- New:
  - `mythic_class_prerequisites.rs`: ten tests, one per pattern, each citing its blueprint;
  - `option_follow_ups::a_living_deity_below_arcanist_20_cant_take_impossible_domain`, which replaces the Living Deity's pre-20 domain test;
  - the two rules tests above.
- Updated, each cited:
  - `a_living_deity_with_impossible_domain_holds_a_third_domain` (MR 1 at 20);
  - `repeats_from_the_header` ×2;
  - `mythic::beneficial_curse_gives_a_slot_of_the_oracles_curses`;
  - `prerequisites_as_components` (652 → 657);
  - `negated_prerequisites`;
  - `wild_talents::extra_wild_talent_needs_the_wild_talent_selection`, whose exact reason is restored: "Needs the Wild Talents selection".

**Checks** (each once after the last source edit)

- `cargo test -p seeds -p corpus`: game_data 610 passed, 104 ignored; links 6, form_proof 22.
- `cargo test -p rules`: 756 passed. `cargo build -p rules --target wasm32-unknown-unknown`: Finished.
- `cargo test -p game-check`: 158 passed.
- `check-data`: "checked … 64 mythic abilities, 56 mythic feats".
- `cargo fmt --all --check`: clean. `scripts/test-modules.sh`: "test-modules OK".
- Game check: exit 0, "3565 known", no stale line. Dry run: "0 fields to write".

One shell edit slipped through: a `sed -i` on one line of `mythic.rs` (a type path). No trailing space was stripped.
