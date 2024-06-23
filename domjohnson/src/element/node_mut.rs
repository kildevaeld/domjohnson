use generational_indextree::{Arena, NodeId};

use crate::{node::Node, NodeRef};

use super::node_ref::Text;

pub struct NodeMut<'a> {
    tree: &'a mut Arena<Node>,
    id: NodeId,
}

impl<'a> core::ops::Deref for NodeMut<'a> {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        self.tree[self.id].get()
    }
}

impl<'a> core::ops::DerefMut for NodeMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree[self.id].get_mut()
    }
}

impl<'a> NodeMut<'a> {
    pub fn html(&self) -> String {
        NodeRef::new(&self.tree, self.id).html()
    }

    /// Returns the inner HTML of this element.
    pub fn inner_html(&self) -> String {
        NodeRef::new(&self.tree, self.id).inner_html()
    }

    pub fn text(&self) -> Text<'_> {
        NodeRef::new(&self.tree, self.id).text()
    }

    pub fn attr(&self, str: impl AsRef<str>) -> Option<&String> {
        if let Some(element) = self.as_element() {
            element.attr(str.as_ref())
        } else {
            None
        }
    }

    pub fn remove(&mut self) {
        self.id.remove_subtree(self.tree)
    }
}
