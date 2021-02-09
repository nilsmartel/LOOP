use nom::bytes::complete::take_while;

pub trait Parse<'a>
where
    Self: Sized,
{
    fn parse(input: &'a str) -> nom::IResult<&'a str, Self>;

    fn parse_ws(input: &'a str) -> nom::IResult<&'a str, Self> {
        fn is_whitespace(c: char) -> bool {
            c == ' ' || c == '\n' || c == '\t' || c == '\r'
        }
        // can't possibly fail
        let (rest, _ws) = take_while(is_whitespace)(input)?;

        Self::parse(rest)
    }
}
