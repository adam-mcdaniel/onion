use crate::context::{Assoc, Context};
use crate::expr::Expr;
use crate::symbol::Symbol;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, none_of, not_line_ending},
    combinator::{cut, map, map_res, opt, recognize, value},
    error::{VerboseError, context, convert_error},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};

// Define a type alias for IResult with VerboseError
type Res<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

fn sp<'a>(input: &'a str) -> Res<'a, &'a str> {
    recognize(many0(alt((
        multispace1,
        recognize(pair(char(';'), not_line_ending)),
    ))))(input)
}

fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> Res<'a, O>
where
    F: FnMut(&'a str) -> Res<'a, O>,
{
    delimited(sp, inner, sp)
}

// --- Atom Parsers ---

fn parse_nil(input: &str) -> Res<Expr> {
    value(Expr::Nil, ws(tag("nil")))(input)
}

fn parse_hex(input: &str) -> Res<Expr> {
    map_res(
        ws(preceded(
            alt((tag("0x"), tag("0X"))),
            take_while1(|c: char| c.is_ascii_hexdigit()),
        )),
        |out: &str| i64::from_str_radix(out, 16).map(Expr::Int),
    )(input)
}

fn parse_int(input: &str) -> Res<Expr> {
    map_res(
        ws(recognize(pair(opt(alt((tag("-"), tag("+")))), digit1))),
        |out: &str| out.parse::<i64>().map(Expr::Int),
    )(input)
}

fn parse_float(input: &str) -> Res<Expr> {
    map_res(
        ws(recognize(tuple((
            opt(alt((tag("-"), tag("+")))),
            digit1,
            tag("."),
            digit1,
        )))),
        |out: &str| out.parse::<f64>().map(Expr::Float),
    )(input)
}

fn parse_str_lit(input: &str) -> Res<Expr> {
    let build_string = escaped_transform(
        none_of("\\\""),
        '\\',
        alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("n")),
        )),
    );

    context(
        "string",
        map(
            preceded(ws(char('"')), cut(terminated(build_string, char('"')))),
            |s: String| Expr::Str(s),
        ),
    )(input)
}

fn parse_symbol_str<'a>(input: &'a str, ctx: &Context) -> Res<'a, &'a str> {
    // 1. Check if input matches a known operator in Context
    let ops = ctx.get_operator_keys();
    // Sort ops by length descending to match longest operator first
    let mut sorted_ops = ops;
    sorted_ops.sort_by(|a, b| b.len().cmp(&a.len()));

    for op in &sorted_ops {
        if input.starts_with(op) {
            // Check boundary if op ends with alphanumeric or _
            // If op is "def", "defun" shouldn't match "def"
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

    // 2. If not operator, parse strict symbol token
    // Allowed: alphanumeric, _, +, -, *, /, <, >, =, !, ? .
    // BUT we must stop at delimiters: whitespace, parens, quotes
    let allowed_special = "+-*/<>=!?_";

    // Custom take_while since we need to respect delimiters
    let is_sym_char = |c: char| c.is_alphanumeric() || allowed_special.contains(c);

    let (input, sym_str) = take_while1(is_sym_char)(input)?;

    // Ensure it's not a number (if it starts with digit, it might have been parsed by int/float,
    // but if we are here, int/float failed or we are in pratt loop looking for token)
    // Actually, `parse_atom` calls `parse_symbol` LAST. So we assume it's acceptable.

    Ok((input, sym_str))
}

fn parse_sym<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    map(ws(|i| parse_symbol_str(i, ctx)), |s: &str| {
        Expr::Sym(Symbol::new(s))
    })(input)
}

// --- Collections ---

fn parse_list<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    context(
        "list",
        map(
            preceded(
                ws(char('(')),
                cut(terminated(many0(|i| parse_expr(i, ctx)), ws(char(')')))),
            ),
            Expr::List,
        ),
    )(input)
}

fn parse_map<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    context(
        "map",
        map(
            preceded(
                ws(char('[')),
                cut(terminated(many0(|i| parse_expr(i, ctx)), ws(char(']')))),
            ),
            |items| {
                let mut map = std::collections::BTreeMap::new();
                for chunk in items.chunks(2) {
                    if chunk.len() == 2 {
                        map.insert(chunk[0].clone(), chunk[1].clone());
                    }
                }
                Expr::Map(map)
            },
        ),
    )(input)
}

