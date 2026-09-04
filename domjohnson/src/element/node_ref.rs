use crate::node::Node;

use generational_indextree::{Arena, NodeEdge, NodeId};
use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy)]
pub struct NodeRef<'a> {
    pub(crate) tree: &'a Arena<Node>,
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
    pub(crate) fn new(tree: &'a Arena<Node>, id: NodeId) -> Self {
        NodeRef { tree, id }
    }

    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a>> {
        self.id
            .children(self.tree)
            .map(|node| NodeRef::new(self.tree, node))
    }

    pub fn reverse_children(&self) -> ChildrenRev<'a> {
        ChildrenRev {
            inner: self.id.reverse_children::<Node>(self.tree),
            area: self.tree,
        }
    }

    pub fn prev_siblings(&self) -> PrevSiblings<'a> {
        let mut inner = self.id.preceding_siblings(&self.tree);
        inner.next();
        PrevSiblings {
            inner,
            arena: &self.tree,
        }
    }

    pub fn next_siblings(&self) -> NextSiblings<'a> {
        let mut inner = self.id.following_siblings(&self.tree);
        inner.next();
        NextSiblings {
            inner,
            arena: &self.tree,
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
        self.tree[self.id].get()
    }

    pub fn node_type(&self) -> String {
        match self.tree[self.id].get() {
            Node::Comment(_) => "comment".to_string(),
            Node::Doctype(_) => "doctype".to_string(),
            Node::Element(el) => el.name().to_string(),
            Node::Fragment => "fragment".to_owned(),
            Node::Text(_) => "text".to_owned(),
            _ => "".to_string(),
        }
    }

    pub fn parent(&self) -> Option<NodeRef<'a>> {
        self.tree[self.id].parent().map(|id| NodeRef {
            tree: &self.tree,
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
            inner: self.id.traverse(&self.tree),
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
    inner: generational_indextree::Traverse<'a, Node>,
    tree: &'a Arena<Node>,
}

impl<'a> Iterator for Traverse<'a> {
    type Item = Edge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            NodeEdge::Start(node) => Some(Edge::Open(NodeRef {
                tree: &self.tree,
                id: node,
            })),
            NodeEdge::End(node) => Some(Edge::Close(NodeRef {
                tree: &self.tree,
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
    inner: generational_indextree::ReverseChildren<'a, Node>,
    area: &'a Arena<Node>,
}

impl<'a> Iterator for ChildrenRev<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(&self.area, idx))
    }
}

pub struct PrevSiblings<'a> {
    inner: generational_indextree::PrecedingSiblings<'a, Node>,
    arena: &'a Arena<Node>,
}

impl<'a> Iterator for PrevSiblings<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(&self.arena, idx))
    }
}

pub struct NextSiblings<'a> {
    inner: generational_indextree::FollowingSiblings<'a, Node>,
    arena: &'a Arena<Node>,
}

impl<'a> Iterator for NextSiblings<'a> {
    type Item = NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|idx| NodeRef::new(&self.arena, idx))
    }
}

#[cfg(test)]
mod tests {
    use super::NodeRef;
    use crate::node::{Comment, Element, Node, Text};
    use generational_indextree::Arena;
    use html5ever::{ns, LocalName, QualName};

    #[test]
    fn node_ref_navigates_and_serializes_a_tree() {
        let mut tree = Arena::new();
        let document = tree.new_node(Node::Document);
        let mut element = Element::new(
            QualName::new(None, ns!(html), LocalName::from("div")),
            Vec::new(),
        );
        element.set_attr("id", "root");
        let div = tree.new_node(Node::Element(element));
        let text = tree.new_node(Node::Text(Text {
            text: "text".into(),
        }));
        let comment = tree.new_node(Node::Comment(Comment {
            comment: "note".into(),
        }));
        let span = tree.new_node(Node::Element(Element::new(
            QualName::new(None, ns!(html), LocalName::from("span")),
            Vec::new(),
        )));

        document.append(div, &mut tree);
        div.append(text, &mut tree);
        div.append(comment, &mut tree);
        div.append(span, &mut tree);

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
