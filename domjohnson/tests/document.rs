use domjohnson::Document;

#[test]
fn parses_html5_structure_and_node_data() {
    let dom = Document::parse(
        r#"<!doctype html><title>Example</title><body><main id="app" class="shell wide"><!--note--><p>Hello &amp; 世界</p></main></body>"#,
    );

    assert_eq!(dom.select("html").len(), 1);
    assert_eq!(dom.select("head > title").len(), 1);
    assert_eq!(dom.select("body > main#app.shell.wide").len(), 1);

    let main = dom.select("main").get(0).unwrap();
    let element = dom.get(main).unwrap().as_element().unwrap();
    assert_eq!(element.name(), "main");
    assert_eq!(element.id(), Some("app"));
    assert!(element.has_class("shell", domjohnson::CaseSensitivity::CaseSensitive));
    assert!(element.has_class("SHELL", domjohnson::CaseSensitivity::AsciiCaseInsensitive));

    let children = dom.children(main).collect::<Vec<_>>();
    assert_eq!(children.len(), 2);
    assert!(dom.get(children[0]).unwrap().is_comment());
    assert!(dom.get(children[0]).unwrap().as_comment().is_some());
    assert!(dom.get(children[1]).unwrap().is_element());
    assert!(dom.get(children[1]).unwrap().as_text().is_none());

    let text = dom.text(main).map(|text| text.as_str()).collect::<Vec<_>>();
    assert_eq!(text, ["Hello & 世界"]);
}

#[test]
fn creates_a_complete_empty_html5_document() {
    let dom = Document::new_html5();

    assert_eq!(
        dom.to_string(),
        "<!DOCTYPE html><html><head></head><body></body></html>"
    );
    assert_eq!(dom.select("html > head").len(), 1);
    assert_eq!(dom.select("html > body").len(), 1);
    assert_eq!(dom.orhpans().count(), 0);
}

#[test]
fn serializes_inner_html_comments_text_and_void_elements() {
    let dom = Document::parse(
        r#"<main><!-- greeting --><p title="a &amp; b">5 &lt; 7 &amp; 8</p><br><script>if (a < b) c++;</script></main>"#,
    );
    let main = dom.select("main").get(0).unwrap();

    assert_eq!(
        dom.inner_html(main),
        r#"<!-- greeting --><p title="a &amp; b">5 &lt; 7 &amp; 8</p><br><script>if (a < b) c++;</script>"#
    );
    assert_eq!(
        dom.inner_html(dom.select("p").get(0).unwrap()),
        "5 &lt; 7 &amp; 8"
    );
}

#[test]
fn handles_templates_and_malformed_markup() {
    let dom = Document::parse(
        "<main><template><span class=inside>template</span></template><p><b>bold</main>",
    );

    assert_eq!(dom.select("template .inside").len(), 1);
    assert_eq!(dom.select("main p b").len(), 1);
    assert_eq!(
        dom.text(dom.select("main").get(0).unwrap())
            .map(|text| text.as_str())
            .collect::<Vec<_>>(),
        ["template", "bold"]
    );
}

#[test]
fn concatenates_adjacent_parser_text_tokens() {
    let dom = Document::parse("<p>one&amp;two&#33;</p>");
    let paragraph = dom.select("p").get(0).unwrap();
    let children = dom.children(paragraph).collect::<Vec<_>>();

    assert_eq!(children.len(), 1);
    assert_eq!(
        dom.get(children[0])
            .unwrap()
            .as_text()
            .unwrap()
            .text
            .as_str(),
        "one&two!"
    );
}
