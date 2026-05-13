pub mod blob;
pub mod genome;
pub mod neurons;
pub mod simulate;

use macroquad::prelude::*;

// TODO: consider setting up like guides, each step changes the code a bit and
// has its own directory.

#[macroquad::main(window_conf)]
async fn main() {
    simulate::Simulator::random_new().run().await;
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Evolution".to_owned(),
        window_width: 512 + 4,
        window_height: 512 + 4,
        fullscreen: true,
        ..Default::default()
    }
}
