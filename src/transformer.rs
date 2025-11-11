//! AST → IR Transformer
//! Converts parsed GemFile (AST) into SceneIR (compile-time representation)

use crate::ast::{GemDecl, GemFile, Value};
use crate::ir::{NodeId, SceneIR, TypedProperty};
use crate::property_type::PropertyType;

pub struct Transformer {
    scene: SceneIR,
}

impl Transformer {
    pub fn new() -> Self {
        Self {
            scene: SceneIR::new(),
        }
    }

    pub fn transform(mut self, ast: GemFile) -> Result<SceneIR, String> {
        self.transform_gem_decl(&ast.root, None)?; // root becomes scene.root implicitly
        Ok(self.scene)
    }

    fn transform_gem_decl(
        &mut self,
        decl: &GemDecl,
        parent: Option<NodeId>,
    ) -> Result<NodeId, String> {
        let node_id = self.scene.add_node(&decl.name, &decl.gem_type);

        // Add properties with type inference
        for prop in &decl.properties {
            let value_str = self.value_to_string(&prop.value);
            let prop_type = PropertyType::infer(&value_str);
            self.scene.set_typed_property(node_id, &prop.key, value_str, prop_type);
        }

        // Attach to parent if provided
        if let Some(parent_id) = parent {
            self.scene.add_child(parent_id, node_id);
        }

        // Transform children
        for child in &decl.children {
            self.transform_gem_decl(child, Some(node_id))?;
        }

        Ok(node_id)
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            Value::Bool(b) => b.to_string(),
            Value::Tuple(vals) => {
                let items: Vec<String> = vals.iter().map(|v| self.value_to_string(v)).collect();
                format!("({})", items.join(", "))
            }
            Value::Directive(parts) => {
                format!("#{}", parts.join(":"))
            }
            Value::Ident(id) => id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn transform_simple_scene() {
        let ast = GemFile {
            root: GemDecl {
                name: "Root".to_string(),
                gem_type: "Gem".to_string(),
                properties: vec![Property {
                    key: "position".to_string(),
                    value: Value::Tuple(vec![Value::Integer(0), Value::Integer(0)]),
                }],
                children: vec![GemDecl {
                    name: "Child".to_string(),
                    gem_type: "Sprite".to_string(),
                    properties: vec![],
                    children: vec![],
                }],
            },
        };

        let transformer = Transformer::new();
        let scene = transformer.transform(ast).unwrap();

        assert_eq!(scene.nodes.len(), 2);
        assert!(scene.root.is_some());

        let root_id = scene.root.unwrap();
        let root = scene.nodes.get(&root_id).unwrap();
        assert_eq!(root.name, "Root");
        assert_eq!(root.class_name, "Gem");
        assert_eq!(root.children.len(), 1);

        let child_id = root.children[0];
        let child = scene.nodes.get(&child_id).unwrap();
        assert_eq!(child.name, "Child");
        assert_eq!(child.class_name, "Sprite");
    }
}
