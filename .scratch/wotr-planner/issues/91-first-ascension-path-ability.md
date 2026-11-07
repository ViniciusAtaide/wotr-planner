# The first ascension's path-linked ability is a choice

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). The app grants the MR 1–2 rows of the path chosen at MR 3 ([ticket 26](26-mythic-path-rules.md): "choosing a path at MR 3 grants its MR 1–2 rows"). So a Build's MR 1–2 path abilities always come from its MR 3 path. The guide says the MR 1 path-linked ability is chosen on its own and doesn't commit the character to that path.

**Repro:** the Enforcer Build from ticket 88 (final link `#b=AdVNPUhCYRT97veeqT1ToYiWSIogSKmkKOgPm7IgsBoqInHIoR-RighrkOBZ9DO4RAqtRVMUxOstIRlZg0sW4SCBk20NQTRI3ZdpSa-lTS7nfOfc795zt2C89sL_bNP5FntCex4xvd_0VCh6NmVBPTzDZexKzeD_Oqs6Ry4TddWSF2f8Z6d4a1vTmjSMvPcq8MthvI6Xc8qOnZPXqM0y4VI706PWgSOrRxvb3Gi40ZUkKiuWXWZTfXNkwSxWl3rb64LHTvdBKChcPvJ20enwJSddqsP1sKcrvIbrLQaBm766HzLx_Q_diVhbY-fYVFw4D_RFUgHj2657_oWfu4veUq2dAAEGQK_XFqs0ZQQlfBPk2z97VG5OIREQueczgwGSKSXhgBAUFJn-JirTwxPSCpfNoF-QislqyPdUDRTQINGcQmJZ9keCBAVIrGSkQyyiCKEGFf0A`), Mythic track. MR 1 offers one slot, "Mythic ability"; MR 2 one slot, "Mythic feat". Its write-up takes the Trickster's rank-1 and rank-2 abilities (Bit of Fun at MR 1; Creative Approach and Summon Will-o'-Wisp at MR 2) and then goes Azata at MR 3. No picker offers Bit of Fun or Creative Approach (they're only rows of `trickster.ron`), and choosing Azata at MR 3 gives the Azata's rank-1–2 rows (Instrument of Freedom, Freedom's Child, Summon Azata Bralani) instead. Jack-in-the-Green takes the Azata's own, so it's unaffected.

**The guide:**
- `mythic-paths#first-ascension`: at the first mythic rank the character picks an ability linked to one of the mythic paths, which doesn't force them down that path, along with a mythic ability; at rank 2 they get a second ability tied to that earlier choice, their first mythic feat and the two fixed abilities (Bypass Epic Damage Reduction, Hard to Kill).
- `trickster` ("Playing a Trickster"): whichever path you mean to take, Creative Approach is the best rank-2 ability, so you'll generally start down the Trickster route.

**The conflict:** [ticket 04](04-progression-model.md) (user, 2026-10-31) has the path chosen at MR 3, MR 1 a general mythic ability and MR 2 a general mythic feat, and ticket 26 settled that the MR 3 path grants its MR 1–2 rows. The guide reads as a separate choice at MR 1. This needs the user's call.

**Recommendation:** add a "first ascension" pick at MR 1: one of the paths that have MR 1–2 rows (the six the MR 3 picker offers); it grants that path's MR 1 row at MR 1 and its MR 2 row at MR 2, and the MR 3 path no longer grants its own MR 1–2 rows. A link without the pick (every link today) reads as having picked its MR 3 path, so old links keep their meaning. Path switches keep MR 1–2 as today.

## Build

Once the user decides: the rules change test-first (`tdd`), the data and seed if the shape changes, the Mythic track's new slot, the link format (a new link group or field, with old links decoding as above), and the behaviour proof.

**Done when:** the repro Build can take the Trickster's first-ascension abilities and go Azata at MR 3, its Mythic track shows Bit of Fun at MR 1 and Creative Approach and Summon Will-o'-Wisp at MR 2, and every older link decodes to the same Build as before.

**User, 2026-11-07:** the recommendation, with the shape settled: the first ascension is a selection **only at MR 1**, of the six path-linked abilities (each path's rank-1 row: Force Reality, Close to the Heavens, Instrument of Freedom, Close to the Abyss, Danse Macabre, Bit of Fun; Gold Dragon has none). **MR 2 has no pick: it just has the features**, the chosen option's follow-ups (that path's rank-2 row) plus the fixed Hard to Kill, as Bypass Epic Damage Reduction is fixed at MR 1. The MR 3 path is chosen independently and no longer grants its own MR 1–2 rows. A link without the MR 1 pick reads as having picked its MR 3 path's option. Check while building: `trickster.ron`'s rank 2 lacks Hard to Kill, and Aeon's rank 2 repeats Force Reality.

## Progress log

- 2026-11-07: started (the user around; small choices settled here). Scratch in `t91/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t91/`). Read `CLAUDE.md`, the map's Notes, tickets 04, 26, 88 and 89. Guide (`guide-cache/mythic-paths.html#first-ascension`, each path's table): Bypass Epic Damage Reduction comes at rank 1 and Hard to Kill at rank 2; the Trickster's rank 2 lists Creative Approach, Hard to Kill and Will-o'-Wisp, and `trickster.ron`'s rank 2 already holds `trickster/hard-to-kill` (the row's keys are in another order, so the quirk isn't one); the Aeon's table repeats Force Reality at rank 2 and its text says it improves then, so the data's repeat is the guide's. No data change. No prerequisite in the data names an MR 1 or 2 row feature, so holding them from MR 1 and 2 changes no answer but the sheet's states.
- 2026-11-07: shape settled here. `Build.first_ascension: Option<MythicPathKey>` (`#[serde(default)]`), `Edit::FirstAscension`, `Choice::FirstAscension`, `FIRST_ASCENSION_RANK = 1`. The options are the paths without a switch rule that have an MR 1 row (computed, nothing stored). `None` takes the MR 3 path's once the Build reaches MR 3. The MR 1 and 2 rows are held from their own ranks (ticket 26 held them from MR 3: the old rows were held from the path choice only because the path was chosen there). `RankSheet.path` at MR 1 and 2 is the first ascension's path. The link: an optional trailing field after the picks, `1, key(mythic path)`, written only when there is one, so every old link reads as `None` and a Build without one writes the link it always did (no new version, the frozen v1 vector unchanged). No schema change: the Build isn't in the database and the game data's shape didn't change.
- 2026-11-07: rules test-first on the small world: the world's Trickster and Aeon MR 1 rows gain Bypass Epic Damage Reduction (to name an option by what sets it apart); new `first_ascension.rs` (11 tests), the codec's round-trip proptest gains the field, the two ticket 26 tests on MR 1–2 timing rewritten to the new rule. Red run with the API stubbed: 12 failures (`t91/red-rules.txt`). Then the walk, picker, withdrawal, apply, link and labels; `cargo test -p rules` 423 + 7 unit pass.
- 2026-11-07: real-data tests (`libs/seeds/tests/game_data/first_ascension.rs`, 4): the six options and their names, Bypass Epic Damage Reduction at MR 1 and Hard to Kill at MR 2 for each, the Enforcer repro, and 8 links recorded before the change (`proof export` at HEAD: one per path, `bench legend40`, and the Enforcer's). Red against the pre-change behaviour (HEAD's walk and the codec and labels stubbed): all 4 (`t91/red-real-data.txt`); green after. `mythic.rs`'s ticket 26 test moved to the new timing. RustRover's `create_new_file` staged the new rules test; unstaged.
- 2026-11-07: every older link: HEAD's harness exported its 2,146 Builds as links and as JSON (`t91/head-links.tsv`, `head-builds.tsv`); a scratch program (`t91/oldlinks/`) decodes each link with the working tree's codec: 2,146 decode to the same Build as HEAD's JSON, with no first ascension, and write back the same link (`t91/oldlinks.txt`).
- 2026-11-07: UI: the Mythic track's MR 1 has a "First ascension" chip (the path-linked ability the rank takes: the Build's own, or its MR 3 path's) opening a picker of the six (each named by its ability, with its path and its MR 2 follow-ups); every reached rank lists the features its path row gives, so MR 2 shows the follow-ups and Hard to Kill. Proof harness: Builds go to and from JSON through serde (a field a revision adds comes along), and five `t91` Builds set the first ascension through serde so the harness still compiles against HEAD.
- 2026-11-07: proof `scripts/proof.sh both t91/proof` against HEAD: 2,162 Builds a side (2,146 + 16 `t91`), none on one side only, 813 differ: exactly the Builds with a path or a first ascension. The 797 older ones differ in `sheet` only (every picker, reason, `allows`, `removable`, `possible` the same). A special breakdown in a scratch copy of the tree (`t91/special/`, `t91/special-out/`) hashed each sheet again without the MR 1 and 2 row features in the states before level 9: all 797 are then equal, so the one change for an older Build is those rows held from MR 1 and 2 instead of from MR 3. The 3 `t91 … / first azata` Builds (first ascension = MR 3 path) differ only in `removable` (the first ascension is a choice) and that timing; the 13 others pick a different option and differ as intended. Ticket 48's shortcut stayed exact on both sides.
- 2026-11-07: browser check on a release build (`CARGO_TARGET_DIR=t91/target`, `LEPTOS_SITE_ROOT=t91/site`, server on `127.0.0.1:3970`, repo database read-only): the Enforcer link's Mythic track shows Instrument of Freedom at MR 1 (its Azata's, as before); the First ascension picker lists the six with their MR 2 follow-ups; picking Bit of Fun shows "Bit of Fun · Bypass Epic Damage Reduction" at MR 1 and "Creative Approach · Hard to Kill · Summon Will-o'-Wisp" at MR 2, MR 3 on Azata; the new link reloads the same; removing the choice gives back the original link exactly. At 375 px the features line spans the row with no horizontal scroll (a first build lacked the CSS rule, fixed and rebuilt). No console message. Server stopped, pane closed.
- 2026-11-07: `scripts/check.sh` ALL-OK, then `--full` ALL-OK (`t91/check-full.txt`). `cargo test -p rules -p seeds -- --list` shows the 16 new tests (rules 11 + the codec unit test, real data 4). Nothing staged or committed.

## Answer

Done 2026-11-07, uncommitted. The first ascension is a choice at MR 1 of one of the six paths' path-linked abilities; MR 2 has no pick and holds that option's follow-ups and Hard to Kill; the MR 3 path no longer gives its own MR 1–2 rows. A link without the choice (every link so far) takes its MR 3 path's, so it opens to the same Build. The Enforcer can take Bit of Fun and go Azata.

### Settled here

- **Stored as a path key**, `Build.first_ascension: Option<MythicPathKey>`: the option is "this path's MR 1 row". The options are computed (paths without a switch rule that have an MR 1 row: Aeon, Angel, Azata, Demon, Lich, Trickster), and the picker names each by what sets its row apart from the others' (every row has Bypass Epic Damage Reduction), so nothing new is stored in the data or the database.
- **`None` follows the MR 3 path**, once the Build reaches MR 3; choosing the MR 3 path's own option sets it explicitly (same answers, but then changing the MR 3 path no longer moves MR 1–2). Remove goes back to following.
- **Timing:** the rows are held from MR 1 and MR 2 (ticket 26 held them from MR 3 only because the path was chosen there). No prerequisite in the data names an MR 1 or 2 row feature, so this changes no answer, only the sheet's states at levels 6–8 (the proof shows exactly that).
- **The link:** an optional field after the picks, `1, key(mythic path)`, written only when there is one. Every old link reads as none and a Build without one writes the same link, so version 1 and its frozen vector stay; no link breaks.
- **Path switches** keep the MR 1–2 rows as before (they're those ranks' path grants); the Devil's "from Aeon or Azata" looks at the MR 3 path.
- **The data quirks are the guide's:** `trickster.ron`'s rank 2 does hold Hard to Kill (in key order, first), and the Aeon's rank 2 repeats Force Reality because its table does (it improves at 2). No data change; README note added.

### What changed

- `rules`: `Build.first_ascension`, `Edit::FirstAscension`, `Choice::FirstAscension`, `FIRST_ASCENSION_RANK`; the walk takes MR 1–2 rows from the first ascension's path (`RankSheet.path` at MR 1–2 is that path); `first_ascension_options`, `first_ascension_name`, `rank_features`, `path_rank_features`; `apply` drops it below MR 1; an invalid or unreached one fails; it's a withdrawable choice; a recheck of a MR 3 path edit starts at MR 1 when the Build follows it; codec as above.
- UI: MR 1's "First ascension" chip and picker (each option with its path and MR 2 follow-ups); each reached rank lists its path row's features.
- Proof harness: Builds to and from JSON through serde; 5 `t91` Builds (plus their full ones).
- Docs: `GLOSSARY.md` (new **First ascension**; **Mythic path**, **Mythic track**), `libs/game-data/README.md`.

### Tests

Rules, small world: `first_ascension.rs` (11), codec proptest and frozen-body unit test, ticket 26's two timing tests rewritten. Real data: `first_ascension.rs` (4: the six options and names, the fixed abilities at 1 and 2, the Enforcer repro, 8 links recorded at HEAD reading the same and re-encoding byte for byte), `mythic.rs`'s timing test. All 2,146 links HEAD's harness exports decode to HEAD's Builds and write back the same links (`t91/oldlinks.txt`).

