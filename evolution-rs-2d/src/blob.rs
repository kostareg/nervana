use crate::genome::*;

use rand::Rng;

/// A blob is a 1x1 creature.
///
/// The x and y positions are measured as a percentage from the center of the
/// board, in order to be easier to fit in a range of -1 to 1.
/// `(x, y) = (-1, -1)` is the bottom left of the board.
#[derive(Debug, Copy, Clone)]
pub struct Blob {
    pub x: f32,
    pub y: f32,
    pub genomes: Genomes,
    pub internal_state: InternalState,
}

#[derive(Debug, Copy, Clone)]
pub struct InternalState {
    pub i0: f32,
    pub i1: f32,
    pub i2: f32,
    pub i3: f32,
}

impl Blob {
    /// Create a randomly generated new blob. Only used in 0th generation.
    pub fn random_new() -> Self {
        let mut rng = rand::rng();

        Self {
            x: rng.random_range(-1. ..=1.),
            y: rng.random_range(-1. ..=1.),
            genomes: std::array::from_fn(|_| Genome::random_new()),
            internal_state: InternalState::random_new(),
        }
    }

    /// Create a blob in a random position with an inherited genome. Used in
    /// all non-zero generations.
    pub fn random_pos(genomes: Genomes) -> Self {
        let mut rng = rand::rng();

        Self {
            x: rng.random_range(-1. ..=1.),
            y: rng.random_range(-1. ..=1.),
            genomes,
            internal_state: InternalState::random_new(),
        }
    }
}

impl InternalState {
    pub fn random_new() -> Self {
        let mut rng = rand::rng();

        Self {
            i0: rng.random_range(0. ..=1.),
            i1: rng.random_range(-1. ..=1.),
            i2: rng.random_range(-1. ..=1.),
            i3: rng.random_range(-1. ..=1.),
        }
    }
}
