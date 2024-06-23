use generational_indextree::{Arena, NodeId};
use html5ever::{
    expanded_name,
    interface::{NodeOrText, QuirksMode, TreeSink},
    local_name, namespace_url, ns,
};

use crate::node::{Comment, Doctype, Element, Node, ProcessingInstruction, Text};

use super::Document;

pub struct DocumentBuilder {
    errors: Vec<std::borrow::Cow<'static, str>>,
    tree: Arena<Node>,
    quirks_mode: QuirksMode,
    root: NodeId,
}

impl DocumentBuilder {
    pub fn new() -> DocumentBuilder {
        let mut tree = Arena::default();

        let root = tree.new_node(Node::Document);

        DocumentBuilder {
            errors: Vec::default(),
            tree,
            root,
            quirks_mode: QuirksMode::NoQuirks,
        }
    }
}

impl TreeSink for DocumentBuilder {
    type Handle = NodeId;
    type Output = Document;

    fn finish(self) -> Self::Output {
        Document::new(self.tree, self.root, self.quirks_mode)
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        self.errors.push(msg)
    }

    fn get_document(&mut self) -> Self::Handle {
        self.root
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> html5ever::ExpandedName<'a> {
        self.tree[*target]
            .get()
            .as_element()
            .unwrap()
            .name
            .expanded()
    }

    fn create_element(
        &mut self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        let node = self
            .tree
            .new_node(Node::Element(Element::new(name.clone(), attrs)));
        if name.expanded() == expanded_name!(html "template") {
            let child = self.tree.new_node(Node::Fragment);
            node.append(child, &mut self.tree);
        }

        node
    }

    fn create_comment(&mut self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.tree.new_node(Node::Comment(Comment {
            comment: text.to_string().into(),
        }))
    }

    fn create_pi(
        &mut self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        self.tree
            .new_node(Node::ProcessingInstruction(ProcessingInstruction {
                target: target.into(),
                data: data.into(),
            }))
    }

    fn append(
        &mut self,
        parent: &Self::Handle,
        child: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        match child {
            NodeOrText::AppendNode(id) => {
                parent.append(id, &mut self.tree);
            }

            NodeOrText::AppendText(text) => {
                let can_concat = parent
                    .reverse_children(&self.tree)
                    .next()
                    .map_or(false, |n| self.tree[n].get().is_text());

                if can_concat {
                    let last_child = parent.reverse_children(&self.tree).next().unwrap();
                    match self.tree[last_child].get_mut() {
                        Node::Text(ref mut t) => t.concat(&text),
                        _ => unreachable!(),
                    }
                } else {
                    let child = self.tree.new_node(Node::Text(Text {
                        text: (&*text).into(),
                    }));
                    parent.append(child, &mut self.tree);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        if self.tree.get(*element).unwrap().parent().is_some() {
            self.append_before_sibling(element, child)
        } else {
            self.append(prev_element, child)
        }
    }

    fn append_doctype_to_document(
        &mut self,
        name: html5ever::tendril::StrTendril,
        public_id: html5ever::tendril::StrTendril,
        system_id: html5ever::tendril::StrTendril,
    ) {
        let doctype = Doctype {
            name: (&*name).into(),
            public_id: (&*public_id).into(),
            system_id: (&*system_id).into(),
        };

        let node = self.tree.new_node(Node::Doctype(doctype));
        self.root.append(node, &mut self.tree);
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        self.tree.get(*target).unwrap().first_child().unwrap()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        if let NodeOrText::AppendNode(id) = new_node {
            id.detach(&mut self.tree);
        }

        let sibling_node = self.tree.get(*sibling).unwrap();
        if sibling_node.parent().is_some() {
            match new_node {
                NodeOrText::AppendNode(id) => {
                    sibling.insert_before(id, &mut self.tree);
                }

                NodeOrText::AppendText(text) => {
                    let can_concat = sibling_node
                        .previous_sibling()
                        .map_or(false, |n| self.tree[n].get().is_text());

                    if can_concat {
                        let prev_sibling = sibling_node.previous_sibling().unwrap();
                        match self.tree[prev_sibling].get_mut() {
                            Node::Text(t) => t.concat(&text),
                            _ => unreachable!(),
                        }
                    } else {
                        let child = self.tree.new_node(Node::Text(Text {
                            text: (&*text).into(),
                        }));
                        sibling.insert_before(child, &mut self.tree);
                    }
                }
            }
        }
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let node = self.tree.get_mut(*target).unwrap();
        let element = match *node.get_mut() {
            Node::Element(ref mut e) => e,
            _ => unreachable!(),
        };

        for attr in attrs {
            element
                .attrs
                .entry(attr.name)
                .or_insert_with(|| attr.value.into());
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        target.detach(&mut self.tree);
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        node.detach(&mut self.tree);
        new_parent.append(*node, &mut self.tree);
    }
}
