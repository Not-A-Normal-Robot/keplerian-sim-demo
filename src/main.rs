use three_d::{
    window::{Window, WindowSettings},
    // FrameOutput,
};

#[cfg(not(target_family = "wasm"))]
::smol_macros::main! {
    async fn main() {
        run().await;
    }
}

pub async fn run() {
    let window = {
        let res = Window::new(WindowSettings {
            title: "Keplerian Orbital Simulator Demo".into(),
            min_size: (1280, 720),
            ..Default::default()
        });
        match res {
            Ok(w) => w,
            Err(e) => {
                println!("Error when creating window: {e}");
                std::process::exit(1);
            }
        }
    };
    let context = window.gl();
}
