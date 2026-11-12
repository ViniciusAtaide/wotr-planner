# Slots a Mythic rank's pick gives are keyed by their rank, not by their place at a level

Type: design
Status: open
Blocked by: 171
Model: Fable to design with the user; it changes slot ids, and so the link.
Map: [WotR Build Planner](../map.md)

## Question

Some slots belong to a character level by their id but are given by a Mythic rank's pick: the Feature selection slot of a feat with an extra pick taken at a rank (Extra Rogue Talent through Extra Feat, or as a Gold Dragon's Dragon Feat: `SlotId::FeatSelection { level, index }`), and an option row's slots reached through a rank's pick (`OptionSelection`, `OptionFeat`). Their `index` is their place among the slots granted at that level, in the order the level grants them: the level's own first, then each rank's in rank order.

Since ticket [171](171-mythic-picks-after-the-levels.md) a rank can be taken at another level. The slot then goes with the rank, and the slots of the ranks after it at either level change place, so a pick could end up in another rank's slot. Ticket 171 refuses such a move (`Engine::shifted_picks`): the pick has to be cleared first, the rank moved, and the pick taken again.

Should these slots be keyed by what gives them (the rank, and which of its picks) rather than by their place at a level?

- **A. Leave them keyed by place.** A move that would shift a filled slot stays locked, as built. Nothing changes in the link.
- **B. Key them by their owner.** A slot given by a rank's pick names the rank (as `MythicExtra { rank }` and `MythicSelection { rank, index }` already do), so it moves with its rank and its pick with it, and no other rank's slot changes place. That also answers ticket 171's open question 2 (carry a filled slot along with its rank). It needs new slot ids, so new addresses and a new group in the link, and a rule for the links that hold such a pick under the old id.

*Recommend:* B, if moving ranks with such picks turns out to be common; A costs nothing and is honest about why the move is locked.

## To decide

- Whether B is wanted at all.
- For B: the new ids and addresses; whether old links holding a `FeatSelection` pick at a rank's level are read as the new id (they can be told apart only by walking the Build) or kept as they are; what the proof and the snapshots show for Builds that hold such picks.
