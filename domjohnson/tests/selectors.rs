use domjohnson::{Document, Matcher, Selection};

fn fixture() -> Document {
    Document::parse(
        r#"
        <main id="app" class="root shell" data-role="page">
          <section id="first" class="card featured" data-kind="primary" data-tags="alpha beta">
            <h1>Title</h1>
            <p class="copy">First</p>
            <p class="copy empty"></p>
          </section>
          <section id="second" class="card" data-kind="secondary">
            <p class="copy">Second</p>
          </section>
          <aside></aside>
        </main>
        "#,
    )
}

fn ids(dom: &Document, selector: &str) -> Vec<String> {
    dom.select(selector)
        .iter()
        .map(|id| {
            dom.get(*id)
                .unwrap()
                .as_element()
                .unwrap()
                .id()
                .unwrap_or("")
                .to_owned()
        })
        .collect()
}

#[test]
fn matches_type_id_class_universal_and_selector_lists() {
    let dom = fixture();

    assert_eq!(dom.select("section").len(), 2);
    assert_eq!(ids(&dom, "#first"), ["first"]);
    assert_eq!(ids(&dom, "section.card.featured"), ["first"]);
    assert_eq!(dom.select("main > *").len(), 3);
    assert!(dom
        .select("*")
        .iter()
        .all(|id| dom.get(*id).unwrap().is_element()));
    assert_eq!(ids(&dom, "#first, #second, #first"), ["first", "second"]);
}

#[test]
fn matches_attribute_operators() {
    let dom = fixture();

    assert_eq!(ids(&dom, "[data-kind]"), ["first", "second"]);
    assert_eq!(ids(&dom, "[data-kind=primary]"), ["first"]);
    assert_eq!(ids(&dom, "[data-tags~=beta]"), ["first"]);
    assert_eq!(ids(&dom, "[data-kind|=primary]"), ["first"]);
    assert_eq!(ids(&dom, "[data-kind^=pri]"), ["first"]);
    assert_eq!(ids(&dom, "[data-kind$=ary]"), ["first", "second"]);
    assert_eq!(ids(&dom, "[data-kind*=cond]"), ["second"]);
    assert_eq!(ids(&dom, "[DATA-KIND=PRIMARY i]"), ["first"]);
}

#[test]
fn matches_combinators_and_structural_pseudo_classes() {
    let dom = fixture();

    assert_eq!(dom.select("main .copy").len(), 3);
    assert_eq!(dom.select("section > .copy").len(), 3);
    assert_eq!(dom.select("h1 + p").len(), 1);
    assert_eq!(dom.select("h1 ~ p").len(), 2);
    assert_eq!(ids(&dom, "main > section:first-child"), ["first"]);
    assert_eq!(ids(&dom, "main > section:nth-child(2)"), ["second"]);
    assert_eq!(ids(&dom, "section:not(.featured)"), ["second"]);
    assert_eq!(dom.select("p:empty").len(), 1);
    assert_eq!(dom.select(":root").len(), 1);
}

#[test]
fn empty_text_nodes_do_not_prevent_matching_empty() {
    let mut dom = Document::new_html5();
    let body = dom.select("body").get(0).unwrap();
    let div = dom.create_element("div");
    let empty_text = dom.create_text("");
    dom.append(div, empty_text);
    dom.append(body, div);

    assert_eq!(dom.select("div:empty").get(0), Some(div));
}

#[test]
fn select_from_excludes_the_starting_node() {
    let dom = fixture();
    let first = dom.select("#first").get(0).unwrap();

    assert!(dom.select_from(first, "#first").is_empty());
    assert_eq!(dom.select_from(first, ".copy").len(), 2);
}

#[test]
fn nested_selection_includes_roots_and_deduplicates_overlaps() {
    let dom = fixture();
    let sections = dom.select("section");

    assert_eq!(
        ids_from_selection(&dom, sections.select(&dom, ".card")),
        ["first", "second"]
    );

    let overlapping = Selection::from(vec![
        dom.select("#app").get(0).unwrap(),
        dom.select("#first").get(0).unwrap(),
    ]);
    assert_eq!(
        ids_from_selection(&dom, overlapping.select(&dom, ".card")),
        ["first", "second"]
    );
}

fn ids_from_selection(dom: &Document, selection: Selection) -> Vec<String> {
    selection
        .into_iter()
        .map(|id| {
            dom.get(id)
                .unwrap()
                .as_element()
                .unwrap()
                .id()
                .unwrap_or("")
                .to_owned()
        })
        .collect()
}

#[test]
fn matcher_rejects_invalid_or_unsupported_selectors() {
    assert!(Matcher::new("").is_err());
    assert!(Matcher::new("div >").is_err());
    assert!(Matcher::new(":hover").is_err());
    assert!(Matcher::new("::before").is_err());
}

#[test]
#[should_panic(expected = "invalid css selector")]
fn document_selection_panics_for_invalid_selectors() {
    fixture().select("div >");
}
