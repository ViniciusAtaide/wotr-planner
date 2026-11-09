# Gaps from the second martial run

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user asked again (2026-11-09) for "the strongest martial build possible, maximum attack damage", this time from scratch. The agent built a Human Two-Handed Fighter 20 with a falchion and branched the Mythic path at MR 3: Demon against Trickster (Perception II opens the Trickster critical chain: Improved Improved Critical, Improved Improved Improved Critical and its multiplier step). Both Builds are legal and full. These are what the planner couldn't settle, or got wrong. The Human skill point it found is fixed in [130](130-skilled-race-skill-points.md).

### 1. The attack line can't tell the two branches apart

`compare` with `weapon: falchion` gave both Builds the same line: +29/+24/+19/+14, 2d4+14, Power Attack −6/+18, **15–20/×2**. The choice between them rested on the agent's own arithmetic (about 1,000 damage a round each on paper; Demon ahead on accuracy).

- **Wrong today:** the Trickster Build holds all three Trickster critical feats, ordinary feats like Improved Critical, which the line counts. Its range should read 13–20 and its multiplier ×3. `attack_leaves_out` doesn't name them, so the line states a wrong fact.
- **Left out, as documented:** what decides a martial build's damage is mostly unconditional: Two-Handed Weapon Training, Backswing, Greater Power Attack and Weapon Mastery (class features), and Power Attack, Improved Critical, Weapon Focus and Weapon Specialization (Mythic).

Options:
- **A.** Count the three Trickster critical feats in the line, as Improved Critical is counted.
- **B.** A, plus the unconditional weapon class features and the weapon mythic feats, each as its own `counts` row.
- **C.** [113](113-effects-as-data.md)'s option C, a modifier model.

*Recommend:* A now, since the line states a wrong fact. B as its own design, sharing [119](119-caster-line.md) and [120](120-initiative-and-skill-totals.md)'s per-item bonus shape.

### 2. Intelligence's skill points

The guide's character creation page (its Intelligence entry): Intelligence "grants bonus skills equal to one point per two character levels for each point of Intelligence bonus". So Int 12 gives a point on odd levels, Int 14 one every level, and Int 18 two. The guide adds that Skilled "is equivalent to +4 Intelligence". The engine adds the whole Int modifier every level, which is twice the guide's figure. The user's skill table ([research/skills-from-user.md](../research/skills-from-user.md)) gives class points only, so it doesn't settle this.

Options:
- **A.** Keep the whole modifier.
- **B.** The guide's rule: half the Int modifier per level, accumulated, so +1 Int gives a point at levels 1, 3, 5 and so on.

*Recommend:* B, since the guide wins where the game and the guide disagree. It changes every Build's skill points, so it's the user's call.

### 3. The Titan Fighter: Weapon Training or not?

The guide contradicts itself on `fighter#titan-fighter`. The intro says it "does not receive Armor Training or Weapon Training", but its level table lists Weapon Training at 5 and Weapon Mastery at 20. The data's rows keep the base fighter's Weapon Training, following the table, while its summary follows the intro.

*Recommend:* the user says which wins. If it's the table, the summary drops "or Weapon Training". If it's the intro, the archetype removes the row at 5.

### 4. `search` caps `limit` at 100 without saying so

A search with `limit: 400` returned 100 of 261 feats. `total` was right, but no `note` said the limit was capped, and the tool description doesn't give the cap.

*Recommend:* state the cap in the description, and add a `note` when `limit` exceeds it.

### 5. The MCP server wasn't running when the session started

`.mcp.json` connects to `http://localhost:3902/mcp` ([129](129-mcp-over-http.md)). Nothing was listening, so the client marked `wotr-planner` failed (ECONNREFUSED). The agent started `WOTR_MCP_PORT=3902 target/release/wotr-mcp` by hand and called it with `curl`, since the client doesn't redial a project server mid-session.

Options:
- **A.** Document it: start the dev server or `wotr-mcp` before a session.
- **B.** A `SessionStart` hook that starts `target/release/wotr-mcp` on 3902 when nothing is listening.
- **C.** Make stdio the default again, keeping HTTP as an option.

*Recommend:* B.

### 6. Smaller cases for open tickets

- **[126](126-refusals-name-the-pick-order.md):** a level-11 general feat can't use Perception II, taken at MR 4, which also lands at level 11. The refusal said "Needs Perception II", which the Build already held, so the agent had to infer the pick order.
- **[125](125-favorite-metamagic-picks-its-metamagic.md):** Stat Focus takes no parameter, though its summary says "a stat of your choice". It's the same shape as Favorite Metamagic.

## Done when

The user has answered 2, 3 and 5. A is built for 1, so the Trickster Build's line reads 13–20/×3, with a real-data test. 4's cap is stated. `scripts/check.sh` passes.

## Progress log

- 2026-11-09, item 4 (branch `t/t-mcp-search`, with tickets 123 and 128): `search`'s tool description and its `limit` argument now state the cap of 100, and a `limit` over it adds a `note` ("`limit` 400 is over the most, 100: at most 100 matches come back …"); a real-data server test (`limit: 400` on feats). The cap itself is unchanged.
