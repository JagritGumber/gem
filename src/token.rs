#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String), // identifier (parser will categorize by first char)

    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Keywords for logic
    On,      // 'on' event handler keyword
    Spawn,   // 'spawn' to create Gem instances
    Extend,  // 'extend' header in logic files
    Fn,      // 'fn' function declaration keyword
    
    Hash,               // '#'
    DocComment(String), // collected from lines starting with '///'
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
