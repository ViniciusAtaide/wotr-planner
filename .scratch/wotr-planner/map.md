# WotR Build Planner

Label: wayfinder:map

## Destination

A working, locally run Leptos app at `~/code/wotr-planner` for planning Pathfinder: Wrath of the Righteous builds. You build a character level by level, and each level offers only the choices the game would allow at that point: class, archetype, feats, mythic feats, mythic abilities, feature selections and skill ranks. The app ends with a summary up to character level 20, or 40 on the Legend mythic path. Builds live entirely in the URL, which updates as you edit and is what you share. It works well on a phone, specifically the user's Galaxy Z Fold 7: both the folded ~21:9 cover screen and the unfolded ~6:5 inner screen. The source of truth is the [GameFAQs guide](https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/introduction).

## Notes

- **This map includes building the app**, not just planning it. Tickets may build things, not only decide them.
- **Stack:**
  - cargo-leptos SSR on Axum, using a dated nightly pinned in `rust-toolchain.toml`.
  - Postgres in docker compose; local only.
  - sqlx with typed serde structs.
- **Game data:**
  - The source of truth is the checked-in data files (user, 2026-11-03, [ticket 75](issues/75-game-data-master.md)). They were extracted from the guide and are corrected by hand; the extractor goes once the files hold what it knows (tickets 81–82).
  - A seed step loads them into ordinary relational tables, prerequisites included (no JSONB).
  - Store prerequisites, a short summary in our own words and a link to the guide, never the guide's full text.
  - When the game and the guide disagree, the guide wins.
  - **Never fetch GameFAQs or the fextralife wiki automatically.** Both robots.txt files disallow ClaudeBot. Agents parse only the pages the user saved to `guide-cache/`, which is git-ignored.
  - Include all DLC content, with no DLC gating. Prestige classes are in scope.
  - Committing extracted facts is OK.
  - **One migration while prototyping** (user, 2026-10-31). Schema changes edit the initial migration, then the dev database is dropped and reseeded; there are no fix-up migrations. Only one agent at a time does this. The steps are in the repo's `CLAUDE.md`.
  - **No derived values in the database** (user, 2026-10-31). Anything computable from other columns or a simple rule belongs in Rust, not in a column. Examples: a `position` that equals `rank − 1`, or ability/feat counts that follow `rank % 2`.
  - **Postgres checks references to missing keys** (user, 2026-11-03): the seed's foreign keys catch them; `GameData::check` stays without a reference check.
- **Rules followed:** the game's intended, current (patched) behaviour. Known exploits are not modelled.
- **Extraction in stages** (user, 2026-11-01): [Extract the guide into data files](issues/10-extract-guide.md) is stage A, where the extractor writes `libs/game-data/` and the seed passes. Then four parallel hand-fix stages, one per category: [feats](issues/17-handfix-feats.md), [base classes](issues/18-handfix-base-classes.md), [prestige classes](issues/19-handfix-prestige-classes.md), [mythic](issues/20-handfix-mythic.md). They validate without the database, and the main session seeds once. Then [Answer the schema questions from the real data](issues/21-schema-questions-real-data.md). Commit a checkpoint after each stage.
- **Order** (user, 2026-10-31): finish the data layer (saved guide pages, extraction, checking the data against the guide) before the UI. [Build the planning sheet screen](issues/12-build-sheet-screen.md) waits on the data-layer tickets 23–38. Tickets 23–28 and 30–38 each edit the one migration, so they run one at a time, in order; design ticket 29 ran with the user alongside. Tickets 32, 33, 35 and 36 build the user's answers to the open questions of 26–35 (2026-11-01); 34 adds backgrounds from the user's list, 37 turns options' class-skill features into class skills, 38 cleans up after it (skills questions take the recommended option, per the user), and 39 swaps validation for impossible states (user, 2026-11-01: picks are removed from the leaves inward), and 40 gives a stale link one generic error instead of the banner and drops picks the Build no longer reaches (user, 2026-11-01), and 41 also drops the path and switch below their ranks, and 42 keeps a switch's earlier picks again (user, 2026-11-02, undoing the rest of 41). 39–42 edit `rules`, not the migration, but still run after 38.
- **Architecture review** (2026-11-03): tickets 75–82 simplify what the review found. Order: [81](issues/81-apply-the-extractor-to-the-files.md) brings what only the extractor knows into the files, [82](issues/82-remove-the-extractor.md) removes the extractor, then 77, 78 and 80; 76 is independent; 79 is closed.
- **Rules:** the rules live in a plain Rust crate with no DB or Leptos code, running in the browser. The browser loads the game data once, filters choices immediately, and decodes and encodes the Build from the URL.
- **Mobile:** design mobile-first. The main targets are the Galaxy Z Fold 7 cover screen (folded, tall and narrow, ~21:9) and inner screen (unfolded, nearly square, ~6:5). Desktop is secondary. Both Fold screens must work for touch: no hover-only interactions, and tap targets large enough for a thumb. The layout should adapt when the phone is folded or unfolded without losing the user's place.
- **Builds and links:** no accounts and no saved-build table. The URL fragment encodes every choice, updates live, and is the share link. `/` with no fragment is a blank Build.
- **Prerequisites checked:**
  - race, ability scores (point-buy and the +1 every 4 levels)
  - BAB
  - caster level, only where the guide states it
  - spell-level access (for example "can cast 3rd-level arcane spells"), which is a different check from caster level
  - skill ranks
  - alignment
  - other feats, class features
