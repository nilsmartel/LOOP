use super::keyword;
use crate::ir;
use super::Parse;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map, recognize},
    multi::many0,
    sequence::pair,
    IResult,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub statements: Vec<Statement>,
}

// This is some of the worst code I've written. 
// Just want to get this to work as swiftly as possible
impl Ast {
    pub fn to_ir(self) -> crate::ir::Program {
        let instructions = map_instructions(self.statements);

        // Deeply uninspired code to extract all mentionings of variables in the instruction set
        fn getvars(ins: &[ir::Instruction], mut vars: &mut Vec<String>) {

            fn variable_of_value(v: &ir::Value) -> Option<String> {
                match v {
                    &ir::Value::Variable(s) => Some(s),
                    _ => None
                }
            }
            
            for i in ins.iter() {
                match i {
                    ir::Instruction::Assign {to, expr} => {
                        vars.push(to.clone());
                        if let Some(v) = variable_of_value(&expr.left) {
                            vars.push(v);
                        }

                        if let Some(v) = variable_of_value(&expr.right) {
                            vars.push(v);
                        }
                    },
                    ir::Instruction::If {condition, body} => {
                        if let Some(v) = variable_of_value(&condition.left) {
                            vars.push(v);
                        }

                        if let Some(v) = variable_of_value(&condition.right) {
                            vars.push(v);
                        }
                        getvars(body, &mut vars);
                    },
                    ir::Instruction::Loop {times, body} => {
                        if let Some(v) = variable_of_value(&times) {
                            vars.push(v);
                        }
                        getvars(body, &mut vars);
                    },
                }
            }
        }

        let variables: Vec<String> = {
            let mut v = Vec::new();
            getvars(&instructions, &mut v);

            // Get distinct elements
            let v = v.into_iter().collect::<std::collections::BTreeSet<String>>();

            // convert to vector
            v.into_iter().collect()
        };

        ir::Program {
            instructions,
            variables,
        }
    }
}

fn map_instructions(statements: Vec<Statement>) -> Vec<ir::Instruction> {
    statements.into_iter().fold(Vec::new(), |mut instructions, statement| {
        instructions.push(map_instruction(statement));
        instructions
    })
}

fn map_instruction(statement: Statement) -> ir::Instruction {
    match statement {
        Statement::Assignment(Assignment { destination, left_hand_side, operation, right_hand_side }) => {
            ir::Instruction::Assign {
                to: destination.name,
                expr: ir::Expr {
                    left: ir::Value::Variable(left_hand_side.name),
                    right: ir::Value::Constant(right_hand_side.value as i64),
                    op: map_op(operation),
                }
            }
        },
        Statement::If(If { variable, condition, instructions }) => {
            let (op, Constant {value}) = match condition {
                Condition::Eq(val) => ( ir::Operation::Equal, val),
                Condition::Neq(val) => ( ir::Operation::NotEqual, val),
            };
            let right = ir::Value::Constant(value as i64);
            ir::Instruction::If {
                condition: ir::Expr {
                    left: ir::Value::Variable(variable.name),
                    op,
                    right,
                },
                body: map_instructions(instructions.statements),
            }
        }
        Statement::Loop(Loop { counter, instruction }) => {
            ir::Instruction::Loop {
                times: ir::Value::Variable(counter.name),
                body: map_instructions(instruction.statements),
            }
        }
    }
}

fn map_op(operation: Operation) -> ir::Operation {
    match operation {
                    Operation::Add => ir::Operation::Plus,
                    Operation::Sub => ir::Operation::Minus,
                    Operation::Mod => ir::Operation::Modulo,
                    Operation::Mul => ir::Operation::Times,
                    Operation::Div => ir::Operation::Divided,
                }

}

