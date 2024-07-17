use std::error::Error;
use std::fmt::{Display,Debug};

use abi_stable::std_types::{RString, RVec};
use ndarray::{Axis, s};
use noisy_float::types::n64;
use padamo_api::calculation_nodes::content::ContentType;
use padamo_api::calculation_nodes::node::CalculationNode;
use padamo_api::lazy_array_operations::{LazyArrayOperation, LazyDetectorSignal, LazyArrayOperationBox};
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use padamo_api::{constants, prelude::*};
use padamo_api::ports;
use rayon::prelude::*;
use ndarray_stats::QuantileExt;
use super::nodes::{category,old_id};

//pub fn calculate_median()


#[derive(Clone,Debug)]
pub struct QuantileNode;

impl QuantileNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let signal = inputs.request_detectorfulldata("Signal")?;
        let l = signal.0.length();
        let q = constants.request_float("Quantile")?;
        let mut signal = signal.0.request_range(0,l).to_ndarray();
        let medians = signal.quantile_axis_skipnan_mut(
                        Axis(0),
                        n64(q),
                        &ndarray_stats::interpolate::Linear).map_err(ExecutionError::from_error)?;
        let medians:ArrayND<f64> = ArrayND::from(medians);
        outputs.set_value("Quantile map", make_lao_box(medians).into())?;
        Ok(())
    }
}

impl CalculationNode for QuantileNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Signal Quantile".into()
    }

    fn category(&self,) -> RVec<abi_stable::std_types::RString>where {
        category()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        old_id("Signal Quantile")
    }

    fn identifier(&self,) -> RString where {
        "padamoflatfielding.estimate.quantile".into()
    }

    fn inputs(&self,) -> RVec<padamo_api::prelude::CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn outputs(&self,) -> RVec<padamo_api::prelude::CalculationIO>where {
        ports!(
            ("Quantile map", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<padamo_api::prelude::CalculationConstant>where {
        constants!(
            ("Quantile", 0.5)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
