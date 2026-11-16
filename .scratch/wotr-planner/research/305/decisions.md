# 305 decisions (main session, 2026-11-16), on the research report
(Research copies: <scratchpad>/305/.)

- Shape: `held: BTreeMap<Fact, HeldFact { holds: u8, values: BTreeMap<Parameter, u8> }>`, Fact = component.rs's enum (grantable kinds), with per-kind views iterating in today's order; zero-hold entries removed. Absorbs feats/feat_values, features, selection_options/option_values, mythic_abilities (+values, open holds), mythic_feats (+values), selections. Today's rank arithmetic per kind kept.
- Hold selections of feats as facts, as the game does (AddFeaturesFromProgression): research shows no offer, number or Requires result changes; drop the `offers_feats` filters at the row-hold sites and hold path-rank, race/subrace and general-slot feat selections. Test it, citing BasicFeatSelection and AddFeaturesFromProgression.
- Q1: key by `Fact` (+ parameter), not GUID. GUID identity is 293's, with the 90 shared-blueprint collisions listed in the note (Stunning Fist feat vs monk feature, Wings, the rogue-talent Weapon Focus option, 39 feature/option pairs, repeated archetype features, 302's selections carrying their feat's GUID).
- Q2: no rank cap at `ranks` in 305 (features count uncapped since 243); a follow-up after 303.
- Q3: 302's nested selections are not held as a second fact; the feat stands for them.
- Q4: option caches (options_through, option_groups, option_selections) stay as they are.
- Q5: no source/level yet.
- Proof: the form proof expects 0 differences (no kind column change); links test; scoped checks.
