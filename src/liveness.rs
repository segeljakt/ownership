use crate::ast::Place;
use crate::mir;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Terminator;

fn compute_liveness(function: &mut Function) {
    let mut changed = true;
    while changed {
        changed = false;

        let old_blocks = function.blocks.clone();

        for block in function.blocks.iter_mut().rev() {
            // Compute live-out for the block
            let mut live_out = Vec::new();
            if let Some(terminator) = &block.terminator {
                // A block's live-out is the union of its successor's live-in
                match terminator {
                    Terminator::Goto(b) | Terminator::ConditionalGoto(_, b, _) => {
                        live_out.extend(old_blocks[*b].live_in.iter().cloned());
                    }
                    Terminator::Return => {}
                }
            }

            // Compute live-out for each statement in reverse order
            for stmt in block.stmts.iter_mut().rev() {
                let old_live_out = stmt.live_out.clone();
                let old_live_in = stmt.live_in.clone();

                // Update live-out for the statement
                stmt.live_out = live_out.clone();

                // Update live-in for the statement
                let used = stmt.op.used();
                if let Some(def) = stmt.op.defined() {
                    // live_in = (live_out - {def}) U use
                    stmt.live_in = stmt
                        .live_out
                        .iter()
                        .filter(|v| def != *v)
                        .cloned()
                        .collect();
                } else {
                    stmt.live_in = stmt.live_out.clone();
                }

                stmt.live_in.extend(used.iter().cloned());

                // Update block_live_out for the next statement
                live_out = stmt.live_in.clone();

                // Check if any changes occurred
                let new_live_out = stmt.live_out.clone();
                let new_live_in = stmt.live_in.clone();

                if old_live_out != new_live_out || old_live_in != new_live_in {
                    changed = true;
                }
            }

            // Update block's live-in and live-out
            block.live_out = live_out.clone();
            block.live_in = block
                .stmts
                .iter()
                .flat_map(|stmt| stmt.live_in.iter().cloned())
                .collect();
        }
    }
}

impl Operation {
    fn used(&self) -> Vec<Place> {
        match self {
            Operation::Assign(_, rv) => match rv {
                Rvalue::Use(op) => op.used().cloned().into_iter().collect(),
                Rvalue::Ref { place, .. } => vec![place.clone()],
            },
            Operation::Call { func, args, .. } => func
                .used()
                .into_iter()
                .chain(args.iter().flat_map(|arg| arg.used()))
                .cloned()
                .collect(),
            Operation::StorageLive(_) | Operation::StorageDead(_) => Vec::new(),
        }
    }

    fn defined(&self) -> Option<&Place> {
        match self {
            Operation::Assign(p, _) => Some(p),
            Operation::Call { dest, .. } => Some(dest),
            Operation::StorageLive(_) | Operation::StorageDead(_) => None,
        }
    }
}

impl Operand {
    fn used(&self) -> Option<&Place> {
        match self {
            Operand::Constant(_) | Operand::Function(_) => None,
            Operand::Copy(p) | Operand::Move(p) => Some(p),
        }
    }
}

impl mir::Function {
    pub fn compute_liveness(mut self) -> Function {
        compute_liveness(&mut self);
        self
    }
}
