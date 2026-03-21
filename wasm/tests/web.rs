use wasm_bindgen_test::*;

use aam_rs_wasm::AamDocument;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen_test]
fn wasm_lookup_smoke() {
    let doc = AamDocument::new("host = localhost\nport = 8080").expect("parse should succeed");
    assert_eq!(doc.find_obj("host"), Some("localhost".to_string()));
}
