# Importer S3, 307n: the carrier-grants run on 66 entries

Type: build (the importer, stage S3)
Status: deferred to a later queue (user, 2026-11-17: research/queue-scope-2026-11-17.md)
Blocked by: 307m, 307l
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [307](307-importer-carriers.md), the importer's stage S3 (the carrier entries), cut to about 15–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/307/slices.md](../research/307/slices.md). The decisions are in [research/307/decisions.md](../research/307/decisions.md).

## Done when

Its section in `slices.md` is built and proved as written there. S3 is a form change, so its census must be empty. The scoped checks pass.

**From 307m's review (main session, 2026-11-17):**
- `a_wyrmshifter_holds_its_dragon_form_level_up_carrier` can't pass as specified. Its three level-up grants are `GrantIf`s on `DragonbloodShifterAspectFeature{Good,Evil,Neutral}`, which have no entry and aren't among S3's 207 carriers. Either add carriers for the three aspect features, naming their GUIDs in `--only`, or drop the test and record why.
- The 3 race grantors stay unwritten, since `owned()` has no races folder.
- Name every carrier GUID in `--only`. 307c makes `carrier-grants` require it, as `carriers` does.
