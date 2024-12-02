use crate::ast::Local;
use crate::ast::Place;
use crate::ast::Type;

pub type Name = String;

pub type BlockId = usize;

#[derive(Debug)]
pub struct Function {
    pub id: String,
    pub params: Vec<Local>,
    pub locals: Vec<Local>,
    pub ty: Type,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Terminator>,
    pub live_in: Vec<Place>,
    pub live_out: Vec<Place>,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub op: Operation,
    pub live_in: Vec<Place>,
    pub live_out: Vec<Place>,
}

impl Stmt {
    pub fn new(op: Operation) -> Stmt {
        Stmt {
            op,
            live_in: Vec::new(),
            live_out: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Assign(Place, Rvalue),
    StorageLive(Local),
    StorageDead(Local),
    Call {
        dest: Place,
        func: Operand,
        args: Vec<Operand>,
    },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return,
    Goto(BlockId),
    ConditionalGoto(Operand, BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    Ref { mutable: bool, place: Place },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Constant(Constant),
    Copy(Place),
    Move(Place),
    Function(String),
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i32),
    Bool(bool),
    String(String),
    Unit,
}
