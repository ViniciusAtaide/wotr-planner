# Impossible Domain for druids

Type: task
Status: resolved
Blocked by: 35
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 35](35-second-follow-ups.md)'s decision 3 (user, 2026-11-01): a druid holding a domain through Nature Bond, a Blight Druid and a Winter Child qualify for Impossible Domain (the guide's prerequisite is "Domain"), and its domain is levelled by druid levels, as ticket 35 did for the Hellknight's Pentamic Faith.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a druid 3 with a Nature Bond domain, a Blight Druid 3 and a Winter Child 3 can each take Impossible Domain, and its domain is level 3; a druid with an animal companion can't.

## Progress log

- 2026-11-01: started. Baseline: HEAD 6b913b5, clean tree; scratch dir `t36/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t36/`). Read the data: the druid's domains are `druid/domain` (via the Nature Bond option `druid/nature-bond/domain`), `druid/blight-druid/domain` and `druid/winter-child/domain`. All three share `cleric/domain/*` options and are levelled by druid levels. `cleric/impossible-domain` (`level_from: OtherOptions`) offers all 37, so a druid's held domain would already set its level. The only change needed would be Impossible Domain's prerequisite: `SelectionOption("druid/nature-bond/domain")`, `Archetype("druid/blight-druid")` and `Archetype("druid/winter-child")`, with no engine, schema or `Build` change.
- 2026-11-01: **stopped before any code.** I read the saved druid page to find the guide phrase for the rule to assert, and the guide rules this case out. In the druid's characteristics (`guide-cache/druid.html`, set off in a blockquote, right after the Nature Bond domain list): "If you choose a domain, you can't take the Impossible Domain mythic ability." Ticket 35's question 3 cited only "Prerequisites: Domain" and missed this line, so decision 3 was made without it. The map says the guide wins over the game, and this ticket says the decision is the user's, so I have asked rather than picked one (below). Nothing changed: no test, code, data, schema or DB. The current data already matches the guide, since no druid source is in Impossible Domain's prerequisite.
- 2026-11-01: the user chose option 1 (below), so I resumed. Real-data test first (`game_data.rs`, 1 new: `no_druid_can_take_impossible_domain_whatever_its_domain`). It passed at once, since the data already follows the guide. To see it fail, I ran it against decision 3's data: `game-data/mythic/abilities.ron` was given each druid source in turn, then restored from `t36/abilities.ron.orig` (`git diff game-data/` empty). It failed each time (`t36/real-data-red.txt`: Nature Bond, Blight Druid, Winter Child each `Some(Ok(()))`). Then the guard `extract::second_follow_ups::no_druid_impossible_domain`, run by `apply` after `impossible_domain`. The one-off `t36/check/` runs it alone over `game-data/`: it passes and changes nothing. It also runs it over copies given each druid source, and each is refused (`t36/check-log.txt`). A fresh extraction (`t36/extract-new/`) is byte-identical to ticket 35's (`t35/extract-new/`): RON and report, 33 items. No data, schema or DB change, so no DB reset. `game-data/README.md` gains one line. Every Verification check passes. Nothing staged. Done.

## Question for the user

**The guide says a druid who chooses a domain can't take Impossible Domain.** The druid page's characteristics give the Nature Bond domain list ("Air, Animal, Earth, Fire, Ice, Plant, Water and Weather"), then: "If you choose a domain, you can't take the Impossible Domain mythic ability." Decision 3 (ticket 35) says the opposite. The guide says nothing separately about the Blight Druid's or Winter Child's domains. Both are the druid's domain under an archetype (the Blight Druid "May choose from the following domains", the Winter Child "Must select a specific domain"), so the sentence most likely covers them too. Relatedly, the Trickster's Lore (Religion) II domains are excluded the same way ("does not allow you to take the Domain Zealot and Impossible Domain mythic abilities").

1. **Follow the guide (recommended).** No druid domain qualifies: Nature Bond, Blight Druid and Winter Child alike. No data change. The ticket closes with a real-data test that pins it (a druid 3 with a Nature Bond domain, a Blight Druid 3 and a Winter Child 3 can't take Impossible Domain) and a guard in `extract::second_follow_ups` that asserts the guide's sentence and that no druid source is in the prerequisite.
2. **Keep decision 3 as is.** All three qualify, levelled by druid levels, as specified, against the guide's explicit line. This is recorded as a fact confirmed by the user, as the skills table was.
3. **Split.** The Nature Bond domain doesn't qualify (the guide says so), but the Blight Druid's and Winter Child's do, reading the sentence as about the base druid's Nature Bond only.

## Decision

**User, 2026-11-01:** option 1, follow the guide ("yeah druid does not qualify for impossible domain"). This reverses ticket 35's decision 3.

## Answer

Done 2026-11-01, uncommitted. No druid can take Impossible Domain. The druid's page says a druid who chooses a domain can't ("If you choose a domain, you can't take the Impossible Domain mythic ability."), and the user chose to follow it over ticket 35's decision 3. The Blight Druid's and Winter Child's domains are the druid's domain under an archetype, so they don't qualify either. The data already matched: Impossible Domain's prerequisite names no druid source. So nothing in the data, schema, database, engine or `Build` changes, and the v1 link vector stays.

### Tests

- Real data (`game_data.rs`, 1 new): a druid 3 / fighter 3 with a Nature Bond domain (Weather), a Blight Druid 3 (Death) and a Winter Child 3 (Ice) each hold their domain at option level 3. Each is refused Impossible Domain at Mythic rank 1, as is a druid with an animal companion. A cleric 3 / fighter 3 is offered it.
- It passed at once on the real data. Against decision 3's data, it failed for each of the three druid sources added alone (`t36/real-data-red.txt`; the file was restored after).

### Guard

`extract::second_follow_ups::no_druid_impossible_domain(guide, data, log)`, run by `apply` after `impossible_domain`. It asserts the druid section's sentence and that no druid class level, archetype or option is among Impossible Domain's prerequisites, and changes no data. It is public so the one-off `t36/check/` (Rust, in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t36/`) could run it alone over `game-data/`. There it passes and changes nothing. It refuses copies given `SelectionOption("druid/nature-bond/domain")`, `Archetype("druid/blight-druid")` or `Archetype("druid/winter-child")`. The extractor was never run over `game-data/`.

### Extractor

A fresh extraction (`t36/extract-new/`) is **byte-identical** to ticket 35's (`t35/extract-new/`), RON and report (33 items, 0 check problems).

### Docs

- `game-data/README.md`: the prerequisite names no druid, and why.
- `GLOSSARY.md` and `game-data/EXTRACTION_REPORT.md`: unchanged.

### Verification

- `cargo test --workspace`: pass (rules 263 + unit 1, extract 84, data files 24 + game_data 102 + postgres 22). The new test is in `cargo test -p data --test game_data -- --list`.
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- Dev DB: not reset, since nothing in the schema or `.ron` files changed (the data hash ignores the README). It still holds ticket 35's reseed of the same files, and the postgres tests pass against it.
- `--check-only` gives 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths (unchanged).
- The v1 link vector is unchanged (the `Build` type didn't change).
