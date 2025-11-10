use abi_stable::std_types::{RString, RVec};
use nalgebra::{Matrix4, Vector3};
use padamo_api::{constants, lazy_array_operations::ArrayND, ports, prelude::*};
use super::{get_all, matrix_err};

#[derive(Clone, Debug)]
pub struct WGS84PositionNode;

const WGS84_A:f64 = 6378.137;
const WGS84_B:f64 = 6356.752314245;

const WGS84_F:f64 = (WGS84_A-WGS84_B)/WGS84_A;
const WGS84_F_INV:f64 = WGS84_A/(WGS84_A-WGS84_B);

const WGS84_A_SQR:f64 = WGS84_A*WGS84_A;
const WGS84_B_SQR:f64 = WGS84_B*WGS84_B;
const WGS84_E_SQR:f64 = WGS84_F* (2.0-WGS84_F);


impl WGS84PositionNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let input = get_all(args.inputs.request_detectorsignal("Matrix")?);
        let input:Matrix4<f64> = input.try_into().map_err(|_| matrix_err("Input matrix must be 4x4"))?;
        let lambda = args.constants.request_float("latitude")?*std::f64::consts::PI/180.0;
        let phi = args.constants.request_float("longitude")?*std::f64::consts::PI/180.0;
        let h = args.constants.request_float("elevation")?/1000.0;

        let s = lambda.sin();
        let n = WGS84_A/(1.0-WGS84_E_SQR*s*s).sqrt();

        let sin_lambda = lambda.sin();
        let cos_lambda = lambda.cos();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        let x = (h+n) * cos_lambda * cos_phi;
        let y = (h+n) * cos_lambda * sin_phi;
        let z = (h+ (1.0 - WGS84_E_SQR)*n) * sin_lambda;

        let vec = nalgebra::Vector3::new(x,y,z);
        let op = nalgebra::Matrix4::new_translation(&vec);

        let output = op*input;
        let output: ArrayND<f64> = output.into();
        args.outputs.set_value("Matrix", make_lao_box(output).into())
    }
}

fn category() -> RVec<RString>where {
    vec![
        "Transform".into(),
        "Geo".into()
    ].into()
}

impl CalculationNode for WGS84PositionNode{
    fn name(&self,) -> RString where {
        "WGS84 ECEF coordinates (km)".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.geo.geoid_coords".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Matrix", ContentType::DetectorSignal)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Matrix", ContentType::DetectorSignal)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("latitude", "Latitude [deg]", 0.0),
            ("longitude", "Longitude [deg]", 0.0),
            ("elevation", "Elevation [m]", 0.0),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone, Debug)]
pub struct DetectorRotatorNode;

impl DetectorRotatorNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let input = get_all(args.inputs.request_detectorsignal("Matrix")?);
        let input:Matrix4<f64> = input.try_into().map_err(|_| matrix_err("Input matrix must be 4x4"))?;

        let mut workon = input;

        // Matrix that aligns detector axis Z with main axis X
        let swapper = Matrix4::new(
            0.0, 0.0, 1.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        if args.constants.request_boolean("align_z")?{
            workon = swapper*workon;
        }

        // Own rotation: in plane YZ
        let own_rotation = args.constants.request_float("own_rot")?*std::f64::consts::PI/180.0;
        let own_rotator = Matrix4::new_rotation(Vector3::new(own_rotation,0.0,0.0));
        workon = own_rotator*workon;

        // Latitudal rotation: in plane XZ
        let latitude = args.constants.request_float("dec")?*std::f64::consts::PI/180.0;
        let latitudal_rotator = Matrix4::new_rotation(Vector3::new(0.0,-latitude,0.0));
        workon = latitudal_rotator*workon;

        // Longitudal rotation: in plane XY
        let longitude = args.constants.request_float("hour_angle")?*std::f64::consts::PI/180.0;
        let longitudal_rotator = Matrix4::new_rotation(Vector3::new(0.0,0.0,longitude));

        workon = longitudal_rotator*workon;

        let output:ArrayND<f64> = workon.into();

        args.outputs.set_value("Matrix", make_lao_box(output).into())

    }
}

impl CalculationNode for DetectorRotatorNode{
    fn name(&self,) -> RString where {
        "Detector Sky rotation".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.geo.rotate_detector".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Matrix", ContentType::DetectorSignal)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Matrix", ContentType::DetectorSignal)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("dec", "Declination [deg]", 0.0),
            ("hour_angle", "Hour angle [deg]", 0.0),
            ("own_rot", "Own rotation [deg]", 0.0),
            ("align_z", "Align local Z axis", true),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
