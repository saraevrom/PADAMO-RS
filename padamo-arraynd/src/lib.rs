pub mod ndim_array;
pub mod indexing;
pub use ndim_array::ArrayND;

#[cfg(feature = "nalgebra")]
pub mod nalgebra_support;
