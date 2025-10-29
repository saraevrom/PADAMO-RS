use abi_stable::std_types::{RResult, RString, RVec};
use nalgebra::Matrix4;
use padamo_api::{constants, lazy_array_operations::ArrayND, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct PositionNode;

#[derive(Clone,Debug)]
pub struct RotationNode{
    name:String,
    id:String,
    axis:nalgebra::Vector3<f64>,
}

#[derive(Clone,Debug)]
pub struct TransformParent;

fn category() -> RVec<RString>where {
    vec![
        "Transform".into()
    ].into()
}

impl PositionNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>where {
        let x = args.constants.request_float("x")?;
        let y = args.constants.request_float("y")?;
        let z = args.constants.request_float("z")?;
        let v = nalgebra::Vector3::new(x, y, z);
        let v = nalgebra::Matrix4::new_translation(&v);
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
        let mut angle = args.constants.request_float("Angle")?;
        if args.constants.request_boolean("Degrees")?{
            angle = angle*std::f64::consts::PI/180.0;
        }

        let v = nalgebra::Matrix4::new_rotation(self.axis*angle);
        let v:ArrayND<f64> = v.into();
        let v = make_lao_box(v);
        args.outputs.set_value("Matrix", v.into())
    }
}

impl TransformParent{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let child = args.inputs.request_detectorsignal("Child")?;
        let parent = args.inputs.request_detectorsignal("Parent")?;

        let child = child.request_range(0,child.length());
        let parent = parent.request_range(0,parent.length());

        let child:Matrix4<f64> = child.try_into().map_err(|_| ExecutionError::OtherError("Child transform must be 4x4 matrix".into()))?;
        let parent:Matrix4<f64> = parent.try_into().map_err(|_| ExecutionError::OtherError("Parent transform must be 4x4 matrix".into()))?;
        let combined = parent*child;
        let combined:ArrayND<f64> = combined.into();
        args.outputs.set_value("Combined", make_lao_box(combined).into())
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
        ports![]
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
        ports![]
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

impl CalculationNode for TransformParent{
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
