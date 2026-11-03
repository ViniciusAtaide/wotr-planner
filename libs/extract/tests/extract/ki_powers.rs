//! Ticket 58: the ki power list `ki_powers` reads from the monk's page, "Lvl | Ki Power | Cost", where some powers
//! have a section of their own under the list and others don't.

use extract::html::parse_page;
use extract::ki_powers::{Power, read_powers};

/// A list in the guide's style: two powers, one with a section of its own.
const PAGE: &str = r#"<html><head><link rel="canonical" href="https://gamefaqs.gamespot.com/x/faqs/80843/monk"></head>
<body><div id="faqwrap">
<h3><a id="ki-powers"></a>Ki Powers</h3>
<p>Ki Powers unlock at a monk level.</p>
<table>
<tr><td>Lvl</td><td>Ki Power</td><td>Cost</td></tr>
<tr><td>4</td><td>Iron Skin</td><td>1</td></tr>
<tr><td>12</td><td>Still Heart</td><td>-</td></tr>
</table>
<h5><a id="still-heart"></a>Still Heart</h5>
<p>Never tires.</p>
</div></body></html>"#;

#[test]
fn a_power_has_its_level_and_its_own_section_only_where_the_guide_gives_one() {
    let page = parse_page("monk", PAGE).unwrap();
    assert_eq!(
        read_powers(&page),
        [
            Power {
                name: "Iron Skin".to_owned(),
                level: 4,
                anchor: None,
            },
            Power {
                name: "Still Heart".to_owned(),
                level: 12,
                anchor: Some("still-heart".to_owned()),
            },
        ]
    );
}
