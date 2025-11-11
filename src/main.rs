mod ast;
mod codegen;
mod display;
mod error;
mod gem;
mod ir;
mod lexer;
mod object;
mod parser;
mod pipeline;
mod property_type;
mod renderer;
mod token;
mod transformer;
mod value;

use display::GemDisplay;
use pipeline::compile_scene;
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

            if is_logic_file {
                match pipeline::lex_source(&content).and_then(pipeline::parse_logic) {
                    Ok(ast) => {
                        println!("[INFO] Parsed logic file successfully!");
                        println!("\nAST:\n{:#?}", ast);
                        println!("\n[INFO] Logic files don't launch renderer - parse only.");
                    }
                    Err(e) => eprintln!("[ERR] Logic parse error: {}", e),
                }
            } else {
                // Determine root directory (folder containing scenes.registry.gem if present), then write to <root>/gen/<relative>.rs
                let root_dir = find_root_dir().unwrap_or_else(|| {
                    Path::new(&chosen_path)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."))
                });
                let relative = Path::new(&chosen_path)
                    .strip_prefix(&root_dir)
                    .unwrap_or_else(|_| Path::new(&chosen_path))
                    .to_path_buf();
                let mut out_path = root_dir.join(".gen").join(&relative);
                out_path.set_extension("rs");
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                match compile_scene(&content, &out_path.to_string_lossy()) {
                    Ok(result) => {
                        // Optionally launch renderer for preview
                        println!("\n[INFO] Launching renderer for preview...");
                        run_renderer(result.ast);
                    }
                    Err(e) => eprintln!("[ERR] Compile error: {}", e),
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

// Return the directory containing scenes.registry.gem if it exists.
fn find_root_dir() -> Option<PathBuf> {
    let registry_path = Path::new("example/scenes.registry.gem");
    if registry_path.exists() {
        return registry_path.parent().map(|p| p.to_path_buf());
    }
    None
}

fn run_renderer(scene_ast: ast::GemFile) {
    println!("\n=== Initializing Renderer ===");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let display = GemDisplay::new(&event_loop, 800, 600, "Gem Engine - Scene Viewer");

    let mut renderer = GemRenderer::new(&display);

    // Simple draw command representing a quad to render
    #[derive(Clone, Copy, Debug)]
    struct DrawCmd {
        x: f32, // pixels
        y: f32, // pixels
        w: f32, // pixels
        h: f32, // pixels
        color: [f32; 4],
    }

    // Extract draw commands from AST nodes (placeholder visuals)
    fn gather_draws(decl: &ast::GemDecl, out: &mut Vec<DrawCmd>) {
        // Defaults
        let mut pos: (f32, f32) = (100.0, 100.0);
        let mut size: (f32, f32) = (160.0, 48.0);
        let mut color: [f32; 4] = [0.6, 0.6, 0.6, 1.0];

        // Basic property parsing for position
        for p in &decl.properties {
            if p.key == "position" {
                if let ast::Value::Tuple(vals) = &p.value {
                    if vals.len() >= 2 {
                        let x = match &vals[0] {
                            ast::Value::Integer(i) => *i as f32,
                            ast::Value::Number(n) => *n as f32,
                            _ => 0.0,
                        };
                        let y = match &vals[1] {
                            ast::Value::Integer(i) => *i as f32,
                            ast::Value::Number(n) => *n as f32,
                            _ => 0.0,
                        };
                        pos = (x, y);
                    }
                }
            }
        }

        // Color/size by gem type (temporary placeholders)
        match decl.gem_type.as_str() {
            "LabelGem" => {
                color = [0.7, 0.2, 0.8, 1.0];
                size = (260.0, 40.0);
            }
            "ButtonGem" => {
                color = [0.2, 0.7, 0.3, 1.0];
                size = (200.0, 56.0);
            }
            _ => { /* keep defaults */ }
        }

        if decl.gem_type != "Gem" {
            out.push(DrawCmd {
                x: pos.0,
                y: pos.1,
                w: size.0,
                h: size.1,
                color,
            });
        }

        for c in &decl.children {
            gather_draws(c, out);
        }
    }

    // Precompute draw list from AST (static for now)
    let mut draws: Vec<DrawCmd> = Vec::new();
    gather_draws(&scene_ast.root, &mut draws);

    // Track framebuffer size for pixel-space to NDC conversion
    let mut fb_w: f32 = 800.0;
    let mut fb_h: f32 = 600.0;

    println!(
        "[INFO] Scene root: {} : {}",
        scene_ast.root.name, scene_ast.root.gem_type
    );
    println!(
        "[INFO] Rendering scene with {} children",
        scene_ast.root.children.len()
    );
    println!("[INFO] Draw commands: {} quads", draws.len());
    for (i, d) in draws.iter().enumerate() {
        println!(
            "  [{}] pos=({:.1},{:.1}) size=({:.1}x{:.1}) color={:?}",
            i, d.x, d.y, d.w, d.h, d.color
        );
    }

    #[allow(deprecated)]
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
                    fb_w = size.width as f32;
                    fb_h = size.height as f32;
                }
                WindowEvent::RedrawRequested => {
                    renderer.begin_frame();

                    for d in &draws {
                        // Convert pixel coordinates to NDC (-1 to 1)
                        // Center position in pixels
                        let cx_px = d.x + d.w * 0.5;
                        let cy_px = d.y + d.h * 0.5;

                        // Convert to NDC: map [0, fb_w] to [-1, 1] and [0, fb_h] to [1, -1] (Y inverted)
                        let cx_ndc = (cx_px / fb_w) * 2.0 - 1.0;
                        let cy_ndc = -((cy_px / fb_h) * 2.0 - 1.0);

                        // Convert size to NDC scale
                        let w_ndc = d.w / fb_w * 2.0;
                        let h_ndc = d.h / fb_h * 2.0;

                        renderer.render_quad(cx_ndc, cy_ndc, w_ndc, h_ndc, d.color);
                    }

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
