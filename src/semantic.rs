use std::collections::HashMap;

use crate::enums::{NType, Program, Statement};

#[derive(Debug, Clone)]
pub enum SymbolLocation {
    StackOffset(i32),
    Global,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: NType,
    pub location: SymbolLocation,
}

#[derive(Debug, Default)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
    pub stack_offset: i32,
}

impl Scope {
    pub fn new_scope(parent: Option<Box<Scope>>) -> Self {
        let initial_offset = parent.as_ref().map_or(0, |p| p.stack_offset);

        Scope {
            symbols: HashMap::new(),
            parent,
            stack_offset: initial_offset,
        }
    }

    pub fn exit_scope(&mut self) -> Option<Scope> {
        self.stack_offset = 0;
        self.symbols.clear();
        self.parent.take().map(|b| *b)
    }

    pub fn insert(&mut self, name: String, ty: NType) {
        let size = ty.size();
        self.stack_offset += size as i32;

        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                ty,
                location: SymbolLocation::StackOffset(self.stack_offset),
            },
        );
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        if let Some(sym) = self.symbols.get(name) {
            Some(sym)
        } else if let Some(ref parent) = self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
    pub program: &'a Program,
    pub current_scope: Scope,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            current_scope: Scope::new_scope(None),
        }
    }

    pub fn analyze(&mut self) -> Result<(), &'static str> {
        for stmt in &self.program.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), &'static str> {
        match stmt {
            Statement::Met {
                name,
                args,
                body,
                ret_ty,
            } => {
                let old_scope = std::mem::take(&mut self.current_scope);

                let mut inner_scope = Scope::new_scope(Some(Box::new(old_scope)));

                for arg in args {
                    inner_scope.insert(arg.name.clone(), arg.ty.clone());
                }

                self.current_scope = inner_scope;

                for stmt in body {
                    self.analyze_statement(stmt)?;
                }

                if let Some(parent_scope) = self.current_scope.exit_scope() {
                    self.current_scope = parent_scope;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
