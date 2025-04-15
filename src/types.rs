use std::fmt;

/// Type system for Boba language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Null,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Function {
        params: Vec<Type>,
        returns: Vec<Type>,
    },
    Any,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::List(elem_type) => write!(f, "[{}]", elem_type),
            Type::Map(key_type, val_type) => write!(f, "[{}:{}]", key_type, val_type),
            Type::Function { params, returns } => {
                write!(f, "fun(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, "): ")?;
                for (i, ret) in returns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ret)?;
                }
                Ok(())
            }
            Type::Any => write!(f, "any"),
        }
    }
}

/// Runtime values in the Boba language
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Function {
        name: String,
        params: Vec<(String, Type)>,
        return_types: Vec<Type>,
        body: Vec<crate::ast::Expr>,
    },
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
            Value::Null => Type::Null,
            Value::List(values) => {
                if values.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    // For simplicity, assume all elements have the same type as the first
                    Type::List(Box::new(values[0].get_type()))
                }
            }
            Value::Map(entries) => {
                if entries.is_empty() {
                    Type::Map(Box::new(Type::Any), Box::new(Type::Any))
                } else {
                    // For simplicity, assume all keys and values have the same types as the first entry
                    Type::Map(
                        Box::new(entries[0].0.get_type()),
                        Box::new(entries[0].1.get_type()),
                    )
                }
            }
            Value::Function {
                params, return_types, ..
            } => Type::Function {
                params: params.iter().map(|(_, t)| t.clone()).collect(),
                returns: return_types.clone(),
            },
        }
    }
}
