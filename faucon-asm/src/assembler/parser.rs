use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use num_traits::{PrimInt, Signed, Unsigned};

fn signed_decimal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Signed,
{
    map_res(
        pair(
            map(opt(alt((tag("+"), tag("-")))), |sign: Option<&str>| {
                sign.map(|s| if s == "-" { true } else { false })
                    .unwrap_or(false)
            }),
            recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        ),
        |out: (bool, &str)| {
            T::from_str_radix(&str::replace(&out.1, "_", ""), 10).and_then(|n| {
                if out.0 {
                    Ok(-n)
                } else {
                    Ok(n)
                }
            })
        },
    )(input)
}

fn unsigned_decimal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Unsigned,
{
    map_res(
        preceded(
            opt(tag("+")),
            recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        ),
        |out: &str| T::from_str_radix(&str::replace(&out, "_", ""), 10),
    )(input)
}

fn signed_hexadecimal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Signed,
{
    map_res(
        pair(
            map(opt(alt((tag("+"), tag("-")))), |sign: Option<&str>| {
                sign.map(|s| if s == "-" { true } else { false })
                    .unwrap_or(false)
            }),
            preceded(
                complete(tag_no_case("0x")),
                recognize(many1(terminated(
                    one_of("0123456789abcdefABCDEF"),
                    many0(char('_')),
                ))),
            ),
        ),
        |out: (bool, &str)| {
            T::from_str_radix(&str::replace(&out.1, "_", ""), 16).and_then(|n| {
                if out.0 {
                    Ok(-n)
                } else {
                    Ok(n)
                }
            })
        },
    )(input)
}

fn unsigned_hexadecimal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Unsigned,
{
    map_res(
        preceded(
            opt(tag("+")),
            preceded(
                complete(tag_no_case("0x")),
                recognize(many1(terminated(
                    one_of("0123456789abcdefABCDEF"),
                    many0(char('_')),
                ))),
            ),
        ),
        |out: &str| T::from_str_radix(&str::replace(&out, "_", ""), 16),
    )(input)
}

fn signed_octal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Signed,
{
    map_res(
        pair(
            map(opt(alt((tag("+"), tag("-")))), |sign: Option<&str>| {
                sign.map(|s| if s == "-" { true } else { false })
                    .unwrap_or(false)
            }),
            preceded(
                complete(tag_no_case("0o")),
                recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
            ),
        ),
        |out: (bool, &str)| {
            T::from_str_radix(&str::replace(&out.1, "_", ""), 8).and_then(|n| {
                if out.0 {
                    Ok(-n)
                } else {
                    Ok(n)
                }
            })
        },
    )(input)
}

fn unsigned_octal<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Unsigned,
{
    map_res(
        preceded(
            opt(tag("+")),
            preceded(
                complete(tag_no_case("0o")),
                recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
            ),
        ),
        |out: &str| T::from_str_radix(&str::replace(&out, "_", ""), 8),
    )(input)
}

fn signed_binary<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Signed,
{
    map_res(
        pair(
            map(opt(alt((tag("+"), tag("-")))), |sign: Option<&str>| {
                sign.map(|s| if s == "-" { true } else { false })
                    .unwrap_or(false)
            }),
            preceded(
                complete(tag_no_case("0b")),
                recognize(many1(terminated(one_of("01"), many0(char('_'))))),
            ),
        ),
        |out: (bool, &str)| {
            T::from_str_radix(&str::replace(&out.1, "_", ""), 2).and_then(|n| {
                if out.0 {
                    Ok(-n)
                } else {
                    Ok(n)
                }
            })
        },
    )(input)
}

fn unsigned_binary<T>(input: &str) -> IResult<&str, T>
where
    T: PrimInt + Unsigned,
{
    map_res(
        preceded(
            opt(tag("+")),
            preceded(
                complete(tag_no_case("0b")),
                recognize(many1(terminated(one_of("01"), many0(char('_'))))),
            ),
        ),
        |out: &str| T::from_str_radix(&str::replace(&out, "_", ""), 2),
    )(input)
}