use three_d::core::Context as ThreeDContext;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext};

#[cfg(target_family = "wasm")]
mod web_heartbeat;
#[cfg(target_family = "wasm")]
mod web_panic_handler;

#[wasm_bindgen(start)]
fn start() {
    #[cfg(target_family = "wasm")]
    {
        web_panic_handler::init_panic_handler();
        web_heartbeat::start_beating();
    }

    let window = web_sys::window().expect("global `window` should exist");
    let document = window.document().expect("`window` should have `document`");

    clear_dom(document.clone());

    let canvas = init_canvas(document);

    let _gl_context = canvas
        .get_context("webgl2")
        .expect("WebGL should be supported")
        .expect("WebGL should be supported")
        .dyn_into::<WebGl2RenderingContext>()
        .expect("cast to WebGl2RenderingContext should work");
}

fn clear_dom(document: Document) {
    let body = document.body().expect("`document` should have `body`");
    body.set_inner_html("");
}

fn init_canvas(document: Document) -> HtmlCanvasElement {
    let canvas = document
        .create_element("canvas")
        .expect("canvas element should be created")
        .dyn_into::<HtmlCanvasElement>()
        .expect("Failed to cast to HtmlCanvasElement");

    document
        .body()
        .expect("body should exist")
        .append_child(&canvas)
        .expect("canvas element should be appendable");

    canvas
}

pub fn render(context: ThreeDContext) {
    dbg!(context);
    todo!();
}
