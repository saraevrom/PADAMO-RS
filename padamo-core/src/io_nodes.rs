use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, make_node_box, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct EnvOutputNode(pub ContentType);


impl EnvOutputNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let key = constants.request_string("Key")?;
        // let value = constants.request_type(&self.0, "Value")?;
        let value = match  environment.request_type(&self.0, &key){
            Ok(v) => v,
            Err(e)=>{
                if let Ok(d) = TryInto::<ConstantContentType>::try_into(self.0){
                    let constant = constants.request_type(&d, "Default")?;
                    constant.into()
                }
                else{
                    return Err(e);
                }
            }
        };

        outputs.set_value("Value", value.into())
    }
}

impl CalculationNode for EnvOutputNode{
    fn name(&self,) -> RString{
        format!("Environment output {:?}", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "Environment".into(),
            "Output".into()
        ]
    }

    fn identifier(&self,) -> RString where {
        let idmark = format!("{:?}",self.0).to_lowercase();
        format!("padamocore.env_output.{}",idmark).into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0.into())
        )
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        RVec::new()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        let mut res = constants!(
            ("Key", "env_key")
        );
        if let Ok(v) = TryInto::<ConstantContentType>::try_into(self.0){
            res.push(CalculationConstant::new("Default", v.default_constant()));
        }
        res
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


#[derive(Clone,Debug)]
pub struct EnvInputNode(pub ContentType);


impl EnvInputNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let key = constants.request_string("Key")?;
        // let value = constants.request_type(&self.0, "Value")?;
        let value = inputs.request_type(&self.0, "Value")?;

        environment.0.insert(key, value.into());
        Ok(())
    }
}

impl CalculationNode for EnvInputNode{
    fn name(&self,) -> RString{
        format!("Environment input {:?}", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "Environment".into(),
            "Input".into()
        ]
    }

    fn is_primary(&self,) -> bool where {
        true
    }

    fn identifier(&self,) -> RString where {
        let idmark = format!("{:?}",self.0).to_lowercase();
        format!("padamocore.env_input.{}",idmark).into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0.into())
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        RVec::new()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Key", "env_key")
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


pub fn nodes()->RVec<CalculationNodeBox>{
    let values:Vec<ContentType> = ContentType::get_variants();
    let mut res:RVec<CalculationNodeBox> = RVec::new();
    for ty in values.iter(){
        res.push(make_node_box(EnvOutputNode(*ty)));
        res.push(make_node_box(EnvInputNode(*ty)));
    }
    res
}
