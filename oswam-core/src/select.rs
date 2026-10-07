use crate::category::CleanupKind;
use crate::risk::RiskLevel;
use crate::scan::{ScanEntry, ScanResult};

#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub safe_only: bool,
    pub categories: Option<Vec<String>>,
}

pub fn is_deletable(entry: &ScanEntry) -> bool {
    entry.risk != RiskLevel::Never && entry.kind != CleanupKind::InfoOnly && !entry.blocked
}

pub fn selectable<'a>(result: &'a ScanResult, sel: &Selection) -> Vec<&'a ScanEntry> {
    in_scope(result, sel)
        .into_iter()
        .filter(|e| is_deletable(e))
        .collect()
}

pub fn blocked<'a>(result: &'a ScanResult, sel: &Selection) -> Vec<&'a ScanEntry> {
    in_scope(result, sel)
        .into_iter()
        .filter(|e| e.blocked)
        .collect()
}

fn in_scope<'a>(result: &'a ScanResult, sel: &Selection) -> Vec<&'a ScanEntry> {
    result
        .categories
        .iter()
        .filter(|c| match &sel.categories {
            Some(ids) => ids.iter().any(|id| id == &c.id),
            None => true,
        })
        .flat_map(|c| c.entries.iter())
        .filter(|e| !sel.safe_only || e.risk == RiskLevel::Safe)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::NativeSpec;
    use crate::scan::ScanCategory;
    use std::path::PathBuf;

    fn entry(risk: RiskLevel, kind: CleanupKind) -> ScanEntry {
        ScanEntry {
            display: "x".into(),
            path: PathBuf::from("/x"),
            kind,
            risk,
            physical_bytes: 10,
            native: matches!(kind, CleanupKind::NativeCommand).then(|| NativeSpec {
                estimate: vec![],
                clean: vec![],
                ..NativeSpec::default()
            }),
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: false,
            partial: false,
            needs_root: false,
            permanent_only: false,
            blocking_app: None,
            blocked: false,
        }
    }

    fn result() -> ScanResult {
        ScanResult {
            categories: vec![
                ScanCategory {
                    id: "system".into(),
                    name: "S".into(),
                    glyph: "s".into(),
                    entries: vec![
                        entry(RiskLevel::Safe, CleanupKind::DeleteContents),
                        entry(RiskLevel::Never, CleanupKind::DeletePath),
                    ],
                    total_bytes: 20,
                },
                ScanCategory {
                    id: "big-data".into(),
                    name: "B".into(),
                    glyph: "b".into(),
                    entries: vec![entry(RiskLevel::Caution, CleanupKind::InfoOnly)],
                    total_bytes: 10,
                },
            ],
            total_bytes: 30,
        }
    }

    #[test]
    fn excludes_never_and_info_only() {
        let r = result();
        let sel = selectable(&r, &Selection::default());
        assert_eq!(sel.len(), 1);
        assert_eq!(sel[0].risk, RiskLevel::Safe);
    }

    #[test]
    fn safe_only_filters_caution() {
        let mut r = result();
        r.categories[0]
            .entries
            .push(entry(RiskLevel::Caution, CleanupKind::DeleteContents));
        let all = selectable(&r, &Selection::default());
        assert_eq!(all.len(), 2);
        let safe = selectable(
            &r,
            &Selection {
                safe_only: true,
                categories: None,
            },
        );
        assert_eq!(safe.len(), 1);
    }

    #[test]
    fn category_filter() {
        let r = result();
        let sel = selectable(
            &r,
            &Selection {
                safe_only: false,
                categories: Some(vec!["big-data".into()]),
            },
        );
        assert!(sel.is_empty());
    }

    #[test]
    fn a_blocked_entry_is_reported_instead_of_selected() {
        let mut r = result();
        let mut held = entry(RiskLevel::Caution, CleanupKind::DeletePath);
        held.blocked = true;
        r.categories[1].entries.push(held);
        assert_eq!(selectable(&r, &Selection::default()).len(), 1);
        assert_eq!(blocked(&r, &Selection::default()).len(), 1);
        let other = Selection {
            safe_only: false,
            categories: Some(vec!["system".into()]),
        };
        assert!(blocked(&r, &other).is_empty());
    }
}
