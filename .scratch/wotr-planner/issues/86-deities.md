# Deities

Type: task
Status: resolved
Blocked by: [85](85-sqlite-instead-of-postgres.md) (it rewrites the one migration, the seed, `.sqlx/`, the README and `CLAUDE.md`'s database steps)
Map: [WotR Build Planner](../map.md)

## Question

Add the deity choice: which deity (or atheism) a character follows, and what that does to alignment, domains, blessings, archetypes and proficiencies. This reverses the map's Out of scope line "Deities: a cleric's deity and its alignment and domain limits" ([ticket 43](43-extraction-report-gaps.md)'s decision 4, built by [ticket 44](44-sorcerer-feat-slayer-deities.md)).

Ticket 43's question 4 rested on a mistake: it said the guide's Deities and Domains page "wasn't saved". It was saved. It is not a page of its own but the last section of `guide-cache/character-creation.html` (`character-creation#deities-and-domains`, saved 2026-10-31 with the rest). The table of contents lists it under Character Creation, which is why it looked like a missing page.

This ticket is the research and the design. It changes no code or data. Scratch output (the parser, the draft data and text dumps of every saved page) is in `t86/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t86/`).

**Settled (user, 2026-11-06): spells stay out of scope for deities too** ("let's leave the spells out for now"). No spell data, and no domain-spell or deity-spell lists. Where the spell tables are, for a later ticket: each domain's and subdomain's 9-row spell table under `character-creation#<name>-domain` / `#<name>-subdomain` (37 tables); the class lists `inquisitor#inquisitor-spells`, `warpriest#warpriest-spells`, `hunter#hunter-spells`, `paladin#paladin-spells` and `witch#witch-spells`; the Hag of Gyronna's pact spells (`witch#dark-pact`). The Cleric Spells page wasn't saved. No saved page has a spell list tied to a single deity.

## What the guide says

### The data: 26 deities

`character-creation#deities` has one table, with columns Deity, LG, NG, CG, LN, TN, CN, LE, NE, CE (✓ or -), Weapon and Domains. It has **26 rows**: Abadar, Achaekek, Apsu, Asmodeus, Calistria, Cayden Cailean, Dahak, Desna, Erastil, Godclaw, Gorum, Gozreh, Green Faith, Gyronna, Iomedae, Irori, Lamashtu, Nethys, Norgorber, Pharasma, Rovagug, Sarenrae, Shelyn, Torag, Urgathoa and Zon-Kuthon.

**The draft is `t86/deities.ron`.** It holds every deity with:

