use crate::ast::{BinaryOperator, Expr, Program, UnaryOperator};
use crate::types::Type;
use std::collections::HashMap;

pub struct TypeChecker {
    variables: HashMap<String, Type>,
    functions: HashMap<String, FunctionType>,
}

#[derive(Clone)]
struct FunctionType {
    param_types: Vec<(String, Type)>,
    return_types: Vec<Type>,
}

pub fn check_types(program: &Program) -> Vec<String> {
    let mut checker = TypeChecker {
        variables: HashMap::new(),
        functions: HashMap::new(),
    };

    let mut errors = Vec::new();

    // Register function signatures
    for (name, func_def) in &program.functions {
        checker.functions.insert(
            name.clone(),
            FunctionType {
                param_types: func_def.params.clone(),
                return_types: func_def.return_types.clone(),
            },
        );
    }

    // Check function bodies
    for (name, func_def) in &program.functions {
        let mut local_checker = TypeChecker {
            variables: HashMap::new(),
            functions: checker.functions.clone(),
        };

        // Add parameters to local scope
        for (param_name, param_type) in &func_def.params {
            local_checker.variables.insert(param_name.clone(), param_type.clone());
        }

        // Check function body
        for expr in &func_def.body {
            if let Err(e) = local_checker.check_expr(expr) {
                errors.push(format!("In function '{}': {}", name, e));
            }
        }

        // Check return type
        if let Some(last_expr) = func_def.body.last() {
            if let Expr::Return(values) = last_expr {
                if values.len() != func_def.return_types.len() {
                    errors.push(format!(
                        "Function '{}' returns {} values, but declared to return {} values",
                        name,
                        values.len(),
                        func_def.return_types.len()
                    ));
                } else {
                    for (i, (value, expected_type)) in values.iter().zip(&func_def.return_types).enumerate() {
                        if let Ok(actual_type) = local_checker.infer_type(value) {
                            if !types_compatible(&actual_type, expected_type) {
                                errors.push(format!(
                                    "Function '{}' return value {} has type {:?}, expected {:?}",
                                    name, i, actual_type, expected_type
                                ));
                            }
                        }
                    }
                }
            } else if !func_def.return_types.is_empty() && func_def.return_types != vec![Type::Null] {
                errors.push(format!(
                    "Function '{}' is missing return statement",
                    name
                ));
            }
        }
    }

    // Check main block
    for expr in &program.main_block {
        if let Err(e) = checker.check_expr(expr) {
            errors.push(e);
        }
    }

    errors
}

impl TypeChecker {
    fn check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IntLiteral(_) => Ok(Type::Int),
            Expr::FloatLiteral(_) => Ok(Type::Float),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::BoolLiteral(_) => Ok(Type::Bool),
            Expr::NullLiteral => Ok(Type::Null),

            Expr::List(items) => {
                if items.is_empty() {
                    return Ok(Type::List(Box::new(Type::Any)));
                }

                let first_type = self.infer_type(&items[0])?;
                
                for (i, item) in items.iter().enumerate().skip(1) {
                    let item_type = self.infer_type(item)?;
                    if !types_compatible(&item_type, &first_type) {
                        return Err(format!(
                            "List contains mixed types: item {} has type {:?}, expected {:?}",
                            i, item_type, first_type
                        ));
                    }
                }

                Ok(Type::List(Box::new(first_type)))
            }

            Expr::Map(entries) => {
                if entries.is_empty() {
                    return Ok(Type::Map(Box::new(Type::Any), Box::new(Type::Any)));
                }

                let first_key_type = self.infer_type(&entries[0].0)?;
                let first_val_type = self.infer_type(&entries[0].1)?;
                
                for (i, (key, val)) in entries.iter().enumerate().skip(1) {
                    let key_type = self.infer_type(key)?;
                    if !types_compatible(&key_type, &first_key_type) {
                        return Err(format!(
                            "Map contains mixed key types: entry {} has key type {:?}, expected {:?}",
                            i, key_type, first_key_type
                        ));
                    }

                    let val_type = self.infer_type(val)?;
                    if !types_compatible(&val_type, &first_val_type) {
                        return Err(format!(
                            "Map contains mixed value types: entry {} has value type {:?}, expected {:?}",
                            i, val_type, first_val_type
                        ));
                    }
                }

                Ok(Type::Map(Box::new(first_key_type), Box::new(first_val_type)))
            }

