//! Ticket 60: the Vivisectionist's lists `read_medical_discoveries` reads from the alchemist's page: one list item per
//! list, its bold heading then the names, "Greater / Grand Mutagen" standing for both.

use extract::borrowed_selections::read_medical_discoveries;
use extract::html::parse_page;

const PAGE: &str = r#"<html><head><link rel="canonical" href="https://gamefaqs.gamespot.com/x/faqs/80843/alchemist"></head>
<body><div id="faqwrap">
<h4>Medical Discovery<a id="medical-discovery"></a></h4>
<p>The Vivisectionist is able to select certain talents and discoveries:</p>
<ul><li><strong>Discoveries</strong><br>Cognatogen, Greater / Grand Mutagen</li>
<li><strong>Rogue Talents</strong><br>Combat Trick, Focusing Attacks</li>
<li><strong>Advanced Rogue Talents</strong><br>Crippling Strike.</li></ul>
<h3>Alchemist Extracts<a id="alchemist-extracts"></a></h3>
<ul><li><strong>Not a list of the section</strong><br>Shield</li></ul>
</div></body></html>"#;

#[test]
fn each_list_has_its_heading_and_names_with_a_slash_read_as_both() {
    let page = parse_page("alchemist", PAGE).unwrap();
    let strings = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(
        read_medical_discoveries(&page),
        vec![
            (
                "Discoveries".to_owned(),
                strings(&["Cognatogen", "Greater Mutagen", "Grand Mutagen"])
            ),
            (
                "Rogue Talents".to_owned(),
                strings(&["Combat Trick", "Focusing Attacks"])
            ),
            (
                "Advanced Rogue Talents".to_owned(),
                strings(&["Crippling Strike"])
            ),
        ]
    );
}
