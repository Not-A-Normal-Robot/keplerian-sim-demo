// use glow::Context as GlowContext;
use three_d::core::Context as ThreeDContext;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext};

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
fn start() {
    web_panic_handler::init_panic_handler();

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

    todo!();
}

mod web_panic_handler {
    use std::{
        panic::PanicHookInfo,
        sync::{Mutex, MutexGuard},
    };

    use wasm_bindgen::JsValue;

    // We DO NOT want any allocations pushing the memory
    // usage past the edge.
    const PANIC_BUFFER_LEN: usize = 16384;
    type PanicBuffer = [u8; PANIC_BUFFER_LEN];
    type PanicBufferGuard<'a> = MutexGuard<'a, PanicBuffer>;
    static PANIC_BUFFER: Mutex<PanicBuffer> = Mutex::new([0; PANIC_BUFFER_LEN]);

    enum PanicDisplayError {
        GetWindowError,
        GetDocumentError,
        GetBodyError,
        AttachDialogError(JsValue),
        CreateDialogError(JsValue),
    }

    impl PanicDisplayError {
        /// Returns the new index, or the first unused byte.
        #[must_use]
        fn write(&self, panic_buffer: &mut PanicBufferGuard<'_>, index: usize) -> usize {
            match self {
                Self::GetWindowError => write_bytes(panic_buffer, index, b"error getting window"),
                Self::GetDocumentError => {
                    write_bytes(panic_buffer, index, b"error getting document")
                }
                Self::GetBodyError => write_bytes(panic_buffer, index, b"error getting body"),
                Self::AttachDialogError(_) => {
                    write_bytes(panic_buffer, index, b"error attaching dialog")
                }
                Self::CreateDialogError(_) => {
                    write_bytes(panic_buffer, index, b"error creating dialog")
                }
            }
        }
    }

    enum PanicAlertError {
        GetWindowError,
        AlertError(JsValue),
    }

    impl PanicAlertError {
        /// Returns the new index, or the first unused byte.
        #[must_use]
        fn write(&self, panic_buffer: &mut PanicBufferGuard<'_>, index: usize) -> usize {
            match self {
                Self::GetWindowError => write_bytes(panic_buffer, index, b"error getting window"),
                Self::AlertError(_) => write_bytes(panic_buffer, index, b"error calling alert()"),
            }
        }
    }

    pub(super) fn init_panic_handler() {
        std::panic::set_hook(Box::new(handle_panic));
    }

    fn handle_panic(info: &PanicHookInfo<'_>) {
        let mut panic_buffer = match PANIC_BUFFER.lock() {
            Ok(l) => l,
            Err(p) => p.into_inner(),
        };
        *panic_buffer = [0; PANIC_BUFFER_LEN];

        let display_res = display_panic(info, &mut panic_buffer);
        *panic_buffer = [0; PANIC_BUFFER_LEN];

        if let Err(e) = display_res {
            let mut index = write_bytes(
                &mut panic_buffer,
                0,
                b"failed to display panic info in GUI: ",
            );
            index = e.write(&mut panic_buffer, index);
            let message = buf_to_str(&mut panic_buffer, 0, index);
            let js_message = JsValue::from_str(message);
            *panic_buffer = [0; PANIC_BUFFER_LEN];

            let additional_info = match e {
                PanicDisplayError::GetWindowError => None,
                PanicDisplayError::GetDocumentError => None,
                PanicDisplayError::GetBodyError => None,
                PanicDisplayError::AttachDialogError(v) => Some(v),
                PanicDisplayError::CreateDialogError(v) => Some(v),
            };

            match additional_info {
                Some(val) => {
                    let len = write_bytes(&mut panic_buffer, 0, b"more info available:");
                    let message = buf_to_str(&mut panic_buffer, 0, len);
                    let js_message_2 = JsValue::from_str(message);
                    web_sys::console::error_3(&js_message, &js_message_2, &val)
                }
                None => {
                    web_sys::console::error_1(&js_message);
                }
            }

            let alert_res = display_alert(info, &mut panic_buffer);
            *panic_buffer = [0; PANIC_BUFFER_LEN];

            if let Err(e) = alert_res {
                let mut index = write_bytes(
                    &mut panic_buffer,
                    0,
                    b"failed to display panic info in alert: ",
                );
                index = e.write(&mut panic_buffer, index);
                let message = buf_to_str(&mut panic_buffer, 0, index);
                let js_message = JsValue::from_str(message);
                *panic_buffer = [0; PANIC_BUFFER_LEN];

                let additional_info = match e {
                    PanicAlertError::GetWindowError => None,
                    PanicAlertError::AlertError(v) => Some(v),
                };

                match additional_info {
                    Some(val) => {
                        let len = write_bytes(&mut panic_buffer, 0, b"more info available:");
                        let message = buf_to_str(&mut panic_buffer, 0, len);
                        let js_message_2 = JsValue::from_str(message);
                        *panic_buffer = [0; PANIC_BUFFER_LEN];

                        web_sys::console::error_3(&js_message, &js_message_2, &val);
                    }
                    None => web_sys::console::error_1(&js_message),
                }
            }
        }

        let _ = log_to_console(info, &mut panic_buffer);
    }

