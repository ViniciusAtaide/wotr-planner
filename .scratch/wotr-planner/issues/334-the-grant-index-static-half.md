# The grant index: its static half

Type: build (engine, tooling)
Status: open
Blocked by: 331
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-G.md, T3](../research/architecture/design-G.md) (interface §1 static half, what goes §2's T3 rows, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

Four crates each answer "what gives this fact" (findings `4-grant-index.md` §1–2), each counting different grant kinds. The carrier guard copies the favoured-weapon rule without its values (308h, 324). Switch features and a feat's selection are in none of them.

## Change

- Add `rules::grants` (design-G §1, static half) and build it in `Catalog::new`.
- Make `catalog.rs`, `check.rs:1925`, game-check's `carrier_guard` and seeds `support` read it (design-G §2's T3 rows).
- `catalog.rs` keeps `GrantedBy` as a temporary projection. `labels.rs` is untouched: ticket 332 edits it.

## Done when

- Every grant kind in design-G §1's `When` has a site, pinned by tests.
- The form proof (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`, recorded on the base in the lane before the first edit) shows 0 differences.
- The game check exits 0.
- `--carrier-audit` is run on the base and the tip and diffed; each changed hit is explained in the Answer. New lines are written `-> ?` for the main session to assign, which is decision 7.
- The wasm and native costs of building the index are measured: under 10 ms native, and under 5% of the browser's "engine" line.

## Files

- `libs/rules/src/{grants.rs (new), lib.rs, engine/catalog.rs, check.rs}`
- `libs/rules/tests/engine/{grants.rs (new), main.rs}`
- `libs/seeds/tests/game_data/{support.rs, grant_index.rs (new), main.rs}`
- `tools/game-check/src/write/carrier_guard.rs`
- `tools/game-check/tests/fixture/prerequisites_write.rs`
- `tools/game-check/carriers-known.txt` (only if hits move)

## Checks

- `cargo test -p rules -p seeds -p corpus -p game-check`
- `cargo build -p rules --target wasm32-unknown-unknown`
- `cargo fmt --all --check`
- the form proof as above
- the game check
- `--carrier-audit` diffed

## Answer

Built design-G §1's static half and §2's T3 rows. Every Done-when holds but the last: the index costs about 3 ms in wasm, about half of `Engine::new`, against a budget of 5% (see Cost). Status stays open until the main session decides that budget.

- **`rules::grants`** (`libs/rules/src/grants.rs`, `pub use grants::*`): `GrantIndex { sites, by_entry, by_fact, offered_in }`, `Site { by, class, when, grants }`, `When`, `GrantRule`, `Grant { fact, value, opens_slot }`, `GrantValue`, `Satisfies`; `new`, `gives`, `given_by`, `offered_in`, `sites`, `Grant::satisfies`. One pass over the data; no IO; never panics on a key the data lacks.
  - The sites: class and archetype level rows (`ClassRow`, `class` = the class or the archetype's base class; features, feats `Every`, `feats_for` `Fixed`, selections opening slots); path rank rows (`Rank`); option level rows (`OptionRow`, chosen feats `Picked`); `Always` for an option's feats and chosen feat, a feat's and a mythic feat's `selection`, a mythic ability's features and selection, a switch's features, a background's feats (`feats_for` `Fixed`), a deity's favoured weapons (Improved Unarmed Strike `Every`, an exotic weapon's proficiency `Fixed` to it, nothing for a simple or martial one), a race's and a subrace's selections; one `Component` site per `Grant`, `GrantAtClassLevel`, `GrantIf`, `GrantOnApply`, `GrantWithValue` on every entry with components (gear too).
- **`Catalog`** builds one in `Catalog::new` (after `canonical()`). `GrantedBy` stays as the projection of the row sites (`GrantedBy::of_row`, `holders(&GrantIndex)`), so `feature_holders`, `selection_holders` and `shared_selection_names` keep their answers and `labels.rs` is untouched. `favoured_weapon_feats` reads the deity's `Always` site.
- **`check.rs`** `counts_toward_features`: a `ClassRow` site of the other class in `given_by`. It builds an index only when some class counts toward a feature (the real data has one).
- **game-check** `carrier_guard`: `Grants` holds a `GrantIndex`; `carry` is the same rule over `gives`, `given_by` and `offered_in`. `sites` and the `favoured_weapon_feats` copy are gone.
- **seeds** `support`: `grants`, `class_rows`, `all_class_grants`, `all_path_grants` keep their names and callers; `RowGrants` is now a `&'static Site` of `support::GRANTS`, and the transitive closure stays in `support`.

### Decisions

- **`satisfies` takes `(fact, value, refused, picked)`** (mine, design-G §1's bracketed alternative): T2's `Leaf` isn't built.
- **A site's grants are sorted, and sites sort by (`by`, `when`, grants)** (mine): `canonical()` sorts a row's features and selections, so without it `new(data) != new(data.canonical())`. Nothing read the data's order inside a row.
- **Three site kinds beyond §1's list** (mine): a race's and a subrace's selections (the walk holds them, `hold_feat_selections`), a selection's `replaced_by` feature (`OptionRow` of the selection), gear components. The audit didn't move.
- **`GrantIndex::sites()`** (mine): the projection and seeds read every site.
- **`row_sites_are_the_old_holders` is a unit test in `catalog.rs`** (mine), since `GrantedBy` is private to the engine. It keeps the old row scan as its reference and goes with `GrantedBy` in 335.
- **The agreement self-check runs on the bench Builds only** (mine): the gold Builds are private to `gold.rs`, and `tools/corpus/tests/self_checks.rs` isn't in this ticket's files. I ran the same check over the corpus once in a temporary test, not committed: 2,157 Builds, 65,214 held facts, 0 misses. The general feat slot's selection is excepted: no entry of the data grants it (the walk holds it at odd levels, as the game's `BasicFeatsProgression` 5b72dd2ca2cb73b49903806ee8986325 does).
- **No new carrier lines**, so no `-> ?` lines and `carriers-known.txt` is unchanged.

### Tests

Each one is in `--list`.

- `libs/rules/tests/engine/grants.rs` (test world): `every_grant_kind_is_a_site` (21 grants: a class row's feat and selection, an archetype row's `feats_for`, an option's chosen feat and its row's, a rank row, a switch's features, a feat's and a mythic feat's selection, a mythic ability's features and selection, a race's selection, a background's feats and `feats_for`, Fist's and Lawgiver's favoured weapons, each grant component under its rule), `a_row_site_names_its_class`, `a_martial_favoured_weapon_gives_nothing`, `given_by_is_the_inverse_of_gives`, `the_index_ignores_the_datas_order` (lists and rows reversed, and `canonical()`), `an_unknown_key_gives_nothing`, `offered_in_lists_the_selections_whose_lists_offer_a_fact`, `a_refused_leaf_is_never_satisfied`, `a_grant_for_one_value_meets_only_that_value`, `a_grant_for_every_value_meets_any`, `a_picked_value_meets_the_picks_value`. Against a stub `new`, 8 failed.
- `libs/rules/src/engine/catalog.rs`: `row_sites_are_the_old_holders`.
- `libs/seeds/tests/game_data/grant_index.rs` (real data), each value from the game:
  - `irori_gives_improved_unarmed_strike`: `IroriFeature` 23a77a5985de08349820429ce1b5a234 grants `ImprovedUnarmedStrike` 7812ad3672a4b9a4fb894ea402095167 by `AddFeatureOnClassLevel` (Cleric 1).
  - `achaekek_gives_exotic_weapon_proficiency_for_the_sawtooth_saber`: `AchaekekFeature` a3189d5b7c4d4d91beaa8bfffac3e38e grants `SawtoothSabreProficiency` 5829f3b193704b8096820875f2eb2368 (`AddProficiencies: SawtoothSabre`) by `AddFeatureOnClassLevel` (Cleric 1).
  - `extra_rage_power_opens_a_rage_power_slot`: `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a is a `BlueprintFeatureSelection`.
  - `legends_switch_grants_its_features`: `LegendProgression` 905383229aaf79e4b8d7e2d316b68715, level 1: `LegendFeature` 7f99961610390044391f300c7ef5e0c8, `Legend_UniqueRestFeature` 27327f8954cd4f91ab3deaebf1f8cfa7.
  - `the_index_agrees_with_the_walk` (the bench Builds), and `grant_index_build_time` (ignored, the timing).
- `tools/game-check/tests/fixture/prerequisites_write.rs`, with the fixture world gaining a deity with an exotic favoured weapon and a path whose switch grants two features:
  - `an_exotic_favoured_weapon_gives_its_proficiency_for_that_weapon` (the deity's site gives `Fixed(fixture-saber)`; `saber-feat` is written);
  - `a_switch_feature_carries_its_fact`. With the switch site turned off, it failed with the gap `feat boon-feat | left whole: feature boon granted by PlantedSwitchBoonFeature …, which feature switch-boon doesn't grant -> 307`.

### Proofs

- **Form proof** (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`, recorded on aedfb6ec in this lane before the first edit): `links: 1992 the same, 0 through an heir, 0 refused, 0 changed`; `offers: 18213559 options recorded, 18213559 now`, `0 statuses changed`; `reasons: 15217937 compared …, 0 changed in 0 edits`; `failures: 0 appeared, 0 gone`; `2157 Builds compared (1992 open), 2157 old links reopened, 0 differences, 0 offer differences, 0 reason problems`.
- **Carrier audit**: 176 hits on the base and on the tip, byte-identical (`diff` empty). No hit moved: the new sites (switch features, a feat's and a mythic feat's selection, race selections, `replaced_by`, values) neither carry a fact a hit named nor break "every site gives both" for one.
- **Game check** exit 0, its report byte-identical to the base's (`Carrier hits (ticket 322): 176, each listed in carriers-known.txt.`).

### Cost

The machine was loaded (another builder ran; load 30 to 100), so these are medians, not quiet-machine numbers.

- **Native** (release, `grant_index_build_time`, median of 20): `GrantIndex::new` 2.4 ms (2,656 sites, 5,146 grants), within the 10 ms budget. `Engine::new` took 9.1 ms at that load.
- **Wasm**: a scratch `cdylib` built with the browser's profile (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`) and run under node over the JSON the server serves, base and tip alternating, median of 40:
  - base `Engine::new` 2.79 ms, tip 5.84 ms (+3.05 ms);
  - `GrantIndex::new` 2.86 ms, **49% of the tip's `Engine::new`**: over the 5% budget.
  - The browser's `check` line also builds one (`counts_toward_features`), about another 2.9 ms.
  - Not measured in the browser: the planner's release `cargo leptos` build runs from the main checkout.
- **Where it goes (native):** reading the sites about 0.8 ms, `offered_in` about 0.5 ms, the rest sorting and the two maps. Each grant clones its key strings.
- **For the main session**, options that would bring it down: build `offered_in` apart (only game-check reads it); key `by_fact` without a clone per grant; build the index lazily in `Catalog`. That would move the cost to the first walk, not remove it. Interning the keys would cut most of it, but that is beyond this ticket. Or relax the budget.
