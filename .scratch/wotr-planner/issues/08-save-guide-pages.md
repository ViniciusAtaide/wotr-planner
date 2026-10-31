# Save the guide pages

Type: task
Status: open
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

GameFAQs can't be fetched automatically. Its robots.txt disallows ClaudeBot, the site's terms forbid scraping, and the fetch returned 403 (see [How the guide is laid out](01-guide-anatomy.md)). So the user saves the pages from their own browser, and the extractor parses the saved files offline.

HITL checklist for the user:

1. Create `~/code/wotr-planner/guide-cache/` and git-ignore it. It is never committed.
2. For each slug below, open `https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/<slug>` and save it as **Webpage, HTML Only** to `guide-cache/<slug>.html`.
   - **Character creation:** `character-creation`
   - **Feats:** `feats`
   - **Base classes:** `alchemist`, `arcanist`, `barbarian`, `bard`, `bloodrager`, `cavalier`, `cleric`, `druid`, `fighter`, `hunter`, `inquisitor`, `kineticist`, `magus`, `monk`, `oracle`, `paladin`, `ranger`, `rogue`, `shaman`, `shifter`, `skald`, `slayer`, `sorcerer`, `warpriest`, `witch`, `wizard`
   - **Prestige classes:** `prestige-classes`
   - **Mythic:** `mythic-paths`, `aeon`, `angel`, `azata`, `demon`, `lich`, `trickster`, `gold-dragon`, `other-mythic-paths`
   - **Test fixtures:** `character-building`, `appendix-default-builds`
3. Note the date you saved them. It becomes the guide-version stamp on the extracted data.

Done when all 40 files exist and each contains a `div#faqwrap`. The agent can check that with a script.
