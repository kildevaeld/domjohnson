use core::fmt;

use self::sink::DocumentBuilder;
use crate::element::node_ref::Text;
use crate::node::{Comment, Doctype, Element, Node};
use crate::selection::Selection;
use crate::{MatchScope, Matcher, Matches, NodeRef};
use trae::{NodeId, Tree};

use html5ever::tendril::TendrilSink;
use html5ever::{interface::QuirksMode, parse_document, ParseOpts};
use html5ever::{ns, LocalName, QualName};
use smol_str::SmolStr;

mod sink;

pub struct Document {
    quirks: QuirksMode,
    tree: Tree<Node>,
    root: NodeId,
}

impl Document {
    pub fn parse(html: &str) -> Document {
        let parser = parse_document(DocumentBuilder::new(), ParseOpts::default());
        parser.one(html)
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks
    }

    pub fn new_html5() -> Document {
        let mut tree = Tree::new();

        let root = tree.alloc(Node::Document);
        let doctype = tree.alloc(Node::Doctype(Doctype {
            name: "html".into(),
            public_id: "".into(),
            system_id: "".into(),
        }));

        tree.append(root, doctype);

        let head_tag = tree.alloc(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("head")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        let body_tag = tree.alloc(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("body")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        let html_tag = tree.alloc(Node::Element(Element {
            name: QualName::new(None, ns!(html), LocalName::from("html")),
            id: None,
            classes: Default::default(),
            attrs: Default::default(),
        }));

        tree.append(html_tag, head_tag);
        tree.append(html_tag, body_tag);

        tree.append(root, html_tag);

        Document {
            quirks: QuirksMode::NoQuirks,
            tree,
            root,
        }
    }
}

impl Document {
    pub(crate) fn new(tree: Tree<Node>, root: NodeId, quirks: QuirksMode) -> Document {
        Document { quirks, tree, root }
    }

    pub(crate) fn tree(&self) -> &Tree<Node> {
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
        self.tree.get(node)
    }

    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut Node> {
        self.tree.get_mut(node)
    }

    pub fn remove(&mut self, node: NodeId) {
        self.tree.detach(node, false)
    }

    pub fn delete(&mut self, node: NodeId) {
        if let Some(parent) = self.tree.parent(node) {
            self.tree.detach(node, parent);
        }
        self.tree.remove(node, false);
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) {
        self.tree.append(parent, child)
    }

    pub fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: NodeId) {
        self.tree.insert_before(parent, reference, child)
    }

    pub fn insert_after(&mut self, parent: NodeId, child: NodeId, reference: NodeId) {
        self.tree.insert_after(parent, reference, child)
    }

    pub fn traverse(&self, node: NodeId) -> trae::Traverse<'_, Node> {
        self.tree.traverse(node)
    }

    pub fn children(&self, node: NodeId) -> trae::Children<'_, Node> {
        self.tree.children(node)
    }

    pub fn inner_html(&self, node: NodeId) -> String {
        NodeRef::new(&self.tree, node).inner_html()
    }

    pub fn text(&self, node: NodeId) -> Text<'_> {
        NodeRef::new(&self.tree, node).text()
    }

    pub fn create_element(&mut self, name: &str) -> NodeId {
        let name = name.to_ascii_lowercase();
        let name = QualName::new(None, ns!(html), LocalName::from(name.as_str()));
        let node = Node::Element(Element::new(name, Vec::new()));
        self.tree.alloc(node)
    }

    pub fn create_text(&mut self, text: impl Into<SmolStr>) -> NodeId {
        let node = Node::Text(crate::node::Text { text: text.into() });
        self.tree.alloc(node)
    }

    pub fn create_comment(&mut self, comment: impl Into<SmolStr>) -> NodeId {
        let node = Node::Comment(Comment {
            comment: comment.into(),
        });
        self.tree.alloc(node)
    }

    pub fn orhpans(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.tree.orphans().filter(move |&id| id != self.root)
    }

    pub fn remove_orphans(&mut self) {
        let roots = self.orhpans().collect::<Vec<_>>();
        for root in roots {
            let subtree = self.tree.decendents(root).collect::<Vec<_>>();
            for node in subtree.into_iter().rev() {
                self.tree.remove(node, false);
            }
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
