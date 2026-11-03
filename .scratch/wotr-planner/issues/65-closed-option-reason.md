# Reason text for an option closed to an archetype

Type: task
Status: resolved
Blocked by: 64
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 59](59-kineticist-infusions.md)'s decision 5 (user, 2026-11-02): the lock reason for an option an archetype can't take reads "Needs not the Kinetic Sharpshooter archetype". It should read "Not for the Kinetic Sharpshooter archetype". Fix it where `rules`' labels render a negated archetype prerequisite (`libs/rules/src/engine/labels.rs`), for every negated archetype, and check other negated prerequisites for the same "Needs not …" wording (fix each to read naturally, and list them in the Answer).

## Build

AFK, test-first (`tdd`); the only agent in the repo. A labels change only: no data, schema or link change. Before/after proof with the latest proof program: only lock-reason text may differ, and only for negated prerequisites.

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus an engine test and a real-data test: a Kinetic Sharpshooter's closed infusion reads "Not for the Kinetic Sharpshooter archetype"; no reason anywhere starts with "Needs not" (a real-data sweep over every picker of the proof's Builds).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD 898b11d (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t65/main-rs-user-edit.diff`, identical to ticket 64's, and left alone). Scratch in `t65/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t65/`): `old-src/` is `git archive HEAD`; the test list (`t65/test-list-before.txt`, 726) equals ticket 64's. Found: a failing `Not(x)` is an `Unmet::Condition { needs: Not(x) }`, and `issue_text` put "Needs " before every reason, so a negation read "Needs not …" alone, ", not …" in a list and "Needs not taken as … or …" in an "or". The data negates four kinds (`Not(Archetype)` 6, `Not(SelectionOption)` 109, `Not(ClassLevel)` 2, `Not(PickedThrough)` 8), each at the top of an option's prerequisite or directly in its top-level `All` or `Any`.
- 2026-11-03: engine tests first (`libs/rules/tests/engine/labels.rs`, 2 new; the old "Needs not Slayer level 6" assertion moved into them): red (`t65/rules-red.txt`). Real-data tests (`game_data.rs`, 2 new, the totem test's reason updated; `data_prerequisites` pulled out of `prerequisite_features`): red with HEAD's `labels.rs` (`t65/real-data-red.txt`: 3 fail, each at a "Needs not" reason).
- 2026-11-03: `libs/rules/src/engine/labels.rs` changed; engine and real-data tests green. Proof (`t65/proof/`, `t65/patch_proof.py`, `t65/proof-old/`, `t65/proof-new/`, `t65/compare.py`, `t65/compare.txt`), UI (`cargo leptos watch` on 3900, headless Chromium: `t65/links/`, `t65/links.txt`, `t65/ui-check.js`, `t65/ui-check.txt`, `t65/ui-*.png`; server stopped), checks (`t65/test-workspace.txt`, `t65/test-list.txt`, `t65/test-list.diff`, `t65/seed.txt`), the map.

## Answer

Done 2026-11-03, uncommitted. A failing negated prerequisite now says what rules the option out: a Kinetic Sharpshooter's closed infusion reads "Not for the Kinetic Sharpshooter archetype", and no reason says "Needs not …" any more. A labels change in `rules` only (`libs/rules/src/engine/labels.rs`): no data, schema, `Build` or link change.

### Wordings

Each kind of negated condition has a lead and an object. Refused, it reads "Not {lead} {object}", or "Only below {object}" for a level or score. As the case of an "or" (below), it reads "{Lead} {object}".

| Negated condition | Before | After |
|---|---|---|
| `Archetype` (in the data) | Needs not the Kinetic Sharpshooter archetype | Not for the Kinetic Sharpshooter archetype |
| `SelectionOption` (in the data) | Needs not Beast Totem, Lesser | Not with Beast Totem, Lesser |
| `ClassLevel` (in the data) | Needs not Slayer level 6 | Only below Slayer level 6 |
| `PickedThrough` (in the data, in an "or") | Needs not taken as Chosen Alignment Blessing or Lawful Evil alignment or Neutral Evil alignment or Chaotic Evil alignment | As Chosen Alignment Blessing, needs Lawful Evil alignment or Neutral Evil alignment or Chaotic Evil alignment |
| `PickedThrough`, alone | Needs not taken as Rogue Talents | Not as Rogue Talents |
| `Race` | Needs not Elf | Not for the Elf race |
| `Alignment` | Needs not Lawful Good alignment | Not for a Lawful Good character |
| `SpellLevel`, `SpontaneousSpellLevel` | Needs not to cast 3rd-level arcane spells | Not for a caster of 3rd-level arcane spells; Not for a spontaneous caster of 1st-level divine spells |
| `Feat`, `FeatFor`, `FeatForSameValue`, `FeatIfValue`, `MythicFeat`, `MythicAbility`, `Feature` (one), `Story` | Needs not Power Attack | Not with Power Attack |
| `NoOptionFrom` | Needs not no Rogue Talents yet | Not with a Rogue Talents pick |
| `NoOtherOptionOfGroup` | Needs not no other Draconic option | Not with another Draconic option |
| `All`, `Any`, `AtLeast` | Needs not dodge, toughness | Not with (dodge, toughness) |
| `BaseAttackBonus`, `AbilityScore`, `SkillRanks`, `CasterLevel`, `CharacterLevel`, `MythicRank`, `Feature` ×n | Needs not BAB +4 | Only below BAB +4 |
| `Not(Not(x))` | Needs not not Power Attack | Needs Power Attack |

How they combine:

- **Beside other unmet conditions:** the refusals lead, each kind once with its objects joined, then what the option needs, separated by "; ". For example, "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 3), Extended Range" (was "Needs not the Kinetic Sharpshooter archetype, Kineticist level 6 (have 3), Extended Range"). Another: "Not with Celestial Totem or Celestial Totem, Lesser; needs Beast Totem, Lesser" (was "Needs not Celestial Totem, not Celestial Totem, Lesser, Beast Totem, Lesser").
- **In an "or":** a failing negated branch means its condition holds, so the reason is that case and what it needs: "As Domain (Separatist), needs Neutral Good alignment or …". An "or" of negations only reads "Not with A and with B together".
- **Deeper inside a group:** a negation reads in line, lower case: "Needs BAB +1 (have +0), (Human or not for the Eldritch Scoundrel archetype or below Slayer level 6)". The data has none.
- **Unchanged:** every reason without a negation, word for word (`issue_text` takes the old path when none of its top-level conditions is negated).

### Code

`labels.rs`:

- `issue_text` renders an `Unmet` through the new `unmet_reason`.
- `held_text` gives each kind its lead (`Lead::For`, `With`, `As`, `From`) and object.
- `prerequisite_text` renders `Not(x)` in line and cancels a double negation.
- `spell_text` is split so "3rd-level arcane spells" serves both a need and a refusal.

The public API is unchanged.

### Tests

- **Engine** (`libs/rules/tests/engine/labels.rs`):
  - `a_negated_condition_says_what_rules_the_option_out`: every kind's refusal, as in the table.
  - `negated_conditions_lead_a_reason_and_read_in_line_inside_a_group`: the combinations above.
- **Real data** (`libs/seeds/tests/game_data.rs`):
  - `a_kinetic_sharpshooters_closed_infusion_reads_not_for_its_archetype`: a fire Sharpshooter 3's Extended Range and Kinetic Blade read "Not for the Kinetic Sharpshooter archetype", and Eruption "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 3), Extended Range". A plain kineticist's Exploding Arrows still reads "Needs the Kinetic Sharpshooter archetype".
  - `every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out`: every `Not` in the data's prerequisites, rendered alone, starts with its kind's lead, and the data negates exactly the four kinds. Real reasons for each kind:
    - a slayer 6's 2nd-level combat style talent: "Only below Slayer level 6";
    - a Celestial-line barbarian 6's Beast Totem: "Not with Celestial Totem or Celestial Totem, Lesser; needs Beast Totem, Lesser";
    - Cinder Dance with the Lame curse: "Not with Lame";
    - a Lawful Good Champion of the Faith's evil blessing, and a Lawful Good Separatist's Chaos Domain, in their "or" forms.
  - **Updated:** the totem test's "Not with Beast Totem, Lesser".

### Proof

Ticket 64's program, plus every picker's lock-reason text:

- per slot, the texts of options whose issue holds no `Not` (hashed) and those of options whose issue holds one (listed);
- the class, path and switch pickers' texts (hashed);
- each Build's count of texts holding "Needs not";
- 14 ticket-65 Builds (a fire Kinetic Sharpshooter 5, a Celestial-line barbarian 6, a slayer 6 and 12, a Lawful Good Champion of the Faith 1, a Lawful Good Separatist 3, a Lame flame oracle 1; each also filled) and their full versions.

It is compiled against HEAD's rules (`proof-old`, A) and the working tree's (`proof-new`, B), both run on HEAD's game data (the working tree's is the same): 1,892 Builds each. `t65/compare.py` (`t65/compare.txt`): 0 problems.

- **A checks the program:** with the added fields removed, A equals ticket 64's proof C on all 1,864 Builds both have.
- **A vs B:** 1,762 Builds are byte-identical. The other 130 differ only in the listed texts of negated reasons: the same options, keys and order in every slot, and every other field equal (sheets, statuses, slots, the non-negated texts, the class, path and switch pickers' texts, `allows`).
- **The sweep:** of 13.9M reason texts in B, none holds "Needs not" (A: 3,017). B's 3,077 negated reasons (100 distinct) each start with "Not for the ", "Not with ", "Only below ", "As " or "Not as ". All 100 before/after pairs are in `t65/compare.txt`.
- **Ticket 48's shortcut** stays exact (1,216,036 feat options, both runs).
- **Run time:** 3 min 41 s (A), 4 min 14 s (B), 8 threads.

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t65/ui-check.txt`):

