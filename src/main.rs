mod error;
mod lexer;
mod token;

use lexer::Lexer;
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn main() {
    println!("Gem lexer demo");

    // Auto-load entry: prefer scenes.registry.gem; fallback to example/main_scene.gem
    let chosen_path = resolve_entry_scene_path();

    match fs::read_to_string(&chosen_path) {
        Ok(content) => {
            println!("Tokenizing: {}", chosen_path);
            let mut lexer = Lexer::new(content);
            match lexer.tokenize() {
                Ok(tokens) => {
                    println!("Tokens:");
                    for (i, token) in tokens.iter().enumerate() {
                        println!("  {}: {:?}", i, token);
                    }
                }
                Err(e) => {
                    println!("Lexer error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error reading file {}: {}", chosen_path, e);
            eprintln!(
                "\nNote: The tool auto-reads example/scenes.registry.gem if present,\nthen falls back to example/main_scene.gem."
            );
        }
    }
}

fn resolve_entry_scene_path() -> String {
    let registry_path = "example/scenes.registry.gem";
    // If registry exists, try to resolve entry scene path from it
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
    // Convert e.g., "example:main_scene.gem" or "example:logic:player_logic" to path
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
