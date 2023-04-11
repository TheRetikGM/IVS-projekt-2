use crate::error::Error;
use crate::token::*;
use crate::Variable;
use crate::{Number, Result};
use std::collections::HashMap;

/// The Engine trait
/// This trait contains 2 parts. `evaluate` and `validate_tokens`
/// The `validate_tokens` ensures that the input is valid for the current `Engine`
///
pub trait Engine {
    /// Validate the given token list to ensure that it's executable
    /// This *only* do the syntatical check shouldn't perform any heavy operation
    fn validate_tokens(
        &mut self,
        token: &[Token],
        variables: &HashMap<String, Variable>,
    ) -> Result<()>;

    /// Evaluate the token list
    /// This function will always be call *after* `validate_tokens`, so it don't need to check for
    /// the correctness of Token list
    fn evaluate(
        &mut self,
        tokens: &[Token],
        variables: &HashMap<String, Variable>,
    ) -> Result<Number>;

    /// Call`validate_tokens` then `evaluate` it immediately
    fn execute(
        &mut self,
        token: &[Token],
        variables: &HashMap<String, Variable>,
    ) -> Result<Number> {
        self.validate_tokens(token, variables)?;
        self.evaluate(token, variables)
    }
}

enum ShuntingYardOperator {
    Operator(Operator),
    OpenParen,
    Variable(Variable),
}

enum Sign {
    Plus,
    Minus,
}

#[derive(Default)]
/// An modification of the shunting yard algorithm for evaluate infix math notation that allows
/// functions/constants being used
pub struct ShuntingYardEngine {
    operators: Vec<ShuntingYardOperator>,
    operands: Vec<Number>,
}

impl Engine for ShuntingYardEngine {
    fn validate_tokens(
        &mut self,
        token: &[Token],
        variables: &HashMap<String, Variable>,
    ) -> Result<()> {
        Ok(())
    }

    fn evaluate(
        &mut self,
        tokens: &[Token],
        variables: &HashMap<String, Variable>,
    ) -> Result<Number> {
        self.operators.clear();
        self.operands.clear();

        for token in tokens {
            match token {
                Token::Number(num) => self.store_operand(num.clone()),
                Token::Operator(op) => self.operator_handle(*op)?,
                Token::FactorialSign => {
                    let num = self.operands.pop().unwrap().factorial()?;
                    self.store_operand(num);
                }

                Token::Bracket(Bracket::ParenLeft) => {
                    self.operators.push(ShuntingYardOperator::OpenParen);
                }
                Token::Bracket(Bracket::ParenRight) => self.closing_bracket_handle()?,
                Token::Bracket(Bracket::VerticalLine) => todo!(),
                Token::Id(id) => {
                    let var = variables.get(id).cloned().unwrap();
                    self.operators.push(ShuntingYardOperator::Variable(var));
                }
                Token::Comma => (),
            }
        }

        Ok(self.operands.pop().unwrap_or_default())
    }
}

fn operator_precedence(op: Operator) -> u8 {
    match op {
        Operator::Plus | Operator::Minus => 0,
        Operator::Multiply | Operator::Divide => 1,
        Operator::Power => 2,
    }
}

fn evaluate_expr(lhs: Number, rhs: Number, op: Operator) -> Result<Number> {
    match op {
        Operator::Plus => lhs.add(rhs),
        Operator::Minus => lhs.sub(rhs),
        Operator::Multiply => lhs.mul(rhs),
        Operator::Divide => lhs.div(rhs),
        Operator::Power => lhs.power(rhs),
    }
}

impl ShuntingYardEngine {
    fn store_operand(&mut self, val: Number) {
        self.operands.push(val);
    }

    fn operator_handle(&mut self, op: Operator) -> Result<()> {
        let current_precedence = operator_precedence(op);

        while let Some(ShuntingYardOperator::Operator(last_op)) = self.operators.last() {
            if current_precedence > operator_precedence(*last_op) {
                break;
            }

            let lhs = self.operands.pop().unwrap();
            let rhs = self.operands.pop().unwrap();
            self.store_operand(evaluate_expr(lhs, rhs, *last_op)?);
            self.operators.pop();
        }

        self.operators.push(ShuntingYardOperator::Operator(op));
        Ok(())
    }

    fn closing_bracket_handle(&mut self) -> Result<()> {
        if let Some(num) = self.finalize()? {
            self.store_operand(num);
            return Ok(());
        }

        if let Some(ShuntingYardOperator::Variable(var)) = self.operators.last() {
            let argc = var.argc();
            let mut argv = Vec::with_capacity(argc as usize);

            for _ in 0..argc {
                argv.insert(0, self.operands.pop().unwrap());
            }

            let val = var.calc(&argv)?;
            self.operators.pop();
            self.store_operand(val);
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<Option<Number>> {
        let mut res = None;

        while let Some(operator) = self.operators.pop() {
            let ShuntingYardOperator::Operator(op) = operator else {
                break;
            };

            let rhs = res.clone().or_else(|| self.operands.pop()).unwrap();
            let lhs = self.operands.pop().unwrap();
            res.replace(evaluate_expr(lhs, rhs, op)?);
        }

        Ok(res)
    }
}
