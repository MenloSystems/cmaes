//! Types related to initializing a [CMAESState]. See [CMAESOptions] for full documentation.

use nalgebra::DVector;

use crate::{CMAESState, ObjectiveFunction, PlotOptions};

/// A builder for [`CMAESState`]. Used to adjust parameters of the algorithm to each particular
/// problem.
///
/// # Examples
///
/// ```
/// # use nalgebra::DVector;
/// # use cmaes::CMAESOptions;
/// let function = |x: &DVector<f64>| x.magnitude();
/// let dim = 3;
/// let cmaes_state = CMAESOptions::new(dim)
///     .initial_mean(vec![2.0; dim])
///     .initial_step_size(5.0)
///     .build(function)
///     .unwrap();
/// ```
#[derive(Clone)]
pub struct CMAESOptions {
    /// Number of dimensions to search (`N`).
    pub dimensions: usize,
    /// Initial mean of the search distribution. This should be set to a first guess at the
    /// solution. Default value is the origin.
    pub initial_mean: DVector<f64>,
    /// Initial step size of the search distribution (`sigma0`). This should be set to a first guess
    /// at how far the solution is from the initial mean. Default value is `0.5`.
    pub initial_step_size: f64,
    /// Number of points to generate each generation (`lambda`). Default value is
    /// `4 + floor(3 * ln(dimensions))`.
    ///
    /// A larger population size will increase the robustness of the algorithm and help avoid local optima,
    /// but will lead to a slower convergence rate. Conversely, a lower value will reduce the robustness of
    /// the algorithm but will lead to a higher convergence rate. Generally, the population size should only
    /// be increased from the default value. It may be useful to restart the algorithm repeatedly
    /// with increasing population sizes.
    pub population_size: usize,
    /// The distribution to use when assigning weights to individuals. Default value is
    /// `Weights::Negative`.
    pub weights: Weights,
    /// The learning rate for adapting the mean. Can be set lower than `1.0` for noisy functions.
    /// Default value is `1.0`.
    pub cm: f64,
    /// The value to use for the [`TerminationReason::TolFun`] termination criterion. Default value
    /// is `1e-12`.
    pub tol_fun: f64,
    /// The value to use for the [`TerminationReason::TolX`] termination criterion. Default value is
    /// `1e-12 * initial_step_size`, used if this field is `None`.
    pub tol_x: Option<f64>,
    /// The seed for the RNG used in the algorithm. Can be set manually for deterministic runs. By
    /// default a random seed is used if this field is `None`.
    pub seed: Option<u64>,
    /// Options for the data plot. Default value is `None`, meaning no plot will be generated. See
    /// [`Plot`].
    pub plot_options: Option<PlotOptions>,
    /// How many function evaluations to wait for in between each automatic
    /// [`CMAESState::print_info`] call. Default value is `None`, meaning no info will be
    /// automatically printed.
    pub print_gap_evals: Option<usize>,
}