- **Facts confirmed by the user** (not researched):
  - Class skills are a nice-to-have, not crucial for planning (2026-11-01): a flat bonus added to a trained skill's total. They never decide which picks are legal, so the sheet shows them and nothing depends on them.
  - Skills questions take the recommended option (2026-11-01): open questions about class skills, skill points, expertise or background skills are decided by the recommendation without asking the user, and recorded as such in the ticket.
  - Skill points, class skills, expertise and backgrounds (2026-11-01): [research/skills-from-user.md](research/skills-from-user.md). Where it disagrees with the guide (Arcane Trickster, Dragon Disciple), the user's table wins.
  - Backgrounds (2026-11-01): [research/backgrounds-from-user.md](research/backgrounds-from-user.md), 38 in eight groups. Its None row isn't a background (2026-11-02): a Build start without one simply has none.
  - Characters gain a feat at every odd character level.
  - The Legend path raises the level cap to 40.
- **Skills to use:**
  - `tdd` for the rules crate and the guide extraction; screens get end-to-end smoke tests only.
  - `grilling` + `domain-modeling` for design tickets; keep [GLOSSARY.md](../../GLOSSARY.md) current.
- **Tracker:** local markdown. Tickets are in `issues/`; research write-ups are in `research/`.

## Decisions so far

