use std::io::Error;

use html5ever::serialize::{Serialize, Serializer, TraversalScope};

use crate::node::Node;

use super::{node_ref::Edge, NodeRef};

impl<'a> Serialize for NodeRef<'a> {
    fn serialize<S: Serializer>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope,
    ) -> Result<(), Error> {
        for edge in self.traverse() {
            match edge {
                Edge::Open(node) => {
                    if node == *self && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    match node.node() {
                        Node::Doctype(ref doctype) => {
                            serializer.write_doctype(doctype.name())?;
                        }
                        Node::Comment(ref comment) => {
                            serializer.write_comment(comment)?;
                        }
                        Node::Text(ref text) => {
                            serializer.write_text(text)?;
                        }
                        Node::Element(ref elem) => {
                            let attrs = elem.attrs.iter().map(|(k, v)| (k, &v[..]));
                            serializer.start_elem(elem.name.clone(), attrs)?;
                        }
                        _ => (),
                    }
                }

                Edge::Close(node) => {
                    if node == *self && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    if let Some(elem) = node.as_element() {
                        serializer.end_elem(elem.name.clone())?;
                    }
                }
            }
        }

        Ok(())
    }
}
