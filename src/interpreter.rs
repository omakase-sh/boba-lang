use crate::ast::{Expr, Program};
use crate::types::{Type, Value};
use std::collections::HashMap;
use std::io::{self, Write};

pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn define_function(&mut self, name: String, value: Value) {
        self.functions.insert(name, value);
    }

    pub fn get_function(&self, name: &str) -> Option<&Value> {
        self.functions.get(name)
    }
}

pub fn interpret(program: Program) -> Result<(), String> {
    let mut env = Environment::new();
    
    // Register functions
    for (name, func_def) in &program.functions {
        let func_value = Value::Function {
            name: name.clone(),
            params: func_def.params.clone(),
            return_types: func_def.return_types.clone(),
            body: func_def.body.clone(),
        };
        env.define_function(name.clone(), func_value);
    }
    
    // Look for main function
    if let Some(Value::Function { body, .. }) = env.get_function("main").cloned() {
        for expr in body {
            evaluate_expr(&expr, &mut env)?;
        }
    } else {
        // Execute main block if no main function
        for expr in program.main_block {
            evaluate_expr(&expr, &mut env)?;
        }
    }
    
    Ok(())
}

fn evaluate_expr(expr: &Expr, env: &mut Environment) -> Result<Value, String> {
    match expr {
        Expr::IntLiteral(n) => Ok(Value::Int(*n)),
        Expr::FloatLiteral(n) => Ok(Value::Float(*n)),
        Expr::StringLiteral(s) => Ok(Value::String(s.clone())),
        Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),
        Expr::NullLiteral => Ok(Value::Null),
        
        Expr::List(items) => {
            let mut values = Vec::new();
            for item in items {
                values.push(evaluate_expr(item, env)?);
            }
            Ok(Value::List(values))
        },
        
        Expr::Map(entries) => {
            let mut values = Vec::new();
            for (key, value) in entries {
                let key_val = evaluate_expr(key, env)?;
                let val_val = evaluate_expr(value, env)?;
                values.push((key_val, val_val));
            }
            Ok(Value::Map(values))
        },
        
        Expr::VarDeclaration(name, value_expr) => {
            let value = evaluate_expr(value_expr, env)?;
            env.define(name.clone(), value.clone());
            Ok(value)
        },
        
        Expr::Identifier(name) => {
            if let Some(value) = env.get(name) {
                Ok(value.clone())
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        },
        
        Expr::Output(args) => {
            let mut values = Vec::new();
            for arg in args {
                let value = evaluate_expr(arg, env)?;
                values.push(value);
            }
            
            // Print values
            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print_value(value);
            }
            println!();
            io::stdout().flush().map_err(|e| e.to_string())?;
            
            Ok(Value::Null)
        },
        
        Expr::OutputFormatted(format_expr) => {
            let format_value = evaluate_expr(format_expr, env)?;
            
            if let Value::String(format_str) = format_value {
                // Simple format string handling
                let result = format_str.replace("{", "").replace("}", "");
                println!("{}", result);
                io::stdout().flush().map_err(|e| e.to_string())?;
            } else {
                return Err("outputf requires a string argument".to_string());
            }
            
            Ok(Value::Null)
        },
        
        Expr::Return(values) => {
            if values.is_empty() {
                Ok(Value::Null)
            } else if values.len() == 1 {
                evaluate_expr(&values[0], env)
            } else {
                // For multiple return values, we'd need a tuple type
                // For simplicity, we'll just return the first value
                evaluate_expr(&values[0], env)
            }
        },
        
        Expr::FunctionCall { name, args } => {
            if let Some(Value::Function { params, body, .. }) = env.get_function(name).cloned() {
                // Create a new environment for the function
                let mut func_env = Environment::new();
                
                // Copy function definitions
                for (fname, fval) in &env.functions {
                    func_env.define_function(fname.clone(), fval.clone());
                }
                
                // Evaluate arguments and bind to parameters
                if args.len() != params.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    ));
                }
                
                for (i, arg) in args.iter().enumerate() {
                    let arg_value = evaluate_expr(arg, env)?;
                    func_env.define(params[i].0.clone(), arg_value);
                }
                
                // Execute function body
                let mut result = Value::Null;
                for expr in body {
                    result = evaluate_expr(&expr, &mut func_env)?;
                    if matches!(expr, Expr::Return(_)) {
                        break;
                    }
                }
                
                Ok(result)
            } else {
                Err(format!("Undefined function: {}", name))
            }
        },
        
        Expr::TypeConversion { expr, target_type } => {
            let value = evaluate_expr(expr, env)?;
            
            match (value, target_type) {
                (Value::Int(n), Type::Float) => Ok(Value::Float(n as f64)),
                (Value::Float(n), Type::Int) => Ok(Value::Int(n as i64)),
                (Value::Int(n), Type::String) => Ok(Value::String(n.to_string())),
                (Value::Float(n), Type::String) => Ok(Value::String(n.to_string())),
                (Value::Bool(b), Type::String) => Ok(Value::String(b.to_string())),
                (Value::String(s), Type::Int) => {
                    match s.parse::<i64>() {
                        Ok(n) => Ok(Value::Int(n)),
                        Err(_) => Err(format!("Cannot convert '{}' to int", s)),
                    }
                },
                (Value::String(s), Type::Float) => {
                    match s.parse::<f64>() {
                        Ok(n) => Ok(Value::Float(n)),
                        Err(_) => Err(format!("Cannot convert '{}' to float", s)),
                    }
                },
                (Value::String(s), Type::Bool) => {
                    match s.to_lowercase().as_str() {
                        "true" => Ok(Value::Bool(true)),
                        "false" => Ok(Value::Bool(false)),
                        _ => Err(format!("Cannot convert '{}' to bool", s)),
                    }
                },
                (v, t) => Err(format!("Cannot convert {:?} to {:?}", v, t)),
            }
        },
        
        // Add other expression types as needed
        _ => Err(format!("Unsupported expression: {:?}", expr)),
    }
}

fn print_value(value: &Value) {
    match value {
        Value::Int(n) => print!("{}", n),
        Value::Float(n) => print!("{}", n),
        Value::String(s) => print!("{}", s),
        Value::Bool(b) => print!("{}", b),
        Value::Null => print!("null"),
        Value::List(items) => {
            print!("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print_value(item);
            }
            print!("]");
        },
        Value::Map(entries) => {
            print!("[");
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print_value(k);
                print!(":");
                print_value(v);
            }
            print!("]");
        },
        Value::Function { name, .. } => {
            print!("<function {}>", name);
        },
    }
}
