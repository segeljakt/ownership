use std::rc::Rc;

use crate::ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::mir;
use crate::mir::BlockId;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Terminator;

impl Function {
    fn do_tree(&self, b: BlockId, loops: &mut Vec<BlockId>) -> Block {
        let merge_nodes = self.domtree[b]
            .clone()
            .into_iter()
            .filter(|&b1| self.is_merge_node(b1))
            .collect::<Vec<_>>();

        if self.is_loop_header(b) {
            loops.push(b);
            let block = self.node_within(b, merge_nodes, loops);
            loops.pop();
            Block {
                stmts: vec![Stmt::Expr(Expr::Loop(Type::Unit, Some(b), Rc::new(block)))],
                expr: None,
            }
        } else {
            self.node_within(b, merge_nodes, loops)
        }
    }

    fn node_within(
        &self,
        b: BlockId,
        mut merge_nodes: Vec<BlockId>,
        loops: &mut Vec<BlockId>,
    ) -> Block {
        if let Some(merge_node) = merge_nodes.pop() {
            let mut block1 = self.do_tree(merge_node, loops);
            let block2 = self.node_within(b, merge_nodes, loops);
            block1.stmts.extend(block2.stmts);
            block1
        } else {
            let mut stmts = Vec::new();

            for s in &self.blocks[b].stmts {
                match &s.op {
                    Operation::Assign(place, rvalue) => {
                        let rhs_expr = match rvalue {
                            Rvalue::Use(op) => self.operand_to_expr(op),
                            Rvalue::Ref { mutable, place } => {
                                if *mutable {
                                    Expr::RefMut(place.ty().clone(), place.clone())
                                } else {
                                    Expr::Ref(place.ty().clone(), place.clone())
                                }
                            }
                        };
                        stmts.push(Stmt::Expr(Expr::Assign(
                            Type::Unit,
                            place.clone(),
                            Rc::new(rhs_expr),
                        )));
                    }
                    Operation::StorageLive(l) => {
                        stmts.push(Stmt::Let(l.clone(), None));
                    }
                    Operation::StorageDead(_) => {}
                    Operation::Call { dest, func, args } => {
                        let Operand::Function(func_name) = func else {
                            unreachable!()
                        };
                        let arg_exprs: Vec<Expr> =
                            args.iter().map(|a| self.operand_to_expr(a)).collect();

                        match func_name.as_str() {
                            "print" => {
                                stmts.push(Stmt::Expr(Expr::Assign(
                                    Type::Unit,
                                    dest.clone(),
                                    Rc::new(Expr::Print(Type::Unit, Rc::new(arg_exprs[0].clone()))),
                                )));
                            }
                            "add" => {
                                stmts.push(Stmt::Expr(Expr::Assign(
                                    Type::Unit,
                                    dest.clone(),
                                    Rc::new(Expr::Add(
                                        Type::Int,
                                        Rc::new(arg_exprs[0].clone()),
                                        Rc::new(arg_exprs[1].clone()),
                                    )),
                                )));
                            }
                            _ => todo!(),
                        }
                    }
                    Operation::Noop => {}
                }
            }

            // Handle terminator
            match self.blocks[b].terminator.clone().unwrap() {
                Terminator::Return => {
                    let local = &self.locals[0];
                    stmts.push(Stmt::Expr(Expr::Return(
                        local.ty.clone(),
                        Rc::new(local.clone().into_expr()),
                    )));
                }
                Terminator::Goto(l) => {
                    stmts.extend(self.do_branch(b, l, loops).stmts);
                }
                Terminator::ConditionalGoto(cond, t, f) => {
                    let cond_expr = self.operand_to_expr(&cond);
                    let then_block = self.do_branch(b, t, loops);
                    let else_block = self.do_branch(b, f, loops);
                    stmts.push(Stmt::Expr(Expr::IfElse(
                        then_block.ty().clone(),
                        Rc::new(cond_expr),
                        Rc::new(then_block),
                        Rc::new(else_block),
                    )));
                }
            };

            Block { stmts, expr: None }
        }
    }

    fn is_loop_header(&self, b: BlockId) -> bool {
        self.predecessors[b]
            .iter()
            .any(|&pred| self.is_backward_edge(pred, b))
    }

    fn is_merge_node(&self, b: BlockId) -> bool {
        self.predecessors[b].len() > 1
    }

    fn is_backward_edge(&self, source: BlockId, target: BlockId) -> bool {
        self.reverse_postorder_number[target] <= self.reverse_postorder_number[source]
    }

    fn do_branch(&self, source: BlockId, target: BlockId, loops: &mut Vec<BlockId>) -> Block {
        if self.is_backward_edge(source, target) {
            Block {
                stmts: vec![Stmt::Expr(Expr::Continue(Type::Unit, Some(target)))],
                expr: None,
            }
        } else if self.is_merge_node(target) {
            if loops.contains(&target) {
                Block {
                    stmts: vec![Stmt::Expr(Expr::Break(Type::Unit, Some(target)))],
                    expr: None,
                }
            } else {
                Block {
                    stmts: Vec::new(),
                    expr: None,
                }
            }
        } else {
            self.do_tree(target, loops)
        }
    }

    fn operand_to_expr(&self, op: &Operand) -> Expr {
        match op {
            Operand::Constant(c) => match c {
                mir::Constant::Int(i) => Expr::Int(Type::Int, *i),
                mir::Constant::Bool(b) => Expr::Bool(Type::Bool, *b),
                mir::Constant::String(s) => Expr::String(Type::String, s.clone()),
                mir::Constant::Unit => Expr::Unit(Type::Unit),
            },
            Operand::Copy(p) | Operand::Move(p) => Expr::Place(p.ty().clone(), p.clone()),
            Operand::Function(_) => todo!(),
        }
    }
}

impl Function {
    pub fn into_ast(self) -> ast::Function {
        let mut block = Block {
            stmts: vec![Stmt::Let(self.locals[0].clone(), None)],
            expr: None,
        };
        let mut env = Vec::new();
        block.stmts.extend(self.do_tree(0, &mut env).stmts);
        ast::Function {
            id: self.id.clone(),
            params: self.params.clone(),
            ty: self.ty.clone(),
            block,
        }
    }
}
