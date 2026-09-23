#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Слова
    KeywordExit,
    KeywordSet,
    KeywordPutChar,
    KeywordIf,
    KeywordElse,
    CRuntimeKeywordPrint,
    KeywordMet,
    KeywordReturn,

    // Операторы (Пока что в общих токенах)
    BinPlus,
    BinMinus,
    BinMul,
    BinDiv,

    // Операторы ()=
    PlusEq,
    SubEq,
    MulEq,
    DivEq,
    BOrEq,
    MaskEq,

    // Вентили
    LogicalAND,
    LogicalOR,
    LogicalNot,
    LogicalGreater,
    LogicalLess,
    LogicalGreaterEqual,
    LogicalLessEqual,
    EqualsEquals,
    LogicalNotEqual,

    // Битовые операторы
    BitOR,
    BitAND,
    BitMask,
    BitNot,

    // Данные/Типы (Обертки)
    Identifier(String),
    IntLiteral(i64),
    Char(u8),
    StringConstLiteral(String),
    UnterminatedString,
    Comment,

    // Символы и 'scrap'
    Exclamation,
    Question,
    Ampersand,
    Equals,
    HashTag,
    Dollar,
    Slash,
    Dot,
    Comma,
    Colon,
    SemiColon,
    OpenParen,
    CloseParen,
    Arrow,

    OpenCurly,
    CloseCurly,
    Unknown,

    Scrap,
}

#[derive(Debug)]
pub enum BinaryOp {
    GreaterEq,
    LessEq,
    Less,
    Greater,
    NotEq,
    Equal,
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    GreaterEq,
    LessEq,
    Less,
    Greater,
    NotEq,
    Equal,
}

#[derive(Debug)]
pub enum Expression {
    Int(i64),
    Var(String),
    Char(u8),
    ConstChar(String),

    AddrOf(Box<Expression>),
    Deref(Box<Expression>),

    Call {
        name: String,
        args: Vec<Expression>,
    },

    Cast {
        target_type: NType,
        expr: Box<Expression>,
    },

    BinaryOp {
        left: Box<Expression>,
        op: Op,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NType {
    U8,
    U16,
    U32,
    U64,

    I8,
    I16,
    I32,
    I64,

    Int,
    Char,

    String,
    Void,
    Ptr(Box<NType>),
}

impl NType {
    pub fn size(&self) -> usize {
        match self {
            NType::U8 | NType::I8 | NType::Char => 1,
            NType::U16 | NType::I16 => 2,
            NType::U32 | NType::I32 | NType::Int => 4,
            NType::U64 | NType::I64 | NType::Ptr(_) | NType::String => 8,
            NType::Void => 0,
        }
    }

    pub fn nasm_data_directive(&self) -> &'static str {
        match self.size() {
            1 => "db",
            2 => "dw",
            4 => "dd",
            8 => "dq",
            _ => "db",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: NType,
}

#[derive(Debug)]
pub enum Statement {
    Exit(Expression),
    PutChar(Expression),
    Set {
        name: String,
        ty: NType,
        val: Option<Expression>,
    },
    CRtmPrint(Expression),
    Return(Option<Expression>),
    Assign {
        name: String,
        val: Expression,
    },
    DerefAssign {
        target: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_br: Vec<Statement>,
        else_br: Option<Vec<Statement>>,
    },
    Met {
        name: String,
        args: Vec<Param>,
        body: Vec<Statement>,
        ret_ty: Option<NType>,
    },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
