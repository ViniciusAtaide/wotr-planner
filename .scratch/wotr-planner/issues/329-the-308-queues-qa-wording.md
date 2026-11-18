# The 308 queue's QA: three reasons that read wrongly

Type: build (texts and data form)
Status: resolved
Blocked by: 325
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

The QA session after the 308 queue (2026-11-17, `main` a129b11f) found every ticket's behaviour right, and three texts the queue made read wrongly:

1. **Several selections named alike.** The prerequisites 306aa and 308c/308d wrote as `Has(Selection)` name each selection by its display name only, so the reason can't tell them apart. `validate` on a Battle Scion 5 with `slot.feat.5.general.0: extra-rage-power` returns `needs: ["the Rage Power selection or the Rage Power selection or the Rage Power selection"]` (the barbarian's, the Instinctual Warrior's and the skald's). The same in Extra Rogue Talent ("the Fey Tricks selection or (the Rogue Talents selection or the Rogue Talent selection) or the Rogue Talents selection"), Extra Arcanist Exploit ("the Arcanist Exploits selection or the Arcanist Exploits selection") and Impossible Domain ("(the Domain selection or the Domain selection or the Domain (Sacred Huntsmaster) selection) or …").
2. **A level clause twice.** Kinetic Knight 5, `options slot.selection.5.0`: Chain Arrows reads "Needs (Kineticist level 16 (have 5), the Kinetic Sharpshooter archetype), Kineticist level 16 (have 5)"; Honed Infusion (12) and Rending Arrows (11) the same. `options/kineticist/infusions/chain-arrows.ron` (308f) holds both a `ClassLevel` and an `ArchetypeLevel` group.
3. **Pentamic Faith** (308f) reads "not with Pentamic Faith or the Pentamic Faith selection, Order of the Godclaw".

## Change

