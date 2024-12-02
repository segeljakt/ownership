use std::rc::Rc;

use crate::ast::Block;
/// Simple type inference
use crate::ast::Expr;
use crate::ast::Function;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::LocalId;
use crate::ast::Place;
use crate::ast::Stmt;
use crate::ast::Type;

struct Context {
    pub stack: Vec<Scope>,
}

pub struct Scope {
    pub bindings: Vec<Local>,
}

impl Context {
    pub fn new() -> Context {
        Context { stack: vec![] }
    }

    pub fn add_binding(&mut self, l: Local) {
        self.stack.last_mut().unwrap().bindings.push(l);
    }

    pub fn lookup(&self, id: &LocalId) -> Option<&Local> {
        self.stack
            .iter()
            .rev()
            .find_map(|s| s.bindings.iter().find(|l| l.id == *id))
    }

    pub fn infer_function(&mut self, f: &Function) -> Function {
        self.stack.push(Scope { bindings: vec![] });
        for l in &f.params {
            self.add_binding(l.clone());
        }
        let body = self.infer_block(&f.body);
        self.stack.pop();
        Function {
            id: f.id.clone(),
            params: f.params.clone(),
            ty: f.ty.clone(),
            body,
        }
    }

    pub fn infer_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Int(_, i) => Expr::Int(Type::Int, *i),
            Expr::Bool(_, b) => Expr::Bool(Type::Bool, *b),
            Expr::Place(_, id) => {
                let p = self.infer_place(id);
                Expr::Place(p.ty().clone(), p)
            }
            Expr::Add(_, e1, e2) => {
                let e1 = self.infer_expr(e1);
                let e2 = self.infer_expr(e2);
                let t = Type::Int;
                Expr::Add(t, Rc::new(e1), Rc::new(e2))
            }
            Expr::IfElse(_, e0, e1, e2) => {
                let e0 = self.infer_expr(e0);
                if *e0.ty() != Type::Bool {
                    panic!("expected bool, found {:?}", e0.ty());
                }
                let b1 = self.infer_block(e1);
                let b2 = self.infer_block(e2);
                let ty = match (b1.expr.ty(), e2.expr.ty()) {
                    (Type::Ref(loans1, t1), Type::Ref(loans2, t2)) => {
                        if t1 != t2 {
                            panic!("mismatched types: {:?} != {:?}", t1, t2);
                        }
                        let loans = loans1
                            .iter()
                            .chain(loans2.iter())
                            .cloned()
                            .collect::<Vec<_>>();
                        Type::Ref(loans, t1.clone())
                    }
                    (Type::RefMut(loans1, t1), Type::RefMut(loans2, t2)) => {
                        if t1 != t2 {
                            panic!("mismatched types: {:?} != {:?}", t1, t2);
                        }
                        let loans = loans1
                            .iter()
                            .chain(loans2.iter())
                            .cloned()
                            .collect::<Vec<_>>();
                        Type::RefMut(loans, t1.clone())
                    }
                    (t1, t2) => {
                        if t1 != t2 {
                            panic!("mismatched types: {:?} != {:?}", t1, t2);
                        }
                        t1.clone()
                    }
                };
                Expr::IfElse(ty, Rc::new(e0), Rc::new(b1), Rc::new(b2))
            }
            // Expr::Let(_, l0, e0, e1) => {
            //     let e0 = self.infer_expr(e0);
            //     let l1 = Local {
            //         id: l0.id.clone(),
            //         ty: e0.ty().clone(),
            //         mutable: l0.mutable,
            //     };
            //     self.add_binding(l1.clone());
            //     let e1 = self.infer_expr(e1);
            //     Expr::Let(e0.ty().clone(), l1, Rc::new(e0), Rc::new(e1))
            // }
            Expr::While(_, e, b) => {
                let e = self.infer_expr(e);
                if *e.ty() != Type::Bool {
                    panic!("expected bool, found {:?}", e.ty());
                }
                let b = self.infer_block(b);
                Expr::While(Type::Unit, Rc::new(e), Rc::new(b))
            }
            Expr::Tuple(_, es) => {
                let es = es.iter().map(|e| self.infer_expr(e)).collect::<Vec<_>>();
                let ts = es.iter().map(|e| e.ty().clone()).collect();
                Expr::Tuple(Type::Tuple(ts), es)
            }
            Expr::Ref(_, p) => {
                let p = self.infer_place(p);
                let loan = Loan {
                    place: p.clone(),
                    mutable: false,
                };
                let t = Type::Ref(vec![loan], Rc::new(p.ty().clone()));
                Expr::Ref(t, p)
            }
            Expr::RefMut(_, p) => {
                let p = self.infer_place(p);
                let loan = Loan {
                    place: p.clone(),
                    mutable: true,
                };
                let t = Type::RefMut(vec![loan], Rc::new(p.ty().clone()));
                Expr::RefMut(t, p)
            }
            Expr::Seq(_, e0, e1) => {
                let e0 = self.infer_expr(e0);
                let e1 = self.infer_expr(e1);
                Expr::Seq(e1.ty().clone(), Rc::new(e0), Rc::new(e1))
            }
            Expr::Assign(_, p, e) => {
                let p = self.infer_place(p);
                if !p.is_mutable() {
                    panic!("cannot assign to immutable variable {:?}", p);
                }
                let e = self.infer_expr(e);
                if p.ty() != e.ty() {
                    panic!("mismatched types: {:?} != {:?}", p.ty(), e.ty());
                }
                Expr::Assign(Type::Unit, p, Rc::new(e))
            }
            Expr::String(_, s) => Expr::String(Type::String, s.clone()),
            Expr::Block(_, b) => {
                let b = self.infer_block(b);
                Expr::Block(b.expr.ty().clone(), Rc::new(b))
            }
            Expr::Unit(_) => Expr::Unit(Type::Unit),
        }
    }

    pub fn infer_block(&mut self, b: &Block) -> Block {
        self.stack.push(Scope { bindings: vec![] });
        let stmts = b
            .stmts
            .iter()
            .map(|s| match s {
                Stmt::Let(l, e) => {
                    let e = self.infer_expr(e);
                    let l = Local {
                        id: l.id.clone(),
                        ty: e.ty().clone(),
                        mutable: l.mutable,
                    };
                    self.add_binding(l.clone());
                    Stmt::Let(l, e)
                }
                Stmt::Expr(e) => {
                    let e = self.infer_expr(e);
                    Stmt::Expr(e)
                }
            })
            .collect::<Vec<_>>();
        self.stack.pop();
        let expr = self.infer_expr(&b.expr);
        Block { stmts, expr }
    }

    pub fn infer_place(&self, p: &Place) -> Place {
        let local = Local {
            id: p.local.id.clone(),
            ty: self.lookup(&p.local.id).unwrap().ty.clone(),
            mutable: false,
        };
        let elems = p.elems.iter().map(|e| e.clone()).collect();
        Place { local, elems }
    }
}

impl Function {
    pub fn infer(&self) -> Function {
        let mut ctx = Context::new();
        ctx.infer_function(self)
    }
}
