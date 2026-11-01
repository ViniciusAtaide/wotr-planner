# Build the rules engine

Type: task
Status: resolved
Blocked by: 09
Map: [WotR Build Planner](../map.md)

## Question

Implement [How a character progresses](04-progression-model.md) in the `rules` crate, test-first (`tdd`) against hand-written fixtures.

- The `Build` type: Build start, then Level-ups, then Mythic ascensions, plus the Path switch. It also needs the **URL codec** from [Saving and sharing Builds](13-saving-sharing-builds.md): version byte, compact binary of text keys plus optional name, compression, base64url. Encode and decode must round-trip, old versions must keep decoding, and unknown keys are reported rather than failing.
- Derived state at any character level:
  - class levels, BAB, saves
  - spell-level access, including the prestige advancement chosen on entry
  - ability scores, including the +1 every 4 levels up to 40
  - skill ranks, capped at character level
  - feats and features held
  - mythic rank and path
- **Slots at each level:** general slots at odd levels, bonus slots from race, class or archetype with their allowed lists (combat feats per `progression.ron`), feature-selection slots, and mythic ability or feat slots and path picks. Mythic ascensions happen at their fixed levels; level 17 has two ranks plus the possible path switch.
- **Available options:** the options for a slot, filtered by evaluating prerequisites against the derived state at that level, with the reason each excluded option fails.
- **Would-break check** (needed by the chosen Sheet screen, which blocks breaking edits): for a proposed edit (pick, class or archetype change, or clear), return the later picks it would invalidate, including chains (removing Cleave also breaks Great Cleave).
- **Whole-Build validation:** used when a link is opened. It reports picks that no longer qualify and keys missing from the current game data, for the stale-link banner.
- **Legend:** keeps only MR 1–2 picks and stays at rank 3. Levels 18–40 get feats at odd levels and ability increases every 4 levels. Caps of 20 levels per class and 10 per prestige class.

AFK: driven by the agent.

## Answer

> **Superseded in part, 2026-10-31,** by [Apply the schema and layering review](14-review-fixes.md). A class's
> archetype and advancement now live in `Build.class_choices` (`LevelUp` has only the class, the ability increase and
> skill ranks), and `Edit::Class` is `{ level, class, choice: Option<ClassChoice> }`. `SlotOption` has no
> `would_break`, `is_combat_feat` is `FeatCategory::is_combat`, path ranks are constants, `LevelCaps` is
> `MythicPath.level_cap`, `rank_after_switch` is derived, and there is no row 0. `Pick::Option` is
> `Pick::SelectionOption`. The `20261031000300_drop_positions.sql` migration was folded into the one migration, and
> the frozen v1 link was regenerated before any link shipped. The rest of this answer is the state when 11 closed.

Built test-first and verified 2026-10-31 (uncommitted). The `rules` crate now has the `Build`, the URL codec and
an `Engine` that walks a Build against `GameData`. It stays pure (serde, postcard, miniz_oxide, base64) and builds for
`wasm32-unknown-unknown`. Mid-ticket, the user also asked for **no derived values in the database**; that change
to `rules` types, `data` and a new migration is described at the end.

### Public API (what the sheet screen calls)