    /// Returns the new index, or the first unused byte.
    #[must_use]
    fn write_bytes(panic_buffer: &mut PanicBufferGuard<'_>, index: usize, bytes: &[u8]) -> usize {
        if index >= PANIC_BUFFER_LEN {
            return PANIC_BUFFER_LEN;
        }

        let mut buf_idx = index;
        for original_idx in 0..bytes.len() {
            match panic_buffer.get_mut(buf_idx) {
                Some(byte) => *byte = bytes[original_idx],
                None => return PANIC_BUFFER_LEN,
            }

            buf_idx = match buf_idx.checked_add(1) {
                Some(i) => i,
                None => return buf_idx,
            };
        }

        PANIC_BUFFER_LEN.min(buf_idx)
    }

    /// Returns the new index, or the first unused byte.
    #[must_use]
    fn write_num(panic_buffer: &mut PanicBufferGuard<'_>, index: usize, mut num: u32) -> usize {
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

    /// Returns the new index, or the first unused byte.
    #[must_use]
    fn write_panic_info(
        panic_buffer: &mut PanicBufferGuard<'_>,
        mut index: usize,
        info: &PanicHookInfo<'_>,
    ) -> usize {
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

        index
    }

    fn buf_to_str<'a>(
        panic_buffer: &'a mut PanicBufferGuard<'_>,
        start_idx: usize,
        len: usize,
    ) -> &'a str {
        let end = start_idx.saturating_add(len);
        let end = end.min(PANIC_BUFFER_LEN);
        for _ in 0..=len {
            let slice = &panic_buffer[start_idx..end];
            let res = core::str::from_utf8(&slice);

            match res {
                Ok(_) => break,
                Err(e) => {
                    // The fact that from_utf8 failed means that
                    // there is at least one byte of invalid
                    // utf-8 within the bounds of the string.
                    // This will never index out of bounds.

                    // Extreme scenario: Invalid UTF-8 byte at last byte
                    // of 8-byte buffer, starting at index 4
                    // 0 1 2 3 [4 5 6 7]
                    // start_idx = 4
                    // len = 4
                    // end = 8
                    // slice = [4 5 6 7]
                    // e.valid_up_to() = 3
                    // first_invalid_idx = 4 + 3 = 7
                    // index 7 is right at the very end!
                    let first_invalid_idx = start_idx + e.valid_up_to();
                    panic_buffer[first_invalid_idx] = b'?';
                }
            }
        }

        let slice = &panic_buffer[start_idx..end];

        match core::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => "[unrecoverable]",
        }
    }

    fn display_panic(
        info: &PanicHookInfo<'_>,
        panic_buffer: &mut PanicBufferGuard<'_>,
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
            p.set_text_content(Some(
                "A catastrophic error occurred and the program cannot continue. \
                Below are details on the error, which you can report to the developer. \
                Opening the console may reveal additional details.",
            ));
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

        index = write_panic_info(panic_buffer, index, info);

        index = index.min(PANIC_BUFFER_LEN);

        let message = buf_to_str(panic_buffer, 0, index);

        if let Some(pre) = pre {
            pre.set_text_content(Some(message));
        } else {
            dialog.set_text_content(Some(message));
        }

        if let Ok(button) = document.create_element("button") {
            button.set_text_content(Some("Dismiss"));
            if button
                .set_attribute("onclick", "this.parentElement.close()")
                .is_ok()
            {
                let _ = dialog.append_child(&button);
            }
        }

        Ok(())
    }

    fn display_alert(
        info: &PanicHookInfo<'_>,
        panic_buffer: &mut PanicBufferGuard<'_>,
    ) -> Result<(), PanicAlertError> {
        let window = web_sys::window().ok_or(PanicAlertError::GetWindowError)?;

        let _ = window.alert_with_message(
            "==[ Panic! ]==\n\
            A catastrophic error occurred and the program cannot continue. \
            Dismiss this prompt to see details on the error, which you can \
            report to the developer. Opening the console may reveal \
            additional details.",
        );

        let len = write_panic_info(panic_buffer, 0, info);

        let message = buf_to_str(panic_buffer, 0, len);

        window
            .alert_with_message(message)
            .map_err(|e| PanicAlertError::AlertError(e))
    }

    fn log_to_console(info: &PanicHookInfo<'_>, panic_buffer: &mut PanicBufferGuard<'_>) {
        let len = write_panic_info(panic_buffer, 0, info);
        let message = buf_to_str(panic_buffer, 0, len);
        let js_str = JsValue::from_str(message);
        web_sys::console::error_1(&js_str);
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
