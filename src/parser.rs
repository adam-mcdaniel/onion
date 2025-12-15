use crate::context::{Assoc, Context, OpInfo};
use crate::expr::Expr;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_while1},
    character::complete::{char, digit1, multispace0, none_of},
    combinator::{map, map_res, opt, recognize, value},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
};

fn ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
    E: nom::error::ParseError<&'a str>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_symbol_str<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, &'a str> {
    let (input, _) = multispace0(input)?;
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    }

    let ops = ctx.get_operator_keys();
    let mut sorted_ops = ops;
    sorted_ops.sort_by(|a, b| b.len().cmp(&a.len()));

    for op in &sorted_ops {
        if input.starts_with(op) {
            let first_c = op.chars().next().unwrap();
            if first_c.is_alphanumeric() || first_c == '_' {
                if input.len() > op.len() {
                    let next_c = input[op.len()..].chars().next().unwrap();
                    if next_c.is_alphanumeric() || next_c == '_' {
                        continue;
                    }
                }
            }
            return Ok((&input[op.len()..], &input[..op.len()]));
        }
    }

    let mut i = 0;
    for (idx, c) in input.char_indices() {
        if c.is_whitespace() || "()[]{},\"".contains(c) {
            break;
        }

        let suffix = &input[idx..];
        let mut found_op = false;
        for op in &sorted_ops {
            if suffix.starts_with(op) {
                let first_c = op.chars().next().unwrap();
                if !first_c.is_alphanumeric() && first_c != '_' {
                    found_op = true;
                    break;
                }
            }
        }
        if found_op {
            break;
        }

        i = idx + c.len_utf8();
    }

    if i == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeWhile1,
        )));
    }

    Ok((&input[i..], &input[..i]))
}

fn parse_nil(input: &str) -> IResult<&str, Expr> {
    value(Expr::Nil, ws(tag("nil")))(input)
}

fn parse_hex(input: &str) -> IResult<&str, Expr> {
    map_res(
        ws(preceded(
            alt((tag("0x"), tag("0X"))),
            take_while1(|c: char| c.is_ascii_hexdigit()),
        )),
        |out: &str| i64::from_str_radix(out, 16).map(Expr::Int),
    )(input)
}

fn parse_int(input: &str) -> IResult<&str, Expr> {
    map_res(
        ws(recognize(pair(opt(alt((tag("-"), tag("+")))), digit1))),
        |out: &str| out.parse::<i64>().map(Expr::Int),
    )(input)
}

fn parse_float(input: &str) -> IResult<&str, Expr> {
    map_res(
        ws(recognize(tuple((digit1, char('.'), digit1)))),
        |s: &str| s.parse::<f64>().map(Expr::Float),
    )(input)
}

fn parse_str_lit(input: &str) -> IResult<&str, Expr> {
    let build_string = escaped_transform(
        none_of("\"\\"),
        '\\',
        alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("n")),
        )),
    );

    map(
        ws(delimited(char('"'), build_string, char('"'))),
        |s: String| Expr::Str(s),
    )(input)
}

fn parse_sym<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    map(|i| parse_symbol_str(i, ctx), |s| Expr::Sym(s.into()))(input)
}

fn parse_parens<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    delimited(
        ws(char('(')),
        map(many0(|i| parse_expr(i, ctx)), |exprs| Expr::List(exprs)),
        ws(char(')')),
    )(input)
}

fn parse_block<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    delimited(
        ws(char('{')),
        map(many0(|i| parse_expr(i, ctx)), |exprs| {
            let mut ops = vec![Expr::sym("do")];
            ops.extend(exprs);
            Expr::List(ops)
        }),
        ws(char('}')),
    )(input)
}

fn parse_sequence<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Vec<Expr>> {
    many0(|i| parse_expr(i, ctx))(input)
}

