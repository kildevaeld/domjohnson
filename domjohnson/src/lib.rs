mod document;
mod element;
mod error;
mod matcher;
pub mod node;
mod selection;

pub use self::{
    document::Document,
    element::NodeRef,
    matcher::{MatchScope, Matcher, Matches},
    node::Node,
    selection::Selection,
};

pub use trae::NodeId;

pub use selectors::attr::CaseSensitivity;
