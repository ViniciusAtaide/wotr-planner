# Carriers the vocabulary can't hold: the shaman's wandering spirit

Type: build (model gap)
Status: open
Blocked by: 307 (it makes the entries)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

S3 (307) makes entries for 9 shaman wandering-spirit carriers, which the game grants through the daily spirit choice. The planner's component vocabulary has nothing that holds them, so S4 (308) leaves the prerequisites that name them unwritten under this ticket. The kineticist blasts' 13 carriers belong to 270. Research: `.scratch/wotr-planner/research/307/`.

## Change

Research the game's form first: how the wandering spirit grants its features (cite blueprint and code). Then model that hold in the vocabulary, so the prerequisites naming these carriers can be written.

## Done when

The prerequisites S4 left under this ticket are written, their `known.txt` lines are gone, the game check exits 0, and the scoped checks pass.
