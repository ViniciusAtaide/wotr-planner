# Mythic Spell Focus doubles the school's focus feats

Type: build (engine rule)
Status: open
Blocked by: none
Model: Opus to build: the caster line's DC parts.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15), half A (`caster_line.rs`).
- **Mythic Spell Focus** (`SpellFocusMythicFeat` 41fa2470ab50ff441b4cfbb2fc725109) has no DC bonus of its own: while it is held, each Spell Focus and Greater Spell Focus of its school gives +2 instead of +1 (`Designers/Mechanics/Facts/SpellFocusParametrized.cs`, `GetBonusDC`, WrathDecompiled 6ce4a6b; `SpellFocus` 16fa59cc9a72a6043b566b49184f53fe, `SpellFocusGreater` 5b04b45b228461c43bad768eb0f7c7bf). With both feats that is +4, where the planner counts +3.
- **Expanded Arsenal** (`ExpandedArsenalSchool` f137089c48364014aa3ec3b92ccaf2e2): for a spell of an arsenal school, the largest per-school sum of the focus feats' `GetBonusDC` (`Designers/Mechanics/Facts/ExpandedArsenalMagicSchools.cs`, `OnEventAboutToTrigger`): Enchantment's 2 + 2 = 4 for a conjuration spell, not 3. Open (242's inference): each Expanded Arsenal fact may run the handler, so a Build with two may add it twice; read the code and record the answer.
- **Arcane Arcana** (`BloodlineArcaneArcana` e8e4f56618dd8b04490aa6a0b75ac24f) adds +1 to a spell with metamagic but not Heighten (`ArcaneBloodlineArcana.OnEventAboutToTrigger`): agrees.

## Change

- The caster line counts Mythic Spell Focus as doubling the school's Spell Focus and Greater Spell Focus, and Expanded Arsenal as the game's largest per-school sum; the breakdown says so.

## Tests 242 marks as waiting on this rule

- `caster_line.rs`: `the_max_dc_runs_enchantment_compulsion_dc_at_20_with_its_breakdown` (SF 2, GSF 2; DC 26 + level, 35, 36), `the_max_dc_runs_dcs_by_level_follow_its_picks` (25, 27, 32, 34, 35 at 7, 8, 15, 16, 20; 1 to 6 agree), `the_max_dc_runs_expanded_arsenal_gives_conjuration_enchantments_spell_focus_feats` (4 at least).

## Done when

- The tests above hold the game's values, cited; the Expanded Arsenal question is answered from the code in the Answer.
- The scoped checks pass.
