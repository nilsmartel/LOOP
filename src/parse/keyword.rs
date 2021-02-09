use super::Parse;
use nom::IResult;

pub struct K_loop;

impl<'a> Parse<'a> for K_loop {
    fn parse(input: &'a str) -> IResult<&'a str, K_loop> {
        let (rest, _) = nom::bytes::complete::tag_no_case("loop")(input)?;
        Ok((rest, K_loop))
    }
}

pub struct K_do;

impl<'a> Parse<'a> for K_do {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, _) = nom::bytes::complete::tag_no_case("do")(input)?;
        Ok((rest, Self))
    }
}

pub struct K_end;

impl<'a> Parse<'a> for K_end {
fn parse(input: &'a str) -> IResult<&'a str, Self> {
    let (rest, _) = nom::bytes::complete::tag_no_case("end")(input)?;
    Ok((rest, Self))
}


pub struct K_if;

impl<'a> Parse<'a> for K_if {
fn parse(input: &'a str) -> IResult<&'a str, Self> {
    let (rest, _) = nom::bytes::complete::tag_no_case("if")(input)?;
    Ok((rest, Self))
}
}
