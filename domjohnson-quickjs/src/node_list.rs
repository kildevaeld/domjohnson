use domjohnson::NodeId;
use locket::LockApi;
use rquickjs::{atom::PredefinedAtom, class::Trace, function::MutFn, Class, Ctx, Function, Object};

use crate::{element::JsElement, lock::Locket};

#[rquickjs::class]
pub struct Children {
    pub dom: Locket<domjohnson::Document>,
    pub node: NodeId,
}

impl<'js> Trace<'js> for Children {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Children {
    pub fn iter<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let res = Object::new(ctx.clone())?;

        let mut nodes = {
            let dom = self.dom.read().unwrap();
            let children = dom.children(self.node);
            children.collect::<Vec<_>>().into_iter()
        };

        let dom = self.dom.clone();

        res.set(
            PredefinedAtom::Next,
            Function::new(
                ctx,
                MutFn::new(move |ctx: Ctx<'js>| -> rquickjs::Result<Object<'js>> {
                    let next = nodes.next();
                    let res = Object::new(ctx.clone())?;
                    res.set(PredefinedAtom::Done, next.is_none())?;

                    if let Some(next) = next {
                        res.set(
                            PredefinedAtom::Value,
                            Class::instance(
                                ctx.clone(),
                                JsElement {
                                    id: next,
                                    dom: dom.clone(),
                                },
                            )?,
                        )?;
                    }

                    Ok(res)
                }),
            ),
        )?;
        Ok(res)
    }
}
