#[macro_export]
macro_rules! implement_onearg_function{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr) =>{
        impl CalculationNode for $structname{
            fn name(&self,) -> RString where {
                ($name).into()
            }

            fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
                abi_stable::std_types::ROption::RSome(format!("Functions/Generic/{}",$old_name).into())
            }

            fn identifier(&self,) -> RString where {
                format!("padamofunctions.generic.{}",$id_name).into()
            }

            fn category(&self,) -> RVec<RString>where {
                $category_fn()
            }

            fn inputs(&self,) -> RVec<CalculationIO>where {
                ports![

                ]
            }

            fn outputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("F", ContentType::Function)
                ]
            }

            fn constants(&self,) -> RVec<CalculationConstant>where {
                constants![]
            }

            fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
                self.calculate(inputs, outputs, constants, environment).into()
            }
        }
    }
}

#[macro_export]
macro_rules! implement_binary_combinator{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr) =>{
        impl CalculationNode for $structname{
            fn name(&self,) -> RString where {
                ($name).into()
            }

            fn category(&self,) -> RVec<RString>where {
                $category_fn()
            }

            fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
                abi_stable::std_types::ROption::RSome(format!("Functions/Generic/{}",$old_name).into())
            }

            fn identifier(&self,) -> RString where {
                format!("padamofunctions.generic.{}",$id_name).into()
            }

            fn inputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("F1", ContentType::Function),
                    ("F2", ContentType::Function)
                ]
            }

            fn outputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("F", ContentType::Function)
                ]
            }

            fn constants(&self,) -> RVec<CalculationConstant>where {
                constants![]
            }

            fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
                self.calculate(inputs, outputs, constants, environment).into()
            }
        }
    }
}

#[macro_export]
macro_rules! implement_unary_combinator{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr) =>{
        impl CalculationNode for $structname{
            fn name(&self,) -> RString where {
                ($name).into()
            }

            fn category(&self,) -> RVec<RString>where {
                $category_fn()
            }

            fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
                abi_stable::std_types::ROption::RSome(format!("Functions/Generic/{}",$old_name).into())
            }

            fn identifier(&self,) -> RString where {
                format!("padamofunctions.generic.{}",$id_name).into()
            }

            fn inputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("F", ContentType::Function)
                ]
            }

            fn outputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("F", ContentType::Function)
                ]
            }

            fn constants(&self,) -> RVec<CalculationConstant>where {
                constants![]
            }

            fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
                self.calculate(inputs, outputs, constants, environment).into()
            }
        }
    }
}
