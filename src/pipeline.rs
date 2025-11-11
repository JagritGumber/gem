use crate::ast;
use crate::codegen;
use crate::ir::SceneIR;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;
use crate::transformer::Transformer;
use std::fs;

pub struct SceneCompileResult {
    pub ast: ast::GemFile,
    pub ir: SceneIR,
    pub generated_path: String,
}

pub fn lex_source(content: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(content.to_string());
    lexer.tokenize().map_err(|e| e.to_string())
}

pub fn parse_scene(tokens: Vec<Token>) -> Result<ast::GemFile, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_scene().map_err(|e| e.message)
}

/// Parse a logic file from tokens.
pub fn parse_logic(tokens: Vec<Token>) -> Result<ast::LogicFile, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_logic().map_err(|e| e.message)
}

/// Compile scene content end-to-end: lex -> parse -> transform -> codegen -> write file.
/// Returns AST + IR + output path on success.
pub fn compile_scene(content: &str, output_path: &str) -> Result<SceneCompileResult, String> {
    println!("\n=== Lexing ===");
    let tokens = lex_source(content)?;
    println!("[INFO] Lexed {} tokens", tokens.len());

    println!("\n=== Parsing ===");
    let ast = parse_scene(tokens)?;
    println!("[INFO] Parsed scene file successfully!");

    println!("\n=== Transforming to IR ===");
    let transformer = Transformer::new();
    let ir = transformer.transform(ast.clone())?;
    println!("[INFO] Transformed to IR: {} nodes", ir.nodes.len());

    println!("\n=== Generating Rust Code ===");
    let codegen = codegen::RustCodegen::new();
    let rust_code = codegen.generate(&ir);

    std::fs::create_dir_all("build").ok();
    match fs::write(output_path, &rust_code) {
        Ok(_) => println!("[INFO] Generated Rust code → {}", output_path),
        Err(e) => return Err(format!("Failed to write {}: {}", output_path, e)),
    }

    println!(
        "\n[INFO] Compilation complete!\n      Generated: {}",
        output_path
    );
    Ok(SceneCompileResult {
        ast,
        ir,
        generated_path: output_path.to_string(),
    })
}
