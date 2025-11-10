mod ast;
mod display;
mod error;
mod lexer;
mod parser;
mod renderer;
mod token;

use display::GemDisplay;
use lexer::Lexer;
use parser::Parser;
use renderer::GemRenderer;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    println!("Gem Engine - Parser & Renderer Demo");

    let chosen_path = resolve_entry_scene_path();

    match fs::read_to_string(&chosen_path) {
        Ok(content) => {
            println!("\n=== Lexing: {} ===", chosen_path);
            let is_logic_file =
                chosen_path.contains("logic") || content.trim_start().starts_with("extend");

            let mut lexer = Lexer::new(content);
            match lexer.tokenize() {
                Ok(tokens) => {
                    println!("[INFO] Lexed {} tokens", tokens.len());

                    println!("\n=== Parsing ===");
                    let mut parser = Parser::new(tokens);

                    if is_logic_file {
                        match parser.parse_logic() {
                            Ok(ast) => {
                                println!("[INFO] Parsed logic file successfully!");
                                println!("\nAST:\n{:#?}", ast);
                                println!(
                                    "\n[INFO] Logic files don't launch renderer - parse only."
                                );
                            }
                            Err(e) => {
                                eprintln!("[ERR] Parse error: {}", e);
                            }
                        }
                    } else {
                        match parser.parse_scene() {
                            Ok(ast) => {
                                println!("[INFO] Parsed scene file successfully!");
                                println!("\n[INFO] Launching renderer...");

                                // Launch the renderer with the parsed scene
                                run_renderer(ast);
                            }
                            Err(e) => {
                                eprintln!("[ERR] Parse error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[ERR] Lexer error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", chosen_path, e);
            eprintln!(
                "\nNote: The tool auto-reads example/scenes.registry.gem if present,\nthen falls back to example/main_scene.gem."
            );
        }
    }
}

fn run_renderer(scene_ast: ast::GemFile) {
    println!("\n=== Initializing Renderer ===");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let display = GemDisplay::new(&event_loop, 800, 600, "Gem Engine - Scene Viewer");

    let renderer = GemRenderer::new(&display);

    println!(
        "[INFO] Scene root: {} : {}",
        scene_ast.root.name, scene_ast.root.gem_type
    );
    println!(
        "[INFO] Rendering scene with {} children",
        scene_ast.root.children.len()
    );

    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("[INFO] Window close requested");
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    display.resize(size.width, size.height);
                    renderer.set_viewport(size.width, size.height);
                }
                WindowEvent::RedrawRequested => {
                    renderer.begin_frame();

                    renderer.render_quad(0.0, 0.0, 0.5, 0.5, [0.2, 0.8, 0.4, 1.0]);

                    display.swap_buffers();
                }
                _ => {}
            },
            Event::AboutToWait => {
                display.window.request_redraw();
            }
            _ => {}
        }
    });
}

fn resolve_entry_scene_path() -> String {
    let registry_path = "example/scenes.registry.gem";
    if Path::new(registry_path).exists() {
        match fs::read_to_string(registry_path) {
            Ok(registry) => {
                if let Some(directive) = parse_registry_for_entry(&registry) {
                    let resolved = directive_to_path(&directive);
                    println!(
                        "Resolved entry from scenes.registry.gem => {} -> {}",
                        directive, resolved
                    );
                    return resolved;
                } else {
                    eprintln!(
                        "Warning: Could not find entry mapping in scenes.registry.gem; using example/main_scene.gem"
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read {}: {}. Falling back to example/main_scene.gem",
                    registry_path, e
                );
            }
        }
    }
    "example/main_scene.gem".to_string()
}

fn parse_registry_for_entry(contents: &str) -> Option<String> {
    let mut entry_name: Option<String> = None;
    let mut map: HashMap<String, String> = HashMap::new();

    for line in contents.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with("///") || t == "{" || t == "}" {
            continue;
        }
        if let Some(rest) = t.strip_prefix("entry:") {
            let name = rest.trim().split_whitespace().next()?.to_string();
            entry_name = Some(name);
            continue;
        }
        if let Some(colon_idx) = t.find(':') {
            let key = t[..colon_idx].trim();
            let rest = t[colon_idx + 1..].trim();
            if let Some(directive) = rest.strip_prefix('#') {
                // remove trailing comma or comments if any
                let directive = directive
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(',');
                if !key.is_empty() && !directive.is_empty() {
                    map.insert(key.to_string(), directive.to_string());
                }
            }
        }
    }

    let name = entry_name?;
    map.get(&name).cloned()
}

fn directive_to_path(directive: &str) -> String {
    let parts: Vec<&str> = directive.split(':').collect();
    let mut pb = PathBuf::new();
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // last segment: add .gem if missing an extension
            if !part.contains('.') {
                pb.push(format!("{}.gem", part));
            } else {
                pb.push(part);
            }
        } else {
            pb.push(part);
        }
    }
    pb.to_string_lossy().to_string()
}
