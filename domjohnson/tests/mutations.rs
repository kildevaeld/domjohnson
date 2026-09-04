use domjohnson::{CaseSensitivity, Document};

#[test]
fn creates_appends_and_reparents_nodes() {
    let mut dom = Document::new_html5();
    let body = dom.select("body").get(0).unwrap();
    let first = dom.create_element("section");
    let second = dom.create_element("aside");
    let text = dom.create_text("hello");
    let comment = dom.create_comment("note");

    assert_eq!(dom.orhpans().count(), 4);
    assert!(dom.get(first).unwrap().is_element());
    assert!(dom.get(text).unwrap().is_text());
    assert!(dom.get(comment).unwrap().is_comment());

    dom.append(body, first);
    dom.append(body, second);
    dom.append(first, text);
    dom.append(first, comment);
    assert_eq!(dom.children(body).collect::<Vec<_>>(), [first, second]);
    assert_eq!(dom.children(first).collect::<Vec<_>>(), [text, comment]);
    assert_eq!(
        dom.text(first)
            .map(|text| text.as_str())
            .collect::<Vec<_>>(),
        ["hello"]
    );

    dom.append(second, text);
    assert_eq!(dom.children(first).collect::<Vec<_>>(), [comment]);
    assert_eq!(dom.children(second).collect::<Vec<_>>(), [text]);
}

#[test]
fn mutates_text_classes_and_attributes_consistently() {
    let mut dom = Document::parse(r#"<div id="old" class="one two" data-old="yes">before</div>"#);
    let div = dom.select("div").get(0).unwrap();
    let text = dom.children(div).next().unwrap();

    dom.get_mut(text)
        .unwrap()
        .as_text_mut()
        .unwrap()
        .set_text("after & more");

    {
        let element = dom.get_mut(div).unwrap().as_element_mut().unwrap();
        element.append_class("three");
        element.remove_class("one");
        element.set_attr("id", "new");
        element.set_attr("data-new", "value");
        element.remove_attr("data-old");
    }

    let element = dom.get(div).unwrap().as_element().unwrap();
    assert_eq!(element.id(), Some("new"));
    assert!(!element.has_class("one", CaseSensitivity::CaseSensitive));
    assert!(element.has_class("two", CaseSensitivity::CaseSensitive));
    assert!(element.has_class("three", CaseSensitivity::CaseSensitive));
    assert_eq!(element.attr("data-new").map(String::as_str), Some("value"));
    assert_eq!(element.attr("data-old"), None);

    assert!(dom.select("#old").is_empty());
    assert_eq!(dom.select("#new.three[data-new=value]").get(0), Some(div));
    assert_eq!(dom.inner_html(div), "after &amp; more");

    let html = dom.to_string();
    assert!(html.contains("id=\"new\""));
    assert!(html.contains("class=\"two three\"") || html.contains("class=\"three two\""));
    assert!(html.contains("data-new=\"value\""));
    assert!(!html.contains("data-old"));
}

#[test]
fn setting_and_removing_id_and_class_attributes_updates_selectors() {
    let mut dom = Document::parse(r#"<div id="before" class="old"></div>"#);
    let div = dom.select("div").get(0).unwrap();

    {
        let element = dom.get_mut(div).unwrap().as_element_mut().unwrap();
        element.set_attr("class", "new other new");
        element.set_attr("id", "after");
    }
    assert!(dom.select("#before, .old").is_empty());
    assert_eq!(dom.select("#after.new.other").get(0), Some(div));
    assert_eq!(
        dom.get(div)
            .unwrap()
            .as_element()
            .unwrap()
            .classes()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from(["new", "other"])
    );

    {
        let element = dom.get_mut(div).unwrap().as_element_mut().unwrap();
        element.remove_attr("id");
        element.remove_attr("class");
    }
    assert!(dom.select("#after, .new, .other").is_empty());
    assert_eq!(dom.get(div).unwrap().as_element().unwrap().id(), None);
    assert_eq!(
        dom.get(div)
            .unwrap()
            .as_element()
            .unwrap()
            .classes()
            .count(),
        0
    );
}

#[test]
fn create_element_normalizes_html_tag_names() {
    let mut dom = Document::new_html5();
    let body = dom.select("body").get(0).unwrap();
    let div = dom.create_element("DiV");
    dom.append(body, div);

    assert_eq!(dom.get(div).unwrap().as_element().unwrap().name(), "div");
    assert_eq!(dom.select("div").get(0), Some(div));
}

#[test]
fn remove_detaches_a_subtree_that_can_be_reattached() {
    let mut dom = Document::parse("<main><section><span>text</span></section></main>");
    let main = dom.select("main").get(0).unwrap();
    let section = dom.select("section").get(0).unwrap();
    let span = dom.select("span").get(0).unwrap();

    dom.remove(section);
    assert!(dom.select("section").is_empty());
    assert_eq!(dom.orhpans().collect::<Vec<_>>(), [section]);
    assert!(dom.get(section).is_some());
    assert!(dom.get(span).is_some());
    assert_eq!(dom.children(section).collect::<Vec<_>>(), [span]);

    dom.append(main, section);
    assert_eq!(dom.select("main > section > span").get(0), Some(span));
    assert_eq!(dom.orhpans().count(), 0);
}

#[test]
fn delete_removes_a_node_and_promotes_its_children() {
    let mut dom = Document::parse("<main><section><b>one</b><i>two</i></section></main>");
    let main = dom.select("main").get(0).unwrap();
    let section = dom.select("section").get(0).unwrap();
    let bold = dom.select("b").get(0).unwrap();
    let italic = dom.select("i").get(0).unwrap();

    dom.delete(section);

    assert!(dom.get(section).is_none());
    assert_eq!(dom.children(main).collect::<Vec<_>>(), [bold, italic]);
    assert_eq!(
        dom.text(main).map(|text| text.as_str()).collect::<Vec<_>>(),
        ["one", "two"]
    );
}

#[test]
fn remove_orphans_deletes_entire_orphaned_subtrees() {
    let mut dom = Document::new_html5();
    let orphan = dom.create_element("div");
    let child = dom.create_element("span");
    let grandchild = dom.create_text("text");
    dom.append(orphan, child);
    dom.append(child, grandchild);

    assert_eq!(dom.orhpans().collect::<Vec<_>>(), [orphan]);
    dom.remove_orphans();

    assert_eq!(dom.orhpans().count(), 0);
    assert!(dom.get(orphan).is_none());
    assert!(dom.get(child).is_none());
    assert!(dom.get(grandchild).is_none());
}

#[test]
fn traversal_visits_every_node_on_entry_and_exit() {
    let dom = Document::parse("<main><p>one</p><p>two</p></main>");
    let main = dom.select("main").get(0).unwrap();

    assert_eq!(dom.traverse(main).count(), 10);
}
