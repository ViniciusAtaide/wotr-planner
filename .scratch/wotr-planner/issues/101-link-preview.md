# A link preview when the site is shared

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Shared on WhatsApp (2026-11-08), the site's link showed only its title and host: the page had no description, no Open Graph tags and no image. The user asked for a nice preview.

## Choices settled

1. **The tags live in the server's `shell`**, which renders only on the server and is the same for every request. `og:type`, `og:site_name`, `og:title`, `og:description`, a `description` and `twitter:card` (`summary_large_image`) are always there. The image tags (`og:image` with its type, size and alt, `twitter:image`) and `og:url` are there only with a site URL. `property` isn't an attribute leptos's view macro knows, so a small `og(property, content)` helper builds them through tachys's `.attr`, as leptos_meta's `<Meta>` does.
2. **An absolute image URL from `SITE_URL`.** Crawlers want `og:image` as an absolute URL, but ticket 100 keeps the repo name out of the code. So the server reads `SITE_URL` (normalised to end in `/`), and the Pages workflow sets it from `actions/configure-pages`' `base_url`, which also covers a custom domain. A dev server has no `SITE_URL`, so it names no image. `build-site.sh` warns without it, and fails if the page names a preview image the site lacks.
3. **The image** is `apps/frontend/public/social-preview.png`: 1200×630 and 161 KB, under WhatsApp's ~300 KB limit. It's rendered with headless Chrome from `apps/frontend/social/preview.html`, whose header holds the command. It shows the app's dark palette, the name, a tagline and a sample sheet. The sample is real data checked against the game files: a Fighter with Power Attack, Cleave and Weapon Focus, and the Angel's Sword of Heaven at MR 3. `public/` is cargo-leptos's `assets-dir`, copied into the site root, so the server's fallback serves it, and `build-site.sh` copies the folder beside `pkg/`. Its `.gitkeep` placeholder went, now that the folder holds a file. The user trimmed the image's tagline to drop "Share it as a link"; the description keeps it.
4. **Tests** in `apps/backend/src/main.rs`:
   - the page describes itself, with the image under the site URL;
   - without a site URL, it names no image and no `og:url`;
   - the image is served from the site root as `image/png`.

## Answer

Done 2026-11-08. A shared link now shows the title, the description and the preview image.
