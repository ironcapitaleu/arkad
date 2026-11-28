//! # Notes Table
//!
//! Provides [`NotesTable`], one of the eight tables of a Notes file.

use std::fmt::{self, Display, Formatter};

/// One of the eight tables that every Notes file holds, each as a tab-separated member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotesTable {
    /// `sub.tsv`: one row per filing.
    Sub,
    /// `num.tsv`: the numeric values.
    Num,
    /// `txt.tsv`: the text values.
    Txt,
    /// `dim.tsv`: the segments of each dimension hash.
    Dim,
    /// `tag.tsv`: the metadata and the labels of each tag.
    Tag,
    /// `pre.tsv`: the position of each value in the statements.
    Pre,
    /// `cal.tsv`: the calculation relationships.
    Cal,
    /// `ren.tsv`: the rendering of each report.
    Ren,
}

impl NotesTable {
    /// The eight tables, in the order of this enum.
    pub const ALL: [Self; 8] = [
        Self::Sub,
        Self::Num,
        Self::Txt,
        Self::Dim,
        Self::Tag,
        Self::Pre,
        Self::Cal,
        Self::Ren,
    ];

    /// Returns the name of the member of the table, for example `num.tsv`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_archive::NotesTable;
    ///
    /// let table = NotesTable::Num;
    ///
    /// let expected_result = "num.tsv";
    ///
    /// let result = table.member_name();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn member_name(self) -> &'static str {
        match self {
            Self::Sub => "sub.tsv",
            Self::Num => "num.tsv",
            Self::Txt => "txt.tsv",
            Self::Dim => "dim.tsv",
            Self::Tag => "tag.tsv",
            Self::Pre => "pre.tsv",
            Self::Cal => "cal.tsv",
            Self::Ren => "ren.tsv",
        }
    }

    /// Returns the table whose member has the given name, or `None` if no table has it.
    #[must_use]
    pub fn from_member_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|table| table.member_name() == name)
    }
}

/// Formats as the name of the member, for example `num.tsv`.
impl Display for NotesTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.member_name())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_notes_table() {
        implements_auto_traits::<NotesTable>();
    }

    #[test]
    fn should_return_the_table_when_the_member_name_names_a_table() {
        let name = "dim.tsv";

        let expected_result = Some(NotesTable::Dim);

        let result = NotesTable::from_member_name(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_member_name_names_no_table() {
        let name = "readme.htm";

        let expected_result = None;

        let result = NotesTable::from_member_name(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_member_name_for_notes_table() {
        let table = NotesTable::Ren;

        let expected_result = "ren.tsv";

        let result = table.to_string();

        assert_eq!(result, expected_result);
    }
}
