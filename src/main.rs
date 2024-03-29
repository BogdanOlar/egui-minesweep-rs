pub mod app;

use std::env;
use eframe::{NativeOptions, epaint::Vec2};
use app::{GameDifficulty, MinesweepRsApp};

fn main() {
    // DEBUG 
    env::set_var("RUST_BACKTRACE", "full");
    
    // DEBUG
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();    
    
    // FIXME: Solve auto resizing
    let size_x = 38.0;
    let size_y = 44.0;

    let options = NativeOptions {
        initial_window_size: Some(
            // Allocate the maximum native window size, corresponding to the HARD game difficulty
            Vec2::new(
                size_x * GameDifficulty::HARD.width as f32,
                size_y * GameDifficulty::HARD.height as f32
            )
        ),
        resizable: false,
        // FIXME: App crashes (on Fedora, with Wayland) when run with `options.run_and_return = true;` and in a `loop`
        run_and_return: true,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "Egui Minesweep-Rs",
        options,
        Box::new(|cc| Box::new(MinesweepRsApp::default().with_context(cc))),
    );

    // TODO: figure out if we can read App `storage` in order to figure out if we should exit or apply new configs

}
