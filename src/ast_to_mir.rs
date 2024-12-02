use crate::ast;
use crate::ast::Expr;
use crate::ast::Local;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Type;
use crate::mir;
use crate::mir::BasicBlock;
use crate::mir::BlockId;
use crate::mir::Constant;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Stmt;
use crate::mir::Terminator;

pub struct Context {
    func: mir::Function,
    temp_counter: usize,
    stack: Vec<Scope>,
}

struct Scope {
    locals: Vec<Local>,
}

impl Context {
    pub fn new(function: mir::Function) -> Context {
        Context {
            func: function,
            temp_counter: 0,
            stack: vec![],
        }
    }
}

impl ast::Function {
    pub fn into_mir(self) -> mir::Function {
        let func = mir::Function {
            id: self.id,
            params: self.params,
            locals: vec![],
            ty: self.ty.clone(),
            blocks: vec![BasicBlock {
                id: 0,
                terminator: None,
                stmts: vec![],
                live_in: vec![],
                live_out: vec![],
            }],
        };
        let mut ctx = Context::new(func);
        ctx.push_scope();
        let l0 = ctx.new_local(self.ty);
        let (b1, l1) = ctx.lower_block(&self.body, 0);
        let r = ctx.use_local(l1);
        ctx.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
            Place {
                local: l0.clone(),
                elems: vec![],
            },
            r,
        )));
        ctx.pop_scope();
        ctx.func.blocks[b1].terminator = Some(Terminator::Return);
        ctx.func
    }
}

impl Context {
    pub fn push_scope(&mut self) {
        self.stack.push(Scope { locals: vec![] });
    }

    fn pop_scope(&mut self) -> Scope {
        self.stack.pop().unwrap()
    }

    pub fn add_storage_live(&mut self, l: Local) {
        self.func
            .blocks
            .last_mut()
            .unwrap()
            .stmts
            .push(Stmt::new(Operation::StorageLive(l.clone())));
    }

    pub fn add_storage_dead(&mut self, l: Local) {
        // Only add storage dead if the value was not moved
        self.func
            .blocks
            .last_mut()
            .unwrap()
            .stmts
            .push(Stmt::new(Operation::StorageDead(l.clone())));
    }

