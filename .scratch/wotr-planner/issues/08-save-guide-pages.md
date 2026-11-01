# Save the guide pages

Type: task
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

GameFAQs can't be fetched automatically. Its robots.txt disallows ClaudeBot, the site's terms forbid scraping, and the fetch returned 403 (see [How the guide is laid out](01-guide-anatomy.md)). So the user saves the pages from their own browser, and the extractor parses the saved files offline.

**Fastest way:** run `scripts/save-guide-pages.sh`. It opens each page, puts its file name on the clipboard, checks the saved file, skips pages already saved, and writes `guide-cache/SAVED_AT`.

HITL checklist for the user:

1. Create `~/code/wotr-planner/guide-cache/` and git-ignore it. It is never committed.
2. For each slug below, open `https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/<slug>` and save it as **Webpage, HTML Only** to `guide-cache/<slug>.html`.
   - **Character creation:** `character-creation`
   - **Feats:** `feats`
   - **Base classes:** `alchemist`, `arcanist`, `barbarian`, `bard`, `bloodrager`, `cavalier`, `cleric`, `druid`, `fighter`, `hunter`, `inquisitor`, `kineticist`, `magus`, `monk`, `oracle`, `paladin`, `ranger`, `rogue`, `shaman`, `shifter`, `skald`, `slayer`, `sorcerer`, `warpriest`, `witch`, `wizard`
   - **Prestige classes:** `prestige-classes`
   - **Mythic:** `mythic-paths`, `aeon`, `angel`, `azata`, `demon`, `lich`, `trickster`, `gold-dragon`, `other-mythic-paths`
3. Note the date you saved them. It becomes the guide-version stamp on the extracted data.

Done when all 38 files exist and each contains a `div#faqwrap`. The agent can check that with a script.

## Comments

**2026-10-31, user:** `character-building` and `appendix-default-builds` are not necessary; the list is now 38 pages.

## Answer

All 38 pages are saved in `guide-cache/` (git-ignored), checked 2026-10-31. Each file holds `div#faqwrap`, and its canonical link matches its file name. `guide-cache/SAVED_AT` = `2026-10-31`; that date is the guide-version stamp for the extracted data. Saved with `scripts/save-guide-pages.sh`. Some early saves used "Webpage, Complete", which left `…_files` folders that nothing needs.
