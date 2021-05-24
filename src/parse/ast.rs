use super::keyword;
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
