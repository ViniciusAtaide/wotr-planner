//! The feat type icons: one small line drawing per [`FeatCategory`], drawn for this app on a 24×24 grid. They
//! take the text's colour (`stroke: currentColor`, in `main.css`'s `.feat-icon`), so they follow the theme.

use leptos::prelude::*;
use rules::FeatCategory;

/// A feat type's icon, `1em` square, with the type's name as its tooltip.
pub fn feat_icon(category: FeatCategory) -> impl IntoView {
    let svg = format!(
        r#"<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">{}</svg>"#,
        drawing(category)
    );
    view! { <span class="feat-icon" title=format!("{} feat", category.name()) inner_html=svg></span> }
}

/// The SVG shapes of a feat type's icon.
fn drawing(category: FeatCategory) -> &'static str {
    match category {
        // A sword, point up and right: blade, crossguard, grip.
        FeatCategory::Weapon => {
            r#"<path d="M20 4v3L10 17l-3-3L17 4z"/><path d="M5 12l7 7M8.5 15.5 4 20"/>"#
        }
        // An upright bow, its string drawn back, with a level arrow.
        FeatCategory::Ranged => {
            r#"<path d="M8 3c8 3 8 15 0 18"/><path d="M8 3l-3 9 3 9"/><path d="M5 12h16M21 12l-3-3M21 12l-3 3"/>"#
        }
        // A closed fist: four knuckles over the palm, the thumb across.
        FeatCategory::UnarmedCombat => {
            r#"<path d="M6 11V8a1.5 1.5 0 0 1 3 0v2M9 10V7a1.5 1.5 0 0 1 3 0v3M12 10V7.5a1.5 1.5 0 0 1 3 0V10M15 10.5V9a1.5 1.5 0 0 1 3 0v4"/><path d="M6 11v3a6 6 0 0 0 12 0v-1"/><path d="M6 13h4.5a1.5 1.5 0 0 0 0-3H9"/>"#
        }
        // A heater shield, split down the middle.
        FeatCategory::ArmourAndShield => {
            r#"<path d="M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6z"/><path d="M12 6.5v11"/>"#
        }
        // Two arrows chasing each other round: trips, grapples, disarms.
        FeatCategory::CombatManoeuvres => {
            r#"<path d="M20 11A8 8 0 0 0 5.7 6.1M5 3v4h4"/><path d="M4 13a8 8 0 0 0 14.3 4.9M19 21v-4h-4"/>"#
        }
        // A horseshoe with its nail holes.
        FeatCategory::MountedCombat => {
            r#"<path d="M7 20C4 17 3.5 9 6 6s9-4 12 0 2 11-1 14"/><circle class="dot" cx="6.2" cy="15" r="1"/><circle class="dot" cx="5.8" cy="10.5" r="1"/><circle class="dot" cx="17.8" cy="15" r="1"/><circle class="dot" cx="18.2" cy="10.5" r="1"/>"#
        }
        // A four-pointed spark, and a small one beside it.
        FeatCategory::Magic => {
            r#"<path d="M11 4c.6 4.8 3.2 7.4 8 8-4.8.6-7.4 3.2-8 8-.6-4.8-3.2-7.4-8-8 4.8-.6 7.4-3.2 8-8z"/><path d="M19 2v4M17 4h4"/>"#
        }
        // A quill.
        FeatCategory::Skill => {
            r#"<path d="M20 3C11 4 6 10 6 18c6-1 11-6 14-15z"/><path d="M6 18l-2.5 3.5M10 14l5-5"/>"#
        }
        // Two linked rings.
        FeatCategory::Teamwork => {
            r#"<circle cx="9" cy="12" r="5"/><circle cx="15" cy="12" r="5"/>"#
        }
        // A head and shoulders.
        FeatCategory::Racial => {
            r#"<circle cx="12" cy="8" r="4"/><path d="M4 21c0-4.4 3.6-7 8-7s8 2.6 8 7"/>"#
        }
        // A pennant on its pole.
        FeatCategory::Class => r#"<path d="M6 3v18"/><path d="M6 4h12l-3 4 3 4H6"/>"#,
        // A cut gem.
        FeatCategory::Miscellaneous => {
            r#"<path d="M7 4h10l4 5-9 11L3 9z"/><path d="M3 9h18M10 4 8.5 9 12 20l3.5-11L14 4"/>"#
        }
    }
}
