use rand::Rng;
use serde::{Deserialize, Serialize};

// For now, just hard-coding the internal neurons. Also, max out multiplication
// by 10.

/// A source is either an internal neuron or a sensory neuron. They produce
/// floating-point values from -1 to 1 that represent various sensors or
/// internal calculations.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Source {
    // Internal:
    I0,
    I1,
    I2,
    I3,

    // Sensory:
    /// Outputs random number from -1 to 1.
    Random,
    /// Position % from center of x axis.
    Px,
    /// Position % from center of y axis.
    Py,
    // TODO: more!
}

/// A sink is either an internal neuron or a sensory neuron. They consume
/// floating-point values from -10 to 10 that represent the likelihood that an
/// action will be taken, or are used for internal calculations.
///
/// For example: a value of -8.98 for My means that there is an 89.8% chance of
/// moving down in the next step.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Sink {
    // Internal:
    I0,
    I1,
    I2,
    I3,

    // Action:
    /// Move x axis.
    Mx,
    /// Move y axis.
    My,
    // TODO: more!
}

impl Source {
    pub fn random_new() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..7) {
            0 => Self::I0,
            1 => Self::I1,
            2 => Self::I2,
            3 => Self::I3,
            4 => Self::Random,
            5 => Self::Px,
            _ => Self::Py,
        }
    }
}

impl Sink {
    pub fn random_new() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..6) {
            0 => Self::I0,
            1 => Self::I1,
            2 => Self::I2,
            3 => Self::I3,
            4 => Self::Mx,
            _ => Self::My,
        }
    }
}
