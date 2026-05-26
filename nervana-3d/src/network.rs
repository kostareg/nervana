use crate::neurons::*;

use rand::prelude::*;
use zerocopy::{Immutable, IntoBytes};

/// One weighted connection in the network, carrying a signal from a source
/// neuron to a sink neuron.
#[derive(Clone, Copy, Debug)]
pub struct Connection {
    /// The neuron the signal is read from (an input or intermediate).
    pub from: Neuron,
    /// The neuron the signal is written to (an intermediate or output).
    pub to: Neuron,
    /// Multiplier applied to the source value, in `-10.0..10.0`.
    pub weight: f32,
}

impl Connection {
    /// Builds a random connection: a source drawn from the inputs and
    /// intermediates, a sink drawn from the intermediates and outputs, and a
    /// random weight.
    pub fn random() -> Self {
        let inp_rng = || rand::random_range(0..INPUTS_N);
        let int_rng = || rand::random_range(0..INTERMEDIATES_N);
        let out_rng = || rand::random_range(0..OUTPUTS_N);

        let from = *[Neuron::Input(inp_rng()), Neuron::Intermediate(int_rng())].choose(&mut rand::rng()).unwrap();
        let to = *[Neuron::Intermediate(int_rng()), Neuron::Output(out_rng())].choose(&mut rand::rng()).unwrap();

        Self {
            from,
            to,
            weight: rand::random_range(-10. .. 10.),
        }
    }
}

/// A fixed-layout, byte-addressable view of a [`Connection`], used to flatten a
/// network into a deterministic byte string.
#[allow(dead_code)] // fields read as raw bytes via zerocopy's as_bytes()
#[derive(Clone, Copy, Debug, Immutable, IntoBytes)]
struct ConnectionBytes {
    from: NeuronBytes,
    to: NeuronBytes,
    weight: f32,
    _padding: [u8; 4],
}

impl From<Connection> for ConnectionBytes {
    fn from(value: Connection) -> Self {
        Self {
            from: value.from.into(),
            to: value.to.into(),
            weight: value.weight,
            _padding: [0; 4],
        }
    }
}

/// A blob's brain: a fixed-size set of connections evaluated each step to turn
/// sensory inputs into movement forces.
#[derive(Clone, Copy, Debug)]
pub struct NeuralNetwork {
    /// The eight connections that define this network's behavior.
    pub connections: [Connection; 8],
}

impl NeuralNetwork {
    /// Builds a network of eight random connections.
    pub fn random() -> Self {
        Self {
            connections: std::array::from_fn(|_| Connection::random()),
        }
    }

    /// Flattens the whole network into a deterministic byte string, used as an
    /// identity for measuring population diversity.
    pub fn all_bytes(&self) -> Vec<u8> {
        let mut v = Vec::<u8>::with_capacity(64);

        for connection in self.connections {
            let connection_bytes: ConnectionBytes = connection.into();
            let bytes = connection_bytes.as_bytes();
            v.extend_from_slice(bytes);
        }

        v
    }

    /// Derives a stable RGB color from the network's bytes, so visually
    /// similar blobs share genetics. Returned as `(r, g, b)`.
    pub fn color(&self) -> (u8, u8, u8) {
        let mut sum_r = 0;
        let mut sum_g = 0;
        let mut sum_b = 0;

        for connection in self.connections {
            let connection_bytes: ConnectionBytes = connection.into();
            let bytes = connection_bytes.as_bytes();

            // we only care about 0, 8, 16, 24, 32-35
            sum_r += (bytes[0] as u16 + bytes[8] as u16 + bytes[16] as u16) / 3;
            sum_g += (bytes[24] as u16 + bytes[32] as u16 + bytes[33] as u16) / 3;
            sum_b += (bytes[34] as u16 + bytes[35] as u16) / 2;
        }

        sum_r /= 8;
        sum_g /= 8;
        sum_b /= 8;

        (sum_r as u8, sum_g as u8, sum_b as u8)
    }
}
