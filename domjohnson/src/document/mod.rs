use core::fmt;

use self::sink::DocumentBuilder;
use crate::element::node_ref::Text;
use crate::node::{Comment, Doctype, Element, Node};
use crate::selection::Selection;
use crate::{MatchScope, Matcher, Matches, NodeRef};
use generational_indextree::{Arena, NodeId};
use html5ever::serialize::TraversalScope;
use html5ever::tendril::TendrilSink;
use html5ever::{interface::QuirksMode, parse_document, ParseOpts};
use html5ever::{namespace_url, ns, LocalName, QualName};
use smol_str::SmolStr;

mod sink;

pub struct Document {
    quirks: QuirksMode,
    tree: Arena<Node>,
    root: NodeId,
}

impl Document {
    pub fn parse(html: &str) -> Document {
        let parser = parse_document(DocumentBuilder::new(), ParseOpts::default());
        parser.one(html)
    }

    pub fn new_html5() -> Document {
        let mut tree = Arena::new();

        let root = tree.new_node(Node::Document);
        let doctype = tree.new_node(Node::Doctype(Doctype {
            name: "html".into(),
            public_id: "".into(),
            system_id: "".into(),
        }));

        root.append(doctype, &mut tree);

        let head_tag = tree.new_node(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("head")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        let body_tag = tree.new_node(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("body")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        let html_tag = tree.new_node(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("html")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        html_tag.append(head_tag, &mut tree);
        html_tag.append(body_tag, &mut tree);

        root.append(html_tag, &mut tree);

        Document {
            quirks: QuirksMode::NoQuirks,
            tree,
            root,
        }
    }
}

impl Document {
    pub(crate) fn new(tree: Arena<Node>, root: NodeId, quirks: QuirksMode) -> Document {
        Document { quirks, tree, root }
    }

    pub(crate) fn tree(&self) -> &Arena<Node> {
        &self.tree
    }

    pub fn select(&self, selector: &str) -> Selection {
        self.select_from(self.root, selector)
    }

    pub fn select_from(&self, node: NodeId, selector: &str) -> Selection {
        let matcher = Matcher::new(selector).expect("invalid css selector");
        Selection::new(
            Matches::from_one(&self.tree, node, matcher, MatchScope::ChildrenOnly).collect(),
        )
    }

    pub fn get(&self, node: NodeId) -> Option<&Node> {
        self.tree.get(node).map(|m| m.get())
    }

    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut Node> {
        self.tree.get_mut(node).map(|m| m.get_mut())
    }

    pub fn remove(&mut self, node: NodeId) {
        node.remove_subtree(&mut self.tree)
    }

    pub fn delete(&mut self, node: NodeId) {
        node.remove(&mut self.tree)
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) {
        parent.append(child, &mut self.tree)
    }

    pub fn traverse(&self, node: NodeId) -> generational_indextree::Traverse<'_, Node> {
        node.traverse(&self.tree)
    }

    pub fn children(&self, node: NodeId) -> generational_indextree::Children<'_, Node> {
        node.children(&self.tree)
    }

    pub fn inner_html(&self, node: NodeId) -> String {
        NodeRef::new(&self.tree, node).inner_html()
    }

    pub fn text(&self, node: NodeId) -> Text<'_> {
        NodeRef::new(&self.tree, node).text()
    }

    pub fn create_element(&mut self, name: &str) -> NodeId {
        let name = QualName::new(None, ns!(html), LocalName::from(name));
        let node = Node::Element(Element::new(name, Vec::new()));
        self.tree.new_node(node)
    }

    pub fn create_text(&mut self, text: impl Into<SmolStr>) -> NodeId {
        let node = Node::Text(crate::node::Text { text: text.into() });
        self.tree.new_node(node)
    }

    pub fn create_comment(&mut self, comment: impl Into<SmolStr>) -> NodeId {
        let node = Node::Comment(Comment {
            comment: comment.into(),
        });
        self.tree.new_node(node)
    }

    pub fn orhpans(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.tree.iter_pairs().filter_map(|(id, node)| {
            if id == self.root || node.parent().is_some() {
                None
            } else {
                Some(id)
            }
        })
    }

    pub fn remove_orphans(&mut self) {
        let nodes = self.orhpans().collect::<Vec<_>>();
        for node in nodes {
            self.delete(node);
        }
    }
}

impl core::ops::Index<NodeId> for Document {
    type Output = Node;
    fn index(&self, index: NodeId) -> &Self::Output {
        self.get(index).expect("node")
    }
}

impl core::ops::IndexMut<NodeId> for Document {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        self.get_mut(index).expect("node")
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", NodeRef::new(&self.tree, self.root).html())
    }
}
