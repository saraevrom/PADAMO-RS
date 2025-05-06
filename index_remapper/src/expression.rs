use nom_language::precedence::{binary_op, precedence, unary_op, Assoc, Operation, Unary};
use nom::{character, IResult, Parser};
use nom::combinator::{map_res, fail};
use nom::sequence::delimited;
use nom::bytes::complete::tag;
use nom::branch::alt;

use crate::utils::IndexCalculator;
use crate::binary_ops::{BinaryOperation, BinaryOperator};
use crate::indices::{parse_index, parse_constant};
use crate::slicing::{parse_slice, SliceDef, Slice};

pub fn parse_expression(i: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
  precedence(
    fail(),// unary_op(1, tag("-")),
    unary_op(1, parse_slice),
    alt((
      binary_op(1, Assoc::Left, tag("**")),
      binary_op(2, Assoc::Left, tag("*")),
      binary_op(2, Assoc::Left, tag("/")),
      binary_op(3, Assoc::Left, tag("+")),
      binary_op(3, Assoc::Left, tag("-")),
      binary_op(3, Assoc::Left, tag("%")),
    )),
    alt((
      parse_index,
      parse_constant,
      delimited(tag("("), parse_expression, tag(")")),
    )),
    |op: Operation<&str, SliceDef, &str, Box<dyn IndexCalculator>>| {
      use nom_language::precedence::Operation::*;
      match op {
        Postfix(x, p2)=> Ok(Box::new(Slice::new(p2, x))),
        Binary(lhs, "*", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Mul, lhs, rhs))),
        Binary(lhs, "/", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Div, lhs, rhs))),
        Binary(lhs, "+", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Add, lhs, rhs))),
        Binary(lhs, "-", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Sub, lhs, rhs))),
        Binary(lhs, "%", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Mod, lhs, rhs))),
        Binary(lhs, "**", rhs) => Ok(Box::new(BinaryOperator::new(BinaryOperation::Pow, lhs, rhs))),
        _ => Err("Invalid combination"),
      }
    }
  )(i)
}
