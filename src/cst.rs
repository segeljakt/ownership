use std::rc::Rc;

pub type Name = String;

pub type LocalId = String;

#[derive(Debug, Clone)]
pub struct Function {
    pub id: Name,
    pub params: Vec<Local>,
    pub ty: Type,
    pub block: Block,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Local {
    pub id: LocalId,
    pub ty: Type,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Expr>,
}

impl Block {
    pub fn ty(&self) -> &Type {
        self.expr.as_ref().map_or(&Type::Unit, |e| e.ty())
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(Local, Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    IfElse(Type, Rc<Expr>, Rc<Block>, Rc<Block>),
    While(Type, Rc<Expr>, Rc<Block>),
    Loop(Type, Option<usize>, Rc<Block>),
    Tuple(Type, Vec<Expr>),
    Ref(Type, Rc<Expr>),
    RefMut(Type, Rc<Expr>),
    Seq(Type, Rc<Expr>, Rc<Expr>),
    Assign(Type, Rc<Expr>, Rc<Expr>),
    Place(Type, Place),
    Var(Type, String),
    Index(Type, Rc<Expr>, usize),
    Deref(Type, Rc<Expr>),
    Add(Type, Rc<Expr>, Rc<Expr>),
    Int(Type, i32),
    Bool(Type, bool),
    String(Type, String),
    Print(Type, Rc<Expr>),
    Unit(Type),
    Return(Type, Rc<Expr>),
    Continue(Type, Option<usize>),
    Break(Type, Option<usize>),
    Block(Type, Rc<Block>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Place {
    pub local: Local,
    pub elems: Vec<PlaceElem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PlaceElem {
    Index(usize),
    Deref,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Bool,
    Unit,
    String,
    Unknown,
    Tuple(Vec<Type>),
    Ref(Vec<Loan>, Rc<Type>),
    RefMut(Vec<Loan>, Rc<Type>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Loan {
    pub place: Place,
    pub mutable: bool,
}

impl Place {
    pub fn is_prefix_of(&self, other: &Place) -> bool {
        if self.local.id != other.local.id {
            return false;
        }
        let mut self_iter = self.elems.iter();
        let mut other_iter = other.elems.iter();
        loop {
            match (self_iter.next(), other_iter.next()) {
                (Some(self_elem), Some(other_elem)) => {
                    if self_elem != other_elem {
                        return false;
                    }
                }
                (None, Some(_)) => return true,
                (Some(_), None) => return false,
                (None, None) => return true,
            }
        }
    }
}

impl Expr {
    pub fn ty(&self) -> &Type {
        match self {
            Expr::IfElse(ty, _, _, _) => ty,
            Expr::While(ty, _, _) => ty,
            Expr::Tuple(ty, _) => ty,
            Expr::Ref(ty, _) => ty,
            Expr::RefMut(ty, _) => ty,
            Expr::Seq(ty, _, _) => ty,
            Expr::Assign(ty, _, _) => ty,
            Expr::Place(ty, _) => ty,
            Expr::Add(ty, _, _) => ty,
            Expr::Int(ty, _) => ty,
            Expr::Bool(ty, _) => ty,
            Expr::String(ty, _) => ty,
            Expr::Block(ty, _) => ty,
            Expr::Unit(ty) => ty,
            Expr::Print(ty, _) => ty,
            Expr::Return(ty, _) => ty,
            Expr::Loop(ty, _, _) => ty,
            Expr::Continue(ty, _) => ty,
            Expr::Break(ty, _) => ty,
            Expr::Var(ty, _) => ty,
            Expr::Index(ty, _, _) => ty,
            Expr::Deref(ty, _) => ty,
        }
    }
}

impl Place {
    pub fn ty(&self) -> &Type {
        let mut t = &self.local.ty;
        for elem in self.elems.iter().rev() {
            t = match elem {
                PlaceElem::Index(i) => match t {
                    Type::Tuple(ts) => &ts[*i],
                    _ => &Type::Unknown,
                },
                PlaceElem::Deref => match t {
                    Type::Ref(_, ty) => ty.as_ref(),
                    Type::RefMut(_, ty) => ty.as_ref(),
                    _ => &Type::Unknown,
                },
            };
        }
        t
    }

    pub fn is_mutable(&self) -> bool {
        if self.elems.is_empty() && self.local.mutable {
            return true;
        }
        self.is_mutable_rec()
    }

    pub fn is_mutable_rec(&self) -> bool {
        let mut t = &self.local.ty;
        for elem in self.elems.iter().rev() {
            t = match elem {
                PlaceElem::Index(i) => match t {
                    Type::Tuple(ts) => &ts[*i],
                    _ => return false,
                },
                PlaceElem::Deref => match t {
                    Type::Ref(_, _) => return false,
                    Type::RefMut(_, ty) => ty,
                    _ => return false,
                },
            };
        }
        true
    }
}

impl Type {
    pub fn loans(&self) -> Vec<Loan> {
        let mut loans = Vec::new();
        self.loans_acc(&mut loans);
        loans
    }

    fn loans_acc(&self, loans: &mut Vec<Loan>) {
        match self {
            Type::Ref(loans2, t) | Type::RefMut(loans2, t) => {
                loans.extend(loans2.clone());
                t.loans_acc(loans);
            }
            Type::Int => {}
            Type::Bool => {}
            Type::Unit => {}
            Type::Unknown => {}
            Type::Tuple(ts) => {
                for t in ts {
                    t.loans_acc(loans);
                }
            }
            Type::String => {}
        }
    }

    pub fn is_copy(&self) -> bool {
        match self {
            Type::Int => true,
            Type::Bool => true,
            Type::Unit => true,
            Type::String => false,
            Type::Unknown => false,
            Type::Tuple(ts) => ts.iter().all(|t| t.is_copy()),
            Type::Ref(_, _) => true,
            Type::RefMut(_, _) => false,
        }
    }

    pub fn downgrade(&self) -> Type {
        if let Type::RefMut(_, t) = self {
            Type::Ref(Vec::new(), t.clone())
        } else {
            self.clone()
        }
    }
}

impl Local {
    pub fn into_expr(self) -> Expr {
        Expr::Place(
            self.ty.clone(),
            Place {
                local: self,
                elems: Vec::new(),
            },
        )
    }
}
