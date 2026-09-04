use cssparser::ParseError;
use generational_indextree::{Arena, NodeId};
use precomputed_hash::PrecomputedHash;
use selectors::{
    matching,
    parser::{self, SelectorList, SelectorParseErrorKind},
    Element,
};
use smol_str::SmolStr;
use std::{
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
};

use crate::{element::NodeRef, node::Node};

/// CSS selector.
#[derive(Clone, Debug)]
pub struct Matcher {
    selector_list: SelectorList<InnerSelector>,
}

impl Matcher {
    /// Greate a new CSS matcher.
    pub fn new<'i>(sel: &'i str) -> Result<Self, ParseError<'i, SelectorParseErrorKind<'i>>> {
        let mut input = cssparser::ParserInput::new(sel);
        let mut parser = cssparser::Parser::new(&mut input);
        selectors::parser::SelectorList::parse(
            &InnerSelectorParser,
            &mut parser,
            parser::ParseRelative::No,
        )
        .map(|selector_list| Matcher { selector_list })
    }

    pub(crate) fn match_element<E>(&self, element: &E) -> bool
    where
        E: Element<Impl = InnerSelector>,
    {
        let mut caches = matching::SelectorCaches::default();
        let mut ctx = matching::MatchingContext::new(
            matching::MatchingMode::Normal,
            None,
            &mut caches,
            matching::QuirksMode::NoQuirks,
            matching::NeedsSelectorFlags::No,
            matching::MatchingForInvalidation::No,
        );

        matching::matches_selector_list(&self.selector_list, element, &mut ctx)
    }
}

#[derive(Debug, Clone)]
pub struct Matches<'a, T> {
    arena: &'a Arena<Node>,
    roots: Vec<T>,
    nodes: Vec<T>,
    matcher: Matcher,
    set: HashSet<NodeId>,
    match_scope: MatchScope,
}

/// Telling a `matches` if we want to skip the roots.
#[derive(Debug, Clone)]
pub enum MatchScope {
    IncludeNode,
    ChildrenOnly,
}

impl<'a, T> Matches<'a, T> {
    pub fn from_one(
        arena: &'a Arena<Node>,
        node: T,
        matcher: Matcher,
        match_scope: MatchScope,
    ) -> Self {
        Self {
            arena,
            roots: vec![node],
            nodes: vec![],
            matcher,
            set: HashSet::new(),
            match_scope,
        }
    }

    pub fn from_list<I: Iterator<Item = T>>(
        arena: &'a Arena<Node>,
        nodes: I,
        matcher: Matcher,
        match_scope: MatchScope,
    ) -> Self {
        Self {
            arena,
            roots: nodes.collect(),
            nodes: vec![],
            matcher,
            set: HashSet::new(),
            match_scope,
        }
    }
}

impl<'a> Iterator for Matches<'a, NodeId> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.nodes.is_empty() {
                if self.roots.is_empty() {
                    return None;
                }

                let root = self.roots.remove(0);

                match self.match_scope {
                    MatchScope::IncludeNode => self.nodes.insert(0, root),
                    MatchScope::ChildrenOnly => {
                        for child in root.reverse_children(&self.arena) {
                            self.nodes.insert(0, child);
                        }
                    }
                }
            }

            while !self.nodes.is_empty() {
                let node = self.nodes.remove(0);

                for node in node.reverse_children(&self.arena) {
                    self.nodes.insert(0, node);
                }

                let node_ref = NodeRef::new(&self.arena, node);

                if node_ref.node().is_element() && self.matcher.match_element(&node_ref) {
                    if self.set.contains(&node) {
                        continue;
                    }

                    self.set.insert(node);
                    return Some(node);
                }
            }
        }
    }
}

pub(crate) struct InnerSelectorParser;

impl<'i> parser::Parser<'i> for InnerSelectorParser {
    type Impl = InnerSelector;
    type Error = parser::SelectorParseErrorKind<'i>;
}

#[derive(Debug, Clone)]
pub struct InnerSelector;

impl parser::SelectorImpl for InnerSelector {
    type ExtraMatchingData<'a> = std::marker::PhantomData<&'a ()>;
    type AttrValue = SelectorAttrValue;
    type Identifier = SelectorString;
    type LocalName = SelectorString;
    type NamespaceUrl = SelectorString;
    type NamespacePrefix = SelectorString;
    type BorrowedLocalName = str;
    type BorrowedNamespaceUrl = str;

    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct SelectorString(SmolStr);

impl AsRef<str> for SelectorString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for SelectorString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SelectorString {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl cssparser::ToCss for SelectorString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        cssparser::serialize_identifier(&self.0, dest)
    }
}

impl PrecomputedHash for SelectorString {
    fn precomputed_hash(&self) -> u32 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as u32
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectorAttrValue(SmolStr);

impl AsRef<str> for SelectorAttrValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SelectorAttrValue {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl cssparser::ToCss for SelectorAttrValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        cssparser::serialize_string(&self.0, dest)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct NonTSPseudoClass;

impl parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = InnerSelector;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }
}

impl cssparser::ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PseudoElement;

impl parser::PseudoElement for PseudoElement {
    type Impl = InnerSelector;
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}
