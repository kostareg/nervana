use crate::blob::*;
use crate::neurons::*;

use ::rand::{self, Rng};
use bincode::Options;
use macroquad::prelude::*;
use rayon::prelude::*;

/// The simulation's population. Each generation will repopulate up to this
/// cap.
const POPULATION: usize = 200;

/// Steps in a single generation dictate the length of the individual
/// generation.
const STEPS_PER_GEN: usize = 300;

/// Number of generations to run. The simulation will freeze when it
/// reaches this cap.
const GENS: usize = 100;

/// The simulation environment.
pub struct Simulator {
    blobs: [Blob; POPULATION],
}

impl Simulator {
    /// Create a new simulation environment, filled with randomly generated
    /// blobs.
    pub fn random_new() -> Self {
        Self {
            blobs: std::array::from_fn(|_| Blob::random_new()),
        }
    }

    /// Runs the whole simulation: each generation steps the population, culls
    /// everyone on the left half, and repopulates from the survivors. Once the
    /// generation cap is hit it draws the final population in a loop.
    pub async fn run(&mut self) {
        let mut rng = rand::rng();
        let mut survived = POPULATION;
        for i in 0..GENS {
            self.run_generation(i, survived).await;

            // delete everyone on the left.
            // TODO: rename to kill method, make modular.
            let surviving_blobs: Vec<_> = self
                .blobs
                .clone()
                .into_iter()
                .filter(|blob| blob.x.is_sign_positive())
                .collect();
            survived = surviving_blobs.len();

            // use the remaining blobs to repopulate.
            self.blobs = std::array::from_fn(|_| {
                // pick a random survivor blob's genomes and copy it.
                let selected_blob = surviving_blobs[rng.random_range(0..surviving_blobs.len())];
                Blob::random_pos(selected_blob.genomes)
            });
        }

        loop {
            let sample = self.blobs[0];
            let config = bincode::DefaultOptions::new()
                .with_varint_encoding()
                .allow_trailing_bytes();
            let sample_code = format!(
                "{}",
                hex::encode(config.serialize(&sample.genomes).unwrap())
            );

            self.draw(GENS, survived, sample, sample_code).await;
            next_frame().await;
        }
    }

    /// Runs a single generation: steps the population `STEPS_PER_GEN` times,
    /// drawing a frame after each step. `i` is the generation number and
    /// `survived` the previous generation's survivor count, both for display.
    pub async fn run_generation(&mut self, i: usize, survived: usize) {
        for _ in 0..STEPS_PER_GEN {
            self.step();

            // sample and draw
            let sample = self.blobs[0];
            let config = bincode::DefaultOptions::new()
                .with_varint_encoding()
                .allow_trailing_bytes();
            let sample_code = format!(
                "{}",
                hex::encode(config.serialize(&sample.genomes).unwrap())
            );

            self.draw(i, survived, sample, sample_code).await;
            next_frame().await;
        }
    }

    /// Simulates one step in the environment.
    fn step(&mut self) {
        // Loop thru each blob and handle its neural network.
        self.blobs.par_iter_mut().for_each(|blob| {
            // TODO: might be smart to move this into genome + unprivatize.

            let random = rand::rng().random_range(-1. ..=1.);

            // Define all of the blob's accumulators for this step.
            let mut mx = 0.;
            let mut my = 0.;
            let mut i0 = 0.;
            let mut i1 = 0.;
            let mut i2 = 0.;
            let mut i3 = 0.;

            // For each genome, read the source and add its value to the
            // accumulator.
            for genome in blob.genomes {
                let source = match genome.source {
                    Source::Px => blob.x,
                    Source::Py => blob.y,
                    Source::Random => random,
                    Source::I0 => blob.internal_state.i0,
                    Source::I1 => blob.internal_state.i1,
                    Source::I2 => blob.internal_state.i2,
                    Source::I3 => blob.internal_state.i3,
                };

                let input_value = source * (genome.weight as f32);

                match genome.sink {
                    // assuming 128x128 board.
                    Sink::Mx => mx += input_value,
                    Sink::My => my += input_value,
                    Sink::I0 => i0 += input_value,
                    Sink::I1 => i1 += input_value,
                    Sink::I2 => i2 += input_value,
                    Sink::I3 => i3 += input_value,
                }
            }

            blob.x += Self::translate(blob.x, mx.tanh()) * (1. / 64.);
            blob.y += Self::translate(blob.y, my.tanh()) * (1. / 64.);
            blob.internal_state.i0 += i0.tanh().abs();
            blob.internal_state.i1 += i1.tanh();
            blob.internal_state.i2 += i2.tanh();
            blob.internal_state.i3 += i3.tanh();
        })
    }