    pub fn lower_block(&mut self, b: &ast::Block, b0: BlockId) -> (BlockId, Local) {
        self.push_scope();
        let b1 = b.stmts.iter().fold(b0, |b1, s| match s {
            ast::Stmt::Let(l, e) => {
                let (b1, l1) = self.lower_expr(e, b1);
                let r = self.use_local(l1);
                self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l.clone(),
                        elems: vec![],
                    },
                    r,
                )));
                b1
            }
            ast::Stmt::Expr(e) => {
                let (b1, _) = self.lower_expr(e, b1);
                b1
            }
        });
        let e = self.lower_expr(&b.expr, b1);
        for l in self.pop_scope().locals {
            self.add_storage_dead(l);
        }
        e
    }
    /// Returns the current block and the local that holds the result of the expression.
    pub fn lower_expr(&mut self, e: &Expr, b0: BlockId) -> (BlockId, Local) {
        match e {
            Expr::Int(t, v) => {
                let l = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l.clone(),
                        elems: vec![],
                    },
                    Rvalue::Use(Operand::Constant(Constant::Int(*v))),
                )));
                (b0, l)
            }
            // Lower addition as a call to a built-in
            Expr::Add(t, e0, e1) => {
                let (b0, l0) = self.lower_expr(e0, b0);
                let (b1, l1) = self.lower_expr(e1, b0);
                let l2 = self.new_live_local(t.clone());
                self.func.blocks[b1].stmts.push(Stmt::new(Operation::Call {
                    dest: Place {
                        local: l2.clone(),
                        elems: vec![],
                    },
                    func: Operand::Function("add".to_string()),
                    args: vec![
                        Operand::Copy(Place {
                            local: l0.clone(),
                            elems: vec![],
                        }),
                        Operand::Copy(Place {
                            local: l1.clone(),
                            elems: vec![],
                        }),
                    ],
                }));
                (b1, l2)
            }
            Expr::IfElse(t, e0, e1, e2) => {
                let (b0, l0) = self.lower_expr(e0, b0);
                let b1_start = self.new_block();
                let b2_start = self.new_block();

                self.func.blocks[b0]
                    .terminator
                    .replace(Terminator::ConditionalGoto(
                        Operand::Copy(Place {
                            local: l0.clone(),
                            elems: vec![],
                        }),
                        b1_start,
                        b2_start,
                    ));

                let (b1, l1) = self.lower_block(e1, b1_start);
                let (b2, l2) = self.lower_block(e2, b2_start);

                let b3 = self.new_block();
                let l3 = self.new_live_local(t.clone());

                self.func.blocks[b1]
                    .terminator
                    .replace(Terminator::Goto(b3));
                self.func.blocks[b2]
                    .terminator
                    .replace(Terminator::Goto(b3));

                let r = self.use_local(l1);
                self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l3.clone(),
                        elems: vec![],
                    },
                    r,
                )));

                let r = self.use_local(l2);
                self.func.blocks[b2].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l3.clone(),
                        elems: vec![],
                    },
                    r,
                )));

                (b3, l3)
            }
            Expr::While(_, e0, e1) => {
                let b0_start = self.new_block();
                let b1_start = self.new_block();
                let b2_start = self.new_block();

                self.func.blocks[b0]
                    .terminator
                    .replace(Terminator::Goto(b0_start));

                let (b0, l0) = self.lower_expr(e0, b0_start);

                self.func.blocks[b0]
                    .terminator
                    .replace(Terminator::ConditionalGoto(
                        Operand::Copy(Place {
                            local: l0.clone(),
                            elems: vec![],
                        }),
                        b1_start,
                        b2_start,
                    ));

                let (b1, _) = self.lower_block(e1, b1_start);

                self.func.blocks[b1]
                    .terminator
                    .replace(Terminator::Goto(b0_start));

                let l2 = self.new_live_local(Type::Unit);
                self.func.blocks[b2_start]
                    .stmts
                    .push(Stmt::new(Operation::Assign(
                        Place {
                            local: l2.clone(),
                            elems: vec![],
                        },
                        Rvalue::Use(Operand::Constant(Constant::Unit)),
                    )));
                (b2_start, l2)
            }
            Expr::Tuple(t, es) => {
                let l0 = self.new_live_local(t.clone());
                let b0 = es.iter().enumerate().fold(b0, |b0, (i, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    let r = self.use_local(l1);
                    self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                        Place {
                            local: l0.clone(),
                            elems: vec![PlaceElem::Index(i)],
                        },
                        r,
                    )));
                    b1
                });
                (b0, l0)
            }
            Expr::Ref(t, p) => {
                let l = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l.clone(),
                        elems: vec![],
                    },
                    Rvalue::Ref {
                        mutable: false,
                        place: p.clone(),
                    },
                )));
                (b0, l)
            }
            Expr::RefMut(t, p) => {
                let l = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l.clone(),
                        elems: vec![],
                    },
                    Rvalue::Ref {
                        mutable: true,
                        place: p.clone(),
                    },
                )));
                (b0, l)
            }
            Expr::Place(t0, p0) => {
                let l0 = self.new_live_local(t0.clone());
                let r = self.use_place(p0.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l0.clone(),
                        elems: vec![],
                    },
                    r,
                )));
                (b0, l0)
            }
            Expr::Seq(_, e0, e1) => {
                let (b0, _) = self.lower_expr(e0, b0);
                self.lower_expr(e1, b0)
            }
            Expr::Assign(_, p0, e0) => {
                let (b0, l0) = self.lower_expr(e0, b0);
                let r = self.use_local(l0);
                self.func.blocks[b0]
                    .stmts
                    .push(Stmt::new(Operation::Assign(p0.clone(), r)));
                let l1 = self.new_live_local(Type::Unit);
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l1.clone(),
                        elems: vec![],
                    },
                    Rvalue::Use(Operand::Constant(Constant::Unit)),
                )));
                (b0, l1)
            }
            Expr::Bool(t, v) => {
                let l = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l.clone(),
                        elems: vec![],
                    },
                    Rvalue::Use(Operand::Constant(Constant::Bool(*v))),
                )));
                (b0, l)
            }
            Expr::String(t, v) => {
                let l0 = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l0.clone(),
                        elems: vec![],
                    },
                    Rvalue::Use(Operand::Constant(Constant::String(v.clone()))),
                )));
                (b0, l0)
            }
            Expr::Block(_, b) => self.lower_block(b, b0),
            Expr::Unit(t) => {
                let l0 = self.new_live_local(t.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place {
                        local: l0.clone(),
                        elems: vec![],
                    },
                    Rvalue::Use(Operand::Constant(Constant::Unit)),
                )));
                (b0, l0)
            }
        }
    }

    fn new_local(&mut self, ty: Type) -> Local {
        let id = self.temp_counter;
        self.temp_counter += 1;
        let l = Local {
            id: format!("_{}", id).into(),
            ty,
            mutable: false,
        };
        self.stack.last_mut().unwrap().locals.push(l.clone());
        self.func.locals.push(l.clone());
        l
    }

    fn new_live_local(&mut self, ty: Type) -> Local {
        let l = self.new_local(ty);
        self.add_storage_live(l.clone());
        l
    }

    fn new_block(&mut self) -> BlockId {
        let block_id = self.func.blocks.len();
        self.func.blocks.push(BasicBlock {
            id: block_id,
            stmts: Vec::new(),
            terminator: None,
            live_in: Vec::new(),
            live_out: Vec::new(),
        });
        block_id
    }

    pub fn use_local(&mut self, l: Local) -> Rvalue {
        if l.ty.is_copy() {
            Rvalue::Use(Operand::Copy(Place {
                local: l,
                elems: vec![],
            }))
        } else {
            self.stack.last_mut().unwrap().locals.retain(|x| x != &l);
            Rvalue::Use(Operand::Move(Place {
                local: l,
                elems: vec![],
            }))
        }
    }

    pub fn use_place(&mut self, p: Place) -> Rvalue {
        if p.ty().is_copy() {
            Rvalue::Use(Operand::Copy(p))
        } else {
            Rvalue::Use(Operand::Move(p))
        }
    }
}
