use abi_stable::std_types::{RResult, RString, RVec};
use nalgebra::Matrix4;
use super::{get_all, matrix_err};
use padamo_api::{constants, lazy_array_operations::{ArrayND, LazyDetectorSignal}, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct IdentityNode;

#[derive(Clone,Debug)]
pub struct PositionNode;

#[derive(Clone,Debug)]
pub struct RotationNode{
    name:String,
    id:String,
    axis:nalgebra::Vector3<f64>,
}

#[derive(Clone,Debug)]
pub struct TransformParentNode;

#[derive(Clone,Debug)]
pub struct ModelViewNode;

fn category() -> RVec<RString>where {
    vec![
        "Transform".into()
    ].into()
}



impl IdentityNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>where {
        let v = nalgebra::Matrix4::identity();
        let v:ArrayND<f64> = v.into();
        let v = make_lao_box(v);
        args.outputs.set_value("Matrix", v.into())
    }
}

impl PositionNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>where {
        let input = get_all(args.inputs.request_detectorsignal("Matrix")?);
        let input:Matrix4<f64> = input.try_into().map_err(|_| matrix_err("Input matrix must be 4x4"))?;

        let x = args.constants.request_float("x")?;
        let y = args.constants.request_float("y")?;
        let z = args.constants.request_float("z")?;
        let v = nalgebra::Vector3::new(x, y, z);
        let v = nalgebra::Matrix4::new_translation(&v) * input;
        let v:ArrayND<f64> = v.into();
        let v = make_lao_box(v);
        args.outputs.set_value("Matrix", v.into())
    }
}

impl RotationNode{
    pub fn new<T1:Into<String>,T2:Into<String>>(name: T1, id:T2, axis: nalgebra::Vector3<f64>) -> Self {
        Self { name:name.into(), axis, id:id.into() }
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>where {
        let input = get_all(args.inputs.request_detectorsignal("Matrix")?);
        let input:Matrix4<f64> = input.try_into().map_err(|_| matrix_err("Input matrix must be 4x4"))?;

        let mut angle = args.constants.request_float("Angle")?;
        if args.constants.request_boolean("Degrees")?{
            angle = angle*std::f64::consts::PI/180.0;
        }

        let v = nalgebra::Matrix4::new_rotation(self.axis*angle) * input;
        let v:ArrayND<f64> = v.into();
        let v = make_lao_box(v);
        args.outputs.set_value("Matrix", v.into())
    }
}

impl TransformParentNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let child = get_all(args.inputs.request_detectorsignal("Child")?);
        let parent = get_all(args.inputs.request_detectorsignal("Parent")?);


        let child:Matrix4<f64> = child.try_into().map_err(|_| matrix_err("Child transform must be 4x4 matrix"))?;
        let parent:Matrix4<f64> = parent.try_into().map_err(|_| matrix_err("Parent transform must be 4x4 matrix"))?;
        let combined = parent*child;
        let combined:ArrayND<f64> = combined.into();
        args.outputs.set_value("Combined", make_lao_box(combined).into())
    }
}

impl ModelViewNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let model = get_all(args.inputs.request_detectorsignal("Model")?);
        let view = get_all(args.inputs.request_detectorsignal("View")?);

        let model:Matrix4<f64> = model.try_into().map_err(|_| matrix_err("Child transform must be 4x4 matrix"))?;
        let view:Matrix4<f64> = view.try_into().map_err(|_| matrix_err("Parent transform must be 4x4 matrix"))?;
        let view = view.try_inverse().ok_or(matrix_err("Cannot construct view matrix. Ensure that View Transform matrix is inversible."))?;

        let combined =  view*model;
        // let combined = parent*child;
        let combined:ArrayND<f64> = combined.into();
        args.outputs.set_value("Combined", make_lao_box(combined).into())
    }
}

impl CalculationNode for IdentityNode{
    fn name(&self) -> RString where {
        "Identity".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.identity".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Matrix", ContentType::DetectorSignal)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for PositionNode{
    fn name(&self,) -> RString where {
        "Position".into()
    }
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.position".into()
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
            ("x", 0.0),
            ("y", 0.0),
            ("z", 0.0),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for RotationNode{
    fn name(&self,) -> RString where {
        self.name.clone().into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        format!("transforms.rotate_{}", self.id).into()
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
            ("Angle", 0.0),
            ("Degrees", false),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for TransformParentNode{
    fn name(&self,) -> RString where {
        "Transform parent".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.parent".into()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Parent", ContentType::DetectorSignal),
            ("Child", ContentType::DetectorSignal),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Combined", ContentType::DetectorSignal),
        ]
    }
}

impl CalculationNode for ModelViewNode{
    fn name(&self,) -> RString where {
        "Model-View".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "transforms.mv".into()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Model", ContentType::DetectorSignal),
            ("View", ContentType::DetectorSignal),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Combined", ContentType::DetectorSignal),
        ]
    }
}
