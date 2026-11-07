# Deities

Type: task
Status: open
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
