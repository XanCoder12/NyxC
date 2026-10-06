use nyxc::driver::{CompileOptions, Driver};
use nyxc::lexer::Lexer;
use nyxc::parser::Parser;
use nyxc::sema::SemanticAnalyzer;
use nyxc::target::Target;
use std::fs;

fn parse(src: &str) -> nyxc::ast::Program {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("tokenize failed");
    let mut parser = Parser::new(tokens);
    parser.parse_program().expect("parse failed")
}

#[test]
fn test_lexer_tokens() {
    let src = "let mut x: i32 = 42\nreturn x";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexer failed");
    assert!(tokens.len() >= 6);
}

#[test]
fn test_parser_basic_function() {
    let src = r#"
        import "nyx/sys"
        fn test_func() -> i32 {
            let a: i32 = 5
            let b: i32 = 10
            return a + b
        }
    "#;
    let prog = parse(src);
    assert_eq!(prog.items.len(), 2);
}

#[test]
fn test_semantic_analysis() {
    let src = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b
        }
        fn main() -> i32 {
            return add(10, 20)
        }
    "#;
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&mut prog).is_ok());
}

#[test]
fn test_let_type_inference() {
    let src = r#"
        fn main() -> i32 {
            let a = 5
            let mut b = 10
            let s = "hi"
            b = a + b
            return b
        }
    "#;
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&mut prog).is_ok());
}

#[test]
fn test_const_inferred_type_parses() {
    let prog = parse("const ANSWER = 42\n");
    match &prog.items[0] {
        nyxc::ast::Item::Const(nyxc::ast::Stmt::Const { ty, .. }) => {
            assert!(ty.is_none(), "const type should be inferred, not explicit");
        }
        _ => panic!("expected a const item"),
    }
}

#[test]
fn test_const_explicit_type_still_parses() {
    let prog = parse("const ANSWER: i32 = 42\n");
    match &prog.items[0] {
        nyxc::ast::Item::Const(nyxc::ast::Stmt::Const { ty, .. }) => {
            assert!(matches!(ty.as_ref(), Some(nyxc::ast::Type::I32)));
        }
        _ => panic!("expected a const item"),
    }
}

#[test]
fn test_semantic_error_carries_span() {
    let src = "fn main() -> i32 { let a = missing }\n";
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    let err = sema.analyze_program(&mut prog).unwrap_err();
    assert_eq!(err.code, "SEM_001");
    let span = err.span.expect("semantic error must carry a span");
    assert_eq!(span.line, 1);
}

#[test]
fn test_const_usage() {
    let src = r#"
        const ANSWER = 42
        fn main() -> i32 {
            return ANSWER
        }
    "#;
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&mut prog).is_ok());
}

#[test]
fn test_mutability_enforced() {
    let src = "fn main() -> i32 { let x = 10; x = 20; return x }\n";
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    let err = sema.analyze_program(&mut prog).unwrap_err();
    assert_eq!(err.code, "SEM_004");
}

#[test]
fn test_for_loop_with_let() {
    let src = r#"
        fn main() -> i32 {
            let mut sum = 0
            for (let mut i = 0; i < 10; i = i + 1) {
                sum = sum + i
            }
            return sum
        }
    "#;
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&mut prog).is_ok());
}

#[test]
fn test_parser_error_carries_span() {
    let mut lexer = Lexer::new("fn {");
    let tokens = lexer.tokenize().expect("tokenize failed");
    let mut parser = Parser::new(tokens);
    let err = parser.parse_program().unwrap_err();
    assert!(err.span.is_some(), "parser error must carry a span");
}

#[test]
fn test_str_len_e2e_elf() {
    fs::create_dir_all("target").expect("failed to create target dir");
    let test_file = "target/str_len_test.nyx";
    let src = r#"
        import "nyx/sys"

        fn main() -> i32 {
            sys::write(1, "str_len ok\n", str_len("str_len ok\n"))
            return 0
        }
    "#;
    fs::write(test_file, src).expect("failed to write test file");

    let options = CompileOptions {
        input_file: test_file.into(),
        output_file: Some("target/str_len_test.elf".into()),
        target: Target::nyxara_x86(),
        emit_asm: false,
        keep_intermediates: true,
    };

    let result = Driver::compile(options);
    assert!(result.is_ok(), "Driver failed: {:?}", result.err());
    assert!(std::path::Path::new("target/str_len_test.elf").exists());

    let asm = fs::read_to_string("target/str_len_test.s").expect("failed to read .s");
    assert!(asm.contains("cmpb [ebx + ecx], 0"), "inline strlen missing from asm");
}

#[test]
fn test_compile_e2e_elf() {
    let test_file = "target/test_program.nyx";
    let src = r#"
        import "nyx/sys"

        fn main() -> i32 {
            let x: i32 = 1
            if x == 1 {
                sys::write(1, "Test OK\n", 8)
            }
            return 0
        }
    "#;
    fs::write(test_file, src).expect("failed to write test file");

    let options = CompileOptions {
        input_file: test_file.into(),
        output_file: Some("target/test_program.elf".into()),
        target: Target::nyxara_x86(),
        emit_asm: false,
        keep_intermediates: true,
    };

    let result = Driver::compile(options);
    assert!(result.is_ok(), "Driver failed: {:?}", result.err());
    assert!(std::path::Path::new("target/test_program.elf").exists());
}

#[test]
fn test_itoa_asm_generation() {
    let test_file = "target/itoa_test.nyx";
    let src = r#"
        import "nyx/sys"

        fn main() -> i32 {
            let s = itoa(42)
            sys::write(1, s, 2)
            return 0
        }
    "#;
    fs::write(test_file, src).expect("failed to write test file");

    let options = CompileOptions {
        input_file: test_file.into(),
        output_file: Some("target/itoa_test.elf".into()),
        target: Target::nyxara_x86(),
        emit_asm: false,
        keep_intermediates: true,
    };

    let result = Driver::compile(options);
    assert!(result.is_ok(), "Driver failed: {:?}", result.err());
    let asm = fs::read_to_string("target/itoa_test.s").expect("failed to read .s");
    assert!(asm.contains(".nyx_itoa:"), "itoa routine missing");
    assert!(asm.contains("div ebx"), "proper division missing from itoa");
}

#[test]
fn test_emit_asm_custom_output() {
    let test_file = "target/custom_asm_test.nyx";
    let out_file = "target/custom_dir/my_asm.s";
    let src = "fn main() -> i32 { return 0 }\n";
    fs::write(test_file, src).expect("failed to write test file");

    let options = CompileOptions {
        input_file: test_file.into(),
        output_file: Some(out_file.into()),
        target: Target::nyxara_x86(),
        emit_asm: true,
        keep_intermediates: false,
    };

    let result = Driver::compile(options);
    assert!(result.is_ok(), "Driver failed: {:?}", result.err());
    assert_eq!(result.unwrap(), std::path::PathBuf::from(out_file));
    assert!(std::path::Path::new(out_file).exists());
}

#[test]
fn test_cannot_assign_to_const() {
    let src = "const N = 100\nfn main() -> i32 { N = 200; return N }\n";
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    let err = sema.analyze_program(&mut prog).unwrap_err();
    assert_eq!(err.code, "SEM_004");
}

#[test]
fn test_addr_of_invalid_expr() {
    let src = "fn main() -> i32 { let p = &(1 + 2); return 0 }\n";
    let mut prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    let err = sema.analyze_program(&mut prog).unwrap_err();
    assert_eq!(err.code, "SEM_013");
}
