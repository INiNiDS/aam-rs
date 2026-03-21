use wasm_bindgen_test::*;

use aam_wasm::AamDocument;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen_test]
fn wasm_lookup_smoke() {
    let doc = AamDocument::new("host = localhost\nport = 8080").expect("parse should succeed");
    assert_eq!(doc.find_obj("host"), Some("localhost".to_string()));
}

#[wasm_bindgen_test]
fn wasm_reverse_lookup_smoke() {
    let doc = AamDocument::new("host = localhost").expect("parse should succeed");
    assert_eq!(doc.find_obj("localhost"), Some("host".to_string()));
}

#[wasm_bindgen_test]
fn wasm_merge_and_find_key() {
    let mut doc = AamDocument::new("role = user").expect("parse should succeed");
    doc.merge("role = admin\nname = alice")
        .expect("merge should succeed");
    assert_eq!(doc.find_obj("role"), Some("admin".to_string()));
    assert_eq!(doc.find_key("alice"), Some("name".to_string()));
}

#[wasm_bindgen_test]
fn wasm_find_deep_chain() {
    let doc = AamDocument::new("a = b\nb = c\nc = terminal").expect("parse should succeed");
    assert_eq!(doc.find_deep("a"), Some("terminal".to_string()));
}
