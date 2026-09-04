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

pub use trae::NodeId;

pub use selectors::attr::CaseSensitivity;
