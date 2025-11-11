use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::object::{ObjectInner, ObjectRef, register_class};
use crate::value::Value;

const NAME_KEY: &str = "name";

#[derive(Clone, Default)]
struct GemPrivate {
    parent: Option<ObjectRef>,
    children: Vec<ObjectRef>,
    in_tree: bool,
    groups: Vec<String>,
}

static GEM_PRIV: OnceLock<RwLock<HashMap<u64, GemPrivate>>> = OnceLock::new();

fn priv_map() -> &'static RwLock<HashMap<u64, GemPrivate>> {
    GEM_PRIV.get_or_init(|| RwLock::new(HashMap::new()))
}
fn init_priv_for(node: &ObjectRef) {
    let mut m = priv_map().write().unwrap();
    m.entry(node.id()).or_insert_with(GemPrivate::default);
}
fn with_priv<R>(node: &ObjectRef, f: impl FnOnce(&mut GemPrivate) -> R) -> R {
    let mut m = priv_map().write().unwrap();
    let p = m.entry(node.id()).or_insert_with(GemPrivate::default);
    f(p)
}
fn read_priv<R>(node: &ObjectRef, f: impl FnOnce(&GemPrivate) -> R) -> R {
    let m = priv_map().read().unwrap();
    let p = m.get(&node.id()).unwrap();
    f(p)
}

