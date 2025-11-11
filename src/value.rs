use std::collections::HashMap;
use std::fmt;

use crate::object::ObjectRef; // forward reference (object.rs will declare ObjectRef)

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Object(ObjectRef),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Int(i) => write!(f, "Int({})", i),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::String(s) => write!(f, "String(\"{}\")", s),
            Value::Array(arr) => write!(f, "Array(len={})", arr.len()),
            Value::Map(map) => write!(f, "Map(len={})", map.len()),
            Value::Object(obj) => write!(f, "Object(class={}, id={})", obj.class_name(), obj.id()),
        }
    }
}

impl Value {
    pub fn as_object(&self) -> Option<ObjectRef> {
        match self {
            Value::Object(o) => Some(o.clone()),
            _ => None,
        }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }
}
