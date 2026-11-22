use crate::types::{Gna2Operation, Gna2Tensor};

use super::{FullyConnectedAffineLayer, LayerDefinition};

/// Fluent builder for constructing and compiling a GNA Model on a given device.
pub struct GnaModelBuilder {
    pub(super) layers: Vec<LayerDefinition>,
}

impl GnaModelBuilder {
    /// Create an empty model builder.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a Fully Connected Affine layer (Dense / Matrix Multiplication + Bias + Activation).
    ///
    /// # Arguments
    /// - `inputs`: Input tensor (`[W x B]` or `[N x W]`)
    /// - `outputs`: Output tensor (`[H x B]`)
    /// - `weights`: Weights matrix (`[H x W]`)
    /// - `biases`: Biases vector (`[H]` or `[H x B]`)
    /// - `activation`: Optional activation function PWL tensor (pass `None` for disabled/identity)
    pub fn add_fully_connected_affine(
        mut self,
        inputs: Gna2Tensor,
        outputs: Gna2Tensor,
        weights: Gna2Tensor,
        biases: Gna2Tensor,
        activation: Option<Gna2Tensor>,
    ) -> Self {
        let act = activation.unwrap_or_else(Gna2Tensor::disabled);
        self.layers
            .push(LayerDefinition::FullyConnectedAffine(Box::new(
                FullyConnectedAffineLayer {
                    input: inputs,
                    output: outputs,
                    weights,
                    biases,
                    activation: act,
                },
            )));
        self
    }

    /// Add a custom pre-initialized Gna2Operation to the network.
    pub fn add_operation(mut self, operation: Gna2Operation) -> Self {
        self.layers.push(LayerDefinition::Raw(operation));
        self
    }
}

impl Default for GnaModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
