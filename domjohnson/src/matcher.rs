use cssparser::ParseError;
use generational_indextree::{Arena, NodeId};
use html5ever::{LocalName, Namespace};
use selectors::{
    matching,
    parser::{self, SelectorList, SelectorParseErrorKind},
    visitor, Element,
};
use std::{collections::HashSet, fmt};

use crate::{element::NodeRef, node::Node};

/// CSS selector.
#[derive(Clone, Debug)]
pub struct Matcher {
    selector_list: SelectorList<InnerSelector>,
}

impl Matcher {
    /// Greate a new CSS matcher.
    pub fn new(sel: &str) -> Result<Self, ParseError<SelectorParseErrorKind>> {
        let mut input = cssparser::ParserInput::new(sel);
        let mut parser = cssparser::Parser::new(&mut input);
        selectors::parser::SelectorList::parse(&InnerSelectorParser, &mut parser)
            .map(|selector_list| Matcher { selector_list })
    }

    pub(crate) fn match_element<E>(&self, element: &E) -> bool
    where
        E: Element<Impl = InnerSelector>,
    {
        let mut ctx = matching::MatchingContext::new(
            matching::MatchingMode::Normal,
            None,
            None,
            matching::QuirksMode::NoQuirks,
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

                if self.matcher.match_element(&node_ref) {
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
    type ExtraMatchingData = String;
    type AttrValue = String;
    type Identifier = LocalName;
    type ClassName = LocalName;
    type PartName = LocalName;
    type LocalName = LocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = LocalName;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = Namespace;

    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
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

    fn has_zero_specificity(&self) -> bool {
        false
    }
}

impl parser::Visit for NonTSPseudoClass {
    type Impl = InnerSelector;

    fn visit<V>(&self, _visitor: &mut V) -> bool
    where
        V: visitor::SelectorVisitor<Impl = Self::Impl>,
    {
        true
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
