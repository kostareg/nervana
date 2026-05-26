use crate::network::*;
use crate::neurons::*;

use bevy::prelude::*;

/// A single blob. Its network maps the force currently acting on it, plus
/// some noise, to the force it should apply next, and `internal_state` carries
/// the intermediate neurons' values between steps.
#[derive(Component, Clone, Copy, Debug)]
pub struct Blob {
    /// The network that decides how this blob moves.
    pub network: NeuralNetwork,
    /// Persistent values of the intermediate neurons, one per hidden neuron.
    pub internal_state: [f32; INTERMEDIATES_N],
}

impl Blob {
    /// Creates a blob with a random network and randomized internal state.
    pub fn random() -> Self {
        Self {
            network: NeuralNetwork::random(),
            internal_state: std::array::from_fn(|_| rand::random_range(0. .. 0.5)),
        }
    }

    /// Advances the blob one tick: feeds the current `force` through the
    /// network, updates internal state, and returns the new clamped force to
    /// apply.
    pub fn step(&mut self, force: Vec3) -> Vec3 {
        let inputs = [force.x, force.y, force.z, rand::random_range(0. .. f32::MAX)];
        let mut result = force;

        for connection in self.network.connections {
            let value = connection.weight * match connection.from {
                Neuron::Input(n) => inputs[n],
                Neuron::Intermediate(n) => self.internal_state[n],
                Neuron::Output(_) => unimplemented!(),
            };

            match connection.to {
                Neuron::Input(_) => unimplemented!(),
                Neuron::Intermediate(n) => self.internal_state[n] = value,
                Neuron::Output(n) => match n {
                    0 => result.x = value / 100.,
                    1 => result.y = value / 100.,
                    2 => result.z = value / 100.,
                    _ => unimplemented!(),
                },
            }
        }

        result.clamp(Vec3 {
            x: -0.0005,
            y: -0.0005,
            z: -0.0005,
        }, Vec3 {
            x: 0.0005,
            y: 0.0005,
            z: 0.0005,
        })
    }
}
