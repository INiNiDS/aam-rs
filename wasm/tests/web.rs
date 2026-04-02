use wasm_bindgen_test::*;

use aam_wasm::AamDocument;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen_test]
fn wasm_lookup_smoke() {
    let doc = AamDocument::new("host = localhost\nport = 8080").expect("parse should succeed");
    assert!(!doc.find("host").is_null());
}

#[wasm_bindgen_test]
fn wasm_reverse_lookup_smoke() {
    let doc = AamDocument::new("host = localhost").expect("parse should succeed");
    assert!(!doc.find("localhost").is_null());
}

#[wasm_bindgen_test]
fn wasm_find_key_and_value_lookup() {
    let doc = AamDocument::new("role = admin\nname = alice").expect("parse should succeed");
    assert!(!doc.find("role").is_null());
    assert!(!doc.find("alice").is_null());
}

#[wasm_bindgen_test]
fn wasm_deep_search_by_pattern() {
    let doc = AamDocument::new("a = b\nb = c\nc = terminal").expect("parse should succeed");
    assert!(!doc.deep_search("a").is_null());
}