fn parse_map<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    let (input, _) = ws(tag("["))(input)?;

    if let Ok((rest, _)) = ws(char::<&str, nom::error::Error<&str>>(']'))(input) {
        return Ok((rest, Expr::Map(Vec::new().into_iter().collect())));
    }

    let (input, items) = parse_sequence(input, ctx)?;
    let (input, _) = ws(char(']'))(input)?;

    let mut map = Vec::new();
    for chunk in items.chunks(2) {
        if chunk.len() == 2 {
            map.push((chunk[0].clone(), chunk[1].clone()));
        }
    }
    Ok((input, Expr::Map(map.into_iter().collect())))
}
fn parse_hash_map<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    let (input, _) = ws(tag("#["))(input)?;

    if let Ok((rest, _)) = ws(char::<&str, nom::error::Error<&str>>(']'))(input) {
        return Ok((rest, Expr::HashMap(Vec::new().into_iter().collect())));
    }

    let (input, items) = parse_sequence(input, ctx)?;
    let (input, _) = ws(char(']'))(input)?;

    let mut map = Vec::new();
    for chunk in items.chunks(2) {
        if chunk.len() == 2 {
            map.push((chunk[0].clone(), chunk[1].clone()));
        }
    }
    Ok((input, Expr::HashMap(map.into_iter().collect())))
}

fn parse_quoted<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    let (input, _) = ws(char('\''))(input)?;
    let (input, expr) = parse_expr(input, ctx)?;
    Ok((input, Expr::Quoted(Box::new(expr))))
}

fn parse_atom<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    alt((
        parse_nil,
        parse_hex,
        parse_float,
        parse_int,
        parse_str_lit,
        |i| parse_quoted(i, ctx),
        |i| parse_block(i, ctx),
        |i| parse_map(i, ctx),
        |i| parse_hash_map(i, ctx),
        |i| parse_parens(i, ctx),
        |i| parse_sym(i, ctx),
    ))(input)
}

pub fn parse_expr<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    pratt_parse(input, ctx, 0)
}

fn pratt_parse<'a>(mut input: &'a str, ctx: &Context, min_bp: u8) -> IResult<&'a str, Expr> {
    let (mut input, mut lhs) = if let Ok((rest, sym)) = parse_symbol_str(input, ctx) {
        if let Some(op_info) = ctx.get_op(sym) {
            if op_info.unary {
                let (next_input, rhs) = pratt_parse(rest, ctx, op_info.precedence)?;

                (next_input, Expr::List(vec![Expr::Sym(sym.into()), rhs]))
            } else {
                parse_atom(input, ctx)?
            }
        } else {
            parse_atom(input, ctx)?
        }
    } else {
        parse_atom(input, ctx)?
    };

    loop {
        let peek_result = parse_symbol_str(input, ctx);

        if let Ok((rest_after_op, op_sym)) = peek_result {
            if let Some(op_info) = ctx.get_op(op_sym) {
                if op_info.precedence < min_bp {
                    break;
                }

                let next_min_bp = match op_info.associativity {
                    Assoc::Left => op_info.precedence + 1,
                    Assoc::Right => op_info.precedence,
                };

                input = rest_after_op;

                let (next_input, rhs) = pratt_parse(input, ctx, next_min_bp)?;
                input = next_input;

                lhs = Expr::List(vec![Expr::Sym(op_sym.into()), lhs, rhs]);

                continue;
            }
        }

        break;
    }

    Ok((input, lhs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Assoc, Context, OpInfo};
    use crate::expr::Expr;

    #[test]
    fn test_pratt_parser() {
        let mut ctx = Context::new();

        ctx.define_op(
            "+",
            OpInfo {
                precedence: 1,
                associativity: Assoc::Left,
                unary: false,
            },
            Expr::Nil,
        );
        ctx.define_op(
            "*",
            OpInfo {
                precedence: 2,
                associativity: Assoc::Left,
                unary: false,
            },
            Expr::Nil,
        );

        ctx.define_op(
            "!",
            OpInfo {
                precedence: 3,
                associativity: Assoc::Right,
                unary: true,
            },
            Expr::Nil,
        );

        let input = "3 + 4 * 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("+".into()),
            Expr::Int(3),
            Expr::List(vec![Expr::Sym("*".into()), Expr::Int(4), Expr::Int(5)]),
        ]);
        assert_eq!(expr, expected);

        let input = "! 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]);
        assert_eq!(expr, expected);

        let input = "! 5 + 3";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("+".into()),
            Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]),
            Expr::Int(3),
        ]);
        assert_eq!(expr, expected);

        let input = "! ! 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("!".into()),
            Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]),
        ]);
        assert_eq!(expr, expected);
    }
}
