//! Ticket 66: the monk powers `instinctual_rage_powers` reads from the barbarian's page, "Name (requires …)" in one
//! sentence, where a requirement is a level or a feat's name.

use extract::html::parse_page;
use extract::instinctual_rage_powers::{MonkPower, Needs, read_monk_powers};

/// A section in the guide's style: a feat, two levels, a last "and".
const PAGE: &str = r#"<html><head><link rel="canonical" href="https://gamefaqs.gamespot.com/x/faqs/80843/barbarian"></head>
<body><div id="faqwrap">
<h4><a id="rage-power-instinctual-warrior"></a>Rage Power (Instinctual Warrior)</h4>
<p>Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki powers: Iron Hand (requires Power Attack), Still Mind (requires 2nd level), Calm Heart (requires 3rd level) and Empty Self (requires 21st level).</p>
</div></body></html>"#;

#[test]
fn a_power_needs_a_level_or_the_feat_its_brackets_name() {
    let page = parse_page("barbarian", PAGE).unwrap();
    let power = |name: &str, needs: Needs| MonkPower {
        name: name.to_owned(),
        needs,
    };
    assert_eq!(
        read_monk_powers(&page),
        [
            power("Iron Hand", Needs::Feat("Power Attack".to_owned())),
            power("Still Mind", Needs::Level(2)),
            power("Calm Heart", Needs::Level(3)),
            power("Empty Self", Needs::Level(21)),
        ]
    );
}