impl<'a> Parse<'a> for Ast {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(many0(Statement::parse_ws), |statements| Ast { statements })(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword() {
        let input = "  =";
        let (rest, _) = keyword::Assign::parse_ws(input).unwrap();
        assert_eq!(rest, "");
    }

    #[test]
    fn full_ast() {
        let code = "
        x1 = x0 + 0;
        x2 = x0 + 3;
        LOOP x2 DO
            x1 = x1 + 1;
        END
        x0 = x1 + 0;";

        let (rest, ast) = Ast::parse_ws(code).unwrap();
        assert_eq!(
            "",
            rest
        );

        assert_eq!(ast.statements.len(), 4)
    }

    #[test]
    fn assignment() {
        let code = "x1 = x0 + 3;";

        let (rest, assignment) = Assignment::parse_ws(code).unwrap();
        assert_eq!("", rest);

        assert_eq!(assignment, Assignment {
            destination: Variable::parse("x1").unwrap().1,
            left_hand_side: Variable::parse("x0").unwrap().1,
            operation: Operation::Add,
            right_hand_side: Constant::parse("3").unwrap().1,
        });
    }

    #[test]
    fn variable() {
        let code = "x1";

        let (rest, variable) = Variable::parse_ws(code).unwrap();
        assert_eq!("", rest);

        assert_eq!(variable, Variable{name: "x1".to_string()});
    }

    #[test]
    fn operation() {
        let code = "+";

        let (rest, operation) = Operation::parse_ws(code).unwrap();
        assert_eq!("", rest);

        assert_eq!(operation, Operation::Add);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Loop(Loop),
    If(If),
}

impl<'a> Parse<'a> for Statement {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(Assignment::parse, Statement::Assignment),
            map(Loop::parse, Statement::Loop),
            map(If::parse, Statement::If),
        ))(input)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Assignment {
    pub destination: Variable,
    pub left_hand_side: Variable,
    pub operation: Operation,
    pub right_hand_side: Constant,
}

impl<'a> Parse<'a> for Assignment {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, destination) = Variable::parse(input)?;
        let (rest, _) = keyword::Assign::parse_ws(rest)?;
        let (rest, left_hand_side) = Variable::parse_ws(rest)?;
        let (rest, operation) = Operation::parse_ws(rest)?;
        let (rest, right_hand_side) = Constant::parse_ws(rest)?;
        let (rest, _) = keyword::Semicolon::parse_ws(rest)?;

        Ok((
            rest,
            Assignment {
                destination,
                left_hand_side,
                operation,
                right_hand_side,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl<'a> Parse<'a> for Operation {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, op) = alt((
            tag("+"),
            tag("-"),
            tag("*"),
            tag("/"),
            tag("%"),
            tag("mul"),
            tag("div"),
            tag("mod"),
        ))(input)?;

        use Operation::*;
        let op = match op {
            "+" => Add,
            "-" => Sub,
            "*" | "mul" => Mul,
            "/" | "div" => Div,
            "%" | "mod" => Mod,
            _ => unreachable!(),
        };

        Ok((rest, op))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
}

impl<'a> Parse<'a> for Variable {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, name) = recognize(pair(
            take_while1(is_alpha),
            take_while(|c| is_alpha(c) || is_number(c) || c == '_'),
        ))(input)?;

        Ok((
            rest,
            Variable {
                name: name.to_string(),
            },
        ))
    }
}

fn is_alpha(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z'
}

fn is_number(c: char) -> bool {
    c >= '0' && c <= '9'
}

// all constants in loop are postive integers
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Constant {
    pub value: u64,
}

impl<'a> Parse<'a> for Constant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, number) = take_while1(is_number)(input)?;

        let value = number.parse().unwrap();

        Ok((rest, Constant { value }))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Loop {
    pub counter: Variable,
    pub instruction: Ast,
}

impl<'a> Parse<'a> for Loop {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = keyword::Loop::parse(input)?;
        let (rest, counter) = Variable::parse_ws(rest)?;
        let (rest, _) = keyword::Do::parse_ws(rest)?;
        let (rest, instruction) = Ast::parse_ws(rest)?;
        let (rest, _) = keyword::End::parse_ws(rest)?;

        Ok((
            rest,
            Loop {
                counter,
                instruction,
            },
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct If {
    pub variable: Variable,
    pub condition: Condition,
    pub instructions: Ast,
}

impl<'a> Parse<'a> for If {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = keyword::If::parse(input)?;
        let (rest, variable) = Variable::parse_ws(rest)?;
        let (rest, condition) = Condition::parse_ws(rest)?;
        let (rest, _) = keyword::Do::parse_ws(rest)?;
        let (rest, instructions) = Ast::parse_ws(rest)?;
        let (rest, _) = keyword::End::parse_ws(rest)?;

        Ok((
            rest,
            If {
                variable,
                condition,
                instructions,
            },
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Condition {
    Eq(Constant),
    Neq(Constant),
}

impl<'a> Parse<'a> for Condition {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, t) = alt((tag("!="), tag("=")))(input)?;

        let (rest, c) = Constant::parse_ws(rest)?;

        let condition = match t {
            "!=" => Condition::Neq(c),
            "=" => Condition::Eq(c),
            _ => unreachable!(),
        };

        Ok((rest, condition))
    }
}
