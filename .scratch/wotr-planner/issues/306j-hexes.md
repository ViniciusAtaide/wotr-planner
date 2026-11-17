# Importer S2, 306j: hexes

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306i
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

**Also from 306f (main session, 2026-11-16):** `rogue/sylvan-trickster/fey-tricks` is written here, not in 306f. The game's 64 include `HexChannelerChannelFeature` b40316f05d4772e4894688e6743602bd, which has no entry until this slice, so 306f left the selection whole. The planner offers 62 and also lacks one rogue talent the game lists. Un-ignore `a_sylvan_tricksters_fey_tricks_are_the_games_64`.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306j-hexes.md`.

**What was built**
- `shaman/hex` lists the game's 34 as its own: 11 new copies `options/shaman/hex/<name>.ron` (`ShamanHex*Feature` blueprints, `aliases: ["witch/hex/<name>"]`). New option `witch/hex/channel` (`HexChannelerChannelFeature` b40316f0…, `ranks: 20`, needs the Hex Channeler archetype). New `magus/hexcrafter/hex` (row 4 of the Hexcrafter, the only selection in that row: no slot moves). New feat `extra-hex-shaman` with `shaman/extra-hex` (no alias; added to the general and dragon feat lists). `extra-hex` and `witch/extra-hex` are the witch's alone now.
- Writer: `--only shaman/hex,witch/hex,witch/extra-hex,shaman/extra-hex,magus/hexcrafter/hex,magus/hexcrafter/hex-arcana,rogue/sylvan-trickster/fey-tricks,shaman/wildland-shaman/spirit` wrote four lists (hexcrafter hex, hex arcana gained the channel, fey tricks = the game's 64: Weapon Snatcher out, Uncanny Dodge and Improved Uncanny Dodge and the channel in, shaman extra hex). `shaman/hex`'s options were moved by hand first, and the two new selections were seeded with one shared option (the writer skips a selection with no list). `wildland-shaman/spirit` stays unwritten (a copy; its `known.txt` line is 316's).
- `known.txt`: removed 293's `shaman/hex` and the `ExtraShamanHexSelection` pool line, 276's fey-tricks, 278's `witch/hex`, 283's `hexcrafter/hex-arcana`; added 278's `shaman/hex/iceplant` (the game's not-`WitchHexIceplantFeature`, as the witch's already is) and 264's `extra-hex-shaman` (also names `WinterWitchShamanHexSelection`); the Wildland Shaman line is 316's. The game check exits 0.
- **Decision (builder, not in the section):** `witch-wandering-hex`'s planner prerequisite (any hex held, until 259) and its planner selection now name the eleven shaman copies and the channel, so a shaman keeps the ability as before; without it 7 picks of a Build were refused. Game: the ability needs witch 1, Sylvan Trickster 2 or Hexcrafter 4, which 259 will write.

**Tests:** `importer_lists::the_shamans_hexes_are_their_own_and_open_old_links_through_aliases`; un-ignored `data::a_sylvan_tricksters_fey_tricks_are_the_games_64` (passes). `follow_ups::a_battle_master_hex_names_no_weapon_and_8_and_16_each_choose_one` still fails (278), so it stays ignored. Updated for the new shapes: `data::a_shaman_has_its_own_eleven_copies_of_witch_hexes` (was shares), the Hexcrafter's hex lists (27 and 42), the dragon feats (262), the Extra feat tables of `extra_picks`, `nested_selections`, `parameterised_feats` (36 repeating feats, the channel Times(20)), `prerequisites_as_components` (634). Citations: ShamanHexSelection 4223fe18…, WitchHexSelection 9846043c…, HexcrafterMagusHexMagusSelection a18b8c3d…, ExtraWitchHexSelection b6054088…, ExtraShamanHexSelection d0b4c824…, SylvanTricksterTalentSelection 290bbcc3…, HexChannelerChannelFeature b40316f0….

**Proof (offers, base `a6876cd6`, this lane):** links: 1,996 open, 1,968 the same, 20 through an heir, 8 refused, 0 changed. The section expected 22 heirs and 3 refused. The 8 refused are 5 Builds' names listed twice (the corpus's and the "full" copies), in `306-refused.txt` (own block): a shaman's Extra Hex holding a shaman hex (the section's), and a Hexcrafter's Extra Hex (the game's ExtraWitchHexSelection needs `WitchHexSelection`, which a Hexcrafter lacks: it holds `HexcrafterMagusHexMagusSelection`; so the slot is gone, toward the game; not in the section). Offers: 18,030,843 recorded, 18,047,962 now; only recorded 3,615, only now 20,734 (the new feat offered at every feat slot); 45,601 statuses changed in 38 Builds, all shamans and Hexcrafters (heirs, refused, and `extra-hex` ok to Unmet for a shaman). No slot-order refusal. No gold build changed.
**Not verified:** why the offers compare reports `witch-wandering-hex` ok to Unmet in 21 Builds; the shaman cannot take it in the game, so the direction is the game's, but I did not trace it to a Build.
