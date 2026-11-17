# 306j: the shaman's hexes are its own; the witch's, Hexcrafter's and Extra Hex lists are the game's

## What the data says now

- `shaman/hex` (`ShamanHexSelection` 4223fe18c75d4d14787af196a04e14e7) lists the game's 34 as its own options. The eleven it
  used to share with the witch are copies: `options/shaman/hex/{ameliorating, aura-of-purity, evil-eye, fortune, healing-hex,
  iceplant, misfortune, nails, protective-luck, slumber, ward}.ron`, each with the shaman's blueprint
  (`ShamanHex<Name>Feature`) and `aliases: ["witch/hex/<name>"]`, so an old link's pick opens as the copy. Name, summary and
  guide are the witch's (design §0, decision 6).
- New option `witch/hex/channel` (`HexChannelerChannelFeature` b40316f05d4772e4894688e6743602bd, own words), owned by
  `witch/hex`, with `ranks: 20` as the game has it. It needs the Hex Channeler archetype (`PrerequisiteArchetypeLevel`
  `HexChannelerWitchArchetype`). It joins `witch/hex` (27 now), `magus/hexcrafter/hex`, `magus/hexcrafter/hex-arcana` and
  `rogue/sylvan-trickster/fey-tricks`; it is also an Extra Hex option.
- New selection `magus/hexcrafter/hex` (`HexcrafterMagusHexMagusSelection` a18b8c3d6251d8641a8094e5c2a7bc78, 27 shared
  options), row 4 of `archetypes/magus/hexcrafter.ron` in place of `witch/hex`. Row 4 holds no other selection, so no slot moves.
- New feat `extra-hex-shaman` (`ExtraShamanHexSelection` d0b4c8245d504b8c9c6d3fccc1f8c5b6) with the selection
  `shaman/extra-hex` (34 shared options, `needs_slot_of: shaman/hex`). No alias: an old `extra-hex` pick stays valid as a feat.
  `witch/extra-hex` lost `shaman/hex` from `needs_slot_of`, and the `extra-hex` feat's prerequisites lost the 23 shaman hexes
  and gained the channel.
- `rogue/sylvan-trickster/fey-tricks` (`SylvanTricksterTalentSelection` 290bbcc3c3bb92144b853fd8fb8ff452) is the game's 64: the
  writer dropped Weapon Snatcher (the game's list lacks it) and added the rogue's Uncanny Dodge and Improved Uncanny Dodge
  and the channel.
- `witch-wandering-hex`'s planner prerequisite (any hex held, until 259) names the eleven shaman copies too, so a shaman holding
  one keeps it open.

## Written by the writer, and by hand

`game-check --write option-lists --only shaman/hex,witch/hex,witch/extra-hex,shaman/extra-hex,magus/hexcrafter/hex,magus/hexcrafter/hex-arcana,rogue/sylvan-trickster/fey-tricks,shaman/wildland-shaman/spirit`
wrote four lists (`magus/hexcrafter/hex`, `magus/hexcrafter/hex-arcana`, `fey-tricks`, `shaman/extra-hex`). The writer skips a
selection with no list, so `magus/hexcrafter/hex` and `shaman/extra-hex` were seeded with one shared option first
(`witch/hex/evil-eye`, `shaman/hex/chant`). `shaman/hex`'s options were moved by hand (the copies must be its own before the
writer can resolve them). `shaman/wildland-shaman/spirit` stays unwritten: its blueprint is another selection's (a copy, 316), so
its `known.txt` line is now 316's.

## Left in `known.txt`

- `shaman/hex/iceplant`: the game gives it `PrerequisiteNoFeature` of `WitchHexIceplantFeature` (and the witch's of the
  shaman's, already 278's line); added as 278's.
- `extra-hex-shaman`: the game's prerequisite also names `WinterWitchShamanHexSelection`; added as 264's, like `extra-hex`.
