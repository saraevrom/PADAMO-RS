use crate::utils::IndexCalculator;


#[derive(Debug)]
pub enum BinaryOperation{
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow
}

impl BinaryOperation{
    pub fn perform(&self, a:i64, b:i64) -> Option<i64>{
        match self {
            Self::Add=>i64::checked_add(a, b),
            Self::Sub=>i64::checked_sub(a, b),
            Self::Mul=>i64::checked_mul(a, b),
            Self::Div=>i64::checked_div(a, b),
            Self::Mod=>i64::checked_rem(a, b),
            Self::Pow=>i64::checked_pow(a, b.try_into().ok()?),
        }
    }
}


#[derive(Debug)]
pub struct BinaryOperator{
    operation:BinaryOperation,
    lhs:Box<dyn IndexCalculator>,
    rhs:Box<dyn IndexCalculator>,
}

impl BinaryOperator {
    pub fn new(operation: BinaryOperation, lhs: Box<dyn IndexCalculator>, rhs: Box<dyn IndexCalculator>) -> Self {
        Self { operation, lhs, rhs }
    }
}

impl IndexCalculator for BinaryOperator{
    fn calculate(&self, index_src:&[usize],index_sizes:&[usize])->Option<i64> {
        let l = self.lhs.calculate(index_src,index_sizes)?;
        let r = self.rhs.calculate(index_src,index_sizes)?;
        self.operation.perform(l, r)
    }
}