| Item | What it is |
| --- | --- |
| `Build { name, start: BuildStart, levels: Vec<LevelUp>, mythic_path, path_switch, picks: BTreeMap<SlotId, Pick> }` | the choices only, all as text keys. `levels[0]` is level 1 |
| `BuildStart { race, subrace, racial_ability, alignment, ability_scores }` | `racial_ability` is where a "+2 to one ability" goes |
| `LevelUp { class, archetype, advances, ability_increase, skill_ranks }` | `archetype` and `advances` (spellcasting class(es) a prestige class advances) are read on the class's first level only |
| `SlotId::{Feat { level, source: Race/Class/General, index }, Selection { level, index }, Mythic { rank }, PathFeat { rank, index }, PathSelection { rank, index }}` | stable slot names; indexes count within their source |
| `Pick::{Feat, Option, MythicAbility, MythicFeat}` | what fills a slot |
| `Edit::{Pick, Clear, Class { level, class, archetype, advances }, AbilityIncrease, SkillRanks, Start, MythicPath, PathSwitch}`, `Build::apply(&Edit)` | the sheet's edits. `Class` at `len + 1` adds a level, and archetype/advances follow the class's first level when a class change moves it |
| `Engine::new(GameData)` | build once (canonicalises and indexes the data); `engine.data()` |
| `engine.evaluate(&Build) -> Sheet` | everything in one walk |
| `Sheet { levels: Vec<LevelSheet>, ranks: Vec<RankSheet>, level_cap, problems }`, `sheet.at(level) -> Option<&CharacterState>`, `sheet.slot(&SlotId)` | |
| `LevelSheet { character_level, class, class_level, ability_increase, skill_points: SkillPoints { available, spent }, slots: Vec<Slot>, mythic_ranks, state }` | one grid row; `mythic_ranks` drives the ✦ MR badge |
| `RankSheet { rank, character_level, status: Reached/Dropped/NotReached, path, slots }` | the Mythic track tab, MR 1–10 always listed |
| `Slot { id, kind: SlotKind::{Feat(FeatSlot), Selection(key), MythicAbility, MythicFeat}, pick }` | one chip |
| `CharacterState` | class levels (`classes`, `class_level()`), BAB, saves, `ability_scores`, `arcane_/divine_spell_level`, `caster_level`, `skill_ranks`, `feats`, `features` (with counts), `selection_options`, `mythic_abilities`, `mythic_feats`, `mythic_rank`, `mythic_path`, race, alignment; `check(&Prerequisite) -> Result<(), Unmet>` |
| `engine.options(&Build, &SlotId) -> Option<Vec<SlotOption { pick, status: Result<(), Issue>, would_break: Vec<Problem> }>>` | every option the slot accepts, in key order, with the reason it's locked, and for available ones what picking it would break. `None` for a slot the Build lacks |
| `engine.would_break(&Build, &Edit) -> Vec<Problem>` | later choices the edit invalidates, chains included, the edited choice itself excluded |
| `engine.validate(&Build) -> Vec<Problem>` | for an opened link; `Problem::is_unknown_key()` splits "removed from game data" from broken picks |
| `engine.state_before(&Build, &SlotId)`, `engine.slot_accepts`, `engine.is_combat_feat` | helpers |
| `Problem { at: Choice, issue: Issue }` | `Choice` locates it (`Slot(SlotId)`, `Class { level }`, `Archetype`, `Advances`, `SkillRanks`, `SkillPoints`, `AbilityIncrease`, `Level`, `Race`, `Subrace`, `AbilityScores`, `MythicPath`, `PathSwitch`) |
| `Issue::{UnknownKey, Unmet(Unmet), AlreadyHeld, NotAllowed, NoSuchSlot, LevelCap, ClassLevelCap, SkillRankCap, SkillPointsOverspent, PointBuy, ScoreOutOfRange, NotChoosable, NoAbilityIncrease}` | |
| `Unmet::{Condition { needs, has }, All(..), Any(..), AtLeast { needed, met, failed }}`, `unmet.conditions()` | reasons shaped like the failing part of the tree; `has` is the current value for numeric conditions ("Needs: BAB +4 (have +1)") |
| `encode_build(&Build) -> String`, `decode_build(&str) -> Result<Build, DecodeError>`, `CODEC_VERSION` | the `#b=` code |
| `mythic_pick_kind(rank) -> MythicPickKind::{Ability, Feat}`, `BASE_LEVEL_CAP` (20), `POINT_BUY_BUDGET` (25) | |

### URL codec

`code = base64url_nopad( [version] ++ body )`. Version 1: `body = raw_deflate(postcard(Build))`, level 9. The version
byte sits outside the compression, so a later version can change anything after it. Decoding never looks at game
data, so unknown keys decode and `validate` reports them. Inflation is capped at 256 KiB. Errors: `Empty`,
`Base64`, `UnsupportedVersion(n)`, `Decompress`, `Format` (including trailing bytes). A 40-level Build with ~50 picks
encodes to under 800 characters.

Crates: **postcard** (serde, `no_std`+`alloc`, varint and length-prefixed, the smallest serde binary format with a
stable documented wire spec), **miniz_oxide** (pure-Rust DEFLATE, no C, `alloc` only; the backend `flate2` would wrap
anyway, without the extra layer), **base64** 0.22.1 (already in the lockfile, `URL_SAFE_NO_PAD`). All three build for
wasm with no features beyond `alloc`.

Format guard: `Build`'s serde shape *is* version 1, so changing a field or variant order would change it. Two
tests freeze it: an uncompressed byte-layout test in `codec.rs`, and a frozen v1 link that must keep decoding. The
rule for later changes is written in the module docs: add version 2 with its own frozen types and keep
`decode_v1`.

### Decisions where the model was silent

- **Picks are keyed by slot** (`BTreeMap<SlotId, Pick>`), and slot ids are positional within their source, so editing
  one level never renames another level's slots. A pick whose slot disappears (class or archetype change, data
  change) is `NoSuchSlot`.
- **Order within a level:** race slots, class-row slots, the general slot, then Feature selection slots; then the
  level's Mythic ascensions in rank order (general pick, then path picks). A pick sees every earlier pick, so a human
  fighter can take Power Attack in the race slot and Cleave in the fighter slot at level 1.
