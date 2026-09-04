use std::cell::{Cell, Ref, RefCell};

use generational_indextree::{Arena, NodeId};
use html5ever::{
    expanded_name,
    interface::{ElemName, NodeOrText, QuirksMode, TreeSink},
    local_name, ns,
};

use crate::node::{Comment, Doctype, Element, Node, ProcessingInstruction, Text};

use super::Document;

pub struct DocumentBuilder {
    errors: RefCell<Vec<std::borrow::Cow<'static, str>>>,
    tree: RefCell<Arena<Node>>,
    quirks_mode: Cell<QuirksMode>,
    root: NodeId,
}

#[derive(Debug)]
pub struct NodeName<'a>(Ref<'a, Node>);

impl ElemName for NodeName<'_> {
    fn ns(&self) -> &html5ever::Namespace {
        &self.0.as_element().unwrap().name.ns
    }

    fn local_name(&self) -> &html5ever::LocalName {
        &self.0.as_element().unwrap().name.local
    }
}

impl DocumentBuilder {
    pub fn new() -> DocumentBuilder {
        let mut tree = Arena::default();
        let root = tree.new_node(Node::Document);

        DocumentBuilder {
            errors: RefCell::default(),
            tree: RefCell::new(tree),
            root,
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }
    }
}

impl TreeSink for DocumentBuilder {
    type Handle = NodeId;
    type Output = Document;
    type ElemName<'a> = NodeName<'a>;

    fn finish(self) -> Self::Output {
        Document::new(self.tree.into_inner(), self.root, self.quirks_mode.get())
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        self.errors.borrow_mut().push(msg)
    }

    fn get_document(&self) -> Self::Handle {
        self.root
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        NodeName(Ref::map(self.tree.borrow(), |tree| tree[*target].get()))
    }

    fn create_element(
        &self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        let mut tree = self.tree.borrow_mut();
        let node = tree.new_node(Node::Element(Element::new(name.clone(), attrs)));
        if name.expanded() == expanded_name!(html "template") {
            let child = tree.new_node(Node::Fragment);
            node.append(child, &mut tree);
        }
        node
    }

    fn create_comment(&self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.tree.borrow_mut().new_node(Node::Comment(Comment {
            comment: text.to_string().into(),
        }))
    }

    fn create_pi(
        &self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        self.tree
            .borrow_mut()
            .new_node(Node::ProcessingInstruction(ProcessingInstruction {
                target: target.into(),
                data: data.into(),
            }))
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut tree = self.tree.borrow_mut();
        match child {
            NodeOrText::AppendNode(id) => parent.append(id, &mut tree),
            NodeOrText::AppendText(text) => {
                let last_child = parent.reverse_children(&tree).next();
                if let Some(last_child) = last_child.filter(|id| tree[*id].get().is_text()) {
                    tree[last_child]
                        .get_mut()
                        .as_text_mut()
                        .unwrap()
                        .concat(&text);
                } else {
                    let child = tree.new_node(Node::Text(Text {
                        text: (&*text).into(),
                    }));
                    parent.append(child, &mut tree);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = self.tree.borrow().get(*element).unwrap().parent().is_some();
        if has_parent {
            self.append_before_sibling(element, child)
        } else {
            self.append(prev_element, child)
        }
    }

    fn append_doctype_to_document(
        &self,
        name: html5ever::tendril::StrTendril,
        public_id: html5ever::tendril::StrTendril,
        system_id: html5ever::tendril::StrTendril,
    ) {
        let doctype = Doctype {
            name: (&*name).into(),
            public_id: (&*public_id).into(),
            system_id: (&*system_id).into(),
        };
        let mut tree = self.tree.borrow_mut();
        let node = tree.new_node(Node::Doctype(doctype));
        self.root.append(node, &mut tree);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.tree
            .borrow()
            .get(*target)
            .unwrap()
            .first_child()
            .unwrap()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let mut tree = self.tree.borrow_mut();
        if let NodeOrText::AppendNode(id) = new_node {
            id.detach(&mut tree);
        }

        if tree.get(*sibling).unwrap().parent().is_none() {
            return;
        }

        match new_node {
            NodeOrText::AppendNode(id) => sibling.insert_before(id, &mut tree),
            NodeOrText::AppendText(text) => {
                let previous_sibling = tree.get(*sibling).unwrap().previous_sibling();
                if let Some(previous_sibling) =
                    previous_sibling.filter(|id| tree[*id].get().is_text())
                {
                    tree[previous_sibling]
                        .get_mut()
                        .as_text_mut()
                        .unwrap()
                        .concat(&text);
                } else {
                    let child = tree.new_node(Node::Text(Text {
                        text: (&*text).into(),
                    }));
                    sibling.insert_before(child, &mut tree);
                }
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let mut tree = self.tree.borrow_mut();
        let element = tree
            .get_mut(*target)
            .unwrap()
            .get_mut()
            .as_element_mut()
            .unwrap();

        for attr in attrs {
            element
                .attrs
                .entry(attr.name)
                .or_insert_with(|| attr.value.into());
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        target.detach(&mut self.tree.borrow_mut());
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut tree = self.tree.borrow_mut();
        let children = node.children(&tree).collect::<Vec<_>>();
        for child in children {
            new_parent.append(child, &mut tree);
        }
    }
}
