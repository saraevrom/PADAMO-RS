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
    pub fn perform(&self, a:usize, b:usize) -> Option<usize>{
        match self {
            Self::Add=>usize::checked_add(a, b),
            Self::Sub=>usize::checked_sub(a, b),
            Self::Mul=>usize::checked_mul(a, b),
            Self::Div=>usize::checked_div(a, b),
            Self::Mod=>usize::checked_rem(a, b),
            Self::Pow=>usize::checked_pow(a, b.try_into().ok()?),
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
    fn calculate(&self, index_src:&[usize],index_sizes:&[usize])->Option<usize> {
        let l = self.lhs.calculate(index_src,index_sizes)?;
        let r = self.rhs.calculate(index_src,index_sizes)?;
        self.operation.perform(l, r)
    }
}
