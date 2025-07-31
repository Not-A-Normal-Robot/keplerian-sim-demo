use three_d::{
    window::{Window, WindowSettings},
    // FrameOutput,
};

fn main() {
    let window = {
        let res = Window::new(WindowSettings {
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

    // keplerian_sim_demo::render(context);
    todo!();
}
