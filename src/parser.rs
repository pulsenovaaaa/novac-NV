use crate::{Expression, NType, Op, Program, Statement, Token};

pub struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn adv(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.cursor);
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
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

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.adv() {
            Some(Token::Ampersand) => {
                let inner = self.parse_primary()?;
                Ok(Expression::AddrOf(Box::new(inner)))
            }

            Some(Token::OpenParen) => {
                let inner = self.parse_expr()?;
                match self.adv() {
                    Some(Token::CloseParen) => Ok(inner),
                    other => Err(format!("Expected ')', found {:?}", other)),
                }
            }

            Some(Token::StringConstLiteral(string)) => Ok(Expression::ConstChar(string.clone())),

            Some(Token::BinMul) => {
                let inner = self.parse_primary()?;
                Ok(Expression::Deref(Box::new(inner)))
            }

            Some(Token::IntLiteral(val)) => Ok(Expression::Int(*val)),
            Some(Token::Identifier(name)) => Ok(Expression::Var(name.clone())),
            Some(Token::Char(c)) => Ok(Expression::Char(*c)),
            Some(token) => Err(format!("Ожидалось выражение, получено {:?}", token)),
            None => Err("Неожиданный EOF при разборе выражения".into()),
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, String> {
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
            }
        }

        Ok(left)
    }

    fn expect(&mut self, exp: Token) -> Result<Token, String> {
        let current = self.adv();

        match current {
            Some(token) if token == &exp => Ok(exp),
            Some(got) => Err(format!(
                "SyntaxError: Expected token '{:?}, got '{:?}''",
                exp, got
            )),
            None => Err(format!(
                "SyntaxError: Expected token '{:?}', but reached End of File",
                exp
            )),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.adv() {
            Some(Token::Identifier(name)) => Ok(name.clone()),
            Some(got) => Err(format!("Ожидалось имя переменной, получено {:?}", got)),
            _ => Err("Неожиданный EOF при разборе имени переменной".into()),
        }
    }

    fn parse_pchar(&mut self) -> Result<Statement, String> {
        self.expect(Token::OpenParen)?;

        let expr = self.parse_expr()?;

        self.expect(Token::CloseParen)?;
        self.expect(Token::SemiColon)?;

        Ok(Statement::PutChar(expr))
    }

    // fn parse_set(&mut self) -> Result<Statement, String> {
    //     let name = self.expect_ident()?;

    // }

    fn parse_type(&mut self) -> Result<NType, String> {
        match self.adv() {
            Some(Token::Identifier(name)) => match name.as_str() {
                "i64" => Ok(NType::I64),
                "u8" => Ok(NType::U8),
                "int" => Ok(NType::Int),
                "char" => Ok(NType::Char),
                "string" => Ok(NType::String),
                _ => Err(format!("Неизвестный тип: {}", name)),
            },
            Some(Token::BinMul) => {
                let inner = self.parse_type()?;
                Ok(NType::Ptr(Box::new(inner)))
            }
            Some(got) => Err(format!("Ожидался тип, получено {:?}", got)),
            None => Err("Неожиданный EOF при разборе типа".into()),
        }
    }

    // Устреет после этапа реконструкции кодгена
    fn parse_c_rtm_print(&mut self) -> Result<Statement, String> {
        self.expect(Token::OpenParen)?;

        let expr = self.parse_expr()?;
        match &expr {
            Expression::Var(s) => {}
            _ => {
                return Err("CRtmPrint: ожидалась переменная".into());
            }
        }

        self.expect(Token::CloseParen)?;
        self.expect(Token::SemiColon)?;

        Ok(Statement::CRtmPrint(expr))
    }

    fn parse_exit_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::OpenParen)?;

        let expr = self.parse_expr()?;
        match expr {
            Expression::Int(_) | Expression::Char(_) | Expression::Var(_) => {}
            _ => return Err("Exit: Expected Var/Char/Int".into()),
        }

        self.expect(Token::CloseParen)?;
        self.expect(Token::SemiColon)?;
        Ok(Statement::Exit(expr))
    }

    fn parse_declaration(&mut self) -> Result<Statement, String> {
        let mut initialized = false;
        /* 1. Имя переменной  */
        let name = self.expect_ident()?;
        /* 2. Проверка на :, если есть, тип явный. Иначе, автоматический */
        let explicit_type = if let Some(Token::Colon) = self.peek() {
            self.adv();
            Some(self.parse_type()?)
        } else {
            None
        };
        /* 3. Ожидаем '=' или ';' */
        match self.adv() {
            Some(Token::Equals) => initialized = true,
            Some(Token::SemiColon) => {}
            _ => {
                return Err("Ожидался '=' или ';'".into());
            }
        };

        /* 4. Если переменная инициализирована, то парсим тип. Иначе, отправляем None */
        let val = if initialized {
            let expr = self.parse_expr()?;
            match self.adv() {
                Some(Token::SemiColon) => {}
                _ => return Err("Ожидалось ';' в конце объявления переменной".into()),
            }
            Some(expr)
        } else {
            None
        };

        let ty = match (explicit_type, &val) {
            // Есть явный тип -> берем его
            (Some(expl), _) => expl,

            // Нет явного типа, но есть простая константа для базового вывода
            (None, Some(Expression::Int(_))) => NType::Int,
            (None, Some(Expression::Char(_))) => NType::Char,
            (None, Some(Expression::AddrOf(_))) => NType::Ptr(Box::new(NType::Int)),

            // Для остальных выражений (переменных, бинарных операций) тип определит Typechecker
            (None, Some(_)) => NType::Non,

            // Ошибка: объявление без типа и без значения (например, `set x;`)
            (None, None) => {
                return Err(format!(
                    "Переменная '{}' должна иметь тип или значение",
                    name
                ));
            }
        };

        Ok(Statement::Set { name, ty, val })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.adv() {
            Some(Token::KeywordPutChar) => self.parse_pchar(),
            Some(Token::KeywordSet) => self.parse_declaration(),

            Some(Token::KeywordExit) => self.parse_exit_statement(),

            Some(Token::CRuntimeKeywordPrint) => self.parse_c_rtm_print(),

            Some(token) => Err(format!("Unknown token: {:?}", token)),
            None => Err("Unexpected EOF".into()),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while self.peek().is_some() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Program { statements })
    }
}
