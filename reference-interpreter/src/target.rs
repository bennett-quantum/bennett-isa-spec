//! Independent target/model checking for ISA 0.9 §§8 and 12.
#[path = "target_model/rational.rs"]
mod rational;
pub use rational::Rational;
#[path = "target_model/interval.rs"]
mod interval;
pub use interval::Interval;
#[path = "target_model/expression.rs"]
mod expression;
pub use expression::{CostExpression, ExpressionBindings};
#[path = "target_model/exact_expression.rs"]
mod exact_expression;
pub use exact_expression::{ExactComplex, ModelExpression};
#[path = "target_model/matrix.rs"]
mod matrix;
#[path = "target_model/process.rs"]
mod process;
pub use matrix::{ComplexInterval, IntervalMatrix};
pub use process::{
    verify_channel_bound, verify_numerical_bound, verify_restricted_numerical_bound,
};
