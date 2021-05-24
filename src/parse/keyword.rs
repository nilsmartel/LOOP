use super::Parse;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::IResult;

pub struct Loop;

impl<'a> Parse<'a> for Loop {
    fn parse(input: &'a str) -> IResult<&'a str, Loop> {
        let (rest, _) = tag_no_case("loop")(input)?;
        Ok((rest, Loop))
    }
}

pub struct Do;

impl<'a> Parse<'a> for Do {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag_no_case("do")(input)?;
        Ok((rest, Self))
    }
}

pub struct End;

impl<'a> Parse<'a> for End {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag_no_case("end")(input)?;
        Ok((rest, Self))
    }
}

pub struct If;

impl<'a> Parse<'a> for If {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag_no_case("if")(input)?;
        Ok((rest, Self))
    }
}

// = as well ass :=
pub struct Assign;

impl<'a> Parse<'a> for Assign {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = alt((tag("="), tag(":=")))(input)?;
        Ok((rest, Self))
    }
}

pub struct Semicolon;

impl<'a> Parse<'a> for Semicolon {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = tag(";")(input)?;
        Ok((rest, Self))
    }
}

#[cfg(test)]
mod keyword_tests {
    use super::*;

    #[test]
    fn assign() {
        assert!(K_assign::parse_ws("=").is_ok());
        assert!(K_assign::parse_ws("  =").is_ok());
    }
}
