use rquickjs::{class::Trace, qjs};

use crate::{element::JsElement, lock::Locket};
use locket::LockApi as _;

#[rquickjs::class(rename = "Document")]
pub struct JsDocument {
    pub inner: Locket<domjohnson::Document>,
}

impl<'js> Trace<'js> for JsDocument {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsDocument {
    pub fn from_doc(doc: domjohnson::Document) -> JsDocument {
        JsDocument {
            inner: <Locket<domjohnson::Document> as locket::LockApi<domjohnson::Document>>::new(
                doc,
            ),
        }
    }
}

#[rquickjs::methods]
impl JsDocument {
    #[qjs(constructor)]
    pub fn new() -> JsDocument {
        JsDocument {
            inner: <Locket<domjohnson::Document> as locket::LockApi<domjohnson::Document>>::new(
                domjohnson::Document::new_html5(),
            ),
        }
    }

    #[qjs(rename = "createElement")]
    pub fn create_element(&self, tag: String) -> rquickjs::Result<JsElement> {
        let mut dom = self.inner.write().unwrap();
        Ok(JsElement {
            dom: self.inner.clone(),
            id: dom.create_element(&tag),
        })
    }

    #[qjs(get)]
    pub fn body(&self) -> Option<JsElement> {
        let dom = self.inner.read().unwrap();
        dom.select("body").get(0).map(|id| JsElement {
            id,
            dom: self.inner.clone(),
        })
    }

    #[qjs(rename = "querySelector")]
    pub fn query_selector(&self, query: String) -> Option<JsElement> {
        let dom = self.inner.read().expect("dom");
        dom.select(&query).get(0).map(|id| JsElement {
            dom: self.inner.clone(),
            id,
        })
    }
}