            Expr::Identifier(name) => {
                if let Some(var_type) = self.variables.get(name) {
                    Ok(var_type.clone())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }

            Expr::VarDeclaration(name, value) => {
                let value_type = self.infer_type(value)?;
                self.variables.insert(name.clone(), value_type.clone());
                Ok(value_type)
            }

            Expr::BinaryOp { left, operator, right } => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                match operator {
                    BinaryOperator::Add => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Int),
                            (Type::Float, Type::Float) => Ok(Type::Float),
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                            (Type::String, Type::String) => Ok(Type::String),
                            _ => Err(format!(
                                "Cannot add values of types {:?} and {:?}",
                                left_type, right_type
                            )),
                        }
                    }
                    BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Int),
                            (Type::Float, Type::Float) => Ok(Type::Float),
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                            _ => Err(format!(
                                "Cannot perform arithmetic on types {:?} and {:?}",
                                left_type, right_type
                            )),
                        }
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        if types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Cannot compare values of incompatible types {:?} and {:?}",
                                left_type, right_type
                            ))
                        }
                    }
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual |
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) | (Type::Float, Type::Float) |
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Bool),
                            (Type::String, Type::String) => Ok(Type::Bool),
                            _ => Err(format!(
                                "Cannot compare values of types {:?} and {:?}",
                                left_type, right_type
                            )),
                        }
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        if left_type == Type::Bool && right_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Logical operators require boolean operands, got {:?} and {:?}",
                                left_type, right_type
                            ))
                        }
                    }
                }
            }

            Expr::UnaryOp { operator, expr } => {
                let expr_type = self.infer_type(expr)?;

                match operator {
                    UnaryOperator::Negate => {
                        match expr_type {
                            Type::Int => Ok(Type::Int),
                            Type::Float => Ok(Type::Float),
                            _ => Err(format!("Cannot negate value of type {:?}", expr_type)),
                        }
                    }
                    UnaryOperator::Not => {
                        if expr_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Cannot apply logical NOT to type {:?}", expr_type))
                        }
                    }
                    UnaryOperator::AddressOf => {
                        // In a real language, this would return a pointer type
                        // For our interpreter, we'll just return a string
                        Ok(Type::String)
                    }
                }
            }

            Expr::If { condition, then_branch, else_if_branches, else_branch } => {
                let cond_type = self.infer_type(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("If condition must be boolean, got {:?}", cond_type));
                }

                // Check then branch
                for expr in then_branch {
                    self.check_expr(expr)?;
                }

                // Check else-if branches
                for (cond, branch) in else_if_branches {
                    let cond_type = self.infer_type(cond)?;
                    if cond_type != Type::Bool {
                        return Err(format!("Else-if condition must be boolean, got {:?}", cond_type));
                    }

                    for expr in branch {
                        self.check_expr(expr)?;
                    }
                }

                // Check else branch
                if let Some(branch) = else_branch {
                    for expr in branch {
                        self.check_expr(expr)?;
                    }
                }

                // If expressions don't have a specific return type in this language
                Ok(Type::Null)
            }

            Expr::Loop { init, condition, update, body } => {
                // Check initialization
                if let Some(init_expr) = init {
                    self.check_expr(init_expr)?;
                }

                // Check condition
                if let Some(cond_expr) = condition {
                    let cond_type = self.infer_type(cond_expr)?;
                    if cond_type != Type::Bool {
                        return Err(format!("Loop condition must be boolean, got {:?}", cond_type));
                    }
                }

                // Check update
                if let Some(update_expr) = update {
                    self.check_expr(update_expr)?;
                }

                // Check body
                for expr in body {
                    self.check_expr(expr)?;
                }

                Ok(Type::Null)
            }

            Expr::Continue | Expr::Break => Ok(Type::Null),

            Expr::Return(values) => {
                for value in values {
                    self.check_expr(value)?;
                }
                Ok(Type::Null)
            }

            Expr::FunctionCall { name, args } => {
                if let Some(func_type) = self.functions.get(name) {
                    if args.len() != func_type.param_types.len() {
                        return Err(format!(
                            "Function '{}' expects {} arguments, got {}",
                            name,
                            func_type.param_types.len(),
                            args.len()
                        ));
                    }

                    for (i, (arg, (_, expected_type))) in args.iter().zip(&func_type.param_types).enumerate() {
                        let arg_type = self.infer_type(arg)?;
                        if !types_compatible(&arg_type, expected_type) {
                            return Err(format!(
                                "Function '{}' argument {} has type {:?}, expected {:?}",
                                name, i, arg_type, expected_type
                            ));
                        }
                    }

                    if func_type.return_types.is_empty() {
                        Ok(Type::Null)
                    } else if func_type.return_types.len() == 1 {
                        Ok(func_type.return_types[0].clone())
                    } else {
                        // For multiple return values, we'd need a tuple type
                        // For simplicity, we'll just return the first value's type
                        Ok(func_type.return_types[0].clone())
                    }
                } else {
                    Err(format!("Undefined function: {}", name))
                }
            }

            Expr::Output(args) => {
                // Check that all arguments are valid expressions
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Null)
            }

            Expr::OutputFormatted(expr) => {
                let expr_type = self.check_expr(expr)?;
                if expr_type != Type::String {
                    return Err(format!("outputf requires a string argument, got {:?}", expr_type));
                }
                Ok(Type::Null)
            }

            Expr::OutputAddress(expr) => {
                self.check_expr(expr)?;
                Ok(Type::Null)
            }

            Expr::Input(expr) => {
                let expr_type = self.check_expr(expr)?;
                if expr_type != Type::String {
                    return Err(format!("input requires a string prompt, got {:?}", expr_type));
                }
                Ok(Type::String)
            }

            Expr::InputFormatted(expr) => {
                let expr_type = self.check_expr(expr)?;
                if expr_type != Type::String {
                    return Err(format!("inputf requires a string argument, got {:?}", expr_type));
                }
                Ok(Type::String)
            }

            Expr::TypeConversion { expr, target_type } => {
                let expr_type = self.infer_type(expr)?;
                
                match (&expr_type, target_type) {
                    (Type::Int, Type::Float) => Ok(Type::Float),
                    (Type::Float, Type::Int) => Ok(Type::Int),
                    (Type::Int, Type::String) => Ok(Type::String),
                    (Type::Float, Type::String) => Ok(Type::String),
                    (Type::Bool, Type::String) => Ok(Type::String),
                    (Type::String, Type::Int) => Ok(Type::Int),
                    (Type::String, Type::Float) => Ok(Type::Float),
                    (Type::String, Type::Bool) => Ok(Type::Bool),
                    _ => Err(format!("Cannot convert from {:?} to {:?}", expr_type, target_type)),
                }
            }

            Expr::TypeCheck { expr, check_type, .. } => {
                // Type check expressions always return a boolean
                self.check_expr(expr)?;
                Ok(Type::Bool)
            }

            _ => Err(format!("Type checking not implemented for {:?}", expr)),
        }
    }

    fn infer_type(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IntLiteral(_) => Ok(Type::Int),
            Expr::FloatLiteral(_) => Ok(Type::Float),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::BoolLiteral(_) => Ok(Type::Bool),
            Expr::NullLiteral => Ok(Type::Null),
            
            Expr::Identifier(name) => {
                if let Some(var_type) = self.variables.get(name) {
                    Ok(var_type.clone())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            },
            
            Expr::TypeConversion { expr, target_type } => {
                // For type inference, we just return the target type
                Ok(target_type.clone())
            },
            
            // For other expressions, we need to evaluate them
            _ => Err(format!("Cannot infer type of complex expression: {:?}", expr)),
        }
    }
}

fn types_compatible(actual: &Type, expected: &Type) -> bool {
    match (actual, expected) {
        (a, b) if a == b => true,
        (_, Type::Any) => true,
        (Type::Int, Type::Float) => true,
        (Type::Float, Type::Int) => true,
        _ => false,
    }
}
