// use glow::Context as GlowContext;
use three_d::core::Context as ThreeDContext;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext};

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn start() {
    web_panic_handler::init_panic_handler();

    let window = web_sys::window().expect("global `window` should exist");
    let document = window.document().expect("`window` should have `document`");

    clear_dom(document.clone());

    panic!("Hello!!!");
    let canvas = init_canvas(document);

    let _gl_context = canvas
        .get_context("webgl2")
        .expect("WebGL should be supported")
        .expect("WebGL should be supported")
        .dyn_into::<WebGl2RenderingContext>()
        .expect("cast to WebGl2RenderingContext should work");

    // todo!();
}

mod web_panic_handler {
    use std::panic::PanicHookInfo;

    use wasm_bindgen::JsValue;

    const PANIC_BUFFER_LEN: usize = 16384;
    static mut PANIC_BUFFER: [u8; PANIC_BUFFER_LEN] = [0; PANIC_BUFFER_LEN];

    enum PanicDisplayError {
        GetWindowError,
        GetDocumentError,
        GetBodyError,
        AttachDialogError(JsValue),
        CreateDialogError(JsValue),
    }

    pub(super) fn init_panic_handler() {
        std::panic::set_hook(Box::new(handle_panic));
    }

    fn handle_panic(info: &PanicHookInfo<'_>) {
        let panic_buffer = unsafe { &mut PANIC_BUFFER };
        *panic_buffer = [0; PANIC_BUFFER_LEN];

        let _ = display_panic(info, panic_buffer);
        *panic_buffer = [0; PANIC_BUFFER_LEN];

        let _ = display_alert(info, panic_buffer);
        *panic_buffer = [0; PANIC_BUFFER_LEN];

        let _ = log_to_console(info, panic_buffer);
    }

    /// Returns the new index, or the first unused byte.
    #[must_use]
    fn write_bytes(panic_buffer: &mut [u8; PANIC_BUFFER_LEN], index: usize, bytes: &[u8]) -> usize {
        if index >= PANIC_BUFFER_LEN {
            return PANIC_BUFFER_LEN;
        }

        let mut buf_idx = index;
        for original_idx in 0..bytes.len() {
            buf_idx = match buf_idx.checked_add(original_idx) {
                Some(i) => i,
                None => return buf_idx,
            };

            match panic_buffer.get_mut(buf_idx) {
                Some(byte) => *byte = bytes[original_idx],
                None => return PANIC_BUFFER_LEN,
            }

            buf_idx += 1;
        }

        PANIC_BUFFER_LEN.min(buf_idx)
    }

    /// Returns the new index, or the first unused byte.
    #[must_use]
    fn write_num(panic_buffer: &mut [u8; PANIC_BUFFER_LEN], index: usize, mut num: u32) -> usize {
        if index >= PANIC_BUFFER_LEN {
            return PANIC_BUFFER_LEN;
        }

        if num == 0 {
            if let Some(byte) = panic_buffer.get_mut(index) {
                *byte = b'0';
                return index + 1;
            }
            return PANIC_BUFFER_LEN;
        }

        let mut temp = num;
        let mut digit_count = 0;
        while temp > 0 {
            temp /= 10;
            digit_count += 1;
        }

        // Write digits from right to left
        let mut write_index = index + digit_count;
        while num > 0 {
            write_index -= 1;
            if let Some(byte) = panic_buffer.get_mut(write_index) {
                *byte = b'0' + (num % 10) as u8;
            }
            num /= 10;
        }

        PANIC_BUFFER_LEN.min(index + digit_count)
    }

    fn display_panic(
        info: &PanicHookInfo<'_>,
        panic_buffer: &mut [u8; PANIC_BUFFER_LEN],
    ) -> Result<(), PanicDisplayError> {
        let window = web_sys::window().ok_or(PanicDisplayError::GetWindowError)?;
        let document = window
            .document()
            .ok_or(PanicDisplayError::GetDocumentError)?;

        let dialog = document
            .create_element("dialog")
            .map_err(|e| PanicDisplayError::CreateDialogError(e))?;

        let _ = dialog.set_attribute("open", "true");

        let body = document.body().ok_or(PanicDisplayError::GetBodyError)?;

        body.append_child(&dialog)
            .map_err(|e| PanicDisplayError::AttachDialogError(e))?;

        // We don't care too much if this fails
        if let Ok(h1) = document.create_element("h1") {
            h1.set_text_content(Some("Panic!"));
            let _ = dialog.append_child(&h1);
        }

        // We don't care too much if this fails
        if let Ok(p) = document.create_element("p") {
            p.set_text_content(Some("A catastrophic error occurred and the program cannot continue. Below are details on the error, which you can report to the developer."));
            let _ = dialog.append_child(&p);
        }

        let pre = {
            let pre = document.create_element("pre").ok();
            if let Some(pre) = pre {
                dialog.append_child(&pre).ok().map(|_| pre)
            } else {
                None
            }
        };

        let mut index = 0;

        index = write_bytes(panic_buffer, index, b"panicked at ");

        match info.location() {
            Some(l) => {
                index = write_bytes(panic_buffer, index, l.file().as_bytes());
                index = write_bytes(panic_buffer, index, b":");
                index = write_num(panic_buffer, index, l.line());
                index = write_bytes(panic_buffer, index, b":");
                index = write_num(panic_buffer, index, l.column());
            }
            None => {
                index = write_bytes(panic_buffer, index, b"?");
            }
        }

        let payload = info.payload();
        if let Some(s) = payload.downcast_ref::<&str>() {
            index = write_bytes(panic_buffer, index, b":\n");
            index = write_bytes(panic_buffer, index, s.as_bytes());
        } else if let Some(s) = payload.downcast_ref::<String>() {
            index = write_bytes(panic_buffer, index, b":\n");
            index = write_bytes(panic_buffer, index, s.as_bytes());
        }

        index = index.min(PANIC_BUFFER_LEN);

        // Clean up all non-utf8 chars
        loop {
            let slice = &panic_buffer[0..index];
            let res = core::str::from_utf8(&slice);

            match res {
                Ok(_) => break,
                Err(e) => {
                    panic_buffer[e.valid_up_to() + 1] = b'?';
                }
            }
        }

        let slice = &panic_buffer[0..index];
        let message = unsafe { core::str::from_utf8_unchecked(&slice) };

        if let Some(pre) = pre {
            pre.set_text_content(Some(message));
        } else {
            dialog.set_text_content(Some(message));
        }

        Ok(())
    }

    fn display_alert(
        _info: &PanicHookInfo<'_>,
        _panic_buffer: &mut [u8; PANIC_BUFFER_LEN],
    ) -> Result<(), &'static str> {
        // TODO
        Ok(())
    }

    fn log_to_console(
        _info: &PanicHookInfo<'_>,
        _panic_buffer: &mut [u8; PANIC_BUFFER_LEN],
    ) -> Result<(), &'static str> {
        // TODO
        Ok(())
    }
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
        .append_child(&canvas)
        .expect("canvas element should be appendable");

    canvas
}

pub fn render(context: ThreeDContext) {
    dbg!(context);
    todo!();
}
