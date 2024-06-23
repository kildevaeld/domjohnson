use crate::{
    matcher::{MatchScope, Matcher, Matches},
    Document,
};
use generational_indextree::NodeId;

#[derive(Debug, Clone)]
pub struct Selection {
    pub(crate) nodes: Vec<NodeId>,
}

impl Selection {
    pub(crate) fn new(nodes: Vec<NodeId>) -> Selection {
        Selection { nodes }
    }

    pub fn select<S: AsRef<str>>(&self, dom: &Document, sel: S) -> Selection {
        let matcher = Matcher::new(sel.as_ref()).expect("Invalid CSS selector");

        Selection::new(
            Matches::from_list(
                dom.tree(),
                self.nodes.iter().copied(),
                matcher.clone(),
                MatchScope::IncludeNode,
            )
            .collect(),
        )
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<NodeId> {
        self.nodes.get(idx).copied()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, NodeId> {
        self.nodes.iter()
    }
}

impl IntoIterator for Selection {
    type IntoIter = std::vec::IntoIter<NodeId>;
    type Item = NodeId;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Selection {
    type IntoIter = std::slice::Iter<'a, NodeId>;
    type Item = &'a NodeId;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}
