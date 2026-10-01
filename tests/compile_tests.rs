use nyxc::driver::{CompileOptions, Driver};
use nyxc::lexer::Lexer;
use nyxc::parser::Parser;
use nyxc::sema::SemanticAnalyzer;
use nyxc::target::Target;
use std::fs;

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
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("tokenize failed");
    let mut parser = Parser::new(tokens);
    let prog = parser.parse_program().expect("parse failed");
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
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("tokenize failed");
    let mut parser = Parser::new(tokens);
    let prog = parser.parse_program().expect("parse failed");
    let mut sema = SemanticAnalyzer::new();
    assert!(sema.analyze_program(&prog).is_ok());
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
