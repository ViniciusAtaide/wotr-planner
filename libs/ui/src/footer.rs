//! The page footer: the credit the Open Game License asks for, and the licence's link.

use leptos::prelude::*;

/// The licence, served from the site root as plain text (`scripts/build-site.sh` copies `OPEN-GAME-LICENSE.md` there).
/// Relative to the page, so it works under a project site's `/<repo>/`.
const LICENCE_PATH: &str = "./open-game-license.txt";

/// The link's words.
const LICENCE_LINK: &str = "Open Game License v1.0a";

/// Who the mechanics come from, and that this is a fan tool: it names Paizo and Owlcat without using their marks.
const CREDIT: &str = "Game mechanics are Open Game Content from Pathfinder: Wrath of the Righteous, \
                      © Owlcat Games, and the Pathfinder rules, © Paizo. A fan tool, not endorsed by either.";

/// The one small line at the foot of the page.
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <span>{CREDIT}</span>
            " "
            <a href=LICENCE_PATH>{LICENCE_LINK}</a>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::{CREDIT, LICENCE_LINK, LICENCE_PATH};

    /// The repo root: this crate is `libs/ui/`.
    fn root_file(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }

    #[test]
    fn the_footer_credits_paizo_and_owlcat_and_links_the_licence_relative_to_the_page() {
        assert!(
            CREDIT.contains("Paizo") && CREDIT.contains("Owlcat"),
            "{CREDIT}"
        );
        // A project site lives under `/<repo>/`, so the link must not start at the host's root.
        assert!(LICENCE_PATH.starts_with("./"), "{LICENCE_PATH}");
        assert!(LICENCE_LINK.contains("Open Game License"), "{LICENCE_LINK}");
    }

    #[test]
    fn the_licence_file_holds_the_licence_and_section_15_with_owlcats_lines_and_ours() {
        let licence = root_file("OPEN-GAME-LICENSE.md");
        for needle in [
            "OPEN GAME LICENSE Version 1.0a",
            "14. Reformation:",
            "15. COPYRIGHT NOTICE",
            "Open Game License v 1.0a Copyright 2000, Wizards of the Coast, Inc.",
            "Pathfinder: Wrath of the Righteous CRPG Reference Document,© 2020, Owlcat Games LTD",
            "Pathfinder Roleplaying Game Core Rulebook.© 2009, Paizo Publishing, LLC",
            "WotR Planner, a build planner",
            "the game mechanics in `libs/game-data/`",
        ] {
            assert!(
                licence.contains(needle),
                "OPEN-GAME-LICENSE.md lacks {needle:?}"
            );
        }
    }

    #[test]
    fn the_readme_and_the_site_build_carry_the_licence() {
        assert!(root_file("README.md").contains("](OPEN-GAME-LICENSE.md)"));
        assert!(root_file("scripts/build-site.sh").contains("OPEN-GAME-LICENSE.md"));
    }
}