pub fn init_gem_class() {
    register_class("Gem", || {
        let obj = ObjectInner::base("Gem");
        obj.set_property(NAME_KEY, Value::String("Gem".into()));
        init_priv_for(&obj);

        // methods
        ObjectInner::insert_method(
            &obj,
            "get_name",
            Arc::new(|this, _| {
                Ok(this
                    .get_property(NAME_KEY)
                    .unwrap_or(Value::String("".into())))
            }),
        );
        ObjectInner::insert_method(
            &obj,
            "set_name",
            Arc::new(|this, args| {
                if let Some(Value::String(s)) = args.get(0).cloned() {
                    this.set_property(NAME_KEY, Value::String(s));
                }
                Ok(Value::Null)
            }),
        );
        // add_child(child)
        ObjectInner::insert_method(
            &obj,
            "add_child",
            Arc::new(|this, args| {
                let child = match args.get(0) {
                    Some(Value::Object(o)) => o.clone(),
                    _ => return Err("add_child expects Object".into()),
                };
                with_priv(this, |p| p.children.push(child.clone()));
                with_priv(&child, |cp| cp.parent = Some(this.clone()));
                if is_in_tree(this) {
                    enter_tree_recursive(&child);
                }
                this.emit_signal("child_entered_tree", &[]);
                Ok(Value::Null)
            }),
        );
        // remove_child(child)
        ObjectInner::insert_method(
            &obj,
            "remove_child",
            Arc::new(|this, args| {
                let target = match args.get(0) {
                    Some(Value::Object(o)) => o.clone(),
                    _ => return Err("remove_child expects Object".into()),
                };
                let mut removed = false;
                with_priv(this, |p| {
                    if let Some(pos) = p.children.iter().position(|o| o.id() == target.id()) {
                        p.children.remove(pos);
                        removed = true;
                    }
                });
                if removed {
                    with_priv(&target, |cp| cp.parent = None);
                    if is_in_tree(this) {
                        exit_tree_recursive(&target);
                    }
                    this.emit_signal("child_exited_tree", &[]);
                }
                Ok(Value::Bool(removed))
            }),
        );
        // get_parent()
        ObjectInner::insert_method(
            &obj,
            "get_parent",
            Arc::new(|this, _| {
                let p = read_priv(this, |pr| pr.parent.clone());
                Ok(p.map(Value::Object).unwrap_or(Value::Null))
            }),
        );
        // get_children()
        ObjectInner::insert_method(
            &obj,
            "get_children",
            Arc::new(|this, _| {
                let arr = read_priv(this, |p| {
                    p.children
                        .iter()
                        .cloned()
                        .map(Value::Object)
                        .collect::<Vec<_>>()
                });
                Ok(Value::Array(arr))
            }),
        );
        // get_child_count()
        ObjectInner::insert_method(
            &obj,
            "get_child_count",
            Arc::new(|this, _| {
                let len = read_priv(this, |p| p.children.len() as i64);
                Ok(Value::Int(len))
            }),
        );
        // get_child(index)
        ObjectInner::insert_method(
            &obj,
            "get_child",
            Arc::new(|this, args| {
                let idx = match args.get(0) {
                    Some(Value::Int(i)) => *i as usize,
                    _ => return Err("get_child expects index (int)".into()),
                };
                if let Some(o) = read_priv(this, |p| p.children.get(idx).cloned()) {
                    return Ok(Value::Object(o));
                }
                Ok(Value::Null)
            }),
        );
        // get_node(path), has_node(path)
        ObjectInner::insert_method(
            &obj,
            "has_node",
            Arc::new(|this, args| {
                let path = match args.get(0) {
                    Some(Value::String(s)) => s,
                    _ => return Err("has_node expects path string".into()),
                };
                Ok(Value::Bool(get_node_by_path(this, path).is_some()))
            }),
        );
        ObjectInner::insert_method(
            &obj,
            "get_node",
            Arc::new(|this, args| {
                let path = match args.get(0) {
                    Some(Value::String(s)) => s,
                    _ => return Err("get_node expects path string".into()),
                };
                if let Some(node) = get_node_by_path(this, path) {
                    Ok(Value::Object(node))
                } else {
                    Ok(Value::Null)
                }
            }),
        );

        // get_index()
        ObjectInner::insert_method(
            &obj,
            "get_index",
            Arc::new(|this, _| {
                let parent = read_priv(this, |p| p.parent.clone());
                if let Some(p) = parent {
                    let idx = read_priv(&p, |pp| {
                        pp.children
                            .iter()
                            .position(|o| o.id() == this.id())
                            .map(|i| i as i64)
                            .unwrap_or(-1)
                    });
                    return Ok(Value::Int(idx));
                }
                Ok(Value::Int(-1))
            }),
        );

        // move_child(child, to_position)
        ObjectInner::insert_method(
            &obj,
            "move_child",
            Arc::new(|this, args| {
                let child_id = match args.get(0) {
                    Some(Value::Object(o)) => o.id(),
                    _ => return Err("move_child expects (child, to_position)".into()),
                };
                let to_pos = match args.get(1) {
                    Some(Value::Int(i)) => *i as usize,
                    _ => return Err("move_child expects (child, to_position)".into()),
                };
                let mut moved = false;
                with_priv(this, |p| {
                    if let Some(pos) = p.children.iter().position(|o| o.id() == child_id) {
                        let val = p.children.remove(pos);
                        let insert_at = to_pos.min(p.children.len());
                        p.children.insert(insert_at, val);
                        moved = true;
                    }
                });
                if moved {
                    return Ok(Value::Bool(true));
                }
                Ok(Value::Bool(false))
            }),
        );

        // is_inside_tree()
        ObjectInner::insert_method(
            &obj,
            "is_inside_tree",
            Arc::new(|this, _| Ok(Value::Bool(is_in_tree(this)))),
        );

        // get_path(), get_path_to(node)
        ObjectInner::insert_method(
            &obj,
            "get_path",
            Arc::new(|this, _| Ok(Value::String(get_path(this)))),
        );
        ObjectInner::insert_method(
            &obj,
            "get_path_to",
            Arc::new(|this, args| {
                let target = match args.get(0) {
                    Some(Value::Object(o)) => o,
                    _ => return Err("get_path_to expects node".into()),
                };
                Ok(Value::String(get_path_to(this, target)))
            }),
        );

        // Groups API: add_to_group(name), is_in_group(name), remove_from_group(name), get_groups()
        ObjectInner::insert_method(
            &obj,
            "add_to_group",
            Arc::new(|this, args| {
                let name = match args.get(0) {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("add_to_group expects name".into()),
                };
                with_priv(this, |p| {
                    if !p.groups.iter().any(|g| g == &name) {
                        p.groups.push(name);
                    }
                });
                Ok(Value::Null)
            }),
        );
        ObjectInner::insert_method(
            &obj,
            "is_in_group",
            Arc::new(|this, args| {
                let name = match args.get(0) {
                    Some(Value::String(s)) => s,
                    _ => return Err("is_in_group expects name".into()),
                };
                let found = read_priv(this, |p| p.groups.iter().any(|g| g == name));
                Ok(Value::Bool(found))
            }),
        );
        ObjectInner::insert_method(
            &obj,
            "remove_from_group",
            Arc::new(|this, args| {
                let name = match args.get(0) {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err("remove_from_group expects name".into()),
                };
                with_priv(this, |p| p.groups.retain(|g| g != &name));
                Ok(Value::Null)
            }),
        );
        ObjectInner::insert_method(
            &obj,
            "get_groups",
            Arc::new(|this, _| {
                let v = read_priv(this, |p| {
                    p.groups
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect::<Vec<_>>()
                });
                Ok(Value::Array(v))
            }),
        );

        // signals set is dynamic; document: child_entered_tree, child_exited_tree, tree_entered, tree_exiting, ready
        obj
    });
}