<!-- one line per closed ticket: [title](link): gist -->
- [How the guide is laid out](issues/01-guide-anatomy.md): about 38 of 70 pages hold build data in parseable HTML; the user saves the pages and we parse them offline.
- [Leptos and toolchain setup](issues/02-leptos-toolchain.md): Leptos 0.8.23, cargo-leptos 0.3.11 and sqlx 0.9 on pinned `nightly-2026-10-30`, in the start-axum-workspace layout.
- [Filling the guide's gaps](issues/07-guide-gaps.md): prestige classes, path mythic picks, alignment and all DLC are in scope; combat feats defined; skill rank cap 20 (40 on Legend).
- [How a character progresses](issues/04-progression-model.md): a Build is a Build start, Level-ups and Mythic ascensions at fixed levels; path chosen at MR 3, switch at MR 8.
- [Create the project skeleton](issues/03-scaffold.md): the workspace builds and runs; dev server on `0.0.0.0:3900`, Postgres 18 on host port 5434.
- [Game data schema and seeding](issues/05-game-data-schema.md): hand-edited RON data files, a relational prerequisite tree and a seed binary *(superseded in part by 23)*.
- [Level-up screen behaviour](issues/06-level-up-screen.md): the Sheet: a level × slot chip grid, a Mythic track and a bottom-sheet or modal picker with locked options.
- [Saving and sharing Builds](issues/13-saving-sharing-builds.md): the URL fragment is the Build, updated live, with no saved-build table *(stale links: reversed by 40)*.
- [Build the game data types, schema and seed](issues/09-game-data-foundation.md): `rules::GameData` is the one shape; the seed loads it and a round-trip test proves it *(superseded in part by 14)*.
- [Build the rules engine](issues/11-rules-engine.md): `Engine` walks a Build into a Sheet; links are versioned and compressed; no derived values in the DB *(superseded in part by 14)*.
- [Apply the schema and layering review](issues/14-review-fixes.md): class choices live in `Build.class_choices`, rule facts moved into `rules`, and `GameData::check` gates every seed.
- [Apply the normalization review](issues/15-normalization-fixes.md): schema normalisation: ability modifiers, the switch's level cap, one per-kind prerequisite CHECK; still 25 tables.
- [Save the guide pages](issues/08-save-guide-pages.md): all 38 pages saved in the git-ignored `guide-cache/` on 2026-10-31.
- [Bonus feats ignore prerequisites](issues/16-bonus-feats-ignore-prerequisites.md): a Bonus-list slot skips the feat's prerequisites but still checks its list and already-held.
- [Extract the guide into data files](issues/10-extract-guide.md): stage A wrote the real `game-data/` and seeded cleanly; `EXTRACTION_REPORT.md` listed 82 items for the hand-fix stages.
- [Hand-fix the extracted feats](issues/17-handfix-feats.md): all 261 feats checked against the guide; two prerequisite errors fixed.
- [Hand-fix the extracted base classes](issues/18-handfix-base-classes.md): 15 of 64 report items fixed; the nested option tables were left to later tickets.
- [Hand-fix the extracted prestige classes](issues/19-handfix-prestige-classes.md): 1 item fixed, 2 narrowed, 4 left; entry requirements and spellcasting advancement rows fixed.
- [Hand-fix the extracted mythic data](issues/20-handfix-mythic.md): 3 of 7 items fixed and the path switch rules encoded; follow-ups listed in the ticket.
- [Drop the type prefix from keys](issues/22-drop-key-type-prefix.md): keys are bare slugs and kinds may share a key; the v1 link vector is regenerated, with no v2.
- [Answer the schema questions from the real data](issues/21-schema-questions-real-data.md): the user took every recommendation (2026-11-01), built as tickets 23–28 with design ticket 29.
- [Store class progressions, not copies](issues/23-class-progressions-not-copies.md): classes store BAB and save progressions, archetypes only the rows that differ; `rules` computes the numbers.
- [Granted feats and named feat lists](issues/24-granted-feats-and-feat-lists.md): lone feats are granted feats, fixed lists are named `FeatList`s, and classes grant the real proficiency feats.
- [Share selection options between selections](issues/25-shared-selection-options.md): a selection can offer another's options under one key, so a shared talent can't be taken twice.
- [Mythic path rules](issues/26-mythic-path-rules.md): a path grants MR 1–2 when chosen; switch keep rules, ability bonuses and `MythicRank` prerequisites; 33 tables.
- [Repeatable and parameterised feats](issues/27-repeatable-and-parameterised-feats.md): feats take a parameter value and a repeat rule, with `FeatFor` prerequisites; 34 tables.
- [Design options that grant over class levels](issues/29-design-option-level-grants.md): an option has rows by option level, fed by the classes its grant names; built in tickets 30 and 31.
- [Class rules from the hand-fix follow-ups](issues/28-class-rules-follow-ups.md): `advances`, `counts_as`, `Story` prerequisites, the Assassin's alignment and Nature Bond; 35 tables.
- [Build option levels](issues/30-build-option-levels.md): bloodlines, schools, orders, spirits, ways and combat styles get option-level rows; 38 tables.
- [Option levels for domains, mysteries, blessings and elements](issues/31-build-option-levels-more-families.md): domains, mysteries, blessings and elements get option-level rows; 39 tables.
- [Feat and rule follow-ups](issues/32-feat-and-rule-follow-ups.md): in-level pick order, per-value repeats, `Not` and `FeatIfValue` prerequisites and mythic feat values; 40 tables.
- [Option follow-ups](issues/33-option-follow-ups.md): option groups, arcana, lagging or borrowed option levels, once-per-option and pool selections.
- [Second round of follow-ups](issues/35-second-follow-ups.md): subrace feat slots, `level_also_from`, `OptionList.at_level` and the bloodrager's own groups.
- [Impossible Domain for druids](issues/36-impossible-domain-druids.md): no druid qualifies, as the guide says (user, 2026-11-01), reversing ticket 35's decision 3.
- [Backgrounds](issues/34-backgrounds.md): a Build start chooses a background; its skills are class skills and its feats granted feats; 41 tables.
- [Class skills from options](issues/37-option-class-skills.md): an option row's skills are class skills while the row is held.
- [Class-skill cleanup](issues/38-class-skill-cleanup.md): ticket 37's decisions 2, 3 and 5.
- [Impossible states instead of validation](issues/39-impossible-states-not-validation.md): the sheet holds only possible Builds; `Engine::removable` and `Engine::allows` replace validation.
- [Stale links get a generic error](issues/40-stale-links-generic-error.md): a link the game data can't hold opens a blank Build with one generic error; unreached ranks' picks are dropped.
- [Drop what a switch or a removed level takes away](issues/41-drop-what-a-switch-takes-away.md): the path and switch drop below their ranks *(the earlier ranks' picks: undone by 42)*.
- [Keep a switch's earlier picks](issues/42-keep-a-switchs-earlier-picks.md): undoes half of 41 (user, 2026-11-02): those picks stay, held until the switch.
- [Build the planning sheet screen](issues/12-build-sheet-screen.md): the Sheet runs the engine in the browser, with one picker behaviour and every edit through `Engine::apply`.
- [Close the extraction report's gaps](issues/43-extraction-report-gaps.md): 3 report items built from the saved pages; 4 became questions for the user.
- [Sorcerer level-1 feat, Extra Slayer Talent, deities](issues/44-sorcerer-feat-slayer-deities.md): a bloodline feat slot at sorcerer 1, Extra Slayer Talent needs slayer 2, deities out of scope.
- [Kineticist wild talents](issues/45-kineticist-wild-talents.md): the kineticist's wild talents are a selection with level, element and talent prerequisites.
- [Shorter links](issues/46-shorter-links.md): links hold 24-bit key hashes in a hand-written v1 layout, about 0.4× the old length.
- [Wild talent follow-ups](issues/47-wild-talent-follow-ups.md): the Skill Focus wild and rogue talents repeat per value; Combat Trick repeats with a combat feat slot.
- [Split engine.rs into submodules](issues/51-split-engine-module.md): a move-only split of `engine.rs` into submodules; public API and tests unchanged.
- [Organise the engine tests](issues/52-organise-engine-tests.md): a move-only reorganisation of `rules/tests/engine/`; the same tests.
- [Folder layout: apps and libs](issues/50-folder-layout.md): the `apps/` and `libs/` folder layout by `git mv`, package names kept (renamed in 53).
- [Faster edits and pickers](issues/48-ui-performance.md): faster rechecks and pickers, and compressed, cached game-data JSON; the engine's answers are unchanged.
- [Extra feats give their extra pick](issues/49-extra-feats-extra-picks.md): one more slot of the feat's selection (`SlotId::FeatSelection`); 42 tables, link group 8.
- [Rename the packages after their folders](issues/53-rename-packages.md): packages renamed `data` → `seeds` and `app` → `ui` (user, 2026-11-02).
- [Paladin mercies and the Arcane Enforcer's exploits become picks](issues/54-mercies-and-enforcer-exploits.md): each is a selection with the guide's list; data only.
- [Mythic Extra Feat and Extra Mythic Ability give their pick](issues/55-mythic-extra-picks.md): a slot at the rank they're taken; link group 9.
- [Pull the game-data lookups out of Engine](issues/56-engine-catalog.md): a refactor: the game-data lookups move into `Catalog`, and `Engine` is `{ catalog, link_keys }`.
- [Greater exploits, and the Exploiter Wizard's exploits become picks](issues/57-enforcer-and-exploiter-greater-exploits.md): slots of the arcanist's exploit selections; data only.
- [Monk ki powers become a selection](issues/58-monk-ki-powers.md): `monk/ki-power` with the guide's 14 powers; data only.
- [Kineticist infusions become a selection](issues/59-kineticist-infusions.md): `kineticist/infusions` from 3rd level, with the guide's 32 infusions; data only.
- [Talents, hexes and discoveries taken through another class become picks](issues/60-borrowed-class-selections.md): slots of the lending class's selection, or a sharing one; data only.
- [Archetype bonus-feat choices become feat slots](issues/61-archetype-bonus-feat-choices.md): `OneOf` slots of their fixed lists; data only.
- [Witch Wandering Hex and Beneficial Curse give their pick](issues/62-wandering-hex-and-beneficial-curse.md): each gives a selection slot (hex, curse); data only.
- [An archetype's alignment requirement replaces its class's](issues/63-archetype-alignment.md): applies on entering the class (user, 2026-11-02), so a Drunken Master can be built.
- [Extra Discovery and Extra Arcana reach the Vivisectionist and the Hexcrafter](issues/64-vivisectionist-hexcrafter-extra-picks.md): each feat's extra pick and prerequisite gain the archetype's own selection; data only.
- [Reason text for an option closed to an archetype](issues/65-closed-option-reason.md): a failing negated prerequisite says what rules the option out, never "Needs not …".
- [The Instinctual Warrior's monk rage powers](issues/66-instinctual-warrior-rage-powers.md): the Instinctual Warrior's five monk powers are rage power options needing the archetype.
- [The Dual-Cursed Oracle's second curse becomes a pick](issues/67-dual-cursed-second-curse.md): a second `oracle/oracles-curse` slot at level 1; data only.
- [Build classes with their picks from the start](issues/68-build-picks-where-owned.md): a refactor: classes are built with their picks from the start; extraction byte-identical.
- [Build the option families where their owners are built](issues/69-option-families-where-owned.md): elements, blessings, mysteries and orders built with their classes; the shaman hexes' "Wind" line stays as extracted (user, 2026-11-03).
- [Build the nature bond, domains and proficiencies where their owners are built](issues/70-nature-bond-and-proficiencies-where-owned.md): a refactor; extraction byte-identical.
- [Build ticket 30's option families where their owners are built](issues/71-ticket-30-families-where-owned.md): a refactor; seven `apply` passes are gone.
- [Fold the repeats into parameters and prune the dead report filters](issues/72-parameters-and-settled-cleanup.md): repeats and mythic values live in `parameters`' tables; seven dead filters gone.
- [Commit the behaviour proof as a harness](issues/73-commit-the-proof-harness.md): the behaviour proof is the `libs/proof` binary, run through `scripts/proof.sh` (user, 2026-11-03).
- [Split the largest files by subject](issues/74-split-the-largest-files.md): a pure move: `build`, `option_families`, `option_levels` and the `game_data` tests are module folders by subject; extraction byte-identical.
- [Decide whether the extractor or the data files are the master](issues/75-game-data-master.md): the files are the master (user, 2026-11-03); the extractor's knowledge goes into them, then the extractor goes (tickets 81–82).
- [Read and write the data files without sqlx](issues/79-game-files-without-sqlx.md): closed, not built; removing the extractor leaves the layout declared once.

## Not yet specified

- **Checking the data:** how we confirm the extracted data matches the guide, by spot checks or golden builds. The guide's sample-build pages aren't saved (the user dropped them), so golden builds would have to come from elsewhere. That page does hold independent BAB, ability-score and skill-rank prerequisite tables, which would cross-check the extracted feats if it's ever saved.
- **End-to-end smoke tests:** tooling and which flows to cover.

## Out of scope

- Spells known or prepared: not part of the class/feat/mythic summary.
- Gear and companions: not part of a character's build choices. This includes gear bonuses toward ability-score prerequisites (user, 2026-10-31). Prerequisites check the character's own scores: point buy, race, level increases and ability bonuses from features such as Legend's +4 (user, 2026-11-01).
- Deployment: this map ends with the app running locally.
- User accounts: Builds are anonymous and exist only as their link.
- Reproducing the guide's full text: copyright; store a summary and a link instead.
- Deities: a cleric's deity and its alignment and domain limits (user, 2026-11-02, [ticket 43](issues/43-extraction-report-gaps.md)'s decision 4).
