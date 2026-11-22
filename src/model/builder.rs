use crate::types::model::{Gna2Operation, Gna2Tensor};

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

    /// Add an Element-Wise Affine layer to the network.
    pub fn add_element_wise_affine(
        mut self,
        inputs: Gna2Tensor,
        outputs: Gna2Tensor,
        weights: Gna2Tensor,
        biases: Gna2Tensor,
        activation: Option<Gna2Tensor>,
    ) -> Self {
        let act = activation.unwrap_or_else(Gna2Tensor::disabled);
        self.layers.push(LayerDefinition::Generic {
            op_type: crate::types::model::Gna2OperationType::ElementWiseAffine,
            tensors: Box::new([inputs, outputs, weights, biases, act]),
        });
        self
    }

    /// Add a Transposition layer to the network.
    pub fn add_transposition(mut self, inputs: Gna2Tensor, outputs: Gna2Tensor) -> Self {
        self.layers.push(LayerDefinition::Generic {
            op_type: crate::types::model::Gna2OperationType::Transposition,
            tensors: Box::new([inputs, outputs]),
        });
        self
    }

    /// Add a Copy layer to the network.
    pub fn add_copy(mut self, inputs: Gna2Tensor, outputs: Gna2Tensor) -> Self {
        self.layers.push(LayerDefinition::Generic {
            op_type: crate::types::model::Gna2OperationType::Copy,
            tensors: Box::new([inputs, outputs]),
        });
        self
    }

    /// Add a Convolution layer to the network.
    pub fn add_convolution(
        mut self,
        inputs: Gna2Tensor,
        outputs: Gna2Tensor,
        filters: Gna2Tensor,
        biases: Gna2Tensor,
        activation: Option<Gna2Tensor>,
    ) -> Self {
        let act = activation.unwrap_or_else(Gna2Tensor::disabled);
        self.layers.push(LayerDefinition::Generic {
            op_type: crate::types::model::Gna2OperationType::Convolution,
            tensors: Box::new([inputs, outputs, filters, biases, act]),
        });
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
