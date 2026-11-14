use crate::{Catalog, Entry, EntryId};
pub(super) fn entries<'a>(catalog: &'a Catalog, folders: &[EntryId]) -> Option<&'a [Entry]> {
    let mut entries = catalog.roots();
    for id in folders {
        entries = entries.iter().find(|e| e.id() == *id)?.children()?;
    }
    Some(entries)
}
impl super::Session {
    pub fn exact_alias(&self, catalog: &Catalog, text: &str) -> Option<super::Action> {
        if self.owner != Some(catalog.owner) {
            return None;
        }
        let super::State::Browsing(path) = &self.state else {
            return None;
        };
        if let Some(entry) = entries(catalog, path)?
            .iter()
            .find(|e| e.presentation.alias.as_deref() == Some(text))
        {
            return Some(match entry.flow() {
                Some(id) => super::Action::StartFlow(id),
                None => super::Action::EnterFolder(entry.id()),
            });
        }
        if path.is_empty() {
            return catalog
                .flows
                .iter()
                .find(|f| f.direct_alias.as_deref() == Some(text))
                .map(|f| super::Action::StartFlow(f.id));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{choose, fixture};
    use crate::{Action, Session, SessionSnapshot};
    #[test]
    fn alias_context() {
        let c = fixture("branching.yaml");
        let mut s = Session::new();
        s.apply(&c, Action::Invoke);
        assert!(matches!(
            s.exact_alias(&c, "open-dd"),
            Some(Action::StartFlow(_))
        ));
        assert!(s.exact_alias(&c, "open").is_none());
        let Some(Action::EnterFolder(id)) = s.exact_alias(&c, "dd") else {
            panic!();
        };
        s.apply(&c, Action::EnterFolder(id));
        assert!(s.exact_alias(&c, "open-dd").is_none());
        let Some(action) = s.exact_alias(&c, "dashboards") else {
            panic!();
        };
        s.apply(&c, action);
        choose(&mut s, &c, 0);
        assert!(matches!(
            s.snapshot(&c).unwrap(),
            SessionSnapshot::Input { .. }
        ));
        assert!(s.exact_alias(&c, "dd").is_none());
        assert!(s.exact_alias(&c, "open-dd").is_none());
    }
}