- **A fire Kinetic Sharpshooter 5's link:** its "Infusion" picker at 5, locked options shown, reads "Not for the Kinetic Sharpshooter archetype" (Extended Range, Kinetic Blade), and "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 5), Extended Range" and the like for Eruption, Blade Whirlwind, Deadly Earth and Fragmentation.
- **A Celestial-line barbarian 6's link:** its "Rage Power" picker at 6 reads "Not with Celestial Totem or Celestial Totem, Lesser" for the other lines, with "; needs Beast Totem, Lesser" and the like.
- **A Lawful Good Champion of the Faith 1's link:** its "Chosen Alignment Blessing" picker reads "As Chosen Alignment Blessing, needs Chaotic Good alignment or Chaotic Neutral alignment or Chaotic Evil alignment" (Chaos) and "… Lawful Evil … Neutral Evil … Chaotic Evil alignment" (Evil).
- No page shows "Needs not". No stale-link error and no console errors. Server stopped.

### Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 105, rules 347 + unit 4, seeds files 25 + game_data 217 + postgres 28, server 4).
- `cargo test --workspace -- --list` (730) vs ticket 64's (726) (`t65/test-list.diff`): the 4 new tests, nothing removed or renamed.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds).
- No schema change, so no DB reset, reseed or `cargo sqlx prepare --check`. No link change: `V1_LINK` stays.
- `cargo leptos build` and `--release`: skipped, as asked.
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t65/main-rs-user-edit.diff`).

### Questions decided by recommendation

1. **The words for each kind:** as in the table. The user's "Not for the … archetype" sets the pattern: "for" whom (archetype, race, alignment, caster), "with" what is held, "as" which slot. A level or score reads "Only below …", since "Not from Slayer level 6" is unclear. The check gives a negation no "have" value, so none is shown.
2. **Several refusals of one kind:** joined once ("Not with Celestial Totem or Celestial Totem, Lesser"), not repeated ("not with …; not with …"). "Or" is right for "not with" (neither may be held). "And" is right for "below" (below each).
3. **Refusals beside needs:** refusals lead, since no level-up lifts them, then "; needs …" in lower case. The reason stays one line.
4. **A negated branch of an "or":** reads as the case it names ("As Domain (Separatist), needs …"). The literal "or not as …" reads as a riddle.
5. **Nested negations** (inside a group inside a group): read in line ("not for …", "below …") rather than restructured. The data has none, and the sweep shows none.

None needs the user. The comma inside names such as "Celestial Totem, Lesser" makes a list harder to read, but it was there before this ticket. The "; " between clauses keeps the clauses apart.
