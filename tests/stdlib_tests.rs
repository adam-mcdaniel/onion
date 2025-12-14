use onion::expr::Expr;
use onion::parser::parse_expr;
use onion::stdlib::stdlib;
use onion::context::eval;

fn run_code(code: &str) -> Expr {
    let mut ctx = stdlib();
    let mut input = code.trim();
    let mut last_res = Expr::Nil;

    while !input.is_empty() {
        match parse_expr(input, &ctx) {
            Ok((rest, expr)) => {
                last_res = eval(expr, &mut ctx);
                input = rest.trim();
            }
            Err(e) => panic!("Parse error at '{}': {:?}", input, e),
        }
    }
    last_res
}

fn assert_int(code: &str, expected: i64) {
    match run_code(code) {
        Expr::Int(n) => assert_eq!(n, expected, "Code: {}", code),
        val => panic!("Expected Int({}), got {:?} for code: {}", expected, val, code),
    }
}

fn assert_float(code: &str, expected: f64) {
    match run_code(code) {
        Expr::Float(f) => assert!((f - expected).abs() < 0.0001, "Expected {}, got {} for code: {}", expected, f, code),
        val => panic!("Expected Float({}), got {:?} for code: {}", expected, val, code),
    }
}

fn assert_str(code: &str, expected: &str) {
    match run_code(code) {
        Expr::Str(s) => assert_eq!(s, expected, "Code: {}", code),
        val => panic!("Expected Str({}), got {:?} for code: {}", expected, val, code),
    }
}

#[test]
fn test_math_module() {
    assert_float("Math.PI", std::f64::consts::PI);
    assert_int("(Math.abs (- 0 10))", 10);
    assert_float("(Math.sqrt 16.0)", 4.0);
    assert_int("(Math.max 1 2 5 3)", 5);
    assert_int("(Math.min 10 (- 0 2) 5)", -2);
}

#[test]
fn test_string_module() {
    assert_int("(String.len \"hello\")", 5);
    assert_str("(String.to_upper \"hello\")", "HELLO");
    assert_str("(String.substring \"hello world\" 0 5)", "hello");
    assert_str("(String.fmt \"Hello {}!\" \"World\")", "Hello World!");
    assert_int("(Math.abs (- 0 10))", 10);
}

#[test]
fn test_reflect_module() {
    // Type.of
    assert_str("(Type.of 123)", "int");
    assert_str("(Type.of \"s\")", "string");
    assert_str("(Type.of [])", "vector");
    assert_str("(Type.of #[])", "map");
    assert_int("(Type.to_int \"123\")", 123);
    assert_str("(Type.to_str 123)", "123");
}

fn assert_nil(code: &str) {
    match run_code(code) {
        Expr::Nil => {},
        val => panic!("Expected Nil, got {:?} for code: {}", val, code),
    }
}

#[test]
fn test_collections_module() {
    // List ops
    assert_str("(String.join (Collections.push [\"a\" \"b\"] \"c\") \",\")", "a,b,c");
    assert_int("(len (Collections.push [] 1))", 1);
    assert_int("(Collections.peek [1 2 3])", 3);
    
    // Reverse
    // We need to verify content.
    let code = "
    (def l [1 2 3])
    (def r (Collections.reverse l))
    (first r)
    ";
    assert_int(code, 3);
    
    // Sort
    let code = "
    (def l [3 1 2])
    (def s (Collections.sort l))
    (first s)
    ";
    assert_int(code, 1);
    
    // Map
    let code = "
    (def m #[ a 1 b 2 ])
    (len (Collections.keys m))
    ";
    assert_int(code, 2);
    
    assert_nil("(Collections.contains_key #[ a 1 ] \"a\")"); // Key "a"?
    // #[ a 1 ] -> Map { a: 1 }. Key is 'a (symbol) or "a" (string) if quoted?
    // parse_hash_map matches `#[` then `parse_sequence`. `a` is identifier -> Symbol `a`.
    // contains_key checks `Expr::Str("a")`.
    // Symbol("a") != Str("a").
    // So `contains_key` fails.
    // Fix: `(Collections.contains_key #[ a 1 ] 'a)`.
    // But we need 'a. `(quote a)`.
    // OR we use strings as keys: `#[ "a" 1 ]`.
    assert_int("(Collections.contains_key #[ \"a\" 1 ] \"a\")", 1);
    assert_int("(len #[ a 1 ])", 1);
}

#[test]
fn test_global_len() {
    assert_int("(len [1 2 3])", 3);
    assert_int("(len \"abc\")", 3);
    assert_int("(len #[ a 1 ])", 1);
}

#[test]
fn test_collections_map_filter() {
    let code = "
    (def l [1 2 3 4])
    (def res (Collections.map l (fun (x) (* x 2))))
    (first res)
    ";
    assert_int(code, 2); // 1*2 = 2

    let code = "
    (def l [1 2 3 4])
    (def res (Collections.filter l (fun (x) (> x 2))))
    (len res)
    ";
    assert_int(code, 2); // 3, 4
}
