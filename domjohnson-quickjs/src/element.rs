use crate::lock::Locket;
use domjohnson::NodeId;
use locket::LockApi;
use rquickjs::{class::Trace, Class, Ctx};

#[rquickjs::class(rename = "Element")]
pub struct JsElement {
    pub dom: Locket<domjohnson::Document>,
    pub id: NodeId,
}

impl<'js> Trace<'js> for JsElement {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsElement {
    pub fn kind(&self, ctx: Ctx<'_>) -> Result<(), rquickjs::Error> {
        let Some(el) = self.dom.read().unwrap().get(self.id) else {
            fail!(ctx, "Element does no longer exists")
        };

        Ok(())
    }

    pub fn append_child<'js>(&self, child: Class<'js, JsElement>) -> rquickjs::Result<()> {
        let mut dom = self.dom.write().unwrap();
        dom.append(self.id, child.try_borrow()?.id);
        Ok(())
    }

    pub fn remove(&self) -> rquickjs::Result<()> {
        let mut dom = self.dom.write().unwrap();
        dom.remove(self.id);
        Ok(())
    }

    #[qjs(get, rename = "innerHTML")]
    pub fn inner_html(&self) -> rquickjs::Result<String> {
        let dom = self.dom.read().unwrap();
        Ok(dom.inner_html(self.id))
    }
}
