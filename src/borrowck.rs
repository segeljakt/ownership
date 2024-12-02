use crate::ast::Loan;
use crate::ast::Place;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;

pub struct Context<'a> {
    function: &'a Function,
}

impl<'a> Context<'a> {
    pub fn new(function: &'a Function) -> Context<'a> {
        Context { function }
    }

    fn check(&self) {
        for block in &self.function.blocks {
            for stmt in &block.stmts {
                match &stmt.op {
                    Operation::Assign(p, r) => {
                        let loan = Loan {
                            place: p.clone(),
                            mutable: true,
                        };
                        if !self.permits(&stmt.live_out, &loan) {
                            panic!("Borrowck error");
                        }
                        match r {
                            Rvalue::Use(o) => match o {
                                Operand::Constant(_) => {}
                                Operand::Copy(_) => {}
                                Operand::Move(p) => {
                                    let loan = Loan {
                                        place: p.clone(),
                                        mutable: false,
                                    };
                                    if !self.permits(&stmt.live_out, &loan) {
                                        panic!("Borrowck error");
                                    }
                                }
                                Operand::Function(_) => {}
                            },
                            Rvalue::Ref { mutable: _, place } => {
                                if !stmt.live_out.contains(place) {
                                    panic!(
                                        "Borrowck error: reference to a dead variable {}",
                                        place
                                    );
                                }
                            }
                        }
                    }
                    Operation::StorageLive(_) => {}
                    Operation::StorageDead(_) => {}
                    Operation::Call {
                        dest: _,
                        func: _,
                        args: _,
                    } => {}
                }
            }
        }
    }

    fn permits(&self, live_out: &[Place], loan1: &Loan) -> bool {
        for place in live_out {
            for loan2 in place.ty().loans() {
                if !self.compatible(loan1, &loan2) {
                    return false;
                }
            }
        }
        true
    }

    fn compatible(&self, loan1: &Loan, loan2: &Loan) -> bool {
        (!loan1.mutable && !loan2.mutable) || self.disjoint(&loan1.place, &loan2.place)
    }

    fn disjoint(&self, p1: &Place, p2: &Place) -> bool {
        !p1.is_prefix_of(p2) && !p2.is_prefix_of(p1)
    }
}

impl Function {
    pub fn borrowck(&self) {
        Context::new(self).check();
    }
}
