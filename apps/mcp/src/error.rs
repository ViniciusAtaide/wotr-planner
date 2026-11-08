//! The server's own refusals: a code an agent can branch on, and a reason it can read. A tool returns one as an
//! error result (`isError: true`) whose text is `<code>: <reason>`; `choose` returns one as a step's rejection.

use std::fmt;

/// A refusal with its code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ToolError {
    pub code: &'static str,
    pub reason: String,
}

impl ToolError {
    /// The Build code doesn't open: it doesn't decode with this game data, or its Build isn't possible.
    pub fn stale_link() -> Self {
        Self {
            code: "stale_link",
            reason: "The Build code doesn't open: it is damaged or hand-edited, or the game data changed since it was \
                     made. Start again with `new_build`, or use a code a tool returned."
                .to_owned(),
        }
    }

    /// The address isn't one, or names a choice the Build doesn't have (or one with no options of its own).
    pub fn no_such_choice(reason: impl Into<String>) -> Self {
        Self {
            code: "no_such_choice",
            reason: reason.into(),
        }
    }

    /// The value given for a choice isn't one its picker or stepper offers.
    pub fn not_an_option(reason: impl Into<String>) -> Self {
        Self {
            code: "not_an_option",
            reason: reason.into(),
        }
    }

    /// The choice can't be withdrawn: something later depends on it, or it isn't withdrawn on its own.
    pub fn not_removable(reason: impl Into<String>) -> Self {
        Self {
            code: "not_removable",
            reason: reason.into(),
        }
    }

    /// The arguments parse but don't make sense together (`compare` given one Build, or five).
    pub fn invalid_arguments(reason: impl Into<String>) -> Self {
        Self {
            code: "invalid_arguments",
            reason: reason.into(),
        }
    }

    /// The engine's own edit made a Build its rules don't allow. The final gate refuses it, so it never comes back;
    /// it would be a bug in the planner.
    pub fn internal(reason: impl Into<String>) -> Self {
        Self {
            code: "internal",
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.reason)
    }
}

impl From<ToolError> for String {
    fn from(error: ToolError) -> Self {
        error.to_string()
    }
}
