use domjohnson::{Document, NodeId, Selection};

#[test]
fn selection_exposes_collection_operations_in_order() {
    let dom = Document::parse("<ul><li id=a></li><li id=b></li><li id=c></li></ul>");
    let selection = dom.select("li");

    assert_eq!(selection.len(), 3);
    assert!(!selection.is_empty());
    assert_eq!(selection.get(0), dom.select("#a").get(0));
    assert_eq!(selection.get(2), dom.select("#c").get(0));
    assert_eq!(selection.get(3), None);

    let from_iter = selection.iter().copied().collect::<Vec<_>>();
    let from_borrowed_into_iter = (&selection).into_iter().copied().collect::<Vec<_>>();
    assert_eq!(from_iter, from_borrowed_into_iter);

    let consumed = selection.clone().into_iter().collect::<Vec<_>>();
    assert_eq!(consumed, from_iter);
}

#[test]
fn selection_and_vec_conversions_preserve_contents() {
    let dom = Document::parse("<div></div><div></div>");
    let nodes = dom.select("div").into_iter().collect::<Vec<NodeId>>();
    let with_duplicate = vec![nodes[0], nodes[1], nodes[0]];

    let selection = Selection::from(with_duplicate.clone());
    let round_trip: Vec<NodeId> = selection.into();
    assert_eq!(round_trip, with_duplicate);

    let empty = Selection::from(Vec::new());
    assert!(empty.is_empty());
    assert_eq!(Vec::<NodeId>::from(empty), Vec::new());
}
