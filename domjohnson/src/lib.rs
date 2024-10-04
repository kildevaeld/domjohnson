mod document;
mod element;
mod error;
mod matcher;
mod node;
mod selection;

pub use self::{
    document::Document,
    element::NodeRef,
    matcher::{MatchScope, Matcher, Matches},
    selection::Selection,
};

pub use generational_indextree::NodeId;

pub use selectors::attr::CaseSensitivity;
