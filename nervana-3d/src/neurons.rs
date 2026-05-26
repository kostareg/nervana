//! The neurons that make up a blob's network, and their stable byte
//! representation used for hashing and color derivation.

use zerocopy::{Immutable, IntoBytes};

/// Number of sensory input neurons available to a network.
pub const INPUTS_N: usize = 4;
/// Number of hidden intermediate neurons that hold internal state.
pub const INTERMEDIATES_N: usize = 10;
/// Number of action output neurons (the x/y/z movement forces).
pub const OUTPUTS_N: usize = 3;

/// A single neuron, tagged by role and carrying its index within that role's
/// pool. Connections wire one neuron to another.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Neuron {
    /// A sensory input, indexed `0..INPUTS_N`.
    Input(usize),
    /// A hidden neuron holding internal state, indexed `0..INTERMEDIATES_N`.
    Intermediate(usize),
    /// An action output, indexed `0..OUTPUTS_N`.
    Output(usize),
}

impl Neuron {
    /// Returns the raw enum discriminant (which role the neuron plays).
    ///
    /// Safety:
    /// doc.rust-lang.org/stable/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
    fn get_discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    /// Returns the neuron's index within its role's pool.
    fn get_value(&self) -> usize {
        match self {
            Self::Input(n) => *n,
            Self::Intermediate(n) => *n,
            Self::Output(n) => *n,
        }
    }
}

/// A fixed-layout, byte-addressable view of a [`Neuron`], so a network can be
/// flattened to bytes deterministically regardless of enum padding.
#[allow(dead_code)] // fields read as raw bytes via zerocopy's as_bytes()
#[derive(Clone, Copy, Debug, Immutable, IntoBytes)]
pub struct NeuronBytes {
    discriminant: u8,
    _padding: [u8; 7],
    value: usize,
}

impl From<Neuron> for NeuronBytes {
    fn from(value: Neuron) -> Self {
        Self {
            discriminant: value.get_discriminant(),
            _padding: [0; 7],
            value: value.get_value(),
        }
    }
}