1. Name a selection by its owner when its name alone is ambiguous in one reason: the class or archetype it belongs to ("the barbarian's Rage Power selection", or the planner's existing owner wording in `libs/rules/src/engine/labels.rs`). One wording for all callers (`CODING_STANDARDS.md`, "One wording per refusal").
2. Find why the class level prints twice (the data's two groups, or the label's flattening) and fix the cause so the game's tree reads once; if the data is the game's (the blueprint's components), fix the label.
3. Word Pentamic Faith's own refusal so it reads as the game's rule (its own blueprint and its selection).

## Done when

Each of the three reads clearly through `validate`, `options` and `choose` (MCP tests in `apps/mcp/tests/server/` or the rules' label tests asserting the exact text), and the scoped checks pass.

## Answer

Built on `ticket/329-qa-wording` from a129b11f. Labels only: no data, no status and no prerequisite tree changed.

1. **Selections named alike.** Cause: `Prerequisite::Selection` read "the {name} selection" alone. Now `Engine::selection_text` (`libs/rules/src/engine/labels.rs`) names the owner, "the Barbarian's Rage Power selection", when another selection that rows name shares the name (`Catalog::selection_shares_name`, from `FeatureFacts`). The owner is `selection_owner`, which already names an option's selection when two options share a name ("Arcane (Sorcerer Bloodline)"). Extra Rage Power now reads "the Instinctual Warrior's Rage Power selection or the Barbarian's Rage Power selection or the Skald's Rage Power selection". Extra Arcanist Exploit reads "the Arcanist's … or the Exploiter Wizard's Arcanist Exploits selection", Extra Rogue Talent "… (the Rogue's Rogue Talents selection or the Rogue Talent selection) or the Dark Lurker's Rogue Talents selection", and Impossible Domain "(the Cleric's Domain selection or the Inquisitor's Domain selection or …)". Decisions (builder):
   - Whether a name is shared is decided over the game data, not within one reason, as `feature_name_apart` decides it for features. A leaf's text can't see its reason's tree, and threading the tree through every label function wasn't worth it for this.
   - Only selections that rows name count. An Extra feat's selection takes its class selection's name, as the game's does (the test world's `rogue/extra-rogue-talent`). It has no owner to tell it apart, and counting it would have made a lone "the Rogue Talents selection" read "the Rogue's …" where nothing else is meant (`held_selections`'s pinned text stays).
2. **The doubled level clause.** Cause: the label. The game's ChainArrowsInfusion 0ff81c5691c64c4aa8b2ecc35c885901 has a `PrerequisiteClassLevel` (KineticistClass 42a455d9ec1ad924d889272429eb8391, 16) and a `PrerequisiteArchetypeLevel` (KineticSharpshooterArchetype e0684d0a6ef44cae9d095f5318618a85, 16), both of group All, so the data's two groups are the game's tree. `compile_requires` makes the archetype's an `All([Archetype, ClassLevel])` inside the outer `All`, and the label bracketed the inner "and". Now an "and" inside an "and" reads as its parts, each once. This holds in the reason (`Unmet::and_parts`, used by `unmet_reason` and `unmet_all_texts`), in the prerequisite text (`all_texts`) and in `validate`'s `needs` (`apps/mcp/src/validate/tree.rs`). Before: "Needs (Kineticist level 16 (have 5), the Kinetic Sharpshooter archetype), Kineticist level 16 (have 5)". After: "Needs Kineticist level 16 (have 5), the Kinetic Sharpshooter archetype". `needs` is now `["Kineticist level 16 (have 5)", "the Kinetic Sharpshooter archetype"]`. Rending Arrows (8896ac251d394dceaf650793aa85e177, 11) and Honed Infusion (cf531860c9fb44c09bda9d8904657314, 12) read the same way. The form is the planner's, not changed in `canonicalize`, so no form proof (builder).
3. **Pentamic Faith.** The text was `describe`'s and `search`'s prerequisite. `options`, `choose` and `validate` show only "Needs Order of the Godclaw" until the Not fails, and a second pick is refused as already taken. HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894 has a `PrerequisiteFeature` (HellKnightOrderOfTheGodclaw 5636564c278583342aec54eb2b409029) and a `PrerequisiteNoFeature` of itself, and the planner keeps that one blueprint as an option and a selection. `held_text`'s "with any of" now leaves out a selection whose blueprint is an option's beside it. An "and"'s prerequisite text now puts what it refuses after what it needs (`all_texts`). Reasons keep refusals first (ticket 65). Before: "not with Pentamic Faith or the Pentamic Faith selection, Order of the Godclaw". After: "Order of the Godclaw, not with Pentamic Faith" (builder).

Tests:
- `apps/mcp/tests/server/wording.rs` (new, real data):
  - `real_data_selections_named_alike_read_with_their_owners` (`validate`, `options`, `choose`);
  - `real_data_an_archetype_level_reads_its_class_level_once` (`options`, `choose`, `validate`);
  - `real_data_pentamic_faith_refuses_itself_once` (`describe`);
  - `real_data_impossible_domain_tells_its_domain_selections_apart` (`describe`).
- `libs/rules/tests/engine/labels.rs`: `an_and_inside_an_and_reads_its_parts_once_and_its_refusals_last`.

Each MCP test cites its game source:
- ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a: RagePowerSelection 28710502f46848d48b3f0d6132817c4e (BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12), InstinctualWarriorRagePowerSelection 609f0e5336084442a0dafa3abd4d31c5 (InstinctualWarriorArchetype adffdd8a99094a89823a79292a503ee9), SkaldRagePowerSelection 2476514e31791394fa140f1a07941c96 (SkaldProgression 26418fed2bc153245972a5b54204ed75).
- ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a: ArcanistExploitSelection b8bf3d5023f2d8c428fdf6438cecaea7 (ArcanistProgression 974911193b8a8b947a8275f96bb265d6), ExploiterExploitSelection 2ba8a0040e0149e9ae9bfcb01a8ff01d (ExploiterWizardArchetype 6d2027b4eec590c41adb5caeb7bb026d).
- ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12: DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2 (DarkLurkerArchetype 33eb002078e5437b8c25db4b80b4a4fe), RogueTalentSelection c074a5d615200494b8f2a9c845799d93 (RogueProgression b57b2a75a5abcaf47a01cf84672b50e9).
- ExtraDomain 213a8480d22206b45acbfa0619ca5aaf: DomainsSelection 48525e5da45c9c243a343fc6545dbdb9, the planner's `cleric/domain` and `inquisitor/domain`.

The order of the alternatives is the canonical tree's, not a game fact.

Checks:
- `cargo test -p rules`: 757 passed;
- `cargo test -p mcp`: 250 passed;
- `cargo test -p seeds -p corpus`: 646 passed, 89 ignored;
- the rules wasm build;
- `cargo fmt --all --check`;
- `scripts/test-modules.sh`;
- `scripts/guarded-files.sh a129b11f`: exit 0.
