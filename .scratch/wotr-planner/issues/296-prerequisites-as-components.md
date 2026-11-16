# Prerequisites as components

Type: build (data form)
Status: resolved
Blocked by: 295
Model: Opus to build: every entry's prerequisite changes form, and the proof must show no behaviour moves.
Map: [WotR Build Planner](../map.md)

## Problem

The component vocabulary ([295](295-the-component-vocabulary.md)) gives the engine the game's prerequisite components. The user decided its staged plan on 2026-11-15. Each blueprint entry still holds the hand-shaped `prerequisite` field. This ticket moves them to the vocabulary's `Requires` form, in the same values and a new form, so the importer ([293](293-the-importer.md)) later writes one form.

The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4.

## Change

- **A rewrite script, kept outside the repo,** turns every blueprint entry's `prerequisite` into `Requires` components from the current data, not from the game.
- **`GameData::check`** refuses an entry holding both the field and the component.
- **The field goes** where nothing still needs it. Planner-made entries keep what they hold.

## Answer

**Built** (2026-11-16) as written: 608 blueprint entries hold their prerequisite as `Requires` components, with the
same values; the 62 others keep the field for a node with no condition yet (below), and so do the planner's entries.
Note: `libs/game-data/notes/296-prerequisites-as-components.md`.

**The rewrite** is a script outside the repo, a small Rust crate on the worktree's `rules` (kept at
`<scratchpad>/game/296/rewrite/`, `cargo run --release -- <game-data dir>`), run on the data at `e1022ba2`, not on
the game. For each entry of `feats/`, `classes/`, `prestige/`, `archetypes/`, `options/`, `mythic/abilities/`,
`mythic/feats/` (never `planner/`) it maps the tree to conditions, asserts that they compile back into the very tree
(both canonical), replaces the field's text in place with `components: [...]` in the files' style (one line in a
one-line file, one component a line in a pretty one; the writer reproduces every old field's text exactly, asserted),
and reads the file back: field gone, components as written, the rest of the entry equal. Re-run on its output it moves
nothing. Files changed: feats 197, options 309, mythic feats 44, mythic abilities 24, archetypes 18, classes 5,
prestige 11 (608; 1,497 components, 124 entries with an `Any` group).

**The form** (mine, the game's where it has one): a top-level `All`'s conditions in the `All` group, and a lone `Any`
among them (not of alignments only) in the `Any` group, as `BlueprintFeature.MeetsPrerequisites` reads the game's
groups; held facts as `Has`, their `Not` as `HasNot`; `Not(Archetype)` as `NoArchetype`; an `Any` of alignments as one
`Alignment` set (`PrerequisiteAlignment`'s mask); the stats as `Stat`; spell levels as `CasterTypeSpellLevel`; nested
groups as the condition's own `All`/`Any`/`Not`. The note has the full mapping.

**Readers keep the same text** (mine): `GameData::checked` compiles each entry's `Requires` into its `prerequisite`
(canonical, as the files' trees are read) and takes them out of its components (`GameData::compile_prerequisites`).
Every reader goes through it (the web server, the MCP server, the game check, the corpus, `check-data`, the browser,
which checks the JSON again), so the MCP's prerequisite text, describe, search and the UI read the tree they read
before. The engine's catalog no longer compiles `Requires` itself. The JSON the server sends is byte-for-byte the
document of `e1022ba2` (2,097,210 bytes, `cmp` of the base data and the tip's, both through the tip's
`check_game_data_files`). The seeds tests' `DATA` is now the checked data (`support.rs`), so their assertions read what
the readers read, unchanged.

**The field stays** (`prerequisite` is needed): for the planner's entries, for the compiled form, and for the 62
blueprint entries whose tree holds a node with no condition yet. Each is the whole tree, since the check refuses a
field beside a `Requires`:
- **Planner logic, stays a field:** `Story` (the 4 Gold Dragon breaths; `GoldDragonBreathAnnihilating`
  e8b9aad5295c49539f1bbf901d314964 needs `CorruptedGoldenDragonFeature` 1e66cfb4f351466fb05636082bce50d6, a story
  fact; story is out of scope).
- **Gaps for 298:** `FeatForSameValue` (8 feats, 3 mythic feats: the parameterised feature's `m_Prerequisite`, the
  header's `values_of`); `FeatIfValue` (Armor Focus: the game's `ArmorFocusSelection` 76d4885a395976547a13c5d6bf95b482
  is a selection of one feature per armour type).
- **Gaps for 293 (with 298 for the selections):** `Worships` (cleric, inquisitor, warpriest, Divine Hunter, Deliverer:
  `ClericClass` 67819271767a9dd4fbfd4ae700befea0 is `PrerequisiteNoFeature` `AtheismFeature`
  92c0d2da0a836ce418a267093c09ca54, a fact the planner doesn't hold) and `Atheist` (Living Deity); `DeityOffers`
  (Angelfire Apostle, Priest of Balance, the 4 alignment blessings) and `PickedThrough` (the 4 alignment domains and
  the 4 blessings): the deity's `…DomainAllowed` facts (`LawBlessingFeature` 9c49504e2e4c66d4aa341348356b47a8 is
  `PrerequisiteFeature` `LawDomainAllowed` 092714336606cfc45a37d2ab39fabfa8); `NoOtherOptionOfGroup` (the 28 draconic
  and elemental bloodlines: a `PrerequisiteCondition` tree over the others, `BloodlineDraconicRedProgression`
  8c6e5b3cf12f71e43949f52c41ae70a8) and `NoOptionFrom` (Dragon Disciple, `DragonDiscipleClass`
  72051275b1dbb2d42ba9118237794f7c, likewise).
- **Gap for 297, then 293:** the Arcane Trickster's `AtLeast(2, [..., Feature(sneak-attack, count: 2)])`: the
  vocabulary's `AtLeast` counts facts, and the game's is `PrerequisiteFullStatValue` `SneakAttack` 2
  (`ArcaneTricksterClass` 9c935a076d4fe4d4999fd48d853e3cf3), a stat the planner doesn't count.
Moving any of them now would change a value (another tree, another reason's text).

**Tests:**
- `libs/rules/tests/engine/components.rs`: `checked_data_holds_the_requires_compiled_into_the_prerequisite` (the
  checked data holds the canonical tree, the other components stay, and it checks again; failed first with `None`).
- `libs/seeds/tests/game_data/prerequisites_as_components.rs` (on the files as read, before the check):
  `a_blueprint_entry_holds_its_prerequisite_as_requires_components` (every field left has a node with no condition,
  with the entries holding each; 608 moved, 62 kept; no planner `Requires`);
  `power_attacks_strength_13_is_a_requires_of_the_stat` (`PowerAttackFeature` 9972f33f977fc724c838e59641b2fca5:
  `PrerequisiteStatValue` Strength 13, group All); `crane_styles_groups_are_the_games` (`CraneStyleFeat`
  0c17102f650d9044290922b0fad9132f: `PrerequisiteFeature` `Dodge` 97e216dbb46ae3c4faef90cf6bbe6fd5 and
  `ImprovedUnarmedStrike` 7812ad3672a4b9a4fb894ea402095167 in All, `PrerequisiteClassLevel` `MonkClass`
  e8f21e5b58e0569468e420ebea456124 1 and `PrerequisiteStatValue` `BaseAttackBonus` 2 in Any; its Intelligence 3, not
  in the data, always holds). All three failed on the data of `e1022ba2`.

**The proof.** The form proof, `record` at `e1022ba2` in a temporary worktree (removed after) and `compare` on this
tip: 2,162 Builds compared (2,001 open), 2,162 old links reopened, 0 differences. The JSON the browser gets is
identical (above), so `allows` and every picker read the same data.

**Checks**, after the last source edit:
- `cargo test -p rules -p seeds -p corpus -p mcp -p ui -p game-check`: every binary ok (rules engine 722 passed,
  1 ignored; seeds game_data 497 passed, 134 ignored, the gold builds among them; seeds files 34; corpus self_checks 1;
  mcp server 246; game-check fixture 30; ui 6).
- `cargo run -p seeds --bin check-data`: `checked …: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, …` (exit 0).
- `cargo build -p rules --target wasm32-unknown-unknown`: finished.
- The game check (`--binz` the main checkout's dump), at `e1022ba2` and on this tip: both exit 0, and the reports
  differ only in the last line's run time (`diff`: `game-check: 1.0 s` / `0.7 s`).
- The form proof `compare`, rerun after the last edit: 2,162 Builds compared (2,001 open), 2,162 old links reopened,
  0 differences.
- `cargo fmt --all --check`, `cargo clippy -p rules -p seeds --all-targets -- -D warnings`: clean.

**Simplify** (the skill, four reviewers): the seeds tests' data checked once (`CHECKED` first, `DATA` its clone);
one read of the files for the new tests; `Component::is_requires` in place of three copies (check.rs's private one
moved there); the new test holds references and `DataAt`, and walks with `Prerequisite::leaves()`. Not taken: reusing
the check's compiled trees in `checked()` (the check would change shape; the cost is one more compile of 608 small
trees at load); moving `NoSelectionIf`'s lowering from the catalog into `checked()` (298's); a game check comparing the
raw `Requires` with the blueprints one to one (293's).

## Done when

- No blueprint entry holds `prerequisite` but through `Requires`.
- The proof is clean:
  - the corpus Builds' names, `Debug` and sheets hashes, and every link's decode, all identical before and after (as 244 did);
  - the self-checks, the gold builds and every real-data test unchanged;
  - `allows` identical over the corpus.
- The scoped checks pass, and the game check exits 0.
