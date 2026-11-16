# Selections as the game has them: the link migration

Type: build (engine)
Status: resolved
Blocked by: 297
Model: Opus to build: every link must keep opening.
Map: [WotR Build Planner](../map.md)

## Rescoped (user, 2026-11-16)

The first build (Build notes below) found this ticket is about six tickets of 297's size. The user chose to split it ("B: forms now, splits in 293"):
- **This ticket** keeps the data check's three fixes already built, and builds the **link migration**: an old option or feat key keeps opening after a later ticket renames or splits it.
- **The five form steps** become [301](301-feat-slots-and-lists-as-selections.md), [302](302-nested-selections.md), [303](303-repeat-from-the-header.md), [304](304-the-progression-header.md) and [305](305-one-held-facts-map.md), in that order.
- **The per-class selection splits and remaps** (244's leftovers, `known.txt` section 298) move to the importer, [293](293-the-importer.md), which writes each selection from its blueprint. Their lines move to `known.txt` section 293 in this ticket.

### Change (as rescoped)

- **A slot-aware link migration** in `Engine::open_link`. An option's or feat's `aliases` may name a key that is no longer live. When a link's pick no longer holds at its slot, the decoder replaces it with the one live entry whose alias names it there. When none or several do, the pick is refused as today. This is the shape 243's and 244's aliases began; 294 and 293 use it.
- **A test over every corpus link and every bench link:** each opens with the same picks. A planted rename with an alias still opens, and one without fails the test.
- **`known.txt`:** section 298's lines move to section 293, unchanged.

### Done when (as rescoped)

- The link test passes over the corpus and the bench, and a planted rename with an alias passes it.
- The form proof (`compare` against the record at `7a793c6a`) shows 0 differences.
- The game check exits 0 with the moved section. The scoped checks pass.

The sections below are the ticket as first written, kept for its history.

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

**A correction to the research above** (main session, 2026-11-16): two blueprints are named `BardTalentSelection`. The research read 40f85fbe8cc35ef4fa96c66e06eeafe8, which only units reference. `BardProgression` 8127f5ff40f5b484b8be98609358b9d2 grants 94e2cd84bf3a8e04f8609fe502892f4f, whose list holds EmboldeningStrike, TerrainMastery, BlindingStrike, Familiar and PetrifyingStrike. The bard counts above are wrong, and the other selections' counts are unchecked. 293 identifies each selection through the progression or archetype that grants it (`game-check refs`), never by name.

## Answer

**Built** (2026-11-16) as rescoped: the data check's three fixes (`1d10d1ab`, Build notes above) and the link migration.
No data and no game fact changed, so there is no data note; the precedent is 243's and 244's `aliases`.

**The migration** (`Engine::open_link`, `libs/rules/src/engine/walk.rs`): a link decodes, and when its Build isn't
possible, while the first failure is a pick at a slot whose key an option's or feat's `aliases` name
(`LinkKeys::heirs`), each such entry is tried in the pick's place with the same value, and the one whose slot then
fails nothing replaces it. None or several: the link is stale. Each slot migrates once. A possible Build is never
touched, so its cost stays one walk. Only a pick whose key the data lacks or its slot doesn't take migrates
(`Issue::UnknownKey` or `NotAllowed`): one its slot still offers that fails there stays refused (main session, from
298's review; `a_pick_its_slot_offers_that_fails_its_prerequisite_is_refused_not_migrated`).

**The decoder** (`LinkKeys`, `libs/rules/src/codec.rs`) keeps, per kind and hash, the text hashed, whether it's a live
key and the live keys naming it as an alias. A live key reads as itself (even when aliases name it: 294's shape); an old
key one entry names reads as that entry (243's behaviour, kept); an old key several name reads as itself, a key no
entry has, which the migration then reads by its slot (293's shape). Collisions are still between different texts.

**Decisions:**
- Feats get `aliases` too (`Feat::aliases`, default empty, not written when empty): the main session's brief, for 293's
  Extra Hex and Extra Channel.
- An alias may name a key still live, or one other entries name (builder: 294 and 293 need both). The data check
  (`check.rs`, `Checker::aliases`) now refuses only an entry's own key as its alias and a malformed alias; 243's test
  (`ranks_and_pools::a_link_naming_an_options_old_key_reads_as_the_option`) changed to say so.
- "Holds at its slot" is the walk's own judgement, no failure at that slot with the heir in place (builder). The
  simplify review suggested the cursor's `pick_fits` instead; kept the walk, the same judge `is_possible` uses.
- The corpus link test runs in the suite (builder): about 45 s in debug.
- `tools/game-check/known.txt`: section 298's 83 lines move to section 293, unchanged but the ticket (user, rescope).
  The game check exits 0.
- `corpus::bench` is public, with `links` and `open`, so the link test and `bench_builds` read the benchmark one way
  (simplify review).

**Tests:**
- `libs/rules/tests/engine/link_migration.rs`, planted renames in the test world (the rogue's Fast Stealth, shared with
  the slayer; a copy of Toughness): `a_renamed_options_old_key_opens_on_its_new_key_with_an_alias_and_is_refused_without`,
  `a_split_that_keeps_the_old_key_opens_each_slot_on_the_entry_it_offers`,
  `an_old_key_two_copies_name_opens_each_slot_on_the_copy_it_offers`, `a_pick_two_live_aliases_hold_at_is_refused`,
  `a_pick_no_alias_holds_at_is_refused`,
  `a_renamed_feats_old_key_opens_on_its_new_key_with_an_alias_and_is_refused_without`,
  `a_split_feats_old_key_opens_on_the_one_copy_that_holds_at_its_slot`.
- `tools/corpus/tests/links.rs`: `every_corpus_and_bench_link_opens_with_the_same_picks` (each of the 2,162 corpus
  Builds' links opens as the Build, but the 161 that aren't possible, which open nothing as at `7a793c6a`; each of the
  5 benchmark links opens with the picks it decodes to, a pick changing key only to an entry whose alias names it),
  and `a_planted_rename_opens_through_its_alias_and_is_refused_without_one` (legend40's first feat nothing else names,
  renamed in the real data: with the alias the link opens on the new key and passes the check above; without, it
  doesn't decode).

**The form proof:** `compare` against the record at `7a793c6a`: 2162 Builds compared (2001 open), 2162 old links
reopened, 0 differences.

**Not done:** no data uses an alias in the new ways yet; 293 and 294 will. The migration's cost on a link that needs
it (a walk per heir per migrated pick) is untimed.

