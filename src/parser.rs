use crate::context::{Assoc, Context, OpInfo};
use crate::expr::Expr;
/// parser.rs
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, recognize, value},
    multi::many0,
    sequence::{delimited, tuple},
};

// --- Helpers ---

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

    // 1. Check if input starts with a known operator
    let ops = ctx.get_operator_keys();
    // Sort ops by length descending to match longest operator first (e.g. `==` before `=`)
    let mut sorted_ops = ops;
    sorted_ops.sort_by(|a, b| b.len().cmp(&a.len()));

    for op in &sorted_ops {
        if input.starts_with(op) {
            let first_c = op.chars().next().unwrap();
            // If op is alphanumeric, check boundary
            if first_c.is_alphanumeric() || first_c == '_' {
                if input.len() > op.len() {
                    let next_c = input[op.len()..].chars().next().unwrap();
                    if next_c.is_alphanumeric() || next_c == '_' {
                        continue; // Not a boundary, ignore this op match
                    }
                }
            }
            return Ok((&input[op.len()..], &input[..op.len()]));
        }
    }

    // 2. If not an operator, consume until we hit a delimiter or the START of an operator
    let mut i = 0;
    for (idx, c) in input.char_indices() {
        // Standard delimiters
        if c.is_whitespace() || "()[]{},\"".contains(c) {
            break;
        }

        // Check if this character starts any operator
        // Optimization: We could check if `input[idx..]` starts with any op.
        // But `x.y` -> parse `x`, stop at `.`.
        // If `.` is an operator, we stop.
        let suffix = &input[idx..];
        let mut found_op = false;
        for op in &sorted_ops {
            if suffix.starts_with(op) {
                // only break if op is punctuation (non-alphanumeric start)
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
        // Should have matched something if not empty and not delimiter start?
        // If input[0] is a delimiter like '(', we return error/Eof effectively
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeWhile1,
        )));
    }

    Ok((&input[i..], &input[..i]))
}

// --- Atom Parsers ---

fn parse_nil(input: &str) -> IResult<&str, Expr> {
    value(Expr::Nil, ws(tag("nil")))(input)
}

fn parse_int(input: &str) -> IResult<&str, Expr> {
    map_res(ws(digit1), |s: &str| s.parse::<i64>().map(Expr::Int))(input)
}

fn parse_float(input: &str) -> IResult<&str, Expr> {
    map_res(
        ws(recognize(tuple((digit1, char('.'), digit1)))),
        |s: &str| s.parse::<f64>().map(Expr::Float),
    )(input)
}

fn parse_str_lit(input: &str) -> IResult<&str, Expr> {
    let inner = is_not("\"");
    map(ws(delimited(char('"'), inner, char('"'))), |s: &str| {
        Expr::Str(s.to_string())
    })(input)
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

/// Helper for list-style sequences [ a b c ]
fn parse_sequence<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Vec<Expr>> {
    many0(|i| parse_expr(i, ctx))(input)
}

fn parse_list<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    // Matches [ ... ]
    let (input, _) = ws(tag("["))(input)?;

    // Handle empty []
    if let Ok((rest, _)) = ws(char::<&str, nom::error::Error<&str>>(']'))(input) {
        return Ok((rest, Expr::Vector(Vec::new())));
    }

    let (input, items) = parse_sequence(input, ctx)?;
    let (input, _) = ws(tag("]"))(input)?;

    Ok((input, Expr::Vector(items)))
}

fn parse_hash_map<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    let (input, _) = ws(tag("#["))(input)?;

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

fn parse_quoted<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    let (input, _) = ws(char('\''))(input)?;
    let (input, expr) = parse_expr(input, ctx)?;
    Ok((input, Expr::Quoted(Box::new(expr))))
}

// An "Atom" is anything that isn't an infix operation
fn parse_atom<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    alt((
        parse_nil,
        parse_float,
        parse_int,
        parse_str_lit,
        |i| parse_quoted(i, ctx),
        |i| parse_block(i, ctx),
        |i| parse_list(i, ctx),
        |i| parse_hash_map(i, ctx),
        |i| parse_parens(i, ctx),
        |i| parse_sym(i, ctx),
    ))(input)
}

