use crate::enums::{
    Expression, NType, Op, Param, Program, Statement,
    Token::{self, OpenCurly},
};

use colored::Colorize;

pub struct ParserNNV<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

#[derive(Debug)]
pub enum ErrorKind {
    NTYPE(NType),
    EXPR(Expression),
    TOKEN(Token),
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub sender: String,
    pub kind: ErrorKind,
    pub cursor: usize,
}

impl ParseError {
    pub fn send_it(msg: String, kind: ErrorKind, cursor: usize, sender: String) -> Self {
        println!("{}: ", "[!] Parsing error".bold().red());
        println!("Message: {}", msg);

        let kind_str = match &kind {
            ErrorKind::NTYPE(ntype) => format!("TypeErr: {:?}", ntype),
            ErrorKind::EXPR(expr) => format!("ExprErr: {:?}", expr),
            ErrorKind::TOKEN(token) => format!("TokenErr: {:?}", token),
        };
        println!("Kind: {}", kind_str);
        println!("Who sent: {}", sender);
        println!("Cursor (In the file being compiled): {}-th token", cursor);

        Self {
            message: msg,
            kind,
            cursor,
            sender,
        }
    }
}

impl<'a> ParserNNV<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn peek_back(&self) -> Option<&Token> {
        if self.cursor == 0 {
            None
        } else {
            self.tokens.get(self.cursor - 1)
        }
    }

    fn peek_back_by(&self, offset: usize) -> Option<&Token> {
        self.cursor
            .checked_sub(offset)
            .and_then(|idx| self.tokens.get(idx))
    }

    fn eat(&mut self, exp: &Token) -> bool {
        if self.peek() == Some(exp) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn adv(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.cursor);
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }

    fn adv_by(&mut self, offset: usize) -> Option<&Token> {
        if offset == 0 {
            return self.peek();
        }

        let target = self.cursor.saturating_add(offset);

        if target <= self.tokens.len() {
            self.cursor = target;
            self.tokens.get(self.cursor - 1)
        } else {
            self.cursor = self.tokens.len();
            None
        }
    }

    fn parse_range(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_comp()?;

        if let Some(tok) = self.peek() {
            let inclusive = match tok {
                Token::DotDot => false,
                Token::DotDotEq => true,
                _ => return Ok(left),
            };

            self.adv();

            let right = self.parse_comp()?;

            return Ok(Expression::Range {
                start: Box::new(left),
                end: Box::new(right),
                inclusive,
            });
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_primary()?;

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::BinMul => Op::Mul,
                Token::BinDiv => Op::Div,
                _ => break,
            };
            self.adv();
            let right = self.parse_primary()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.adv() {
            // Address-of
            Some(Token::Ampersand) => {
                let inner = self.parse_primary()?;
                Ok(Expression::AddrOf(Box::new(inner)))
            }

            // Inner expression
            Some(Token::OpenParen) => {
                let inner = self.parse_expr()?;
                match self.adv() {
                    Some(Token::CloseParen) => Ok(inner),
                    other => {
                        let other = other.unwrap().clone();

                        Err(ParseError::send_it(
                            format!("Expected ')', found {:?}", other),
                            ErrorKind::TOKEN(other),
                            self.cursor,
                            "parse_primary".to_string(),
                        ))
                    }
                }
            }

            // String literal
            Some(Token::StringConstLiteral(string)) => Ok(Expression::ConstChar(string.clone())),

            // Pointer dereference
            Some(Token::BinMul) => {
                let inner = self.parse_primary()?;
                Ok(Expression::Deref(Box::new(inner)))
            }

            // Integer literal
            Some(Token::IntLiteral(val)) => Ok(Expression::Int(*val)),

            // Character literal
            Some(Token::Char(c)) => Ok(Expression::Char(*c)),

            // Identifier
            Some(Token::Identifier(name)) => {
                let name = name.clone();

                if self.peek() == Some(&Token::OpenParen) {
                    self.adv();
                    let mut args = Vec::new();

                    if self.peek() != Some(&Token::CloseParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) {
                                self.adv();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::CloseParen)?;
                    Ok(Expression::Call { name, args })
                } else {
                    Ok(Expression::Var(name))
                }
            }

            // If the token is not a valid expression, return an error
            Some(token) => Err(ParseError::send_it(
                format!("expected expression, found {:?}", token),
                ErrorKind::TOKEN(token.clone()),
                self.cursor,
                "parse_primary/token".to_string(),
            )),

            None => Err(ParseError::send_it(
                "expected expression, found EOF".into(),
                ErrorKind::TOKEN(Token::Scrap),
                self.cursor,
                "parse_primary/token".to_string(),
            )),
        }
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::BinPlus => Op::Add,
                Token::BinMinus => Op::Sub,
                _ => break,
            };
            self.adv();
            let right = self.parse_term()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comp(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive()?;

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::LogicalGreaterEqual => Op::GreaterEq,
                Token::LogicalLessEqual => Op::LessEq,
                Token::EqualsEquals => Op::Equal,
                Token::LogicalNotEqual => Op::NotEq,
                Token::LogicalGreater => Op::Greater,
                Token::LogicalLess => Op::Less,
                _ => break,
            };
            self.adv();
            let right = self.parse_additive()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Expression, ParseError> {
        self.parse_range()
    }

    fn expect(&mut self, exp: Token) -> Result<Token, ParseError> {
        let current = self.adv();

        match current {
            Some(token) if token == &exp => Ok(exp),
            Some(got) => Err(ParseError::send_it(
                format!("expected {:?}, but got: {:?}", exp, got),
                ErrorKind::TOKEN(got.clone()),
                self.cursor,
                "expect/token".to_string(),
            )),
            None => {
                return Err(ParseError::send_it(
                    format!("expected {:?}, but reached end of file", exp),
                    ErrorKind::TOKEN(Token::Scrap),
                    self.cursor,
                    "expect/token".to_string(),
                ));
            }
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.adv() {
            Some(Token::Identifier(name)) => Ok(name.clone()),
            Some(got) => Err(ParseError::send_it(
                format!("expected identifier, got: {:?}", got),
                ErrorKind::TOKEN(got.clone()),
                self.cursor,
                "expect_ident/token".to_string(),
            )),
            _ => Err(ParseError::send_it(
                "expected identifier, but reached end of file".to_string(),
                ErrorKind::TOKEN(Token::Scrap),
                self.cursor,
                "expect_ident/token".to_string(),
            )),
        }
    }

    fn parse_pchar(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::OpenParen)?;

        let expr = self.parse_expr()?;

        self.expect(Token::CloseParen)?;
        self.expect(Token::SemiColon)?;

        Ok(Statement::PutChar(expr))
    }

    pub fn parse_assign(&mut self, idt_name: &String) -> Result<Statement, ParseError> {
        // Skip '='
        self.expect(Token::Equals)?;

        // Get expression of assignment
        let expr = self.parse_expr()?;

        // Check semicolon
        self.expect(Token::SemiColon)?;

        // Return ASGN statement
        Ok(Statement::Assign {
            name: idt_name.to_string(),
            val: expr,
        })
    }

    fn parse_type(&mut self) -> Result<NType, ParseError> {
        match self.adv() {
            Some(Token::Identifier(name)) => match name.as_str() {
                "i8" => Ok(NType::I8),
                "i16" => Ok(NType::I16),
                "i32" => Ok(NType::I32),
                "i64" => Ok(NType::I64),

                "u8" => Ok(NType::U8),
                "u16" => Ok(NType::U16),
                "u32" => Ok(NType::U32),
                "u64" => Ok(NType::U64),

                "int" => Ok(NType::Int),
                "char" => Ok(NType::Char),
                "str" => Ok(NType::String),
                _ => Err(ParseError::send_it(
                    format!("unknown type: {}", name),
                    ErrorKind::TOKEN(Token::Scrap),
                    self.cursor,
                    "parse_type/token".to_string(),
                )),
            },
            Some(Token::BinMul) => {
                let inner = self.parse_type()?;
                Ok(NType::Ptr(Box::new(inner)))
            }
            Some(Token::Ampersand) => {
                let inner = self.parse_type()?;
                Ok(NType::Ref(Box::new(inner)))
            }
            Some(got) => Err(ParseError::send_it(
                format!("expected type, but got: {:?}", got),
                ErrorKind::TOKEN(got.clone()),
                self.cursor,
                "parse_type/token".to_string(),
            )),
            None => Err(ParseError::send_it(
                "expected type, but reached end of file".to_string(),
                ErrorKind::TOKEN(Token::Scrap),
                self.cursor,
                "parse_type/token".to_string(),
            )),
        }
    }

    fn parse_exit_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::OpenParen)?;

        let expr = self.parse_expr()?;
        match expr {
            Expression::Int(_) | Expression::Char(_) | Expression::Var(_) => {}
            _ => {
                return Err(ParseError::send_it(
                    format!("expected Var/Char/Int, but got: {:?}", expr),
                    ErrorKind::EXPR(expr),
                    self.cursor,
                    "parse_exit_statement/expr".to_string(),
                ));
            }
        }

        self.expect(Token::CloseParen)?;
        self.expect(Token::SemiColon)?;
        Ok(Statement::Exit(expr))
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(Token::OpenParen)?;
        let mut params = Vec::new();

        if self.peek() != Some(&Token::CloseParen) {
            loop {
                let name = self.expect_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;

                params.push(Param { name, ty });

                if self.peek() == Some(&Token::Comma) {
                    self.adv();
                } else {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_method_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect_ident()?;
        let args = self.parse_params()?;

        self.adv();
        let ret_ty = if self.peek() == Some(&Token::Arrow) {
            self.adv();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::OpenCurly)?;

        let mut body = Vec::new();
        while self.peek() != Some(&Token::CloseCurly) {
            body.push(self.parse_statement()?);
        }

        self.expect(Token::CloseCurly)?;

        Ok(Statement::Met {
            name,
            args,
            body,
            ret_ty,
        })
    }

    fn parse_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect_ident()?; // Имя переменной

        let exp_type = if self.peek() == Some(&Token::Colon) {
            // Проверка на явный тип
            self.adv();
            Some(self.parse_type()?)
        } else {
            None
        };

        let val = if self.peek() == Some(&Token::Equals) {
            // Проверка и выражение инициализации
            self.adv();
            Some(self.parse_expr()?)
        } else {
            None
        };

        // EXPECTED SEMI-SUKA-COLON
        self.expect(Token::SemiColon)?;

        // Определение типа
        let ty = match (exp_type, &val) {
            (Some(expl), _) => expl,
            (None, Some(Expression::Int(_))) => NType::Int,
            (None, Some(Expression::Char(_))) => NType::Char,
            (None, Some(Expression::AddrOf(_))) => NType::Ptr(Box::new(NType::Int)),
            (None, Some(_)) => NType::Void,
            (None, None) => {
                return Err(ParseError::send_it(
                    format!("The variable '{}' must have a type or a value.", name),
                    ErrorKind::TOKEN(Token::Scrap),
                    self.cursor,
                    "parse_declaration/type_token".to_string(),
                ));
            }
        };

        // Давай газуй в вектор епта
        Ok(Statement::Set { name, ty, val })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        // Условие
        let expr = self.parse_expr()?;

        // Скобка
        self.expect(Token::OpenCurly)?;

        let mut then_br = Vec::new();
        while self.peek() != Some(&Token::CloseCurly) {
            then_br.push(self.parse_statement()?);
        }
        self.expect(Token::CloseCurly)?;

        Ok(Statement::If {
            condition: expr,
            then_br,
            else_br: None,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expr()?;
        self.expect(Token::SemiColon)?;

        Ok(Statement::Return(Some(expr)))
    }

    fn parse_pointer_call(&mut self) -> Result<Statement, ParseError> {
        // Выражение 1
        let target = self.parse_expr()?;

        // Нужен '=' или ошибка
        self.expect(Token::Equals)?;

        // Выражение 2
        let value = self.parse_expr()?;
        self.expect(Token::SemiColon)?;

        // *<expr1> = &<expr2> для примера:
        // *ptr = &str;

        Ok(Statement::DerefAssign { target, value })
    }

    fn parse_use(&mut self) -> Result<Statement, ParseError> {
        let statement = match self.peek() {
            Some(Token::KeywordImport) => {
                self.adv(); // Сожрать import

                self.parse_import()?
            }
            Some(Token::KeywordModule) => {
                self.adv(); // Сожрать module

                self.parse_module()?
            }
            _ => {
                return Err(ParseError::send_it(
                    "Expected tokens: import, module".to_string(),
                    ErrorKind::TOKEN(Token::Scrap),
                    self.cursor,
                    "parse_use/*".to_string(),
                ));
            }
        };

        Ok(Statement::Use(Box::new(statement)))
    }

    fn parse_import(&mut self) -> Result<Statement, ParseError> {
        let import_lib = self.parse_path_02()?;

        if matches!(self.peek(), Some(Token::SemiColon)) {
            self.adv();
        }

        Ok(Statement::Import(import_lib))
    }

    fn parse_module(&mut self) -> Result<Statement, ParseError> {
        let module_use = self.expect_ident()?;

        self.expect(Token::SemiColon)?;
        Ok(Statement::Module(module_use))
    }

    fn parse_path_02(&mut self) -> Result<String, ParseError> {
        let mut pth = String::new();

        loop {
            match self.peek() {
                Some(Token::Identifier(name)) => {
                    pth.push_str(name);
                    self.adv();
                }
                Some(Token::Dot) => {
                    pth.push('.');
                    self.adv();
                }
                Some(Token::BinMul) => {
                    pth.push('*');
                    self.adv();
                }
                _ => break,
            }
        }

        if pth.is_empty() {
            return Err(ParseError::send_it(
                "Expected import path after 'import'".to_string(),
                ErrorKind::TOKEN(Token::Scrap),
                self.cursor,
                "parse_path_02/token".to_string(),
            ));
        }

        Ok(pth)
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        let ident = self.expect_ident()?;

        self.expect(Token::KeywordIn)?;

        let val = self.parse_expr()?;

        self.expect(Token::OpenCurly)?;

        let mut body = Vec::new();
        while let Some(tok) = self.peek() {
            if tok == &Token::CloseCurly {
                break;
            }
            body.push(self.parse_statement()?);
        }
        self.expect(Token::CloseCurly)?;

        Ok(Statement::For { ident, val, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.adv() {
            Some(Token::KeywordPutChar) => self.parse_pchar(),
            Some(Token::KeywordSet) => self.parse_declaration(),
            Some(Token::KeywordExit) => self.parse_exit_statement(),
            Some(Token::KeywordIf) => self.parse_if_statement(),
            Some(Token::KeywordReturn) => self.parse_return(),

            Some(Token::KeywordFor) => self.parse_for(),

            Some(Token::KeywordUse) => self.parse_use(),

            Some(Token::Identifier(ident)) => {
                let ident = ident.clone();
                self.parse_assign(&ident)
            }

            Some(Token::BinMul) => self.parse_pointer_call(),

            Some(Token::KeywordMet) => self.parse_method_declaration(),

            Some(token) => Err(ParseError::send_it(
                format!("Parser: Unknown token: {:?}", token),
                ErrorKind::TOKEN(token.clone()),
                self.cursor,
                "parse_statement/token".to_string(),
            )),
            None => Err(ParseError::send_it(
                "Unexpected EOF".to_string(),
                ErrorKind::TOKEN(Token::Scrap),
                self.cursor,
                "parse_statement/eof".to_string(),
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Program { statements })
    }
}