- **Broken picks don't count as held.** That settles 06's flag-cascade question: removing Cleave breaks Great Cleave,
  which breaks Cleaving Finish. The same rule serves would-break and stale links.
- **No duplicates:** a feat, option, mythic ability or mythic feat already held (including a feat granted by an
  option, such as Combat Trick) is `AlreadyHeld`.
- **Class entry** (prestige requirements, a base class's alignment rule, the archetype's prerequisite) is checked once,
  on the class's first level, against the character before that level. A broken entry is a problem at
  `Choice::Class`/`Archetype` and is part of would-break; the levels still count.
- **Spell-level access:** each class's effective level is its class level plus the levels of prestige classes that
  chose it on entry, counting `advances_arcane` rows if it casts arcane spells and `advances_divine` rows if it casts
  divine ones, capped at its last row. No choice means no advancement.
- **Caster level:** the highest effective level among classes that can cast at that level. It's only checked where a
  `CasterLevel` prerequisite exists.
- **Skill points:** class (or archetype) skill points plus the Int modifier at that level, at least 1. Overspending
  is a problem. No racial skill points, since the data has none. **The skill rank cap** is the character level, which
  gives 20, or 40 on Legend.
- **Point buy:** Pathfinder's cost table (7→−4 … 18→17), scores 7–18, budget 25. Underspending is fine.
- **Paths:** only paths with no switch rule can be chosen at MR 3, and only paths with one can be switched to
  (`NotChoosable` otherwise). On a switch, grants from ranks outside `kept_ranks` are removed: general picks, path
  picks, path features, and feats granted by dropped options. Those ranks show as `Dropped`. With
  `further_ascensions: false` (Legend), the rank becomes `rank_after_switch`, MR 8–10 are `NotReached`, and the
  switch path's `level_caps` apply: the character cap is 40, base classes 20 and prestige classes 10. Otherwise the
  new path's rank rows apply from the switch rank (Gold Dragon gets its MR 8 row). Picks stored for ranks never
  reached are ignored, not reported.
- **Level cap:** 20 unless the switched-to path raises it; levels above it are `LevelCap`. A class level beyond its
  table (or the path cap) is `ClassLevelCap`.
- **Row 0** of a class table (what taking the class grants), if present, applies on the first level ahead of row 1.
- An ability increase recorded at a level that isn't a multiple of 4 is `NoAbilityIncrease`, and it's ignored.
- `options` reports `would_break` only for options that can be picked. Its cost is one walk per option, a few ms for
  the fixture world.

### Data change: no derived values in the database

At the user's request (relayed mid-ticket):

- `rules::MythicAscension` is now just `{ rank, character_level }`. The general pick is the `mythic_pick_kind(rank)`
  rule (odd: ability, even: feat), tested.
- No table stores `position` any more. Order follows a natural key or means nothing. `GameData::canonicalize()`
  (and `canonical()`) put a value in its one order: catalogs by key, class level rows by level, rank rows and
  ascensions by rank, and multisets (granted features and selections, feat slots, ability modifiers, prerequisite
  children) sorted by value. `Prerequisite::canonicalize()` sorts group children recursively. Both the files reader
  and the loader return canonical data, and `Engine::new` canonicalises too.
- True sets became `BTreeSet`, with unchanged RON syntax: `Class.class_skills`, `Archetype.class_skills`,
  `FeatSlot::OneOf`, `PathSwitch.kept_ranks` and `Progression.combat_feat_categories`. Catalogs and multisets stay
  `Vec`s, sorted on load: a `BTreeMap` catalog would change the file syntax and repeat each key, and multisets can
  hold duplicates (two identical feat slots at one level). `Ord` is derived on `Ability`, `SpellType`, `Alignment`,
  `FeatCategory`, `AbilityModifier`, `FeatSlot` and `Prerequisite`.
- New migration `20261031000300_drop_positions.sql` (the applied ones are untouched). The loader orders by natural
  keys, the round-trip test still asserts files → `GameData` equals files → Postgres → `GameData` (both canonical),
  and `cargo sqlx prepare` was re-run (54 queries).
- **Positions kept: none.** No list's order is authored information. The prerequisite order "as the guide lists
  it" is set semantics by the user's rule, and display order is the UI's job.

