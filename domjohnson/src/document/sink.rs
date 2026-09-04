use std::cell::{Cell, Ref, RefCell};

use trae::{NodeId, Tree};

use html5ever::{
    expanded_name,
    interface::{ElemName, NodeOrText, QuirksMode, TreeSink},
    local_name, ns,
};

use crate::node::{Comment, Doctype, Element, Node, ProcessingInstruction, Text};

use super::Document;

pub struct DocumentBuilder {
    errors: RefCell<Vec<std::borrow::Cow<'static, str>>>,
    tree: RefCell<Tree<Node>>,
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
        let mut tree = Tree::default();
        let root = tree.alloc(Node::Document);

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
        NodeName(Ref::map(self.tree.borrow(), |tree| &tree[*target]))
    }

    fn create_element(
        &self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        let mut tree = self.tree.borrow_mut();
        let node = tree.alloc(Node::Element(Element::new(name.clone(), attrs)));
        if name.expanded() == expanded_name!(html "template") {
            let child = tree.alloc(Node::Fragment);
            tree.append(node, child);
        }
        node
    }

    fn create_comment(&self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.tree.borrow_mut().alloc(Node::Comment(Comment {
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
            .alloc(Node::ProcessingInstruction(ProcessingInstruction {
                target: target.into(),
                data: data.into(),
            }))
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut tree = self.tree.borrow_mut();
        match child {
            NodeOrText::AppendNode(id) => tree.append(*parent, id),
            NodeOrText::AppendText(text) => {
                let last_child = tree.reverse_children(*parent).next();
                if let Some(last_child) = last_child.filter(|id| tree[*id].is_text()) {
                    tree[last_child].as_text_mut().unwrap().concat(&text);
                } else {
                    let child = tree.alloc(Node::Text(Text {
                        text: (&*text).into(),
                    }));
                    tree.append(*parent, child);
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
        let has_parent = self.tree.borrow().parent(*element).is_some();
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
        let node = tree.alloc(Node::Doctype(doctype));
        tree.append(self.root, node);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.tree.borrow().children(*target).next().unwrap()
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
            tree.detach(id, false);
        }

        let Some(parent) = tree.parent(*sibling) else {
            return;
        };

        match new_node {
            NodeOrText::AppendNode(id) => tree.insert_before(parent, *sibling, id),
            NodeOrText::AppendText(text) => {
                let mut proceeding = tree.proceeding_siblings(*sibling);
                proceeding.next();
                let previous_sibling = proceeding.next();
                if let Some(previous_sibling) =
                    previous_sibling.filter(|id| tree[*id].is_text())
                {
                    tree[previous_sibling].as_text_mut().unwrap().concat(&text);
                } else {
                    let child = tree.alloc(Node::Text(Text {
                        text: (&*text).into(),
                    }));
                    tree.insert_before(parent, *sibling, child);
                }
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let mut tree = self.tree.borrow_mut();
        let element = tree.get_mut(*target).unwrap().as_element_mut().unwrap();

        for attr in attrs {
            element
                .attrs
                .entry(attr.name)
                .or_insert_with(|| attr.value.into());
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.tree.borrow_mut().detach(*target, false);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut tree = self.tree.borrow_mut();
        let children = tree.children(*node).collect::<Vec<_>>();
        for child in children {
            tree.append(*new_parent, child);
        }
    }
}
