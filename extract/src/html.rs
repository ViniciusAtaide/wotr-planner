//! Reading a saved guide page: the flat children of `div#faqwrap`, rebuilt into a section tree by heading level.

use scraper::{ElementRef, Html, Node, Selector};

/// One content element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A paragraph, split into lines at `<br>`.
    Para(Vec<String>),
    /// Table rows of cell text; a `<br>` inside a cell becomes a newline.
    Table(Vec<Vec<String>>),
    /// List items.
    List(Vec<String>),
    Quote(String),
}

/// A heading and everything up to the next heading of the same or a higher level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Section {
    /// 2 to 5 for `h2`..`h5`; 1 for the page itself.
    pub level: u8,
    pub anchor: String,
    pub name: String,
    pub blocks: Vec<Block>,
    pub children: Vec<Section>,
}

impl Section {
    /// This section and every section below it, depth first.
    pub fn walk(&self) -> Vec<&Section> {
        let mut all = vec![self];
        for child in &self.children {
            all.extend(child.walk());
        }
        all
    }

    pub fn find(&self, anchor: &str) -> Option<&Section> {
        self.walk()
            .into_iter()
            .find(|section| section.anchor == anchor)
    }

    pub fn paragraphs(&self) -> impl Iterator<Item = &Vec<String>> {
        self.blocks.iter().filter_map(|block| match block {
            Block::Para(lines) => Some(lines),
            _ => None,
        })
    }

    pub fn tables(&self) -> impl Iterator<Item = &Vec<Vec<String>>> {
        self.blocks.iter().filter_map(|block| match block {
            Block::Table(rows) => Some(rows),
            _ => None,
        })
    }

    pub fn list_items(&self) -> impl Iterator<Item = &String> {
        self.blocks
            .iter()
            .filter_map(|block| match block {
                Block::List(items) => Some(items),
                _ => None,
            })
            .flatten()
    }

    /// All the section's own text, lines joined by spaces.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for block in &self.blocks {
            let part = match block {
                Block::Para(lines) => lines.join(" "),
                Block::List(items) => items.join(" "),
                Block::Quote(quote) => quote.clone(),
                Block::Table(_) => continue,
            };
            text.push_str(&part);
            text.push(' ');
        }
        text
    }
}

/// A saved page: its slug and its section tree.
#[derive(Clone, Debug)]
pub struct Page {
    pub slug: String,
    pub root: Section,
}

impl Page {
    /// `page#anchor`, or the page alone for an empty anchor.
    pub fn guide(&self, anchor: &str) -> String {
        if anchor.is_empty() {
            self.slug.clone()
        } else {
            format!("{}#{anchor}", self.slug)
        }
    }
}

/// Parses a saved page. The slug comes from the canonical link and must match `expected_slug`.
pub fn parse_page(expected_slug: &str, html: &str) -> Result<Page, String> {
    let document = Html::parse_document(html);
    let canonical = Selector::parse("link[rel=canonical]").expect("valid selector");
    let slug = document
        .select(&canonical)
        .next()
        .and_then(|link| link.value().attr("href"))
        .and_then(|href| href.trim_end_matches('/').rsplit('/').next())
        .ok_or_else(|| format!("{expected_slug}: no canonical link"))?
        .to_owned();
    if slug != expected_slug {
        return Err(format!("{expected_slug}: the canonical link is for {slug}"));
    }
    let wrap = Selector::parse("div#faqwrap").expect("valid selector");
    let wrap = document
        .select(&wrap)
        .next()
        .ok_or_else(|| format!("{slug}: no div#faqwrap"))?;

    let mut stack = vec![Section {
        level: 1,
        ..Section::default()
    }];
    for child in wrap.children().filter_map(ElementRef::wrap) {
        let tag = child.value().name();
        let block = match tag {
            "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse().expect("h2..h6");
                let anchor = anchor(child);
                let name = clean(&text(child));
                close_to(&mut stack, level);
                stack.push(Section {
                    level,
                    anchor,
                    name,
                    ..Section::default()
                });
                continue;
            }
            "p" => Block::Para(lines(child)),
            "table" => Block::Table(table(child)),
            "ul" | "ol" => Block::List(
                child
                    .children()
                    .filter_map(ElementRef::wrap)
                    .filter(|item| item.value().name() == "li")
                    .map(|item| lines(item).join("\n"))
                    .collect(),
            ),
            "blockquote" => Block::Quote(clean(&text(child))),
            _ => continue,
        };
        stack
            .last_mut()
            .expect("the page root is never popped")
            .blocks
            .push(block);
    }
    close_to(&mut stack, 2);
    let root = stack.pop().expect("the page root");
    Ok(Page { slug, root })
}

/// Closes every open section at `level` or deeper, attaching each to its parent.
fn close_to(stack: &mut Vec<Section>, level: u8) {
    while stack.len() > 1 && stack.last().is_some_and(|section| section.level >= level) {
        let section = stack.pop().expect("checked");
        stack
            .last_mut()
            .expect("the page root")
            .children
            .push(section);
    }
}

fn anchor(heading: ElementRef) -> String {
    let anchors = Selector::parse("a[id]").expect("valid selector");
    heading
        .select(&anchors)
        .next()
        .and_then(|a| a.value().attr("id"))
        .unwrap_or_default()
        .to_owned()
}

/// Text with `<br>` as `\n`.
fn text(element: ElementRef) -> String {
    let mut out = String::new();
    push_text(element, &mut out);
    out
}

fn push_text(element: ElementRef, out: &mut String) {
    for node in element.children() {
        match node.value() {
            Node::Text(text) => out.push_str(text),
            Node::Element(child) if child.name() == "br" => out.push('\n'),
            Node::Element(_) => {
                if let Some(child) = ElementRef::wrap(node) {
                    push_text(child, out);
                }
            }
            _ => {}
        }
    }
}

fn lines(element: ElementRef) -> Vec<String> {
    text(element).split('\n').map(clean).collect()
}

fn table(element: ElementRef) -> Vec<Vec<String>> {
    let rows = Selector::parse("tr").expect("valid selector");
    let cells = Selector::parse("th, td").expect("valid selector");
    element
        .select(&rows)
        .map(|row| {
            row.select(&cells)
                .map(|cell| {
                    text(cell)
                        .split('\n')
                        .map(clean)
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .collect()
        })
        .collect()
}

/// Collapses whitespace and straightens typographic quotes.
pub fn clean(text: &str) -> String {
    text.replace(['\u{2019}', '\u{2018}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"")
        .replace(['\u{2013}', '\u{2014}'], "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
