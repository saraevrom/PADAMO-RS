use abi_stable::rvec;
use abi_stable::std_types::{RVec, RString, RResult};
use padamo_api::{constants, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct SaveHDF5Node;

impl SaveHDF5Node{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError> {
        let signal = inputs.request_detectorfulldata("Signal")?;
        let file_path = inputs.request_string("File path")?.to_string();
        let h5_file = hdf5::File::create(file_path).map_err(ExecutionError::from_error)?;

        let chunk_size = constants.request_integer("chunk")?;
        let chunk_size:usize = chunk_size.try_into().map_err(ExecutionError::from_error)?;

        let spatial_name = constants.request_string("spatial_field")?.into_string();
        let temporal_name = constants.request_string("temporal_field")?.into_string();

        let spatial = signal.0.request_range(0,signal.0.length());
        let deflate = constants.request_boolean("deflate")?;

        let ds_shape:Vec<usize> = spatial.shape.clone().into();
        let mut chunk_3d = ds_shape.clone();
        chunk_3d[0] = chunk_size;

        let mut space_ds = h5_file.new_dataset::<f64>()
            .chunk(chunk_3d)
            .shape(ds_shape);
        if deflate{
            let deflate_level = constants.request_integer("deflate_level")?;
            let deflate_level = deflate_level.try_into().map_err(ExecutionError::from_error)?;
            space_ds = space_ds.deflate(deflate_level);
        }
        let space_ds = space_ds.create(spatial_name.as_str()).map_err(ExecutionError::from_error)?;
        space_ds.write(&spatial.to_ndarray()).map_err(ExecutionError::from_error)?;

        let temporal = signal.1.request_range(0,signal.0.length());

        let mut time_ds = h5_file.new_dataset::<f64>()
            .chunk(chunk_size)
            .shape(vec![temporal.len()]);
        if deflate{
            let deflate_level = constants.request_integer("deflate_level")?;
            let deflate_level = deflate_level.try_into().map_err(ExecutionError::from_error)?;
            time_ds = time_ds.deflate(deflate_level);
        }

        let time_ds = time_ds.create(temporal_name.as_str()).map_err(ExecutionError::from_error)?;
        time_ds.write(&temporal).map_err(ExecutionError::from_error)?;

        Ok(())
    }
}

impl CalculationNode for SaveHDF5Node{
    fn name(&self,) -> RString {
        "Save HDF5 signal".into()
    }
    fn category(&self,) -> RVec<RString>{
        rvec![
            "HDF5".into()
        ]
    }

    fn is_primary(&self,) -> bool where {
        true
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal",ContentType::DetectorFullData),
            ("File path",ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("deflate",true),
            ("deflate_level",3),
            ("spatial_field","pdm_2d_rot_global"),
            ("temporal_field","unixtime_dbl_global"),
            ("chunk",16)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
