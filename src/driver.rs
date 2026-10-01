use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::codegen::generate_x86_assembly;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::sema::SemanticAnalyzer;
use crate::target::{Target, TargetOs};

pub struct CompileOptions {
    pub input_file: PathBuf,
    pub output_file: Option<PathBuf>,
    pub target: Target,
    pub emit_asm: bool,
    pub keep_intermediates: bool,
}

pub struct Driver;

impl Driver {
    pub fn compile(options: CompileOptions) -> Result<PathBuf, String> {
        let source = fs::read_to_string(&options.input_file)
            .map_err(|e| format!("Failed to read input file {:?}: {}", options.input_file, e))?;

        // 1. Lexer
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;

        // 2. Parser
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;

        // 3. Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze_program(&program)?;

        // 4. Codegen (x86 GNU as assembly)
        let asm_code = generate_x86_assembly(&program, &analyzer)?;

        let stem = options
            .input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("out");

        let parent_dir = options
            .input_file
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let asm_path = parent_dir.join(format!("{}.s", stem));
        fs::write(&asm_path, &asm_code)
            .map_err(|e| format!("Failed to write assembly output {:?}: {}", asm_path, e))?;

        if options.emit_asm {
            return Ok(asm_path);
        }

        // 5. Assemble via `as --32`
        let obj_path = parent_dir.join(format!("{}.o", stem));
        let as_status = Command::new("as")
            .arg("--32")
            .arg(&asm_path)
            .arg("-o")
            .arg(&obj_path)
            .status()
            .map_err(|e| format!("Failed to execute 'as --32': {}", e))?;

        if !as_status.success() {
            return Err("Assembler (as --32) reported errors".into());
        }

        // 6. Link via `ld -m elf_i386`
        let out_elf = options.output_file.unwrap_or_else(|| {
            parent_dir.join(format!("{}.elf", stem))
        });

        let mut ld_cmd = Command::new("ld");
        ld_cmd.arg("-m").arg("elf_i386");
        ld_cmd.arg("-nostdlib");

        // Write or locate linker script
        let ld_script_content = include_str!("../runtime/user.ld");
        let temp_ld = parent_dir.join(".temp_user.ld");
        if options.target.os == TargetOs::Nyxara {
            fs::write(&temp_ld, ld_script_content)
                .map_err(|e| format!("Failed to write temporary linker script: {}", e))?;
            ld_cmd.arg("-T").arg(&temp_ld);
        }

        ld_cmd.arg("-o").arg(&out_elf);
        ld_cmd.arg(&obj_path);

        let ld_status = ld_cmd
            .status()
            .map_err(|e| format!("Failed to execute 'ld -m elf_i386': {}", e))?;

        if temp_ld.exists() {
            let _ = fs::remove_file(temp_ld);
        }

        if !ld_status.success() {
            return Err("Linker (ld -m elf_i386) reported errors".into());
        }

        if !options.keep_intermediates {
            let _ = fs::remove_file(&asm_path);
            let _ = fs::remove_file(&obj_path);
        }

        Ok(out_elf)
    }
}
