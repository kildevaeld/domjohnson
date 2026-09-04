use super::NodeRef;

use crate::matcher::{
    InnerSelector, NonTSPseudoClass, PseudoElement, SelectorAttrValue, SelectorString,
};

use html5ever::ns;
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

    fn is_part(&self, _name: &SelectorString) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        element!(self).name == element!(other).name
    }

    fn imported_part(&self, _: &SelectorString) -> Option<SelectorString> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_siblings().find(|sibling| sibling.is_element())
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.next_siblings().find(|sibling| sibling.is_element())
    }

    fn first_element_child(&self) -> Option<Self> {
        self.children().find(|child| child.is_element())
    }

    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Is there more to this?
        element!(self).name.ns == ns!(html)
    }

    fn has_local_name(&self, name: &str) -> bool {
        element!(self).name.local.as_ref() == name
    }

    fn has_namespace(&self, namespace: &str) -> bool {
        element!(self).name.ns.as_ref() == namespace
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&SelectorString>,
        local_name: &SelectorString,
        operation: &AttrSelectorOperation<&SelectorAttrValue>,
    ) -> bool {
        element!(self).attrs.iter().any(|(key, value)| {
            !matches!(*ns, NamespaceConstraint::Specific(url) if url.as_ref() != key.ns.as_ref())
                && local_name.as_ref() == key.local.as_ref()
                && operation.eval_str(value)
        })
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &NonTSPseudoClass,
        _context: &mut matching::MatchingContext<Self::Impl>,
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

    fn apply_selector_flags(&self, _flags: matching::ElementSelectorFlags) {}

    fn is_link(&self) -> bool {
        element!(self).name() == "link"
    }

    fn is_html_slot_element(&self) -> bool {
        element!(self).name() == "slot"
    }

    fn has_id(&self, id: &SelectorString, case_sensitivity: CaseSensitivity) -> bool {
        match element!(self).id {
            Some(ref val) => case_sensitivity.eq(id.as_ref().as_bytes(), val.as_bytes()),
            None => false,
        }
    }

    fn has_class(&self, name: &SelectorString, case_sensitivity: CaseSensitivity) -> bool {
        element!(self)
            .classes
            .iter()
            .any(|class| case_sensitivity.eq(name.as_ref().as_bytes(), class.as_bytes()))
    }

    fn has_custom_state(&self, _name: &SelectorString) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        !self.children().any(|child| {
            child.node().is_element()
                || child
                    .node()
                    .as_text()
                    .is_some_and(|text| !text.text.is_empty())
        })
    }

    fn is_root(&self) -> bool {
        self.parent()
            .map_or(false, |parent| parent.node().is_document())
    }

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false
    }
}
