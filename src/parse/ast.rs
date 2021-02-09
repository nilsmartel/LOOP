use super::keyword;
use super::Parse;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::{combinator::recognize, IResult, sequence::pair};

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
        let (rest, name) = recognize(pair(take_while1(is_alpha), take_while(|c| c == '_' || is_alpha(c) || is_number(c))))(input)?;

    Ok((rest, Variable{name: name.to_string()}))
    }
}

fn is_alpha(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z'
}

fn is_number(c: char) -> bool {
    c >= '0' && c <= '9'
}

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
        let (rest, instruction) = Ast::parse_ws(rest)?;

        Ok((
            rest,
            Loop {
                counter,
                instruction,
            },
        ))
    }
}
