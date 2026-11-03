//! Ticket 59: the infusion list `infusions` reads from the kineticist's page: "Level N" groups of one section per
//! infusion, each with a table of `Element`, `Type` and maybe `Prerequisites` rows and its text.

use extract::html::parse_page;
use extract::infusions::{Infusion, read_infusions};

/// A list in the guide's style: a misspelt Fire infusion, a Universal one for the Sharpshooter only, and one needing
/// either of two others.
const PAGE: &str = r#"<html><head><link rel="canonical" href="https://gamefaqs.gamespot.com/x/faqs/80843/kineticist"></head>
<body><div id="faqwrap">
<h3><a id="infusions"></a>Infusions</h3>
<p>Infusions modify the blasts.</p>
<h4><a id="level-1"></a>Level 1</h4>
<h5><a id="dazziing-infusion"></a>Dazziing Infusion</h5>
<table>
<tr><td>Element</td><td>Fire</td></tr>
<tr><td>Type</td><td>Substance infusion</td></tr>
<tr><td>Burn</td><td>1</td></tr>
<tr><td>The target is dazzled.</td></tr>
</table>
<h5><a id="exploding-arrows"></a>Exploding Arrows</h5>
<table>
<tr><td>Element</td><td>Universal</td></tr>
<tr><td>Type</td><td>Form infusion</td></tr>
<tr><td>Creatures nearby take half.<br>This infusion may be only be used by a Kinetic Sharpshooter wielding a ranged weapon.</td></tr>
</table>
<h4><a id="level-5"></a>Level 5</h4>
<h5><a id="grappling-infusion"></a>Grappling Infusion</h5>
<table>
<tr><td>Prerequisites</td><td>Cloud, Wall</td></tr>
<tr><td>Element</td><td>Air, Water</td></tr>
<tr><td>Type</td><td>Substance infusion</td></tr>
<tr><td>A foe in your Cloud or Wall is grappled.</td></tr>
</table>
</div></body></html>"#;

#[test]
fn an_infusion_has_its_level_elements_prerequisites_and_who_may_use_it() {
    let page = parse_page("kineticist", PAGE).unwrap();
    assert_eq!(
        read_infusions(&page),
        [
            Infusion {
                anchor: "dazziing-infusion".to_owned(),
                name: "Dazzling Infusion".to_owned(),
                level: 1,
                elements: vec!["fire".to_owned()],
                prerequisites: vec![],
                only_sharpshooter: false,
            },
            Infusion {
                anchor: "exploding-arrows".to_owned(),
                name: "Exploding Arrows".to_owned(),
                level: 1,
                elements: vec![],
                prerequisites: vec![],
                only_sharpshooter: true,
            },
            Infusion {
                anchor: "grappling-infusion".to_owned(),
                name: "Grappling Infusion".to_owned(),
                level: 5,
                elements: vec!["air".to_owned(), "water".to_owned()],
                prerequisites: vec!["Cloud".to_owned(), "Wall".to_owned()],
                only_sharpshooter: false,
            },
        ]
    );
}
