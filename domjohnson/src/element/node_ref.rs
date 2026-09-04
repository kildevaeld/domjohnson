use crate::node::Node;

use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use smol_str::SmolStr;
use trae::{NodeEdge, NodeId, Tree};

#[derive(Debug, Clone, Copy)]
pub struct NodeRef<'a> {
    pub(crate) tree: &'a Tree<Node>,
    pub(crate) id: NodeId,
}

impl<'a> PartialEq for NodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> Eq for NodeRef<'a> {}

impl<'a> std::ops::Deref for NodeRef<'a> {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        self.node()
    }
}

impl<'a> NodeRef<'a> {
    pub(crate) fn new(tree: &'a Tree<Node>, id: NodeId) -> Self {
        NodeRef { tree, id }
    }

    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a>> {
        self.tree
            .children(self.id)
            .map(|node| NodeRef::new(self.tree, node))
    }

    pub fn reverse_children(&self) -> ChildrenRev<'a> {
        ChildrenRev {
            inner: self.tree.reverse_children(self.id),
            area: self.tree,
        }
    }

    pub fn prev_siblings(&self) -> PrevSiblings<'a> {
        let mut inner = self.tree.proceeding_siblings(self.id);
        inner.next();
        PrevSiblings {
            inner,
            arena: self.tree,
        }
    }

    pub fn next_siblings(&self) -> NextSiblings<'a> {
        let mut inner = self.tree.forward_siblings(self.id);
        inner.next();
        NextSiblings {
            inner,
            arena: self.tree,
        }
    }

    fn serialize(&self, traversal_scope: TraversalScope) -> String {
        let opts = SerializeOpts {
            scripting_enabled: false, // It's not clear what this does.
            traversal_scope,
            create_missing_parent: false,
        };
        let mut buf = Vec::new();
        serialize(&mut buf, self, opts).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn node(&self) -> &'a Node {
        &self.tree[self.id]
    }

    pub fn node_type(&self) -> String {
        match &self.tree[self.id] {
            Node::Comment(_) => "comment".to_string(),
            Node::Doctype(_) => "doctype".to_string(),
            Node::Element(el) => el.name().to_string(),
            Node::Fragment => "fragment".to_owned(),
            Node::Text(_) => "text".to_owned(),
            _ => "".to_string(),
        }
    }

    pub fn parent(&self) -> Option<NodeRef<'a>> {
        self.tree.parent(self.id).map(|id| NodeRef {
            tree: self.tree,
            id,
        })
    }

    /// Returns the HTML of this element.
    pub fn html(&self) -> String {
        self.serialize(TraversalScope::IncludeNode)
    }

    /// Returns the inner HTML of this element.
    pub fn inner_html(&self) -> String {
        self.serialize(TraversalScope::ChildrenOnly(None))
    }

    /// Returns an iterator over descendent text nodes.
    pub fn text(&self) -> Text<'a> {
        Text {
            inner: self.traverse(),
        }
    }

    pub fn attr(&self, str: impl AsRef<str>) -> Option<&String> {
        if let Some(element) = self.as_element() {
            element.attr(str.as_ref())
        } else {
            None
        }
    }

    pub fn traverse(&self) -> Traverse<'a> {
        Traverse {
            inner: self.tree.traverse(self.id),
            tree: self.tree,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Edge<'a> {
    Open(NodeRef<'a>),
    Close(NodeRef<'a>),
}

pub struct Traverse<'a> {
    inner: trae::Traverse<'a, Node>,
    tree: &'a Tree<Node>,
}

impl<'a> Iterator for Traverse<'a> {
    type Item = Edge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            NodeEdge::Start(node) => Some(Edge::Open(NodeRef {
                tree: self.tree,
                id: node,
            })),
            NodeEdge::End(node) => Some(Edge::Close(NodeRef {
                tree: self.tree,
                id: node,
            })),
        }
    }
}

pub struct Text<'a> {
    inner: Traverse<'a>,
}

impl<'a> Iterator for Text<'a> {
    type Item = &'a SmolStr;

    fn next(&mut self) -> Option<&'a SmolStr> {
        for edge in &mut self.inner {
            if let Edge::Open(node) = edge {
                if let Node::Text(ref text) = node.node() {
                    return Some(&text.text);
                }
            }
        }
        None
    }
}

pub struct ChildrenRev<'a> {
    inner: trae::ReverseChildren<'a, Node>,
    area: &'a Tree<Node>,
}

impl<'a> Iterator for ChildrenRev<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(self.area, idx))
    }
}

pub struct PrevSiblings<'a> {
    inner: trae::ProceedingSiblings<'a, Node>,
    arena: &'a Tree<Node>,
}

impl<'a> Iterator for PrevSiblings<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(self.arena, idx))
    }
}

pub struct NextSiblings<'a> {
    inner: trae::ForwardSiblings<'a, Node>,
    arena: &'a Tree<Node>,
}

impl<'a> Iterator for NextSiblings<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(self.arena, idx))
    }
}

#[cfg(test)]
mod tests {
    use super::NodeRef;
    use crate::node::{Comment, Element, Node, Text};
    use html5ever::{ns, LocalName, QualName};
    use trae::Tree;

    #[test]
    fn node_ref_navigates_and_serializes_a_tree() {
        let mut tree = Tree::new();
        let document = tree.alloc(Node::Document);
        let mut element = Element::new(
            QualName::new(None, ns!(html), LocalName::from("div")),
            Vec::new(),
        );
        element.set_attr("id", "root");
        let div = tree.alloc(Node::Element(element));
        let text = tree.alloc(Node::Text(Text {
            text: "text".into(),
        }));
        let comment = tree.alloc(Node::Comment(Comment {
            comment: "note".into(),
        }));
        let span = tree.alloc(Node::Element(Element::new(
            QualName::new(None, ns!(html), LocalName::from("span")),
            Vec::new(),
        )));

        tree.append(document, div);
        tree.append(div, text);
        tree.append(div, comment);
        tree.append(div, span);

        let node = NodeRef::new(&tree, div);
        assert_eq!(node.node_type(), "div");
        assert_eq!(node.attr("id").map(String::as_str), Some("root"));
        assert_eq!(node.parent().unwrap().node_type(), "");
        assert_eq!(
            node.children()
                .map(|child| child.node_type())
                .collect::<Vec<_>>(),
            ["text", "comment", "span"]
        );
        assert_eq!(
            node.reverse_children()
                .map(|child| child.node_type())
                .collect::<Vec<_>>(),
            ["span", "comment", "text"]
        );

        let comment = NodeRef::new(&tree, comment);
        assert_eq!(
            comment
                .prev_siblings()
                .map(|sibling| sibling.node_type())
                .collect::<Vec<_>>(),
            ["text"]
        );
        assert_eq!(
            comment
                .next_siblings()
                .map(|sibling| sibling.node_type())
                .collect::<Vec<_>>(),
            ["span"]
        );

        assert_eq!(
            node.html(),
            r#"<div id="root">text<!--note--><span></span></div>"#
        );
        assert_eq!(node.inner_html(), "text<!--note--><span></span>");
        assert_eq!(
            node.text().map(|text| text.as_str()).collect::<Vec<_>>(),
            ["text"]
        );
        assert_eq!(node.traverse().count(), 8);
    }
}