- its key (the name's slug, as for backgrounds: `cayden-cailean`, `green-faith`, `zon-kuthon`);
- its name;
- `guide: "character-creation#deities"` (the table has no anchor per deity);
- `alignments`, the ✓ columns;
- `favoured_weapons`, as `weapons.ron` keys (Lamashtu has two: Falchion and Kukri);
- `domains`, as `cleric/domain/*` option keys.

**How it was parsed.** `t86/deities.py` reads `character-creation.html` from the `deities-and-domains` anchor up to `air-domain`, with Python's `html.parser`. It asserts three tables and their headers, and asserts that each alignment cell is exactly ✓ or -. Weapons and domains are matched by name with case and punctuation ignored. It writes the RON and `t86/report.txt`. The count matches the table: 26 rows and 26 keys, with no key twice. Atheism is not a row (see the design).

- **Weapons:** 26 of 27 names match a `weapons.ron` key. **One doesn't:** Asmodeus's "Mace". The data has `light-mace` and `heavy-mace` but no `mace` (question 9). "Longbow" matches `long-bow` and "Unarmed strike" matches `unarmed-strike`. The only exotic favoured weapon is Achaekek's Sawtooth Saber.
- **Domains:** every name matches one of the 37 `cleric/domain` options, and every option is offered by at least one deity. The three subdomains (Ice: Pharasma; Murder: Achaekek; Undead: Urgathoa) are ordinary options already, as the guide's Domains section treats them. **No domain is unmatched.**
- **Each deity's own alignment** isn't in the table, and it isn't needed. Every row's ✓ set is the one-step neighbourhood of one alignment ("you must be within one position of your chosen deity"), except Godclaw's (LG, LN, LE). The alignment domains on each deity's list (Law, Good, Chaos, Evil) equal that alignment's parts for all 26 (`t86/check_alignment_domains.py`: 0 differences). So "a good deity" can be read from the data as "offers the Good domain".

### Every table and list tied to deities

| Where | What it holds | Rows | Modelled? |
| --- | --- | --- | --- |
| `character-creation#deities` | The deity table above | 26 | **New data** (`deities.ron`) |
| `character-creation#domains`, first table | Domain → the alignments that can take it, and the deities that offer it | 37 | **Not stored.** It is the deity table inverted, with alignment domains only at their own three alignments (question 7). 24 rows derive exactly. 13 differ: it leaves out Apsu, Dahak and Irori wherever they appear, and Dahak and Rovagug under Evil; it **adds Torag under Glory and Zon-Kuthon under Undead**, which their deity rows don't have (question 8). It also misspells Urgathoa as "Urgothoa" and "Urgagthoa". |
| `character-creation#domains`, second table | Domain pairs "worth taking", with alignments and deities | 35 | **Not stored.** It is advice, and the guide says it isn't exhaustive. 29 rows derive. 6 differ: Shelyn's four pairs show LN instead of CG and TN, and Charm + Trickery drops CG. |
| `character-creation#domains`, list | The classes that choose one or two domains at creation: Cleric (Angelfire Apostle, Ecclesitheurge, Separatist choose two; Crusader, Demonbane Priest, Herald Caller, Priest of Balance one), Druid (Defender of the True World, Feyspeaker, Blight Druid), Divine Hunter, Inquisitor (not the Faith Hunter) | 6 lines | Already modelled as the domain selections ([ticket 31](31-build-option-levels-more-families.md)) |
| `character-creation#<domain>-domain` / `-subdomain` | 37 sections (34 domains and 3 subdomains): the powers in prose, plus a spell table | 37 | Powers are modelled (`cleric/domain/*` option rows and features, ticket 31). Spells are out of scope. |
| `warpriest#blessings` | 25 blessings, each with a minor power at 1 and a major one at 10. None for Artifice, Charm, Community, Glory, Ice, Knowledge, Murder, Plant, Rune, Scalykind, Undead or War. | 25 | Modelled (`warpriest/blessings`, ticket 31). **New:** each blessing names its domain. |

### What a deity does, per class and feature

Lines marked **stated** are what the pages say, in summary. Lines marked **open** are what the pages leave unsaid.

- **Everyone** (`character-creation#deities-and-domains`). **Stated:** any character may choose a deity or atheism; "certain character classes" gain domain powers from it. Choosing a deity restricts the character's alignment to within one step of the deity, and it restricts which domains can be chosen. **Open:** whether the alignment limit applies to non-clerics (question 1).
- **Cleric** (`cleric`). **Stated:**
  - the alignment must be within one step of the chosen deity's;
  - proficiency with the deity's weapon (Cleric Proficiencies, which Crusader, Herald Caller, Priest of Balance and Separatist keep);
  - the Ecclesitheurge's own proficiencies name the deity's favoured weapon too;
  - the Angelfire Apostle's and the Demonbane Priest's proficiency lines don't name it (question 11);
  - channel energy's type follows the deity's alignment for a neutral cleric (not modelled; no pick depends on it);
  - the Angelfire Apostle "may only select a good-aligned deity";
  - the Priest of Balance "may only choose neutral-aligned deities" (question 12);
  - the Separatist's second domain is "not on his deity's domain list" and can't be an alignment domain that conflicts with his own alignment (the alignment half is already modelled);
  - the Ecclesitheurge's Domain Mastery gives spells from the deity's other domains (spells only).
  
  **Open:** whether a cleric can be an atheist (question 5).
- **Inquisitor** (`inquisitor`). **Stated:** proficiency with the deity's favoured weapon (Inquisitor Proficiencies, which every archetype keeps). The Faith Hunter "cannot select a domain"; his Sworn Enemy is an alignment axis opposed to his deity's (a feature, not a pick: nothing to model). The Sacred Huntsmaster can't take the Animal domain (modelled). **Open:** the inquisitor page doesn't say the domain comes from the deity. Only the general line on character creation does (question 6).
- **Warpriest** (`warpriest`). **Stated:**
  - he selects two Blessings "from among those granted by the domains of his chosen deity";
  - Sacred Weapon blesses the deity's favoured weapon (damage only: no pick changes);
  - Fervor heals or harms by the deity's alignment (not modelled);
  - the Champion of the Faith's chosen alignment must match his own and one "offered by his deity", and he gains that Blessing even if the deity doesn't offer the domain (question 13);
  - the Feral Champion may only select Erastil, Gozreh or Green Faith;
  - the Mantis Zealot may only select Achaekek;
  - the Cult Leader's note suggests Lamashtu for a kukri sacred weapon (advice, and damage only).
  
  **Open:** whether a warpriest gains proficiency with the favoured weapon. The page says it doesn't: only damage. Nothing changes for him either way, since he is martial-proficient and the only exotic favoured weapon is Achaekek's, which the Mantis Zealot already holds.
- **Divine Hunter** (`hunter#divine-hunter`). **Stated:** "dedicated to a deity"; chooses "from the domains granted by her deity".
- **Druid** (`druid`). **Stated:** its own list (Air, Animal, Earth, Fire, Ice, Plant, Water, Weather), the Blight Druid's list of 12, and the Winter Child's Ice. No deity is mentioned. **Open:** the creation page lists the druid among the classes choosing domains, but never ties the druid to a deity (question 6).
- **Slayer, Deliverer** (`slayer#deity`). **Stated:** "in the service of a deity"; gains proficiency in the deity's sacred weapon; "Choosing Irori gives you Improved Unarmed Strike" (question 10). Determined Zeal and Divine Anathema key off the target's alignment, not the deity's.
- **Witch, Hag of Gyronna** (`witch#hag-of-gyronna`). **Stated:** "Must select Gyronna as a deity". The data gives the archetype no alignment rule today; Gyronna allows CN, NE and CE.
- **Arcanist, Living Deity** (`arcanist#mysterious-gift`). **Stated:** "can only be an Atheist". Hollow Spark lets her select two domains at 20, with no deity limit.
- **Mythic, Impossible Domain** (`mythic-paths#impossible-domain`). **Stated:** "You may select any domain". There is no deity limit, and the cleric page recommends it for "a domain you really, really want".
- **Hellknight, Pentamic Faith** (`prestige-classes#pentamic-faith`). **Stated:** its own list of 11 domains, after the Order of the Godclaw. No deity is mentioned (the order is not the deity).
- **Paladin** (`paladin#channel-positive-energy`). **Stated:** she channels "in the same way as a Cleric of her deity". Nothing else, so no rule.
- **No build effect** (named here so nobody hunts for them):
  - the Cavalier's Order of the Star and "same faith" allies;
  - the monk's Drunken Technique "Cayden's Trick" (a name only);
  - the demon aspects (Deskari, Nocticula, Socothbenoth are demon lords, not deities from the table);
  - the Gold Dragon's Vow of Dahak, which stays `Story(VowOfDahak)` (a story choice, not the deity choice);
  - the Mantis Zealot's Aspect of the Mantis, which names Achaekek and is already a feature.
- **No feat, mythic feat or mythic ability names a deity** (`feats.html`, `mythic-paths.html` and `other-mythic-paths.html` searched for every deity name, "deity" and "atheis").

## Build

AFK, test-first (`tdd`). Start only after ticket 85 has landed, as the only agent touching the migration and the database, with the reset steps in the repo's `CLAUDE.md` as ticket 85 leaves them. Apply the user's answers to the questions below. Where this section and an answer disagree, the answer wins, and the rest of the design adjusts to it.

### Data (`libs/game-data/deities.ron`, `rules::GameData`)

- **The deity.** `Deity { key: DeityKey, name, guide, alignments: BTreeSet<Alignment>, favoured_weapons: BTreeSet<WeaponKey>, domains: BTreeSet<SelectionOptionKey> }`. `GameData.deities` is canonical by key, and `counts` gains "Deities". `deities.ron` is a required file, written from `t86/deities.ron` with question 9's weapon. A README row and note go in `libs/game-data/README.md`. The deity's own alignment is not stored (no derived values, and the guide doesn't give it). A rule that needs it reads the deity's alignment domains instead (questions 12 and 13).
- **Atheism is not a deity row.** It is a Build start choice of its own (below), as the user's 2026-11-02 decision on the "None" background keeps non-entries out of the data.
- **Blessings name their domain:** `domain: "cleric/domain/air"` on each of the 25 `warpriest/blessings/*` options. The keys differ (`air-domain`, `destruction`), so the link is data, not derived.
- **A selection says how the deity limits it:** `deity_domains: Offered` (only options whose domain the deity offers: `cleric/domain` with every cleric archetype's use of it, `inquisitor/domain`, `inquisitor/sacred-huntsmaster/domain`, `hunter/divine-hunter/domain`, `warpriest/blessings`) or `NotOffered` (`cleric/separatist/domain`). If it is absent, the deity doesn't matter: the druid's, the Blight Druid's and the Winter Child's, `cleric/impossible-domain`, `arcanist/magic-deceiver/domain`, `hellknight/pentamic-faith`, the Champion's and the Feral Champion's. An option's domain is itself for a `cleric/domain` option, or its `domain` link for a blessing. Follow question 6's answer.
- **A feature grants proficiency with the favoured weapons:** `favoured_weapon_proficiency: true` on `cleric/cleric-proficiencies`, `cleric/ecclesitheurge/ecclesitheurge-proficiencies`, `inquisitor/inquisitor-proficiencies` and `slayer/deliverer/deity`. The rule is in `rules`, not the data: an exotic favoured weapon is held as `Exotic Weapon Proficiency` for that weapon (Achaekek: Sawtooth Saber). The unarmed strike is held as `Improved Unarmed Strike` (question 10). A simple or martial favoured weapon gives nothing: no feat checks proficiency with one weapon (ticket 34's decision 1). The sheet notes it ("Proficient with longbow (Erastil)").
- **Archetype and class prerequisites** (each a condition of its own in a top-level `All`, as ticket 63's alignment rules are):
  - `Worships` (a deity, not atheism): cleric, inquisitor, warpriest, `hunter/divine-hunter`, `slayer/deliverer` (question 2);
  - `Deity("achaekek")`: Mantis Zealot (question 1 says what happens to its user-fact alignment rule);
  - `Any([Deity("erastil"), Deity("gozreh"), Deity("green-faith")])`: Feral Champion;
  - `Deity("gyronna")`: Hag of Gyronna;
  - `Atheist`: the Living Deity option (`arcanist/magic-deceiver/ways-of-the-deceiver/living-deity`);
  - `DeityOffers("cleric/domain/good")`: Angelfire Apostle;
  - `All([Worships, Not(DeityOffers("cleric/domain/good")), Not(DeityOffers("cleric/domain/evil"))])`: Priest of Balance (question 12);
  - the Champion's four alignment blessings, when picked through `warpriest/champion-of-the-faith/chosen-alignment`: their alignment as today, plus `DeityOffers` of that alignment's domain (question 13).
- **Check** (`GameData::check`, no reference check, per the map):
  - a deity key used once;
  - at least one alignment and one favoured weapon;
  - every `domains` entry and blessing `domain` an option of `cleric/domain`;
  - `deity_domains` only on a selection all of whose options have a domain;
  - `DeityOffers` naming a `cleric/domain` option;
  - `favoured_weapon_proficiency` only on a feature some class row grants.

### Schema (the one migration, as ticket 85 leaves it: SQLite, `STRICT`, named CHECKs, inline deferred FKs)

- `deity` (`key` with the `text_key` CHECK, `name`, `guide`).
- `deity_alignment` (`deity_key` FK, `alignment` CHECKed to the nine as in `prerequisite_node`, primary key both).
- `deity_weapon` (`deity_key` FK, `weapon_key` FK → `weapon`, primary key both).
- `deity_domain` (`deity_key` FK, `domain_key` FK → `selection_option`, primary key both).
- `feature_selection.deity_domains TEXT CONSTRAINT deity_domains CHECK (deity_domains IN ('offered', 'not_offered'))`, NULL for no limit.
- `selection_option.domain_key TEXT REFERENCES selection_option (key)`, NULL except on blessings.
- `feature.favoured_weapon_proficiency INTEGER NOT NULL CONSTRAINT boolean`.
- `prerequisite_node`:
  - `kind` gains `'deity'`, `'atheist'`, `'worships'` and `'deity_offers'`;
  - a new column, `deity_key TEXT REFERENCES deity (key)`;
  - the column-set CHECK gains `'deity' → 'deity_key'`, `'atheist' → ''`, `'worships' → ''` and `'deity_offers' → 'selection_option_key'`.

That makes 42 + 4 = 46 tables. Seed, loader, `columns` and the fixture follow: the fixture's `deities.ron` gets two deities, one whose domains the fixture's domain options hold and one with an exotic favoured weapon. Atheism and the Build aren't in the database.

### Build start and link

- `BuildStart.deity: Option<Faith>`, with `enum Faith { Deity(DeityKey), Atheism }`. `None` means not chosen yet. It sits beside `alignment` and `background`.
- New `Choice::Deity`, which is among `Edit::Start`'s choices and in `Engine::removable`'s withdrawals (deity → `None`).
- `CharacterState` holds the faith.
- **Codec:** a new `KeyKind::Deity`. In `start`, after `opt(key(background))`, add `faith = 0 [none] | 1 [atheism] | 2, key(deity)`. A `Faith` gives a `LinkKeys` kind and one more byte. The layout changes, so question 14 decides: replace version 1's body and regenerate the frozen `V1_LINK` (with a comment), as tickets 34, 46, 49 and 55 did. Every older link then opens as a stale link with the generic error. The alternative is a version 2 that still decodes version 1 as "no deity". Either way, `no_two_keys_of_one_kind_share_a_link_hash` covers the deity keys.

### Engine

- **The start check** (`Walker::start`, after the background):
  - an unknown deity key is `UnknownKey` at `Choice::Deity`;
  - a deity whose `alignments` lack the start's alignment fails `Choice::Deity` (question 1). If question 1 limits the rule to clerics, it becomes a `DeityAlignment` leaf in the cleric's prerequisite instead.
  - `Engine::allows` ignores failures at the edit's own choices, so the start pickers need the fit directly: `Engine::faith_fits(&BuildStart) -> Result<(), Unmet>`. The deity picker and the alignment wheel lock a misfit with a reason, for example "Abadar allows Lawful Good, Lawful Neutral, True Neutral or Lawful Evil". They still lock, with no reason, an edit `allows` refuses.
- **New prerequisite leaves:** `Deity(DeityKey)`, `Atheist`, `Worships` and `DeityOffers(SelectionOptionKey)`, with their text ("Needs Achaekek as deity", "Needs atheism", "Needs a deity", "Needs a deity offering the Good domain"). `Worships` and `Deity` fail while the deity isn't chosen, as `Alignment` does today, so a cleric is locked in the class picker until a deity is chosen.
- **The domain filter:** a slot of a selection with `deity_domains` checks the picked option's domain against the deity's `domains` (`Offered`: it must be on the list; `NotOffered`: it must not be). The check sits beside the option's prerequisite, and a failure is a new `Issue` with its own reason ("Not a domain of Erastil" / "A domain of Erastil"). Ticket 48's picker shortcut must stay exact.
- **Favoured weapons:** holding a `favoured_weapon_proficiency` feature holds the feats above for the start's deity, as granted feats with no slot. They come and go with the deity.
- **A held domain when the deity changes.** Choices come off from the leaves inward (ticket 39). A deity that doesn't offer a held domain or blessing (or offers the Separatist's) breaks that pick, so the deity picker locks it with no reason. To change deity, the player removes the domains first. The same goes for a deity whose favoured-weapon feat a later pick needs (Irori's Improved Unarmed Strike under Stunning Fist), and for removing the deity while a `Worships` class is held. Nothing is dropped silently.

### UI

- The start panel gets a **Deity** row between Alignment and Background ("No deity" until chosen, otherwise the deity's name or "Atheism").
- A `DeityPicker` (a `StartPicker`) lists "Atheism" first, then the 26 deities by name. Each one's detail line shows its alignments' short names, favoured weapons and domains ("LG LN TN LE · Light crossbow · Earth, Law, Nobility, Protection, Travel"). It has a remove control while removable, and the locks above.
- The alignment wheel also locks the alignments the deity doesn't allow.
- Domain and blessing pickers show the deity reasons.
- The summary shows the deity and the favoured-weapon note.
- Mobile-first as ever: check the cover and inner Fold widths.

### Tests

- **Rules, test world first** (new `libs/rules/tests/engine/deities.rs`). The world gains three deities (one lawful good with Good and Law and an exotic favoured weapon; one true neutral; one whose weapon is the unarmed strike), a `Worships` class with an `Offered` domain selection, a `NotOffered` one, a blessing-like selection with `domain` links, a `Deity(..)` archetype and an `Atheist` option. Red first. The tests:
  - the start check, both ways, and `faith_fits`'s reason;
  - each new leaf and its text;
  - `Offered` and `NotOffered` slots;
  - favoured-weapon feats held and dropped with the deity;
  - `removable`: a deity is locked while a held domain or a dependent pick needs it, and free once that comes off;
  - an unknown deity key is a stale link.
  
  Also: `check` tests for each new refusal; the codec round trip with `None`, `Atheism` and a deity, plus the frozen layout; `canonical` (counts).
- **Seeds:** `files` (deities.ron required); the database round trip (deity rows, the new prerequisite kinds, `deity_domains`, `domain_key`, the feature flag); a CHECK/FK test (an unknown alignment, a deity domain naming no option, `deity` kind without `deity_key`).
- **Real data** (new `libs/seeds/tests/game_data/deities.rs`, red on the current data first):
  - the 26 deities with the table's alignments, weapons and domains;
  - an LG cleric of Erastil is offered exactly Animal, Community, Good, Law and Plant in both domain slots;
  - a TN Build can't choose Erastil, and an Erastil Build can't be Chaotic Neutral;
  - a cleric with no deity is locked with "Needs a deity";
  - a Separatist of Erastil's second domain offers every domain but Erastil's five (less the conflicting alignment domains, as today);
  - a warpriest of Torag is offered the Earth, Good, Law and Protection blessings only;
  - a Divine Hunter of Desna is offered Chaos, Good, Liberation, Luck and Travel;
  - the Feral Champion is open only to Erastil, Gozreh and Green Faith; the Mantis Zealot only to Achaekek; the Hag of Gyronna only to Gyronna; the Living Deity only to atheism;
  - an inquisitor of Achaekek holds Exotic Weapon Proficiency (Sawtooth Saber); a Deliverer of Irori holds Improved Unarmed Strike and can take Stunning Fist;
  - the druid's domain, Impossible Domain and Pentamic Faith are unchanged by the deity;
  - the Angelfire Apostle and the Priest of Balance follow question 12.
- **Proof.** This changes the `Build` type, the link format, `rules` and `libs/game-data/`. As in tickets 34 and 63, record the existing Builds on both sides, and the deity Builds on the new side only. Expected:
  - every Build with no cleric, inquisitor, warpriest, Divine Hunter, Deliverer, Hag of Gyronna, Mantis Zealot, Feral Champion or Living Deity walks exactly as before (class pickers aside);
  - each Build with one of those becomes impossible without a fitting faith;
  - given a fitting deity (or atheism), it walks as before, except: domain and blessing pickers lock the deity's non-domains; Achaekek holds the Sawtooth Saber proficiency; Irori holds Improved Unarmed Strike (question 10); Angelfire Apostle and Priest of Balance limits apply;
  - Mantis Zealot Builds answer the same as today at every alignment (question 1);
  - Hag of Gyronna Builds outside CN, NE and CE become impossible.

  Ticket 48's shortcut stays exact on the proof's Builds.

**Done when:**

- `scripts/check.sh --schema` passes, and `scripts/check.sh --full` at the end of the queue.
- Every test above shows in `cargo test -p <crate> -- --list`. The red runs are recorded in the progress log.
- `seed --check-only` counts 26 deities, and the real seed loads them (the database's foreign keys pass).
- The proof shows only the expected differences.
- The sheet works in the browser pane at both Fold widths:
  - choose Erastil, and a CN alignment is locked with its reason;
  - a cleric's domain picker offers Erastil's five, and the others are locked "Not a domain of Erastil";
  - with Animal held, every deity without Animal is locked;
  - after removing Animal, the deity changes;
  - atheism opens the Living Deity.
- Docs:
  - `libs/game-data/README.md`: the `deities.ron` row and a ticket-86 note;
  - `GLOSSARY.md`: new **Deity** and **Faith** (atheism); Build start and Prerequisite updated;
  - the map: drop the deities line from Out of scope, add a Decisions-so-far line, and note that ticket 43's question 4 misread the saved page.

## Questions for the user

1. **Who the alignment limit binds.** Character creation: "Your choice of deity restricts which alignment you can be (you must be within one position of your chosen deity)", for any character with a deity. The cleric page says it for clerics. **Recommendation:** every character who chooses a deity (the guide's general line; the guide wins). Two consequences:
   - The Mantis Zealot's alignment rule, your 2026-11-02 fact `Any([LN, LE, NE])`, equals Achaekek's ✓ set. Replace it with `Deity("achaekek")` so the fact lives in one place.
   - The Hag of Gyronna becomes CN, NE or CE only. The witch page warns "you may find the alignment choice restrictive".
2. **Which classes need a deity.** The pages tie a feature to "his/her deity" for the cleric, inquisitor, warpriest, Divine Hunter and Deliverer, and name a specific deity for the Mantis Zealot, Feral Champion and Hag of Gyronna. The paladin's "Cleric of her deity" ties nothing a Build holds. **Recommendation:** `Worships` on those five (archetypes by their class, the Divine Hunter and Deliverer as archetypes), and nothing on the paladin.
3. **When the deity is chosen.** The guide puts it in character creation, for any character. **Recommendation:** a Build start choice, beside alignment and background, needed by a deity-bound class at whatever level it's first taken. The alternative, choosing it on first entering a deity-bound class (as an archetype is), would let a Build's alignment be checked against a deity chosen later, which the creation-time rule doesn't allow.
4. **Whether non-divine characters can pick one.** "Any character may choose a deity … which is something that will crop up in conversation." **Recommendation:** yes, any character. It costs them only the alignment limit (question 1), and the sheet shows it.
5. **Atheism.** Offered at creation ("or, alternatively, choose atheism"). The Living Deity "can only be an Atheist". **Recommendation:** an explicit choice, distinct from "not chosen yet", allowing every alignment. It is required by the Living Deity and refused by the deity-bound classes, since a cleric's domains and an inquisitor's weapon come from a deity.
6. **Which domain picks the deity limits.** Stated: the cleric (creation and cleric pages), the warpriest's blessings, the Divine Hunter, and the Separatist (the opposite way). Not stated for the inquisitor's own page. Stated as unlimited: Impossible Domain ("any domain"). Given their own lists: the druid, Blight Druid, Winter Child and Pentamic Faith. The Living Deity is an atheist. **Recommendation:** limit the cleric's, the inquisitor's (by the creation page's general line; the Sacred Huntsmaster too), the Divine Hunter's and the warpriest's blessings; the Separatist's second must be off the list. Leave the druids, Impossible Domain, Pentamic Faith, Living Deity and the Champion's and Feral Champion's fixed blessings unlimited.
7. **Alignment domains need a matching alignment.** The Domains table offers Chaos, Evil, Good and Law only at their own three alignments, for example no Chaos for a True Neutral follower of Calistria. No sentence says so, and the data doesn't check it today (only the Separatist's "conflicts" rule is modelled). **Recommendation:** check it on the deity-limited picks of question 6, as `Any([Not(PickedThrough(..)), Alignment ×3])`-style prerequisites, or one engine rule if that reads better. Keep the Separatist's own "doesn't conflict" rule, and leave Impossible Domain and the Living Deity unchecked ("any domain").
8. **Torag's Glory and Zon-Kuthon's Undead.** The Domains table lists Torag under Glory and Zon Kuthon under Undead, and its Undead alignments include LN, which only Zon-Kuthon gives. The deity table gives Torag Artifice, Earth, Good, Law and Protection, and Zon-Kuthon Darkness, Death, Destruction, Evil and Law. The other 11 differences are omissions in the Domains table. **Recommendation:** the deity table, which is the per-deity source this ticket parses; the Domains table is a summary with known omissions. Say if your game shows otherwise.
9. **Asmodeus's "Mace".** The weapon list has a light and a heavy mace, no plain mace. It changes nothing a Build holds: both are simple weapons, and every class that gets the favoured weapon is already simple-proficient. **Recommendation:** `heavy-mace`, with a README note that the guide says "Mace".
10. **Irori's unarmed strike.** Only the Deliverer's page says "Choosing Irori gives you Improved Unarmed Strike". Clerics and inquisitors also gain their deity's weapon proficiency, and an unarmed strike has no proficiency feat of its own. **Recommendation:** an Irori follower with any favoured-weapon proficiency feature (cleric, Ecclesitheurge, inquisitor, Deliverer) holds Improved Unarmed Strike. The Deliverer line says what that proficiency means. It opens the unarmed feats (Stunning Fist, the styles).
11. **The Angelfire Apostle's and Demonbane Priest's favoured weapon.** Their proficiency lines list simple weapons and armour without the deity's weapon. The other archetypes keep "Cleric Proficiencies", and the Ecclesitheurge names it. **Recommendation:** follow the pages, so they don't get it. That matters only for a Demonbane Priest of Achaekek or Irori; the Apostle's deity is good.
12. **"Good-aligned" and "neutral-aligned" deities** (Angelfire Apostle, Priest of Balance). The table gives no deity alignment, but each deity's alignment domains match its alignment for all 26. **Recommendation:** a good deity offers the Good domain (Apsu, Cayden Cailean, Desna, Erastil, Iomedae, Sarenrae, Shelyn, Torag). A neutral one offers neither Good nor Evil (Abadar, Calistria, Godclaw, Gorum, Gozreh, Green Faith, Irori, Nethys, Pharasma), matching the Priest of Balance's own LN, TN or CN. The other reading, any deity with a neutral part including NG and NE, would let a Priest of Balance follow Sarenrae.
13. **The Champion of the Faith's "alignment offered by his deity".** **Recommendation:** the deity offers that alignment's domain (an Iomedae Champion may choose Good or Law; a Gozreh Champion none). This is the same reading as question 12, and gives the same answer as the deity's own alignment for all 26.
14. **The link format.** A new start field changes version 1's layout. Ticket 46 replaced version 1 because "no link has shipped", and tickets 34, 49 and 55 regenerated the vector since. **Recommendation:** the same again: replace version 1's body and regenerate `V1_LINK`, unless you have links you want to keep opening. In that case: a version 2, with version 1 decoding as "no deity".

**User, 2026-11-06:** all fourteen recommendations above. Spells stay out of scope (user, 2026-11-06).

## Progress

- 2026-11-06: read `CLAUDE.md`, the map's Notes and this ticket. Re-ran `t86/deities.py` on the saved
  `guide-cache/character-creation.html`: the same 26 rows as the draft (`diff` empty). The draft became
  `libs/game-data/deities.ron`, with `heavy-mace` for Asmodeus (question 9).
- Rules, test-first. The test world gained three deities (the Lawgiver: LG, NG, LN, fire and war, the dueling
  sword; the Balance: TN ±1, water, the longsword; the Fist: war and water, the unarmed strike), a Priest (`Worships`,
  an `Offered` domain, a `NotOffered` heresy, a blessing selection whose options name their domain, a Zealot needing
  the Lawgiver and an Apostle needing a deity offering fire), a Seeker (creeds needing atheism or a deity) and
  Stunning Fist. Red run with the types stubbed: 33 failing (22 in `deities`, the codec's 5, and the world's lists);
  `check`'s 6 new tests red before the checks. Then green: `cargo test -p rules` 403 passed.
- Seeds: the migration (4 tables, 46 in all), seed, loader, `columns`, `files`, the fixture (two deities, a cleric
  with `Worships`, `Offered`/`NotOffered` selections, a blessing's domain, the favoured-weapon flag, the four new
  leaves). Real-data tests in `libs/seeds/tests/game_data/deities.rs`: red on the data before its edits (13 of 18
  failing; the 26-deity table and four "as before" tests passed), green after. The 32 older real-data tests that
  built a deity-bound class without a faith were given one: `support::faithful` picks the first of atheism and the
  deities (key order) a Build is possible with; seven tests swapped a domain pair no deity offers (fire + nobility →
  fire + glory, Sarenrae's, and so on) or now check the alignment-domain rule. `cargo test -p seeds` all pass.
- Dev database rebuilt per `CLAUDE.md` (delete, `sqlx database create`, `migrate run`, seed, `cargo sqlx prepare`):
  the seed loads 26 deities and the foreign keys pass.
- UI: a Deity row in the Build start, the deity picker, the wheel's locks with the deity's reason, the summary's faith
  and favoured-weapon note. Browser pane (scratch build on 127.0.0.1:3950, stopped after): see the Answer.
- Proof: `scripts/proof.sh rules` can't read the new data on side A, so a `both` mode was added; it also exposed a
  shared-target bug in `proof.sh` (fixed). Breakdown in the Answer. `check.sh --schema` and `--schema --full`: ALL-OK.

## Answer

Built as designed, with the user's fourteen answers (2026-11-06); spells stay out.

**Data.** `libs/game-data/deities.ron`: the guide's 26 deities (`character-creation#deities`), each with its
alignments (the row's ticks), favoured weapons (`weapons.ron` keys) and domains (`cleric/domain` options), from the
parser's draft re-run on the saved page. Asmodeus's "Mace" is `heavy-mace` (question 9; both maces are simple, so no
Build depends on it). The classes: the 25 warpriest blessings name their domain; `deity_domains: Offered` on
`cleric/domain`, `inquisitor/domain`, `inquisitor/sacred-huntsmaster/domain`, `hunter/divine-hunter/domain` and
`warpriest/blessings`, `NotOffered` on `cleric/separatist/domain`; `favoured_weapon_proficiency` on Cleric,
Ecclesitheurge and Inquisitor Proficiencies and the Deliverer's Deity; the prerequisites below. README row and
ticket-86 note.

**Schema** (46 tables): `deity`, `deity_alignment`, `deity_weapon`, `deity_domain`;
`feature_selection.deity_domains` (`offered`/`not_offered`, NULL), `selection_option.domain_key`,
`feature.favoured_weapon_proficiency`; `prerequisite_node` kinds `deity` (`deity_key`), `atheist`, `worships`,
`deity_offers` (`selection_option_key`). Seed, loader, `columns`, files reader (`deities.ron` required), fixture.

**Link.** Version 1's start gains `faith = 0 [none] | 1 [atheism] | 2, key(deity)` after the background
(`KeyKind::Deity`); `V1_LINK` regenerated with the Balance as its faith (question 14). Every older link opens as a
stale link. The proof's `bench-links.tsv` was rewritten into the new layout (one 0 byte inserted, by a scratch
script).

**Rules.** `BuildStart.deity: Option<Faith>` (`Faith::Deity(DeityKey) | Atheism`), `Choice::Deity` (in
`Edit::Start`'s choices and `removable`'s withdrawals), `CharacterState.faith` and `deity_domains`.
- The start: an unknown deity, or one whose alignments lack the start's alignment, fails `Choice::Deity` (so the link
  is stale). `Engine::faith_fits(&BuildStart) -> Result<(), Issue>` gives the start pickers the fit;
  `Issue::DeityAlignment` reads "Erastil allows Lawful Good, Neutral Good or Lawful Neutral". (A design choice: an
  `Issue`, not an `Unmet`, so the reason reads as the ticket's example.)
- Leaves `Deity`, `Atheist`, `Worships`, `DeityOffers`: "Needs Achaekek as deity", "Needs atheism", "Needs a deity",
  "Needs a deity offering the Good Domain"; negated, "Not for a deity offering the Evil Domain".
- Where they apply: `Worships` on the cleric, inquisitor and warpriest classes and the Divine Hunter and Deliverer;
  `Deity("achaekek")` on the Mantis Zealot (its user-fact alignment rule, the same three alignments, question 1);
  `Any` of Erastil, Gozreh and Green Faith on the Feral Champion; `Deity("gyronna")` on the Hag of Gyronna; `Atheist`
  on the Living Deity way; `DeityOffers(good)` beside the Angelfire Apostle's alignment; `Not(DeityOffers)` of Good and
  of Evil beside the Priest of Balance's (question 12; no `Worships` there, the class has it). Question 7: each
  alignment domain needs its alignment when picked through one of the four deity-limited domain selections, and each
  alignment blessing through `warpriest/blessings`; question 13: as the Champion's chosen alignment, a deity
  offering its domain too.
- The domain filter: a pick in a `deity_domains` selection checks its domain (`domain`, or itself) against the
  deity's; `Issue::DeityDomain` reads "Not a domain of Erastil" / "A domain of Erastil"; without a deity it's "Needs a
  deity".
- Favoured weapons: while a flagged feature is held, an exotic favoured weapon is held as Exotic Weapon Proficiency
  for it and the unarmed strike as Improved Unarmed Strike (question 10), granted with no slot.
  `Engine::favoured_weapons_note` reads "Proficient with Long Bow (Erastil)".
- `GameData::check`: a deity key once; at least one alignment and favoured weapon; deity domains, blessing domains
  and `DeityOffers` name `cleric/domain` options; a deity-limited selection offers only options with a domain; a
  favoured-weapon feature is granted by some class or archetype row.
- Reasons: an "and" of "or"s that share one negated case reads that case once ("As Chosen Alignment Blessing, needs
  Lawful Evil alignment or … , and a deity offering the Evil Domain").

**UI.** A Deity row between Alignment and Background ("Choose…", the deity's name or "Atheism"); the start summary
says "No deity" until chosen. The deity picker lists Atheism, then the 26 by name with "LG LN TN LE · Light Crossbow ·
Earth, Law, Nobility, Protection, Travel"; misfits are locked with the alignment reason, edits `allows` refuses with
none; a remove control while removable. The alignment wheel locks the deity's misfits and says the deity's limit
under the wheel. Domain and blessing pickers show the deity reasons. The sheet summary shows "Faith: Iomedae ·
Proficient with Longsword (Iomedae)".

**Tests.** Rules: `tests/engine/deities.rs` (23: the start check and `faith_fits`'s reason, each leaf and its text,
`Offered`, `NotOffered` and blessing slots, favoured-weapon feats held and gone with the deity, Stunning Fist,
`removable`, a stale link, the one-case reason), 6 `check` tests, the codec (faith round trip, the regenerated vector,
the frozen body, a tag the encoder never writes); the world's list tests updated for the Priest, the Seeker and
Stunning Fist. Seeds: `files` (deities read, a missing `deities.ron` fails), `database` (every new leaf round trips; a
schema test for the deity tables, the new node kinds, `deity_domains`, `domain_key` and the flag). Real data:
`tests/game_data/deities.rs` (18, every case the ticket lists). Red runs: see the progress log.

**Browser** (scratch build on 127.0.0.1:3950, cover width 412 and inner 884, no console errors, server stopped):
Erastil chosen, the wheel locks CG, TN, CN, LE, NE and CE with "Erastil allows Lawful Good, Neutral Good or Lawful
Neutral"; a cleric's domain picker offers Animal, Community, Good, Law and Plant and locks the other 32 "Not a domain
of Erastil"; with Animal held every other deity is locked (Gozreh's with its alignment reason at LG, the rest with
none) and Atheism too; after removing Animal, Iomedae was picked and the summary showed her longsword; with atheism a
Magic Deceiver's Ways of the Deceiver offer the Living Deity.

**Proof.** `scripts/proof.sh rules` can't run this ticket: side A (`HEAD`'s crates) can't read the working tree's
game data (unknown fields such as `deity_domains`, unknown `Worships`), and `HEAD`'s codec can't read the new link
layout. So the harness gained a `both` mode: as `rules`, but side A records `HEAD`'s own game data and both sides
record the working tree's Builds as JSON lines (`export --json`, `record --builds`). The harness still compiles
against `HEAD`'s crates: the faith is set through serde (`generate::with_faith`; `HEAD`'s Build start drops it),
and a Build that's impossible without a faith gets the first of atheism and the deities (key order) it's possible
with (`generate::faithful`; recorded decision). New `t86 …` Builds (a cleric 3 of each deity, and each deity-bound
class or archetype with the deity the guide names; empty and filled) are recorded on side B only, their full Builds
too, and ticket 48's shortcut is checked on them (`SHORTCUT_CHECKED` gains `t86 `). The sheet's hash leaves out an
empty faith (`faith: None, deity_domains: {}`), a field side A's states don't have.

Found on the way: `proof.sh` built both sides in one `CARGO_TARGET_DIR`. Cargo fingerprints a workspace's crates by
their paths within it and the files' times, and `git archive` gives side A's files their commit's time, so side A's
build counted as up to date and copied side B's harness: **side A was the working tree's crates**. The first run
here showed it (A's harness asked for `deities.ron`). Each side now builds in its own folder (`A/`, `B/`). Earlier
`rules` proofs (tickets 76–85) that reported no differences may have compared the working tree with itself; worth
re-running one if they matter.

Result against `HEAD` (`42398fe`): A 1994 Builds, B 2146 (152 only in B: the deity Builds and their full Builds),
all 1994 differing. Every difference was traced, with a scratch breakdown tool (outside the repo) that lists each
side's class picker per option, every slot picker's statuses, each `allows` edit, `removable` and the pickers of the
Builds that differ, then compared line by line:
- **1833 Builds differ only in the class pickers.** In every Build the cleric, inquisitor and warpriest (with their
  archetypes), the Divine Hunter, the Deliverer and the Hag of Gyronna are locked "Needs a deity" (or by their deity
  rule); no other class option changed except in the 4 Builds that became impossible (below), where every later
  class shows `BreaksLater`.
- **134 Builds with a faith** (`faithful` gave a cleric, inquisitor, warpriest, Divine Hunter, Deliverer or a deity
  archetype one): the sheet holds the faith, the deity's domains and the favoured-weapon feats; slots: domain and
  blessing pickers lock the deity's non-domains ("Not a domain of Abadar") and the Separatist's second its domains ("A
  domain of Abadar") (4,322 option statuses in all), alignment domains and blessings need their alignment (112,
  question 7), the Champion's alignment blessing a deity offering it (21, question 13); none of these Builds follows
  Achaekek or Irori, so the favoured-weapon feats show only in the deity Builds (side B only: an inquisitor of
  Achaekek holds Exotic Weapon Proficiency (Sawtooth Saber), a cleric or Deliverer of Irori Improved Unarmed Strike,
  which opens Stunning Fist and the styles' need of it), and in the tests; `removable`: the
  alignment can't come off while an alignment domain needs it, a Separatist's archetype can't (his second domain
  would land in a deity-limited slot), the Mantis Zealot's alignment now can (its rule is Achaekek's, question 1);
  `allows`: changing the alignment breaks a held alignment domain, and a Mantis Zealot's misfit alignment is now
  refused by the wheel's `faith_fits`, not by `allows` (the deity is the edit's own choice).
- **27 Builds without a faith, with more than the class pickers:** 3 Magic Deceiver Builds (the Living Deity way is
  locked "Needs atheism"); 13 cleric 10 / prestige 10 Builds and 6 other deity-bound Builds that are impossible on
  both sides (the prestige entry, or the alignment: an Angelfire Apostle or Mantis Zealot at True Neutral), where the
  cleric levels now also say "Needs a deity" and the edits that would have fixed them on side A still need one; and
  **4 Builds that became impossible**: the Hag of Gyronna at True Neutral (Gyronna allows CN, NE, CE) and three
  cleric domain pairs no deity offers at True Neutral (Animal + Knowledge, Trickery + Air, Scalykind + Law).

No difference falls outside a deity rule. Ticket 48's shortcut held on the 955 Builds it was checked on (1,392,409
feat options), the deity Builds included.

**Checks.** `scripts/check.sh --schema`: ALL-OK; `scripts/check.sh --schema --full` (once, at the end): ALL-OK.
`seed --check-only` counts 26 deities; the real seed loads them and the foreign keys pass. `cargo fmt --all --check`
passes. Nothing committed or staged.

**Lead, 2026-11-06: re-proof of tickets 76–85 with the fixed script.** `scripts/proof.sh rules` (each side in its own target folder) in a checkout of 923d018, the commit before this ticket, against 31341d7, the commit before ticket 76: 1,999 Builds a side, 0 only in A, 0 only in B, 0 differing. The two harnesses are different binaries, and side A's source still has `Engine::data()` and `slot_ignores_prerequisites`, so it really is the pre-76 code. Tickets 76, 77, 78, 80, 83 and 85 changed no behaviour.
