use domjohnson::Document;

static HTML: &str = r#"
<html>
<body>
    <h1>Title</h1>
</body>
</html>
"#;

fn main() {
    let mut dom = Document::parse(HTML);

    let root = dom.select("html").get(0).unwrap();

    let h1 = dom.select("h1").get(0).expect("h1");
    println!("{:?}", dom[h1].as_element().unwrap());

    dom.remove(h1);

    let text = dom.create_text("Hello, World!");
    dom.append(dom.select("body").get(0).unwrap(), text);

    println!("{}", dom);

    let h1text = dom.children(h1).next().unwrap();

    dom[h1text].as_text_mut().unwrap().set_text("Hello Title!");

    for node in dom.orhpans().collect::<Vec<_>>() {
        println!("O1: {:?}", dom.inner_html(node));
        dom.delete(node);
    }

    let mut dom = Document::new_html5();

    let title_tag = dom.create_element("title");
    let title = dom.create_text("Page title");
    dom.append(title_tag, title);

    dom.append(dom.select("head").get(0).unwrap(), title_tag);

    println!("{dom}");
}
