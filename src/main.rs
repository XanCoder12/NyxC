use std::env;
use std::path::PathBuf;
use std::process;

use nyxc::driver::{CompileOptions, Driver};
use nyxc::lexer::Lexer;
use nyxc::parser::Parser;
use nyxc::sema::SemanticAnalyzer;
use nyxc::target::Target;

fn print_help() {
    println!("NyxC - Modern Systems Programming Language for NyxaraOS");
    println!("Usage:");
    println!("  nyxc build <file.nyx> [options]    Compile NyxC source to NyxaraOS ELF32");
    println!("  nyxc check <file.nyx>              Verify syntax and semantics without linking");
    println!("  nyxc emit-asm <file.nyx>           Compile NyxC source to x86-32 assembly (.s)");
    println!();
    println!("Options:");
    println!("  -o <path>                          Specify output binary path");
    println!("  --target <triple>                  Target triple (default: nyxara-x86)");
    println!("  -k, --keep-temps                   Keep intermediate .s and .o files");
    println!("  -h, --help                         Show help information");
    println!("  -v, --version                      Show version");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        process::exit(1);
    }

    let mut command = "build";
    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut target = Target::nyxara_x86();
    let mut emit_asm = false;
    let mut keep_intermediates = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("nyxc 0.1.0 (target: i386-nyxara)");
                return;
            }
            "build" | "compile" => {
                command = "build";
            }
            "check" => {
                command = "check";
            }
            "emit-asm" => {
                command = "build";
                emit_asm = true;
            }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: -o requires an output path");
                    process::exit(1);
                }
                output_file = Some(PathBuf::from(&args[i]));
            }
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --target requires a target name");
                    process::exit(1);
                }
                match Target::from_str(&args[i]) {
                    Ok(t) => target = t,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            }
            "-k" | "--keep-temps" => {
                keep_intermediates = true;
            }
            "--emit-asm" => {
                emit_asm = true;
            }
            _ => {
                if !arg.starts_with('-') && input_file.is_none() {
                    input_file = Some(PathBuf::from(arg));
                } else {
                    eprintln!("Unknown argument: {}", arg);
                    process::exit(1);
                }
            }
        }
        i += 1;
    }

    let input = match input_file {
        Some(path) => path,
        None => {
            eprintln!("Error: No input file specified");
            process::exit(1);
        }
    };

    if command == "check" {
        let source = match std::fs::read_to_string(&input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", input, e);
                process::exit(1);
            }
        };

        let mut lexer = Lexer::new(&source);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer error: {}", e);
                process::exit(1);
            }
        };

        let mut parser = Parser::new(tokens);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Parser error: {}", e);
                process::exit(1);
            }
        };

        let mut analyzer = SemanticAnalyzer::new();
        if let Err(e) = analyzer.analyze_program(&program) {
            eprintln!("Type/Semantic error: {}", e);
            process::exit(1);
        }

        println!("Check passed: {:?} is valid NyxC code.", input);
        return;
    }

    let options = CompileOptions {
        input_file: input,
        output_file,
        target,
        emit_asm,
        keep_intermediates,
    };

    match Driver::compile(options) {
        Ok(out) => {
            println!("Compiled successfully -> {:?}", out);
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}
