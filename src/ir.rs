//! Intermediate Representation (IR) for Gem compiler.
//! These structures are compile-time only: no runtime mutation, no Arc/RwLock.
//! They model classes (Object/Gem), nodes, and scenes similar to Godot's Node tree.

use crate::property_type::PropertyType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SignalDecl {
    pub name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PropertyDecl {
    pub name: String,
    pub default: Option<String>, // store literal repr for now
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub properties: Vec<PropertyDecl>,
    pub methods: Vec<MethodSig>,
    pub signals: Vec<SignalDecl>,
    pub base: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone)]
pub struct TypedProperty {
    pub value: String, // literal value
    pub prop_type: PropertyType,
}

#[derive(Debug, Clone)]
pub struct NodeIR {
    pub id: NodeId,
    pub name: String,
    pub class_name: String, // e.g. "Gem" or future specialized classes
    pub properties: HashMap<String, TypedProperty>, // typed properties
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl NodeIR {
    pub fn new(id: NodeId, name: impl Into<String>, class_name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            class_name: class_name.into(),
            properties: HashMap::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SceneIR {
    pub nodes: HashMap<NodeId, NodeIR>,
    pub root: Option<NodeId>,
    next_id: u32,
}

impl SceneIR {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
        }
    }

    pub fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }

    pub fn add_node(&mut self, name: impl Into<String>, class_name: impl Into<String>) -> NodeId {
        let id = self.alloc_id();
        let node = NodeIR::new(id, name, class_name);
        if self.root.is_none() {
            self.root = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.push(child);
        }
        if let Some(c) = self.nodes.get_mut(&child) {
            c.parent = Some(parent);
        }
    }

    pub fn set_property(&mut self, node: NodeId, key: impl Into<String>, value: impl Into<String>) {
        let value_str = value.into();
        let prop_type = PropertyType::infer(&value_str);
        if let Some(n) = self.nodes.get_mut(&node) {
            n.properties.insert(
                key.into(),
                TypedProperty {
                    value: value_str,
                    prop_type,
                },
            );
        }
    }

    pub fn set_typed_property(
        &mut self,
        node: NodeId,
        key: impl Into<String>,
        value: impl Into<String>,
        prop_type: PropertyType,
    ) {
        if let Some(n) = self.nodes.get_mut(&node) {
            n.properties.insert(
                key.into(),
                TypedProperty {
                    value: value.into(),
                    prop_type,
                },
            );
        }
    }

    pub fn get_path(&self, node: NodeId) -> Option<String> {
        let mut cur = self.nodes.get(&node)?;
        let mut segments = vec![cur.name.clone()];
        while let Some(pid) = cur.parent {
            cur = self.nodes.get(&pid)?;
            segments.push(cur.name.clone());
        }
        segments.reverse();
        Some(format!("/{}", segments.join("/")))
    }

    pub fn find_by_path(&self, path: &str) -> Option<NodeId> {
        if path.is_empty() {
            return None;
        }
        let trimmed = path.trim();
        let parts: Vec<&str> = trimmed
            .trim_start_matches('/')
            .split('/')
            .filter(|p| !p.is_empty())
            .collect();
        let mut current = self.root?;
        let mut idx = 0usize;
        // root must match first segment
        if let Some(root_node) = self.nodes.get(&current) {
            if root_node.name != parts[0] {
                return None;
            }
        }
        idx += 1;
        while idx < parts.len() {
            let seg = parts[idx];
            let mut found: Option<NodeId> = None;
            if let Some(cur_node) = self.nodes.get(&current) {
                for cid in &cur_node.children {
                    if let Some(child) = self.nodes.get(cid) {
                        if child.name == seg {
                            found = Some(*cid);
                            break;
                        }
                    }
                }
            }
            current = found?;
            idx += 1;
        }
        Some(current)
    }
}
