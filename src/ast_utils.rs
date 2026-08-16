use crate::{Expression, NType, Program, Statement};

fn show_bin_op(binop: &Expression) {
    match binop {
        Expression::BinaryOp { left, op, right } => {
            
        }
        _ => {}
    }
}

pub fn show_ast(ast: &Program) {
    println!("--- Program AST ---");
    for stmt in &ast.statements {
        match stmt {
            Statement::Set { name, ty, val } => {
                let unw_val = match val {
                    Some(Expression::Int(i)) => format!("{}", i),
                    Some(Expression::Var(s)) => s.clone(),
                    Some(Expression::Char(c)) => (*c).to_string(),
                    Some(Expression::ConstChar(ch)) => ch.clone(),
                    Some(Expression::AddrOf(addr)) => {
                        let boxed = &*addr;
                        "s".to_string()
                    }


                    None => format!("None"),
                    _ => format!("Unknown"),
                };
                println!("├ Set (Name: {}): ", name);
                println!("│ ├ Type:  {:?}", ty);
                println!("│ └ Value: {:?}", unw_val);
                println!("│ ");
            }
            Statement::Exit(expr) => {
                println!("├ Exit: ");
                println!("│ └ Value: {:?}", expr);
                println!("│ ");
            }
            Statement::CRtmPrint(str1) => {
                println!("├ Print (C runtime version): ");
                println!("│ └ String: {:?}", str1);
                println!("│ ");
            }
            _ => {}
        }
    }
}
