use std::fmt::Debug;

use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants};
use padamo_api::{ports,nodes_vec};

#[derive(Debug,Clone)]
pub struct Printer(pub ContentType);

impl Printer{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let value = inputs.request_type(&self.0, "Value")?;
        println!("Value: {:?}", value);
        Ok(())
    }
}

impl CalculationNode for Printer{
    fn name(&self,) -> RString{
        format!("Print {:?}", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "IO".into(),
            "Print".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome(format!("IO/Print {:?}", self.0).into())
    }

    fn identifier(&self,) -> RString where {
        let idmark = format!("{:?}",self.0).to_lowercase();
        format!("padamocore.print.{}",idmark).into()
    }

    fn is_primary(&self,) -> bool {
        true
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        RVec::new()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}



#[derive(Debug,Clone)]
pub struct StringCaster(pub ContentType);

impl StringCaster{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let value = inputs.request_type(&self.0, "Value")?;
        // let precision = constants.request_integer("Precision")?;
        // let precision:usize = precision.try_into().map_err(ExecutionError::from_error)?;
        let cast = match value {
            Content::Integer(i)=>i.to_string(),
            //Content::Float(f)=>format!("{:.1$}",f,precision),
            Content::Boolean(b)=>b.to_string(),
            Content::String(s)=>s.to_string(),
            _=>{return Err(ExecutionError::OtherError("Unsupported output".into()));}
        };
        outputs.set_value("Value", cast.into())?;
        // println!("Value: {:?}", value);
        Ok(())
    }
}

impl CalculationNode for StringCaster{
    fn name(&self,) -> RString{
        format!("{:?} to string", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "IO".into(),
            "Cast".into(),
        ]
    }


    fn identifier(&self,) -> RString where {
        let idmark = format!("{:?}",self.0).to_lowercase();
        format!("padamocore.cast_to_string.{}",idmark).into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Value", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

#[derive(Debug,Clone)]
pub struct FloatToStringCaster;


impl FloatToStringCaster{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let value = inputs.request_float("Value")?;
        let precision = constants.request_integer("Precision")?;
        let precision:usize = precision.try_into().map_err(ExecutionError::from_error)?;
        let cast = format!("{:.1$}",value,precision);
        outputs.set_value("Value", cast.into())?;
        // println!("Value: {:?}", value);
        Ok(())
    }
}


impl CalculationNode for FloatToStringCaster{
    fn name(&self,) -> RString{
        format!("Float to string").into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "IO".into(),
            "Cast".into(),
        ]
    }


    fn identifier(&self,) -> RString where {
        "padamocore.cast_to_string.float".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", ContentType::Float)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Value", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Precision", 3)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}



pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        Printer(ContentType::Integer),
        Printer(ContentType::Float),
        Printer(ContentType::String),
        Printer(ContentType::Boolean),

        StringCaster(ContentType::Integer),
        FloatToStringCaster,
        //StringCaster(ContentType::Float),
        StringCaster(ContentType::String),
        StringCaster(ContentType::Boolean),
    ]

}
