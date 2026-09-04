#![cfg(feature = "deterministic")]

use domjohnson::Document;

#[test]
fn deterministic_feature_preserves_attribute_order() {
    let mut dom = Document::parse(r#"<div z="3" a="1" m="2"></div>"#);
    let div = dom.select("div").get(0).unwrap();

    assert_eq!(
        dom.get(div)
            .unwrap()
            .as_element()
            .unwrap()
            .attrs()
            .collect::<Vec<_>>(),
        [("z", "3"), ("a", "1"), ("m", "2")]
    );

    dom.get_mut(div)
        .unwrap()
        .as_element_mut()
        .unwrap()
        .set_attr("b", "4");
    assert_eq!(
        dom.get(div)
            .unwrap()
            .as_element()
            .unwrap()
            .attrs()
            .collect::<Vec<_>>(),
        [("z", "3"), ("a", "1"), ("m", "2"), ("b", "4")]
    );
}