impl CMAESOptions {
    /// Creates a new `CMAESOptions` with default values. Set individual options using the provided
    /// methods.
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            initial_mean: DVector::zeros(dimensions),
            initial_step_size: 0.5,
            population_size: 4 + (3.0 * (dimensions as f64).ln()).floor() as usize,
            weights: Weights::Negative,
            cm: 1.0,
            tol_fun: 1e-12,
            tol_x: None,
            seed: None,
            plot_options: None,
            print_gap_evals: None,
        }
    }

    /// Changes the initial mean from the origin.
    pub fn initial_mean<V: Into<DVector<f64>>>(mut self, initial_mean: V) -> Self {
        self.initial_mean = initial_mean.into();
        self
    }

    /// Changes the initial step size from the default value. Must be positive.
    pub fn initial_step_size(mut self, initial_step_size: f64) -> Self {
        self.initial_step_size = initial_step_size;
        self
    }

    /// Changes the population size from the default value. Must be at least 4.
    pub fn population_size(mut self, population_size: usize) -> Self {
        self.population_size = population_size;
        self
    }

    /// Changes the weight distribution from the default value. See [`Weights`] for
    /// possible distributions.
    pub fn weights(mut self, weights: Weights) -> Self {
        self.weights = weights;
        self
    }

    /// Changes the learning rate for the mean from the default value. Must be between `0.0` and
    /// `1.0`.
    pub fn cm(mut self, cm: f64) -> Self {
        self.cm = cm;
        self
    }

    /// Changes the value for the `TolFun` termination criterion from the default value (see
    /// [`TerminationReason`][crate::TerminationReason]).
    pub fn tol_fun(mut self, tol_fun: f64) -> Self {
        self.tol_fun = tol_fun;
        self
    }

    /// Changes the value for the `TolX` termination criterion from the default value (see
    /// [`TerminationReason`][crate::TerminationReason]).
    pub fn tol_x(mut self, tol_x: f64) -> Self {
        self.tol_x = Some(tol_x);
        self
    }

    /// Sets the seed for the RNG.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Enables recording of a data plot for various state variables of the algorithm. See [`Plot`].
    pub fn enable_plot(mut self, plot_options: PlotOptions) -> Self {
        self.plot_options = Some(plot_options);
        self
    }

    /// Enables automatic printing of various state variables of the algorithm. A minimum of
    /// `min_gap_evals` function evaluations will be waited for between each
    /// [`CMAESState::print_info`] call, but it will always be called for the first few generations.
    pub fn enable_printing(mut self, min_gap_evals: usize) -> Self {
        self.print_gap_evals = Some(min_gap_evals);
        self
    }

    /// Attempts to build the [`CMAESState`] using the chosen options. See [`CMAESState`] for
    /// information about the lifetime parameter.
    pub fn build<'a, F: ObjectiveFunction + 'a>(
        self,
        objective_function: F,
    ) -> Result<CMAESState<'a>, InvalidOptionsError> {
        CMAESState::new(Box::new(objective_function), self)
    }
}

/// The distribution of weights for the population. The default value is `Negative`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weights {
    /// Weights are higher for higher-ranked selected individuals and are zero for the rest of the
    /// population.
    Positive,
    /// Similar to `Positive`, but non-selected individuals have negative weights. With this
    /// setting, the algorithm is known as active CMA-ES or aCMA-ES.
    Negative,
    /// Weights for selected individuals are equal and are zero for the rest of the population. This
    /// setting will likely perform much worse than the others.
    Uniform,
}

impl Default for Weights {
    fn default() -> Self {
        Self::Negative
    }
}

/// Represents invalid options for CMA-ES.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidOptionsError {
    /// The number of dimensions is set to zero.
    Dimensions,
    /// The dimension of the initial mean does not match the chosen dimension.
    MeanDimensionMismatch,
    /// The population size is too small (must be at least 4).
    PopulationSize,
    /// The initial step size is non-positive.
    InitialStepSize,
    /// The learning rate is outside the valid range (`0.0` to `1.0`).
    Cm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let dummy_function = |_: &DVector<f64>| 0.0;
        assert!(CMAESOptions::new(5).build(dummy_function).is_ok());
        assert!(CMAESOptions::new(5)
            .population_size(3)
            .build(dummy_function)
            .is_err());
        assert!(CMAESOptions::new(5)
            .initial_step_size(-1.0)
            .build(dummy_function)
            .is_err());
        assert!(CMAESOptions::new(5)
            .initial_mean(vec![1.0; 2])
            .build(dummy_function)
            .is_err());
        assert!(CMAESOptions::new(0).build(dummy_function).is_err());
        assert!(CMAESOptions::new(0).cm(2.0).build(dummy_function).is_err());
        assert!(CMAESOptions::new(0).cm(-1.0).build(dummy_function).is_err());
    }
}