fn is_in_tree(node: &ObjectRef) -> bool {
    read_priv(node, |p| p.in_tree)
}

fn root_of(node: &ObjectRef) -> ObjectRef {
    let mut cur = node.clone();
    loop {
        let parent = read_priv(&cur, |p| p.parent.clone());
        match parent {
            Some(p) => cur = p,
            None => break,
        }
    }
    cur
}

fn get_path(node: &ObjectRef) -> String {
    // absolute path from root using names
    let mut names = vec![];
    let mut cur = node.clone();
    loop {
        let name = match cur.get_property(NAME_KEY) {
            Some(Value::String(s)) => s,
            _ => "".to_string(),
        };
        names.push(name);
        let parent = read_priv(&cur, |p| p.parent.clone());
        match parent {
            Some(p) => cur = p,
            None => break,
        }
    }
    names.reverse();
    format!("/{}", names.join("/"))
}

fn get_path_to(from: &ObjectRef, to: &ObjectRef) -> String {
    let a = get_path(from);
    let b = get_path(to);
    // if a is prefix of b, return relative tail, else return b
    if b.starts_with(&a) {
        let tail = b[a.len()..].trim_start_matches('/').to_string();
        return tail;
    }
    b
}

fn get_node_by_path(from: &ObjectRef, path: &str) -> Option<ObjectRef> {
    if path.is_empty() {
        return None;
    }
    let start = if path.starts_with('/') {
        root_of(from)
    } else {
        from.clone()
    };
    let mut cur = start;
    for seg in path.split('/') {
        if seg.is_empty() {
            continue;
        }
        // find child by name
        let children = read_priv(&cur, |p| p.children.clone());
        let mut found: Option<ObjectRef> = None;
        for o in children {
            if let Some(Value::String(n)) = o.get_property(NAME_KEY) {
                if n == seg {
                    found = Some(o);
                    break;
                }
            }
        }
        if let Some(n) = found {
            cur = n;
        } else {
            return None;
        }
    }
    Some(cur)
}

fn enter_tree_recursive(node: &ObjectRef) {
    with_priv(node, |p| p.in_tree = true);
    node.emit_signal("tree_entered", &[]);
    let children = read_priv(node, |p| p.children.clone());
    for o in children {
        enter_tree_recursive(&o);
    }
}

fn exit_tree_recursive(node: &ObjectRef) {
    let children = read_priv(node, |p| p.children.clone());
    for o in children {
        exit_tree_recursive(&o);
    }
    node.emit_signal("tree_exiting", &[]);
    with_priv(node, |p| p.in_tree = false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{init_object_class, object_new};
    use crate::value::Value;

    #[test]
    fn gem_add_child() {
        init_object_class();
        init_gem_class();

        let root = object_new("Gem");
        root.call_method("set_name", &[Value::String("Root".into())])
            .unwrap();
        let child = object_new("Gem");
        child
            .call_method("set_name", &[Value::String("Child".into())])
            .unwrap();

        root.call_method("add_child", &[Value::Object(child.clone())])
            .unwrap();

        let cnt = root.call_method("get_child_count", &[]).unwrap();
        match cnt {
            Value::Int(n) => assert_eq!(n, 1),
            _ => panic!("expected int"),
        }

        let got = root
            .call_method("get_node", &[Value::String("Child".into())])
            .unwrap();
        match got {
            Value::Object(o) => assert_eq!(o.id(), child.id()),
            _ => panic!("expected object"),
        }
    }
}
