pub mod ndim_array;
pub mod indexing;
pub use ndim_array::ArrayND;
pub mod operators;

#[cfg(feature = "nalgebra")]
pub mod nalgebra_support;

// Because semvers are cursed.
#[cfg(feature = "ndarray")]
pub use ndarray;

pub use ndim_array::calculate_offset;
pub use ndim_array::calculate_offset_f;
