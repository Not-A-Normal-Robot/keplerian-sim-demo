#![allow(special_module_name)]
#![cfg(target_family = "wasm")]

use wasm_bindgen::prelude::*;
use web_sys::Document;

mod main;
mod web_heartbeat;
mod web_panic_handler;

#[wasm_bindgen(start)]
fn start() {
    web_panic_handler::init_panic_handler();
    web_heartbeat::start_beating();

    let window = web_sys::window().expect("global `window` should exist");
    let document = window.document().expect("`window` should have `document`");

    clear_dom(&document);
    init_canvas(&document);

    main::main();
}

fn clear_dom(document: &Document) {
    let body = document.body().expect("`document` should have `body`");
    body.set_inner_html("");
}

fn init_canvas(document: &Document) {
    let canvas = document
        .create_element("canvas")
        .expect("canvas creation should work");
    document
        .body()
        .expect("body should exist")
        .append_child(&canvas)
        .expect("canvas attachmint should work");
}
