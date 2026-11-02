# Saving and sharing Builds

Type: grilling
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

How are Builds saved to Postgres and shared by URL? These are already decided: no accounts, each save is a snapshot that can't be changed and gets its own link, and a shared Build opens on the planning sheet.

- **Storage:** what does a saved Build row hold? Options include the serialised Build (in which format), a name, the data hash it was built against, and timestamps. Is it relational per level, or one serialised value? Game data is relational, but a Build is a document.
- **URL:** the slug scheme (random short ID vs content hash, length), the route shape, and whether an unsaved Build can live entirely in the URL.
- **Stale Builds:** what happens when a saved Build no longer validates after game data changes (it's server-validated on load)?
- **Drafts:** does an unsaved Build in progress survive a reload or fold/unfold? For example, local storage until it's saved.

## Comments

**2026-10-31, user (round 1):** a saved Build *can* change: open it, edit it, save again. The shared link is a hash of all the choices made, so the link is the Build's whole state. A Build is never "closed"; you share all its choices through that link. This replaces the earlier "snapshot that can't be changed" assumption, which came from an unanswered question (Q8 of the first charting round).

Open: whether the link *contains* the choices (encoded in the URL) or *refers to* them (a short hash stored in Postgres). Asked in round 2.

**2026-10-31, user (round 2):** **choices in the URL.** The link encodes every choice in the Build and updates live on each edit, so there's no Save step and no `saved_build` table. Postgres holds only game data. The previous round's Q1, Q2 and Q3 no longer apply, and Q4 (drafts) is mostly covered by the URL itself.

**2026-10-31, user (round 3):**

- **Q10:** `/` with no Build in the URL opens a blank new Build, to start from scratch. Nothing is kept in local storage.
- Not answered, so the recommendations are taken unless the user objects:
  - **Q7:** text keys, in a versioned compact binary format, compressed and made URL-safe.
  - **Q8:** the Build goes in the URL fragment (`/#b=…`).
  - **Q9:** stale links open with a banner, with broken picks and removed keys marked. *(Reversed by the user, 2026-11-01, built in [ticket 40](40-stale-links-generic-error.md): a stale link gets one generic error.)*
  - **Q11:** an optional name inside the URL.

## Answer

- **The URL is the Build.** The link encodes every choice in the URL fragment, `/#b=<code>`, and is updated live (`history.replaceState`) on every edit. There's no Save step, a Build is never closed, and sharing means copying the address. Postgres holds only game data; there's no saved-build table.
- **Encoding:** a format-version byte, then the Build in a compact binary layout, compressed with raw DEFLATE, in base64url. Each game-data key (`power-attack`) is written as a 24-bit hash of its text (FNV-1a, cut), looked up among the current game data's keys of its kind when the link opens, plus an optional name as text. Hashes keep links valid across reseeds and every game-data change that keeps the keys; a hash the data no longer has is a stale link. The version byte allows future format changes. A 20-level Build with every slot filled is about 270–370 characters ([ticket 46](46-shorter-links.md), which replaced the text keys: they were 630–890). The layout is specified in `rules`' codec module docs.
- **Opening a link:** decoded in the browser by the `rules` crate; the server never sees it.
- **`/` with no fragment:** a blank new Build. Nothing is kept in local storage.
- **Stale links**, when game data changed since the link was made, and **hand-edited or damaged links**: a link that doesn't decode, or whose Build isn't possible with the current game data (a key gone, a pick that no longer fits, any rule broken, a pick for a slot or rank the Build doesn't reach), opens nothing. The sheet shows one generic error, "This link no longer works with the current game data", and a blank Build: no banner, no marked picks, no reasons ([ticket 40](40-stale-links-generic-error.md), reversing Q9's banner and [ticket 39](39-impossible-states-not-validation.md)'s × on broken picks). `rules` checks it in one call, `Engine::open_link`.
