#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    Fun, // 'fun' keyword instead of 'fn'
    If,
    Else,
    Takethis, // 'takethis' keyword for output
    No,       // 'no' keyword (negation/else-like)

    Ident(String),
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Eq,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,

    Plus,
    Minus,
    Multiply,
    Divide,

    And,
    Or,
    EqEq,
    NotEq,

    Not,

    Comma, // ,
    Colon, // :
    Dot,   // .

    Less,      // <
    Greater,   // >
    LessEq,    // <=
    GreaterEq, // >=
}
