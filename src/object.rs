use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::value::Value;

#[derive(Clone)]
pub struct ObjectRef(Arc<ObjectInner>);

impl PartialEq for ObjectRef {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for ObjectRef {}

type MethodFn = dyn Fn(&ObjectRef, &[Value]) -> Result<Value, String> + Send + Sync + 'static;
type SignalFn = dyn Fn(&[Value]) + Send + Sync + 'static;

static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static CLASS_REGISTRY: OnceLock<RwLock<HashMap<String, fn() -> ObjectRef>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<String, fn() -> ObjectRef>> {
    CLASS_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_class(name: &str, ctor: fn() -> ObjectRef) {
    let reg = registry();
    reg.write().unwrap().insert(name.to_string(), ctor);
}

pub fn object_new(class_name: &str) -> ObjectRef {
    let reg = registry();
    let map = reg.read().unwrap();
    let ctor = map
        .get(class_name)
        .unwrap_or_else(|| panic!("Class '{}' not registered", class_name));
    ctor()
}

pub struct ObjectInner {
    id: u64,
    class_name: String,
    properties: RwLock<HashMap<String, Value>>,
    methods: RwLock<HashMap<String, Arc<MethodFn>>>,
    signals: RwLock<HashMap<String, Vec<Arc<SignalFn>>>>,
}

impl ObjectInner {
    pub fn base(class_name: &str) -> ObjectRef {
        ObjectRef(Arc::new(ObjectInner {
            id: OBJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            class_name: class_name.to_string(),
            properties: RwLock::new(HashMap::new()),
            methods: RwLock::new(HashMap::new()),
            signals: RwLock::new(HashMap::new()),
        }))
    }
}

impl ObjectInner {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn class_name(&self) -> &str {
        &self.class_name
    }
}

impl ObjectInner {
    pub(crate) fn insert_method(this: &ObjectRef, name: &str, f: Arc<MethodFn>) {
        this.0.methods.write().unwrap().insert(name.to_string(), f);
    }
    #[allow(dead_code)]
    pub(crate) fn ensure_property(this: &ObjectRef, key: &str, default: Value) {
        let mut props = this.0.properties.write().unwrap();
        props.entry(key.to_string()).or_insert(default);
    }
}

impl ObjectRef {
    pub fn id(&self) -> u64 {
        self.0.id()
    }
    pub fn class_name(&self) -> &str {
        self.0.class_name()
    }

    pub fn set_property(&self, key: &str, value: Value) {
        self.0
            .properties
            .write()
            .unwrap()
            .insert(key.to_string(), value);
    }
    pub fn get_property(&self, key: &str) -> Option<Value> {
        self.0.properties.read().unwrap().get(key).cloned()
    }
    pub fn call_method(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let methods = self.0.methods.read().unwrap();
        let m = methods
            .get(name)
            .ok_or_else(|| format!("Method '{}' not found on {}", name, self.class_name()))?;
        m(self, args)
    }
    pub fn connect(&self, signal: &str, callback: Arc<SignalFn>) {
        let mut sigs = self.0.signals.write().unwrap();
        sigs.entry(signal.to_string()).or_default().push(callback);
    }
    pub fn emit_signal(&self, signal: &str, args: &[Value]) {
        if let Some(list) = self.0.signals.read().unwrap().get(signal) {
            for cb in list {
                cb(args);
            }
        }
    }
}

pub fn init_object_class() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        register_class("Object", || {
            let obj = ObjectInner::base("Object");
            ObjectInner::insert_method(
                &obj,
                "to_string",
                Arc::new(|this, _| {
                    Ok(Value::String(format!(
                        "<Object {}:{}>",
                        this.class_name(),
                        this.id()
                    )))
                }),
            );
            obj
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn object_basic() {
        init_object_class();
        let o = object_new("Object");
        o.set_property("foo", Value::Int(42));
        assert_eq!(o.get_property("foo"), Some(Value::Int(42)));
        let s = o.call_method("to_string", &[]).unwrap();
        if let Value::String(txt) = s {
            assert!(txt.contains("Object"));
        } else {
            panic!("Expected string");
        }
    }
}
