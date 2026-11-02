//! Turning guide names into key slugs and comparable forms.

/// The guide's anchor style: lowercase words joined by `-`, apostrophes dropped ("Hunter's Surprise" is
/// `hunters-surprise`).
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.chars() {
        if c == '\'' {
            continue;
        }
        if c.is_ascii_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c.to_ascii_lowercase());
        } else {
            dash = true;
        }
    }
    out
}

/// A name for comparing: its slug with "-" between words, so case, punctuation and spacing don't matter.
pub fn norm(name: &str) -> String {
    slug(&name.replace('-', " "))
}

/// Simple English plural/singular forms of a slug, for matching "Rogue Talent" to "Rogue Talents" and "Hex"
/// to "Hexes".
pub fn number_forms(slug: &str) -> Vec<String> {
    let mut forms = vec![slug.to_owned(), format!("{slug}s"), format!("{slug}es")];
    if let Some(stem) = slug.strip_suffix("es") {
        forms.push(stem.to_owned());
    }
    if let Some(stem) = slug.strip_suffix('s') {
        forms.push(stem.to_owned());
    }
    forms
}
