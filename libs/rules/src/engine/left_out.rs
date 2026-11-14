//! What a computed number leaves out, one thing at a time, in two lengths: the short form a tool response carries on
//! every call, and the full one, with its examples, that the web app shows and the MCP server serves once.

/// One thing a line or the totals leave out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeftOut {
    /// In a few words: "bonuses that grow with class level".
    pub short: &'static str,
    /// In full, with its examples: "bonuses that grow with class level (Danger Sense, Bravery, …)".
    pub full: &'static str,
}
