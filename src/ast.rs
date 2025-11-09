/// AST nodes for Gem scene files and logic scripts

#[derive(Debug, Clone, PartialEq)]
pub struct GemFile {
    pub root: GemDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemDecl {
    pub name: String,
    pub gem_type: String,
    pub properties: Vec<Property>,
    pub children: Vec<GemDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Integer(i64),
    String(String),
    Bool(bool),
    Tuple(Vec<Value>),
    Directive(Vec<String>), // e.g., #assets:player.png -> ["assets", "player.png"]
    Ident(String),
}

// Logic file AST
#[derive(Debug, Clone, PartialEq)]
pub struct LogicFile {
    pub extend_type: String,
    pub doc_comment: Option<String>,
    pub events: Vec<Event>,
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assignment {
        target: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Spawn {
        gem_type: String,
        properties: Vec<Property>,
    },
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Integer(i64),
    String(String),
    Bool(bool),
    Ident(String),
    Tuple(Vec<Expr>),
    Directive(Vec<String>),
    Call {
        name: String,
        args: Vec<Expr>,
    },
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Not,
    Minus,
}
