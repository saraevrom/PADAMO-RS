#[macro_export]
macro_rules! implement_onearg_function{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr, $main_expr:expr) =>{

        #[derive(Clone,Debug)]
        pub struct $structname;

        impl $structname{
            fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
                //let v = inputs.request_float("Value")?;
                //let f = make_function_box(crate::ops::Linear);
                let f:DoubleFunctionOperatorBox = ($main_expr).into();
                args.outputs.set_value("F", f.into())?;
                Ok(())
            }
        }

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

            fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
                self.calculate(args).into()
            }
        }
    }
}

#[macro_export]
macro_rules! implement_binary_combinator{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr, $main_expr:expr) =>{

        #[derive(Clone,Debug)]
        pub struct $structname;

        impl $structname{
            fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
                let f1 = args.inputs.request_function("F1")?;
                let f2 = args.inputs.request_function("F2")?;
                //let f = make_function_box(crate::ops::TwoSum(f1,f2));
                let f = f1.map2(f2, $main_expr);
                args.outputs.set_value("F", f.into())?;
                Ok(())
            }
        }

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

            fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
                self.calculate(args).into()
            }
        }
    }
}

#[macro_export]
macro_rules! implement_unary_combinator{
    ($structname:ident, $name:expr, $category_fn:ident, $id_name:expr, $old_name:expr, $main_expr:expr) =>{

        #[derive(Clone,Debug)]
        pub struct $structname;

        impl $structname{
            fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
                let f0 = args.inputs.request_function("F")?;
                let f = f0.map($main_expr);
                //let f = make_function_box(crate::ops::Log(f0));
                args.outputs.set_value("F", f.into())?;
                Ok(())
            }
        }

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

            fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
                self.calculate(args).into()
            }
        }
    }
}
