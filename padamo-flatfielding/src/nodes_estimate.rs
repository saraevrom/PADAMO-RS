use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use abi_stable::std_types::{RString, RVec};
use padamo_api::calculation_nodes::content::ContentType;
use padamo_api::calculation_nodes::node::CalculationNode;
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use padamo_api::{constants, prelude::*};
use padamo_api::ports;
use rayon::iter::ParallelIterator;
use rayon::iter::ParallelBridge;
use standalone_quantiles::quantile;
use super::nodes::{category,old_id};

//pub fn calculate_median()


#[derive(Clone,Debug)]
pub struct QuantileNode;

impl QuantileNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let signal = args.inputs.request_detectorfulldata("Signal")?;
        let l = signal.0.length();
        let q = args.constants.request_float("Quantile")?;
        let signal:ArrayND<f64> = signal.0.request_range(0,l);
        let target_shape = signal.shape.iter().skip(1).map(|x|*x).collect::<Vec<usize>>();

        let medians = Arc::new(Mutex::new(ArrayND::defaults(target_shape)));

        let iterators = signal.make_pixel_iterators();

        iterators.enumerate().par_bridge().for_each(|index|{
            let mut src:Vec<f64> = iterators[&index].clone().collect();
            medians.lock().unwrap()[&index] = quantile(&mut src, q);
        });

        // let medians = signal.quantile_axis_skipnan_mut(
        //                 Axis(0),
        //                 n64(q),
        //                 &ndarray_stats::interpolate::Linear).map_err(ExecutionError::from_error)?;
        // let medians:ArrayND<f64> = ArrayND::from(medians);
        let medians = Arc::try_unwrap(medians).unwrap().into_inner().unwrap();
        args.outputs.set_value("Quantile map", make_lao_box(medians).into())?;
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

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
