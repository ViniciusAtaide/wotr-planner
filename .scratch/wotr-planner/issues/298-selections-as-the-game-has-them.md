# Selections as the game has them

Type: build (data form and engine)
Status: open
Blocked by: 297
Model: Opus to build: slots, lists and repeats change shape, and every link must keep opening.
Map: [WotR Build Planner](../map.md)

## Problem

The staged component plan (user, 2026-11-15; [295](295-the-component-vocabulary.md)) ends its form changes here.

**What the planner holds today** that the game doesn't:
- feat slots and feat lists, where the game lists every option explicitly (`BasicFeatSelection` names 276 feats);
- `extra_pick_of`, `MythicExtraPick` and a mythic ability's own selections;
- `Repeat`;
- `option_levels` and its kin;
- five held-fact maps in the state.

**What [244](244-the-game-data-laid-out-by-blueprint.md)'s layout left here** (main session, 2026-11-15): `tools/game-check/known.txt`'s former 244 lines, now under 298. They are:
- the per-class selections the game keeps separate: bard, skald, slayer 6 and 10, Dark Lurker, Instinctual Warrior, Exploiter, Provocateur, Hexcrafter, Order of the Paw, Elemental Witch, Nine-Tailed Heir, the third elemental focus;
- entries holding several blueprints: Blessing and Second Blessing, the Aeon gaze tiers, the kineticist element feat lists, Advanced Weapon Training, Extra Channel, Extra Hex, Magical Tail, Abundant Smite;
- the oracle mystery copies (Divine Herbalist, Enlightened Philosopher) and the revelations' prerequisites on them, and `shaman/hex`;
- the 11 differing selection groups, which need research in the game;
- the ranger archery feat-list pairs.

The domains are [294](294-the-domain-split-with-a-slot-aware-link-migration.md)'s. The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4.

## Change

- **The game's shapes:**
  - feat slots and lists become selections;
  - nested selections replace `extra_pick_of`, `MythicExtraPick` and a mythic ability's selections;
  - `Repeat` is derived from the header (`Ranks`, selection or parametrised, `PrerequisiteCondition`);
  - progression header fields replace `option_levels` and its kin;
  - the held-fact maps become one, with the kinds kept as views for links and the sheet.
- **244's leftovers above** are split or remapped to the game's own selections, each read from its blueprint. Where a split changes an option key a link names, the old key reads through an alias or a slot-aware migration (as 243 and 294 do).

## Done when

- Every link opens with the same picks: a test over the corpus links and the bench links.
- The proof is clean: the hashes, the self-checks and the gold builds, with no real-data expected value changed but where the game says so, each cited.
- `known.txt`'s 298 lines are gone, and the game check exits 0.
- The scoped checks pass.

## Build notes (2026-11-16, partial: the ticket stays open)

**Built:** the three low findings from 297's review, in `libs/rules/src/check.rs`, one test each:
- an `ExpandedArsenal` whose `focuses` name an entry with no `SpellFocus` is refused (`DataIssue::NotASpellFocus`;
  `check::an_expanded_arsenals_focuses_each_hold_a_spell_focus`);
- `HighestAbilityBonus`, `AbilityAtLeast`, `BaseAttackAtLeast` and `GoldenDragonSkills` sit only on a feature, as
  `Scales` does (`check::the_gold_dragons_kinds_sit_on_a_feature`);
- a `Scales` reads a level (a class level naming a class, or the character level: `DataIssue::ScalesWithoutLevel`), and
  a `Custom` progression with no band is refused anywhere (`DataIssue::EmptyCustom`;
  `feature_steps::a_scales_reads_a_level_and_a_custom_progression_has_a_band`).

The real data passes all three (`check-data` exit 0). No data, no link and no engine behaviour moved.

**Not built, and why** (builder, 2026-11-16): the rest is several tickets' work and needs decisions first.
- **The game's lists change offers.** Using each per-class selection's own game list (what the game check compares)
  removes talents some classes are offered today: bard (`BardTalentSelection` 40f85fbe8cc35ef4fa96c66e06eeafe8: 9
  talents with no `Requires`, and 10 advanced talents from the Archaeologist's 10th level), skald
  (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57: 7), the Inciter's (`ProvocateurTalentSelection`
  aa88cb5263a54791b8d1e471bc192ad4: 6), the slayer's three lists (4, and 2 more from 10), the skald's and barbarian's
  rage powers (the Instinctual Warrior's 5). Copying the planner's lists instead keeps the offers but fails the game
  check. Which to take, and what a link picking a removed talent opens as, is a decision.
- **Engine questions the splits raise:** `extra_pick_of` gives a slot of the first listed selection reached, so a
  slayer's Extra Slayer Talent would stay on the 2nd-level list (losing the 6th- and 10th-level combat style feats);
  `once_per_option` doesn't reach across selections, which the 15th-level element (`ThirdElementalFocusSelection`
  e2c1718828fc843479f18ab4d75ded86) needs; the Dark Lurker's talent rows are inherited and need rows of its own.
- **Link migrations:** the oracle mystery copies, the shaman hex copies, Extra Hex and Extra Channel change keys that
  corpus links name (92 corpus Builds name a mystery, 120 a witch hex, 21 Extra Hex, 7 Extra Channel): they need the
  slot-aware migration in `Engine::open_link` (an option's or feat's alias read only where the old key no longer
  holds), not built.
- **The form steps** (feat slots and lists as selections, nested selections, `Repeat` from the header, the progression
  header, one facts map) are each about 297's size.

Research for the split: `<scratchpad>/game/298/research/selections-spec.md` (the main session's scratchpad), with the
proposed keys and every place that names a shared selection. The form proof's record at `7a793c6a` is in
`<scratchpad>/game/298/proof-before/`.
