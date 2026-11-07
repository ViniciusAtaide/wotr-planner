# The first ascension's path-linked ability is a choice

Type: task
Status: open
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
