# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`domjohnson` is a Rust HTML DOM library: parse HTML5 into an arena-based tree, query it with CSS selectors (jQuery/scraper-style `.select("div.foo")`), and mutate/serialize it back to HTML. `domjohnson-quickjs` is a QuickJS (rquickjs) binding that exposes a small `document.querySelector`-style JS API backed by the Rust `Document`.

This is a two-crate Cargo workspace:

- `domjohnson` — the core DOM/HTML library, no JS involved.
- `domjohnson-quickjs` — JS bindings on top of `domjohnson`, depends on it via a path dependency.

## Commands

Build/test everything from the workspace root:

```
cargo build --workspace
cargo test --workspace
```

Work on a single crate:

```
cargo build -p domjohnson
cargo test -p domjohnson
cargo test -p domjohnson --test selection      # one integration test file
cargo test -p domjohnson selection_exposes     # one test by name (substring match)
```

The `deterministic` feature on `domjohnson` switches `Attributes`/attribute iteration from `HashMap` to `indexmap::IndexMap` for order-preserving (de)serialization — enable it when order-sensitive round-tripping matters:

```
cargo test -p domjohnson --features deterministic
```

Run the example binaries (useful for manually exercising the API end-to-end):

```
cargo run -p domjohnson --example dom
cargo run -p domjohnson-quickjs --example jsdom
```

`domjohnson-quickjs` has a `send` feature that swaps its internal `Locket` alias from `Rc<RefCell<_>>` to `Arc<Mutex<_>>` (see Architecture below) — build with `--features send` when the JS bindings need to cross thread boundaries.

Integration tests for the core crate live in `domjohnson/tests/*.rs` (one file per concern: `deterministic.rs`, `document.rs`, `mutations.rs`, `selection.rs`, `selectors.rs`); unit tests are inline (`#[cfg(test)]`) in individual `src/` modules, e.g. `domjohnson/src/element/node_ref.rs`.

## Architecture (domjohnson core)

**Storage**: `Document` (`src/document/mod.rs`) wraps a `generational_indextree::Arena<Node>` plus a `root: NodeId`. All nodes — document root, doctype, elements, text, comments, processing instructions, and the `Fragment` marker used inside `<template>` contents — are variants of the `Node` enum (`src/node/mod.rs`). There is no separate node type per kind; code branches on `Node::as_element()`, `.is_text()`, etc. `NodeId` (re-exported from `generational_indextree`) is the stable, `Copy`able handle used everywhere instead of references, so nodes can be freely moved/removed without lifetime headaches — `Document` methods take/return `NodeId` and index via `Index`/`IndexMut`.

**Parsing**: `Document::parse` feeds `html5ever::parse_document` a custom `TreeSink` implementation, `DocumentBuilder` (`src/document/sink.rs`). This is where html5ever's tree-construction callbacks (`create_element`, `append`, `append_before_sibling`, text-node coalescing, `<template>` → `Fragment` handling, etc.) get translated into arena operations. If HTML parsing/serialization behaves unexpectedly, start here.

**CSS selectors**: built on the `selectors` + `cssparser` crates, not a hand-rolled matcher. `src/matcher.rs` defines the `InnerSelector` `SelectorImpl` (with `SelectorString`/`SelectorAttrValue` newtypes wrapping `SmolStr`, and no-op `NonTSPseudoClass`/`PseudoElement` since pseudo-classes/elements aren't supported). `src/element/selector.rs` implements the `selectors::Element` trait for `NodeRef`, which is what lets `matching::matches_selector_list` walk the arena tree (parent/sibling/child lookups, id/class/attr matching, `:empty`/`:root`, etc.) directly against `Node` data. `Matches` (in `matcher.rs`) is the lazy iterator that walks matching descendants of one or more root nodes — `MatchScope::IncludeNode` vs `ChildrenOnly` controls whether the root itself can match. `Document::select`/`select_from` and `Selection::select` (`src/selection/mod.rs`, a thin ordered/deduped `Vec<NodeId>` wrapper) are both built on top of `Matches`.

**Read vs write access**: `NodeRef<'a>` (`src/element/node_ref.rs`) is the immutable, `Copy`able view (tree + id) used for traversal, serialization (`html()`/`inner_html()` via `html5ever::serialize`), and selector matching. `NodeMut<'a>` (`src/element/node_mut.rs`) is the mutable counterpart. Both are thin wrappers recreated on demand from `(&Arena<Node>, NodeId)` rather than long-lived cursors.

**Deterministic serialization**: `Element::attrs`'s type (`Attributes`) is feature-gated — `IndexMap` under `deterministic`, `HashMap` otherwise — so attribute order in serialized output is only guaranteed when that feature is enabled. `remove_attr` has matching `#[cfg]` branches (`shift_remove` vs `remove`); keep both in sync when touching attribute storage.

## Architecture (domjohnson-quickjs bindings)

Exposes a `domjohnson` ES module (via `#[rquickjs::module]` in `src/lib.rs`) with a `parse(input: String) -> Document` function. `JsDocument` (`src/document.rs`) and `JsElement` (`src/element.rs`) are `#[rquickjs::class]` wrappers holding a shared handle to the underlying `domjohnson::Document` plus (for elements) a `NodeId`. The shared handle type is `Locket<T>`, an alias defined in `src/lock.rs`: `Rc<RefCell<T>>` by default, `Arc<Mutex<T>>` under the `send` feature — this is what lets multiple JS-side `Element`/`Document` objects mutate the same Rust tree. JS method names are mapped to DOM conventions with `#[qjs(rename = "...")]` (e.g. `createElement`, `querySelector`, `innerHTML`); `Children` (`src/node_list.rs`) implements a manual JS iterator protocol (`next`/`done`/`value` via `PredefinedAtom`) over a node's children. The `fail!` macro (`src/macros.rs`) is the standard way to throw a JS exception from a method body.

This binding layer is intentionally partial — it mirrors only the subset of `domjohnson`'s API that's been wired up (e.g. `NodeList::item` and the free-standing `NodeList` struct in `node_list.rs` are currently unused scaffolding). When adding JS-exposed functionality, prefer extending existing classes over introducing new wrapper types unless there's a clear DOM-API reason to.

## External dependencies of note

- `trae` and `locket` are pulled from `git` (kildevaeld/trae-rs, kildevaeld/locket-rs) rather than crates.io — expect `cargo update` to fetch from those repos, and check them if a build fails on a fresh checkout.
- `generational-indextree`, `html5ever`, `selectors`, and `cssparser` are the load-bearing parsing/tree/selector dependencies; version bumps to these are the most likely source of breaking API changes in `src/document/sink.rs`, `src/matcher.rs`, and `src/element/selector.rs`.
