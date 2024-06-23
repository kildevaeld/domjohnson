#[macro_use]
mod macros;

mod document;
mod element;
mod lock;
mod node_list;

pub use self::{document::JsDocument, element::JsElement, node_list::Children};

#[rquickjs::module(rename_vars = "camelCase", rename = "Module")]
pub mod domjohnson {

    pub use super::JsDocument as Document;

    #[rquickjs::function]
    pub fn parse(input: String) -> rquickjs::Result<Document> {
        let doc = domjohnson::Document::parse(&input);
        Ok(Document::from_doc(doc))
    }
}