| Table | Before | After |
| --- | --- | --- |
| `mythic_ascension` | rank PK, `position` UNIQUE, `character_level`, `mythic_abilities`, `mythic_feats` | `(rank PK, character_level)` |
| `mythic_rank` | `position`, UNIQUE (path_key, position) and (path_key, rank) | no `position`; UNIQUE (path_key, rank) |
| `class_level` | `position` + UNIQUE (owner, position) | no `position`; UNIQUE (owner, level) remains |
| `path_switch_kept_rank` | PK (path_key, position) | PK (path_key, rank) |
| `class_skill` | UNIQUE (owner, position) | UNIQUE NULLS NOT DISTINCT (class_key, archetype_key, skill_key) |
| `feat_slot_option` | PK (feat_slot_id, position) | PK (feat_slot_id, feat_key) |
| `prerequisite_node` | `position`, UNIQUE (parent_id, position) | no `position`; children are a set |
| `skill`, `race`, `subrace`, `feat`, `class`, `archetype`, `mythic_ability`, `mythic_feat`, `mythic_path`, `feature`, `feature_selection`, `selection_option`, `combat_feat_category` | `position` (+ UNIQUE) | no `position`; ordered by key |
| `ability_modifier`, `granted_feature`, `granted_selection`, `feat_slot` | `position` + UNIQUE (owner, position) | no `position`; multisets, no uniqueness |

Files: `rules/src/{game_data,prerequisite}.rs`, `data/src/{files,load,seed,prerequisites,columns}.rs`,
`data/tests/{files,postgres}.rs` (expectations now canonical), the fixture `progression.ron`, and
`game-data/README.md` (order means nothing; the mythic pick rule).

### Tests

`rules/tests/engine/` is one test crate over a hand-built world in `support/mod.rs`: fighter, rogue (with
Eldritch Scoundrel), wizard, Arcane Trickster, Trickster, Legend and Gold Dragon, with 20-level tables and a
Cleave → Great Cleave → Cleaving Finish chain. 57 integration tests plus 3 unit tests:

- **derived (8):** multiclass BAB and saves; racial modifiers and the increase every 4 levels; human chosen bonus;
  subrace replaces race; skill ranks and skill points; spell-level access from the table; prestige advancement of
  the class chosen on entry (and caster level); no advancement without a choice.
- **slots (6):** general slots at odd levels; the human level-1 bonus slot; fighter bonus slots by class level;
  rogue talent selection slots; archetype rows replace class rows; ability increases at every 4th level.
- **mythic (6):** the odd/even pick-kind rule; ascensions at 6/7/9/11/14/15/16/17×2/19; ability and feat alternate,
  unreached ranks listed; the path at MR 3 with its own picks; a Legend switch at MR 8 that keeps MR 1–2, stays at
  rank 3, drops MR 3–8 and leaves MR 9–10 unreached, with levels 18–40 still getting odd-level feats and +1 every 4
  levels (Fighter 20 / Rogue 20, no problems); a Gold Dragon switch that keeps ascending.
- **options (11):** fighter bonus slots offer only combat feats (and a non-combat pick there is `NotAllowed`); AND
  reports each failing condition; OR holds on any branch; at-least counts branches; same-level ordering and
  `AlreadyHeld`; selection options with reasons; an option granting a feat; ability, skill-rank, spell-level and
  caster-level checks; race and alignment; mythic slots by rank; no options for a missing slot.
- **would_break (7):** removing Cleave breaks Great Cleave and Cleaving Finish; the reason is at the broken pick; a
  class change; an archetype change; an earlier skill edit breaks a prestige entry; the edited choice is not
  reported; options carry `would_break`.
- **validation (11):** unknown keys reported, not fatal; picks that no longer qualify after a data change (with the
  cascade); a pick for a vanished slot; the skill rank cap; skill point overspend; levels above 20 without Legend;
  Legend class caps; class and archetype entry requirements; the point buy; non-choosable paths; duplicate picks.
- **codec (6 + 1 unit):** proptest round-trip of arbitrary Builds; a frozen v1 link still decodes; the version byte
  comes first; a link with unknown keys decodes and `validate` reports them; bad links are errors, not panics; a
  40-level link is under 800 characters; the frozen uncompressed v1 byte layout (unit).
- **canonical (2):** canonical `GameData` doesn't depend on list order; catalogs ordered by key and levels by level.

### Commands (all verified)

```sh
cargo test --workspace                                 # rules 3 + 57, data files 7, data postgres 6: all pass
cargo clippy -p rules -p data -p server --all-targets  # clean
cargo fmt --all --check                                # clean
cargo build -p rules --target wasm32-unknown-unknown   # OK
SQLX_OFFLINE=true cargo leptos build                   # OK
cd data && cargo sqlx prepare --check                  # OK after re-running prepare
```

### Not done

- The engine tests use the hand-built world, not the RON fixture. The fixture's tables stop at class level 3, too
  short for ascensions. `data`'s tests cover the fixture.
- No caching across calls: each `evaluate` re-walks the Build. `options` costs one walk per option.
- Path-switch rules for Gold Dragon, Swarm That Walks and Devil are data (`kept_ranks`); the real values wait on
  the extraction.
