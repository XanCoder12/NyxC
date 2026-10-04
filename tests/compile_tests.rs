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
    let prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&prog).is_ok());
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
    let prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&prog).is_ok());
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
    let prog = parse(src);
    let mut sema = SemanticAnalyzer::new();
    let err = sema.analyze_program(&prog).unwrap_err();
    assert_eq!(err.code, "SEM_001");
    let span = err.span.expect("semantic error must carry a span");
    assert_eq!(span.line, 1);
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
