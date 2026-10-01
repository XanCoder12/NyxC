pub mod runtime;
pub mod x86_gas;

use crate::ast::Program;
use crate::sema::SemanticAnalyzer;
use self::x86_gas::X86GasCodegen;

pub fn generate_x86_assembly(program: &Program, analyzer: &SemanticAnalyzer) -> Result<String, String> {
    let mut codegen = X86GasCodegen::new(analyzer);
    codegen.generate(program)
}
