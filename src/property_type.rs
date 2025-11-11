//! Property type inference from literal values

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyType {
    String,
    Int,
    Float,
    Bool,
    Vec2,     // (x, y)
    Vec3,     // (x, y, z)
    Color,    // (r, g, b, a) or hex
    SceneRef, // #path:to:scene
}

impl PropertyType {
    /// Infer type from literal value string
    pub fn infer(value: &str) -> Self {
        let trimmed = value.trim();

        // Scene reference
        if trimmed.starts_with('#') {
            return PropertyType::SceneRef;
        }

        // Tuple literals
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            return match parts.len() {
                2 => PropertyType::Vec2,
                3 => PropertyType::Vec3,
                4 => PropertyType::Color,
                _ => PropertyType::String,
            };
        }

        // String literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return PropertyType::String;
        }

        // Boolean
        if trimmed == "true" || trimmed == "false" {
            return PropertyType::Bool;
        }

        // Float (contains decimal point)
        if trimmed.contains('.') && trimmed.parse::<f64>().is_ok() {
            return PropertyType::Float;
        }

        // Int
        if trimmed.parse::<i64>().is_ok() {
            return PropertyType::Int;
        }

        // Default to string
        PropertyType::String
    }

    /// Get Rust type string for static const
    pub fn to_rust_type(&self) -> &'static str {
        match self {
            PropertyType::String => "&'static str",
            PropertyType::Int => "i32",
            PropertyType::Float => "f32",
            PropertyType::Bool => "bool",
            PropertyType::Vec2 => "(i32, i32)",
            PropertyType::Vec3 => "(i32, i32, i32)",
            PropertyType::Color => "(u8, u8, u8, u8)",
            PropertyType::SceneRef => "&'static str",
        }
    }

    /// Get Rust type string for mutable runtime
    pub fn to_rust_type_mut(&self) -> &'static str {
        match self {
            PropertyType::String => "String",
            PropertyType::SceneRef => "String",
            _ => self.to_rust_type(), // primitives stay the same
        }
    }

    /// Convert literal string to Rust const value
    pub fn parse_to_rust_const(&self, value: &str) -> String {
        let trimmed = value.trim();
        match self {
            PropertyType::String => {
                // Keep quotes for &'static str
                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    trimmed.to_string()
                } else {
                    format!("\"{}\"", trimmed)
                }
            }
            PropertyType::SceneRef => {
                // Already has quotes typically
                if trimmed.starts_with('"') {
                    trimmed.to_string()
                } else {
                    format!("\"{}\"", trimmed)
                }
            }
            PropertyType::Vec2 | PropertyType::Vec3 => {
                // Already in tuple form: (x, y) or (x, y, z)
                trimmed.to_string()
            }
            PropertyType::Color => {
                // Parse (r, g, b, a) tuple
                trimmed.to_string()
            }
            PropertyType::Bool | PropertyType::Int | PropertyType::Float => trimmed.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference() {
        assert_eq!(PropertyType::infer("\"hello\""), PropertyType::String);
        assert_eq!(PropertyType::infer("42"), PropertyType::Int);
        assert_eq!(PropertyType::infer("3.14"), PropertyType::Float);
        assert_eq!(PropertyType::infer("true"), PropertyType::Bool);
        assert_eq!(PropertyType::infer("(100, 200)"), PropertyType::Vec2);
        assert_eq!(PropertyType::infer("(1, 2, 3)"), PropertyType::Vec3);
        assert_eq!(
            PropertyType::infer("(255, 128, 0, 255)"),
            PropertyType::Color
        );
        assert_eq!(
            PropertyType::infer("#example:scene"),
            PropertyType::SceneRef
        );
    }
}
