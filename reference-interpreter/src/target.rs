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
#[path = "target_model/exact_matrix.rs"]
mod exact_matrix;
#[path = "target_model/native.rs"]
mod native;
#[path = "target_model/native_schema.rs"]
mod native_schema;
pub use native::{verify_native_model, ModelContext, ModelOperands, ModelVerification};
#[path = "target_model/document.rs"]
mod document;
#[path = "target_model/document_schema.rs"]
mod document_schema;
pub use document::{NativeTarget, ResolvedNativeEntry};
#[path = "target_model/frames.rs"]
mod frames;
pub use frames::frame_slot_classes;
#[path = "target_model/resources.rs"]
mod resources;
pub(crate) use resources::canonical_rows;
pub use resources::{
    charged_wire_occupancy, resource_display, verify_resource_contexts, verify_resource_rows,
    ResolvedResourceContext,
};
