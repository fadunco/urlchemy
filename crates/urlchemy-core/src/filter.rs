use crate::{Row, RowId};
#[derive(Clone, Copy, Debug)]
pub enum MatchMode {
    CaseSensitiveSubstring,
    CaseInsensitiveSubstring,
}
pub fn filter_rows(rows: &[Row], query: &str, mode: MatchMode) -> Vec<RowId> {
    let query = match mode {
        MatchMode::CaseSensitiveSubstring => query.into(),
        MatchMode::CaseInsensitiveSubstring => query.to_lowercase(),
    };
    rows.iter()
        .filter(|row| match mode {
            MatchMode::CaseSensitiveSubstring => row.label().contains(&query),
            MatchMode::CaseInsensitiveSubstring => row.label().to_lowercase().contains(&query),
        })
        .map(Row::id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntryId, Row, RowId};
    #[test]
    fn ordered_label_filter() {
        let rows: Vec<Row> = ["Accounts", "Billing", "accounts API"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| Row {
                id: RowId::Entry(EntryId {
                    owner: 1,
                    index: i as u32,
                }),
                label: label.into(),
                emoji: None,
            })
            .collect();
        assert_eq!(
            filter_rows(&rows, "", MatchMode::CaseSensitiveSubstring),
            vec![rows[0].id(), rows[1].id(), rows[2].id()]
        );
        assert_eq!(
            filter_rows(&rows, "accounts", MatchMode::CaseSensitiveSubstring),
            vec![rows[2].id()]
        );
        assert_eq!(
            filter_rows(&rows, "accounts", MatchMode::CaseInsensitiveSubstring),
            vec![rows[0].id(), rows[2].id()]
        );
        assert!(filter_rows(&rows, "absent", MatchMode::CaseSensitiveSubstring).is_empty());
    }
}
