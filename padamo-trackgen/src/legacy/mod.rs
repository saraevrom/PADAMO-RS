use abi_stable::std_types::RVec;
use padamo_api::{calculation_nodes::node::CalculationNodeBox, nodes_vec};

pub mod nodes_rev1;
pub mod lc_nodes_rev1;
pub mod ops_rev1;


pub fn nodes()->RVec<CalculationNodeBox>{
    let mut res = nodes_vec!();
    res.extend(nodes_rev1::nodes());
    res.extend(lc_nodes_rev1::nodes());
    res
}
