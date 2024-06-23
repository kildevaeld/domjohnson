use rquickjs::{Class, Context, Function, Module, Runtime};

fn print(msg: String) {
    println!("{msg}");
}

fn main() -> rquickjs::Result<()> {
    let runtime = Runtime::new()?;
    let ctx = Context::full(&runtime)?;

    ctx.with(|ctx| {
        let global = ctx.globals();
        global
            .set(
                "print",
                Function::new(ctx.clone(), print)
                    .unwrap()
                    .with_name("print")
                    .unwrap(),
            )
            .unwrap();
        let _ = Module::declare_def::<domjohnson_quickjs::js_domjohnson, _>(ctx, "domjohnson")?;
        rquickjs::Result::Ok(())
    })?;

    ctx.with(|ctx| {
        Module::evaluate(
            ctx,
            "main",
            r#"
            import { parse } from 'domjohnson';

            const dom = parse("<html><body>Hello, World</body></html>")

            print(dom.body.innerHTML)
        
        "#,
        )?;
        rquickjs::Result::Ok(())
    })?;

    Ok(())
}
