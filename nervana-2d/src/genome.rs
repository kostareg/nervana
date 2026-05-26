use crate::neurons::*;

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A list of genomes creates the genetic makeup of a blob.
pub type Genomes = [Genome; 8]; // for a base: 8 genomes.

/// A genome describes one single neural pathway in the blob's neural network.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub source: Source,
    pub sink: Sink,
    pub weight: i8, // from -10 to 10 for now.
}

impl Genome {
    /// Builds a random pathway: a random source and sink wired together with a
    /// random weight in `-10..=10`.
    pub fn random_new() -> Self {
        let mut rng = rand::rng();

        Self {
            source: Source::random_new(),
            sink: Sink::random_new(),
            weight: rng.random_range(-10..=10),
        }
    }
}