fn parse_hashmap<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    context(
        "hashmap",
        map(
            preceded(
                ws(tag("#[")),
                cut(terminated(many0(|i| parse_expr(i, ctx)), ws(char(']')))),
            ),
            |items| {
                let mut map = std::collections::HashMap::new();
                for chunk in items.chunks(2) {
                    if chunk.len() == 2 {
                        map.insert(chunk[0].clone(), chunk[1].clone());
                    }
                }
                Expr::HashMap(map)
            },
        ),
    )(input)
}

fn parse_quote<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    context(
        "quote",
        map(
            preceded(ws(char('\'')), cut(|i| parse_expr(i, ctx))),
            |expr| Expr::Quoted(Box::new(expr)),
        ),
    )(input)
}

fn parse_block<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    context(
        "block",
        map(
            preceded(
                ws(char('{')),
                cut(terminated(many0(|i| parse_expr(i, ctx)), ws(char('}')))),
            ),
            |exprs| {
                let mut block = vec![Expr::Sym(Symbol::new("do"))];
                block.extend(exprs);
                Expr::List(block)
            },
        ),
    )(input)
}

// --- Pratt Parsing Support ---

fn parse_atom<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    alt((
        parse_nil,
        parse_hex,
        parse_float,
        parse_int,
        parse_str_lit,
        |i| parse_quote(i, ctx),
        |i| parse_hashmap(i, ctx),
        |i| parse_map(i, ctx),
        |i| parse_block(i, ctx),
        |i| parse_list(i, ctx),
        |i| parse_sym(i, ctx),
    ))(input)
}

fn pratt_parse<'a>(input: &'a str, ctx: &Context, min_bp: u8) -> Res<'a, Expr> {
    // 1. Prefix Phase
    let (mut input, mut lhs) = parse_atom(input, ctx)?;

    // Check if the atom we just parsed is a Symbol that acts as a specific prefix operator
    if let Expr::Sym(ref s) = lhs {
        if let Some(op_info) = ctx.get_op(s.as_str()) {
            if op_info.unary {
                let (next_input, rhs) = pratt_parse(input, ctx, op_info.precedence)?;
                input = next_input;
                // Prefix: (op rhs)
                lhs = Expr::List(vec![lhs.clone(), rhs]);
            }
        }
    }

    // 2. Infix Phase
    loop {
        // Peek next token to see if it's an operator
        let peek = ws(|i| parse_symbol_str(i, ctx))(input);

        if let Ok((rest_after_op, op_sym)) = peek {
            if let Some(op_info) = ctx.get_op(op_sym) {
                // Unary operators cannot be used as infix
                if op_info.unary {
                    break;
                }

                // If operator precedence is lower than min_bp, we stop (binding tighter to the left)
                if op_info.precedence < min_bp {
                    break;
                }

                let (_l_bp, r_bp) = match op_info.associativity {
                    Assoc::Left => (op_info.precedence, op_info.precedence + 1),
                    Assoc::Right => (op_info.precedence, op_info.precedence),
                };

                // Consume the operator
                input = rest_after_op;

                // Parse RHS
                let (next_input, rhs) = pratt_parse(input, ctx, r_bp)?;
                input = next_input;

                // Combine: (op lhs rhs)
                lhs = Expr::List(vec![Expr::Sym(Symbol::new(op_sym)), lhs, rhs]);
                continue;
            }
        }
        break;
    }

    Ok((input, lhs))
}

pub fn parse_expr<'a>(input: &'a str, ctx: &Context) -> Res<'a, Expr> {
    pratt_parse(input, ctx, 0)
}

pub fn convert_error_to_string(input: &str, e: VerboseError<&str>) -> String {
    convert_error(input, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atoms() {
        let ctx = Context::new();
        // Int
        assert_eq!(parse_expr("123", &ctx).unwrap().1, Expr::Int(123));
        assert_eq!(parse_expr("-456", &ctx).unwrap().1, Expr::Int(-456));
        // Float
        match parse_expr("3.14", &ctx).unwrap().1 {
            Expr::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
        // String
        assert_eq!(
            parse_expr("\"hello\"", &ctx).unwrap().1,
            Expr::Str("hello".to_string())
        );
        // List
        // assert_eq!(parse_expr("(1 2)", &ctx).unwrap().1, Expr::List(vec![Expr::Int(1), Expr::Int(2)]));
    }
}
