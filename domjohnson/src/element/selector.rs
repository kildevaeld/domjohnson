use super::NodeRef;

use crate::matcher::{InnerSelector, NonTSPseudoClass, PseudoElement};

use html5ever::{namespace_url, ns, LocalName, Namespace};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching;
use selectors::OpaqueElement;

macro_rules! element {
    ($this: expr) => {{
        match $this.node().as_element() {
            Some(el) => el,
            None => return false,
        }
    }};
}

impl<'a> selectors::Element for NodeRef<'a> {
    type Impl = InnerSelector;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.node())
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent()
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn is_part(&self, _name: &LocalName) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        element!(self).name == element!(other).name
    }

    fn exported_part(&self, _: &LocalName) -> Option<LocalName> {
        None
    }

    fn imported_part(&self, _: &LocalName) -> Option<LocalName> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_siblings().find(|sibling| sibling.is_element())
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.next_siblings().find(|sibling| sibling.is_element())
    }

    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Is there more to this?
        element!(self).name.ns == ns!(html)
    }

    fn has_local_name(&self, name: &LocalName) -> bool {
        &element!(self).name.local == name
    }

    fn has_namespace(&self, namespace: &Namespace) -> bool {
        &element!(self).name.ns == namespace
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &LocalName,
        operation: &AttrSelectorOperation<&String>,
    ) -> bool {
        element!(self).attrs.iter().any(|(key, value)| {
            !matches!(*ns, NamespaceConstraint::Specific(url) if *url != key.ns)
                && *local_name == key.local
                && operation.eval_str(value)
        })
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        _pc: &NonTSPseudoClass,
        _context: &mut matching::MatchingContext<Self::Impl>,
        _flags_setter: &mut F,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        element!(self).name() == "link"
    }

    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(&self, id: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        match element!(self).id {
            Some(ref val) => case_sensitivity.eq(id.as_bytes(), val.as_bytes()),
            None => false,
        }
    }

    fn has_class(&self, name: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        element!(self).has_class(name, case_sensitivity)
    }

    fn is_empty(&self) -> bool {
        !self
            .children()
            .any(|child| child.node().is_element() || child.node().is_text())
    }

    fn is_root(&self) -> bool {
        self.parent()
            .map_or(false, |parent| parent.node().is_document())
    }
}