pub fn parse_expr<'a>(input: &'a str, ctx: &Context) -> IResult<&'a str, Expr> {
    pratt_parse(input, ctx, 0)
}

/// Recursive Pratt Parser
fn pratt_parse<'a>(mut input: &'a str, ctx: &Context, min_bp: u8) -> IResult<&'a str, Expr> {
    // --- 1. PREFIX PHASE ---

    // Peek at the start to see if it is an operator
    let (mut input, mut lhs) = if let Ok((rest, sym)) = parse_symbol_str(input, ctx) {
        if let Some(op_info) = ctx.get_op(sym) {
            // Is it a defined UNARY operator?
            if op_info.unary {
                // Yes: Consume the operator symbol
                // Recurse with the operator's precedence (prefix usually binds right)
                // For right-associative prefix, we recurse with `precedence`.
                let (next_input, rhs) = pratt_parse(rest, ctx, op_info.precedence)?;

                (next_input, Expr::List(vec![Expr::Sym(sym.into()), rhs]))
            } else {
                // It's a defined operator, but NOT unary (e.g. "+" used at start).
                // Treat it as a standard atom (variable name)
                parse_atom(input, ctx)?
            }
        } else {
            // Not an operator. Parse as atom.
            parse_atom(input, ctx)?
        }
    } else {
        // Not a symbol (literal, parens, etc). Parse as atom.
        parse_atom(input, ctx)?
    };

    // --- 2. INFIX PHASE ---

    loop {
        // Peek at next token
        let peek_result = parse_symbol_str(input, ctx);

        if let Ok((rest_after_op, op_sym)) = peek_result {
            // Check if this symbol is a defined operator in our Context
            if let Some(op_info) = ctx.get_op(op_sym) {
                // If the operator found is Unary-only, it cannot be in infix position.
                // However, some ops are both (e.g. "-").
                // Standard Pratt: If it exists as infix, we process it.
                // Note: You might want a check here `if !op_info.infix { break; }` if you split definitions.
                // For now assuming OpInfo covers infix behavior if precedence > 0.

                if op_info.precedence < min_bp {
                    break;
                }

                let next_min_bp = match op_info.associativity {
                    Assoc::Left => op_info.precedence + 1,
                    Assoc::Right => op_info.precedence,
                };

                // Consume operator
                input = rest_after_op;

                // Parse RHS
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
        let mut ctx = Context::new(); // Note: Ensure Context::new() initializes parsing inner

        // Define Binary Ops
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

        // Define Unary Op
        ctx.define_op(
            "!",
            OpInfo {
                precedence: 3,
                associativity: Assoc::Right,
                unary: true,
            },
            Expr::Nil,
        );

        // Test 1: Standard Infix
        let input = "3 + 4 * 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("+".into()),
            Expr::Int(3),
            Expr::List(vec![Expr::Sym("*".into()), Expr::Int(4), Expr::Int(5)]),
        ]);
        assert_eq!(expr, expected);

        // Test 2: Unary Prefix
        let input = "! 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        // (! 5)
        let expected = Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]);
        assert_eq!(expr, expected);

        // Test 3: Mixed Unary and Infix
        // ! 5 + 3 -> ((! 5) + 3) because ! binds tighter (3) than + (1)
        let input = "! 5 + 3";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("+".into()),
            Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]),
            Expr::Int(3),
        ]);
        assert_eq!(expr, expected);

        // Test 4: Nested Unary
        // ! ! 5
        let input = "! ! 5";
        let (_, expr) = parse_expr(input, &ctx).unwrap();
        let expected = Expr::List(vec![
            Expr::Sym("!".into()),
            Expr::List(vec![Expr::Sym("!".into()), Expr::Int(5)]),
        ]);
        assert_eq!(expr, expected);
    }
}
