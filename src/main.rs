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

    window.render_loop(|_f| {
        // keplerian_sim_demo::render(f.context);
        todo!();
        // FrameOutput {
        //     ..Default::default()
        // }
    });
}
