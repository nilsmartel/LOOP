use super::keyword;
use super::Parse;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::{combinator::recognize, sequence::pair, IResult};

pub struct Ast {
    statements: Vec<Statement>,
}
impl<'a> Parse<'a> for Ast {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        todo!()
    }
}

pub enum Statement {
    Assignment(Assignment),
    Loop(Loop),
    // TODO
    // If(If)
}
impl<'a> Parse<'a> for Statement {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        todo!()
    }
}

pub struct Assignment {
    pub destination: Variable,
    pub op_var: Variable,
    pub op_const: Constant,
}
impl<'a> Parse<'a> for Assignment {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        todo!()
    }
}

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

pub struct Loop {
    pub counter: Variable,
    pub instruction: Ast,
}

impl<'a> Parse<'a> for Loop {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = keyword::K_loop::parse(input)?;
        let (rest, counter) = Variable::parse_ws(rest)?;
        let (rest, _) = keyword::K_do::parse_ws(rest)?;
        let (rest, instruction) = Ast::parse_ws(rest)?;
        let (rest, _) = keyword::K_end::parse_ws(rest)?;

        Ok((
            rest,
            Loop {
                counter,
                instruction,
            },
        ))
    }
}

pub struct If {
    pub variable: Variable,
    pub condition: Condition,
    pub instructions: Ast,
}

impl<'a> Parse<'a> for If {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = keyword::K_if::parse(input)?;
        let (rest, variable) = Variable::parse_ws(rest)?;
        let (rest, condition) = Condition::parse_ws(rest)?;
        let (rest, _) = keyword::K_do::parse_ws(rest)?;
        let (rest, instructions) = Ast::parse_ws(rest)?;
        let (rest, _) = keyword::K_end::parse_ws(rest)?;

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

pub enum Condition {
    Eq(Constant),
    Neq(Constant),
}

impl<'a> Parse<'a> for Condition {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        use nom::branch::alt;
        use nom::bytes::complete::tag;

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