    /// Returns either 0 or +/-1 based on probability as provided. Always
    /// 0 if passing borders.
    fn translate(position: f32, probability: f32) -> f32 {
        // If we want to go off the screen, return 0.
        if (position + (1. / 64.) * probability.signum()).abs() >= 1. {
            return 0.;
        }

        if rand::rng().random_range(0. ..1.) < probability.abs() {
            return 1. * probability.signum(); // direction
        } else {
            return 0.;
        }
    }

    /// Renders one frame: the kill region, the population, and a stats panel
    /// describing the simulation and the sampled `sample` blob. Also handles
    /// the pause (`p`) and quit (`q`) keys.
    async fn draw(&self, i: usize, survived: usize, sample: Blob, sample_code: String) {
        let fill = || {
            clear_background(BLACK);
            draw_text_ex(
                format!("Generation {}. Hold p to pause, press q to quit.", i).as_str(),
                612.,
                60.,
                TextParams::default(),
            );
            draw_text_ex(
                format!(
                    "{}% survival rate, killed {}.",
                    100. * (survived as f32) / (POPULATION as f32),
                    POPULATION - survived
                )
                .as_str(),
                612.,
                80.,
                TextParams::default(),
            );

            draw_text_ex(
                format!("Analyzing blob {}:", sample_code).as_str(),
                612.,
                120.,
                TextParams::default(),
            );
            draw_text_ex(
                format!("(x, y) = ({}, {})", sample.x, sample.y).as_str(),
                632.,
                140.,
                TextParams::default(),
            );
            draw_multiline_text_ex(
                format!("{:#?}", sample.genomes).as_str(),
                632.,
                160.,
                Some(1.),
                TextParams::default(),
            );
            draw_multiline_text_ex(
                format!("{:#?}", sample.internal_state).as_str(),
                888.,
                160.,
                Some(1.),
                TextParams::default(),
            );

            draw_text_ex("Constants:", 1200., 60., TextParams::default());
            draw_text_ex(
                format!("Population cap of {}.", POPULATION).as_str(),
                1220.,
                80.,
                TextParams::default(),
            );
            draw_text_ex(
                format!("Running {} steps/generation.", STEPS_PER_GEN).as_str(),
                1220.,
                100.,
                TextParams::default(),
            );
            draw_text_ex(
                format!("Running up to {} generations.", GENS).as_str(),
                1220.,
                120.,
                TextParams::default(),
            );
            // elimination rect
            draw_rectangle(50., 50., 256., 512. + 4., RED);

            let scale = 4.0; // Scale factor to zoom in (1x1 becomes 4x4 pixels)

            let mut f = true;
            for blob in &self.blobs {
                // TODO: color for biodiversity?
                let color = if f { BLUE } else { WHITE };
                let (screen_x, screen_y) = self.to_screen_coords(blob.x, blob.y);
                draw_rectangle(screen_x, screen_y, scale, scale, color);
                f = false;
            }
        };

        if is_key_down(KeyCode::P) {
            while !is_key_released(KeyCode::P) {
                fill();
                next_frame().await;
            }
        }

        if is_key_pressed(KeyCode::Q) {
            std::process::exit(0);
        }

        fill();
    }

    /// Maps a blob's `-1..=1` board position to on-screen pixel coordinates.
    fn to_screen_coords(&self, x: f32, y: f32) -> (f32, f32) {
        let screen_x = 50. + ((x + 1.0) * 64.0) * 4.0; // Scale coordinates
        let screen_y = 50. + ((y + 1.0) * 64.0) * 4.0;
        (screen_x, screen_y)
    }
}
