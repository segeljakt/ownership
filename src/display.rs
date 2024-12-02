use std::fmt::Display;

use crate::ast;
use crate::ast::Expr;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Place;
use crate::ast::Type;
use crate::mir;
use crate::mir::BasicBlock;
use crate::mir::Constant;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Stmt;
use crate::mir::Terminator;

struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    indent_level: usize,
    verbose: bool,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer {
            f,
            indent_level: 0,
            verbose: false,
        }
    }

    fn lit(&mut self, s: impl Display) -> std::fmt::Result {
        write!(self.f, "{}", s)
    }

    fn space(&mut self) -> std::fmt::Result {
        self.lit(" ")
    }

    fn indent(&mut self) -> std::fmt::Result {
        for _ in 0..self.indent_level {
            self.lit("    ")?;
        }
        Ok(())
    }

    fn newline(&mut self) -> std::fmt::Result {
        self.lit("\n")?;
        self.indent()
    }

    fn local(&mut self, l: &Local) -> std::fmt::Result {
        if l.mutable {
            self.lit("mut")?;
            self.space()?;
        }
        self.lit(&l.id)
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Int => self.lit("i32"),
            Type::Bool => self.lit("bool"),
            Type::Unit => self.lit("()"),
            Type::Tuple(tys) => {
                self.lit("(")?;
                for (i, ty) in tys.iter().enumerate() {
                    if i > 0 {
                        self.lit(",")?;
                        self.space()?;
                    }
                    self.ty(ty)?;
                }
                self.lit(")")
            }
            Type::Ref(loans, ty) => {
                self.lit("&")?;
                self.lit("{")?;
                for (i, loan) in loans.iter().enumerate() {
                    if i > 0 {
                        self.lit(",")?;
                        self.space()?;
                    }
                    self.loan(&loan)?;
                }
                self.lit("}")?;
                self.space()?;
                self.ty(ty)
            }
            Type::RefMut(loans, ty) => {
                self.lit("&mut")?;
                self.lit("{")?;
                for (i, loan) in loans.iter().enumerate() {
                    if i > 0 {
                        self.lit(",")?;
                        self.space()?;
                    }
                    self.loan(&loan)?;
                }
                self.lit("}")?;
                self.space()?;
                self.ty(ty)
            }
            Type::Unknown => self.lit("?"),
            Type::String => self.lit("String"),
        }
    }

    fn loan(&mut self, loan: &Loan) -> std::fmt::Result {
        if loan.mutable {
            self.lit("mut")?;
        } else {
            self.lit("shared")?;
        }
        self.lit("(")?;
        self.place(&loan.place)?;
        self.lit(")")
    }

    fn place(&mut self, place: &Place) -> std::fmt::Result {
        if self.verbose {
            self.lit("(")?;
        }
        self.local(&place.local)?;
        for elem in &place.elems {
            match elem {
                ast::PlaceElem::Index(i) => {
                    self.lit(".")?;
                    self.lit(&i)?;
                }
                ast::PlaceElem::Deref => {
                    self.lit(".*")?;
                }
            }
        }
        if self.verbose {
            self.lit(")")?;
            self.lit(":")?;
            self.ty(&place.ty())?;
        }
        Ok(())
    }

    fn ast_function(&mut self, f: &ast::Function) -> std::fmt::Result {
        self.lit("fn")?;
        self.space()?;
        self.lit(&f.id)?;
        self.lit("(")?;
        self.typed_locals(&f.params)?;
        self.lit(")")?;
        self.lit(":")?;
        self.space()?;
        self.ty(&f.ty)?;
        self.space()?;
        self.ast_block(&f.body)
    }

    fn typed_local(&mut self, l: &Local) -> std::fmt::Result {
        self.local(l)?;
        self.lit(":")?;
        self.space()?;
        self.ty(&l.ty)
    }

    fn typed_locals(&mut self, locals: &[Local]) -> std::fmt::Result {
        for (i, l) in locals.iter().enumerate() {
            if i > 0 {
                self.lit(",")?;
                self.space()?;
            }
            self.typed_local(l)?;
        }
        Ok(())
    }

    fn mir_function(&mut self, f: &crate::mir::Function) -> std::fmt::Result {
        self.lit("fn")?;
        self.space()?;
        self.lit(&f.id)?;
        self.lit("(")?;
        self.typed_locals(&f.params)?;
        self.lit(")")?;
        self.lit(":")?;
        self.space()?;
        self.ty(&f.ty)?;
        self.space()?;
        self.lit("{")?;
        self.indent_level += 1;
        for l in &f.locals {
            self.newline()?;
            self.lit("let")?;
            self.space()?;
            self.typed_local(l)?;
            self.lit(";")?;
        }
        for block in &f.blocks {
            self.newline()?;
            self.mir_block(block)?;
        }
        self.indent_level -= 1;
        self.newline()?;
        self.lit("}")
    }

    fn ast_stmt(&mut self, stmt: &ast::Stmt) -> std::fmt::Result {
        match stmt {
            ast::Stmt::Let(l, e) => {
                self.lit("let")?;
                self.space()?;
                self.local(l)?;
                self.lit(":")?;
                self.space()?;
                self.ty(&l.ty)?;
                self.space()?;
                self.lit("=")?;
                self.space()?;
                self.expr(e)
            }
            ast::Stmt::Expr(e) => self.expr(e),
        }
    }

    fn mir_stmt(&mut self, stmt: &Stmt) -> std::fmt::Result {
        match &stmt.op {
            Operation::Assign(place, rvalue) => {
                self.place(place)?;
                self.space()?;
                self.lit("=")?;
                self.space()?;
                self.rvalue(rvalue)
            }
            Operation::StorageLive(l) => {
                self.lit("StorageLive")?;
                self.lit("(")?;
                self.lit(&l.id)?;
                self.lit(")")
            }
            Operation::StorageDead(l) => {
                self.lit("StorageDead")?;
                self.lit("(")?;
                self.lit(&l.id)?;
                self.lit(")")
            }
            Operation::Call {
                dest: destination,
                func,
                args,
            } => {
                self.place(destination)?;
                self.space()?;
                self.lit("=")?;
                self.space()?;
                self.operand(func)?;
                for arg in args {
                    self.space()?;
                    self.operand(arg)?;
                }
                Ok(())
            }
        }
    }

    fn mir_block(&mut self, block: &BasicBlock) -> std::fmt::Result {
        self.lit("bb")?;
        self.lit(&block.id)?;
        self.lit(":")?;
        self.space()?;
        self.lit("{")?;
        self.indent_level += 1;
        for stmt in &block.stmts {
            self.newline()?;
            self.mir_stmt(stmt)?;
            self.lit(";")?;
            self.space()?;
            self.lit("//")?;
            self.space()?;
            self.lit("live_in =")?;
            self.space()?;
            self.lit("[")?;
            self.places(&stmt.live_in)?;
            self.lit("],")?;
            self.space()?;
            self.lit("live_out =")?;
            self.space()?;
            self.lit("[")?;
            self.places(&stmt.live_out)?;
            self.lit("]")?;
        }
        if let Some(ref terminator) = block.terminator {
            self.newline()?;
            self.terminator(terminator)?;
        }
        self.indent_level -= 1;
        self.newline()?;
        self.lit("}")
    }

    fn places(&mut self, places: &[Place]) -> std::fmt::Result {
        for (i, place) in places.iter().enumerate() {
            if i > 0 {
                self.lit(",")?;
                self.space()?;
            }
            self.place(place)?;
        }
        Ok(())
    }

    fn ast_block(&mut self, block: &ast::Block) -> std::fmt::Result {
        self.lit("{")?;
        self.indent_level += 1;
        for stmt in &block.stmts {
            self.newline()?;
            self.ast_stmt(stmt)?;
            self.lit(";")?;
        }
        self.newline()?;
        self.expr(&block.expr)?;
        self.indent_level -= 1;
        self.newline()?;
        self.lit("}")
    }

    fn terminator(&mut self, terminator: &Terminator) -> std::fmt::Result {
        match terminator {
            Terminator::Return => self.lit("return"),
            Terminator::Goto(block_id) => {
                self.lit("goto")?;
                self.space()?;
                self.lit(&block_id)
            }
            Terminator::ConditionalGoto(cond, block_id1, block_id2) => {
                self.lit("if")?;
                self.space()?;
                self.operand(cond)?;
                self.space()?;
                self.lit("goto")?;
                self.space()?;
                self.lit(&block_id1)?;
                self.space()?;
                self.lit("else")?;
                self.space()?;
                self.lit("goto")?;
                self.space()?;
                self.lit(&block_id2)
            }
        }
    }

    fn rvalue(&mut self, rvalue: &Rvalue) -> std::fmt::Result {
        match rvalue {
            Rvalue::Use(operand) => self.operand(operand),
            Rvalue::Ref { mutable, place } => {
                if *mutable {
                    self.lit("&mut")?;
                } else {
                    self.lit("&")?;
                }
                self.place(place)
            }
        }
    }

    fn operand(&mut self, operand: &Operand) -> std::fmt::Result {
        match operand {
            Operand::Constant(c) => self.constant(c),
            Operand::Copy(place) => {
                self.lit("copy")?;
                self.lit("(")?;
                self.place(place)?;
                self.lit(")")
            }
            Operand::Move(place) => {
                self.lit("move")?;
                self.lit("(")?;
                self.place(place)?;
                self.lit(")")
            }
            Operand::Function(name) => self.lit(&name),
        }
    }

    fn constant(&mut self, c: &Constant) -> std::fmt::Result {
        match c {
            Constant::Int(i) => self.lit(i),
            Constant::Bool(b) => self.lit(b),
            Constant::String(s) => self.lit(&s),
            Constant::Unit => self.lit("()"),
        }
    }

    fn expr(&mut self, e: &Expr) -> std::fmt::Result {
        if self.verbose {
            self.lit("(")?;
        }
        match e {
            // Expr::Let(_, l, e1, e2) => {
            //     self.lit("let")?;
            //     self.space()?;
            //     self.local(l)?;
            //     self.lit(":")?;
            //     self.space()?;
            //     self.ty(&l.ty)?;
            //     self.space()?;
            //     self.lit("=")?;
            //     self.space()?;
            //     self.expr(e1)?;
            //     self.space()?;
            //     self.lit("in")?;
            //     self.space()?;
            //     self.expr(e2)?;
            // }
            Expr::IfElse(_, e1, b2, b3) => {
                self.lit("if")?;
                self.space()?;
                self.expr(e1)?;
                self.space()?;
                self.lit("then")?;
                self.space()?;
                self.ast_block(b2)?;
                self.space()?;
                self.lit("else")?;
                self.space()?;
                self.ast_block(b3)?;
            }
            Expr::While(_, e, b) => {
                self.lit("while")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.lit("do")?;
                self.space()?;
                self.ast_block(b)?;
            }
            Expr::Tuple(_, es) => {
                self.lit("(")?;
                for (i, e) in es.iter().enumerate() {
                    if i > 0 {
                        self.lit(",")?;
                        self.space()?;
                    }
                    self.expr(e)?;
                }
                self.lit(")")?;
            }
            Expr::Ref(_, place) => {
                self.lit("&")?;
                self.place(place)?;
            }
            Expr::RefMut(_, place) => {
                self.lit("&mut")?;
                self.place(place)?;
            }
            Expr::Seq(_, e1, e2) => {
                self.lit("seq")?;
                self.lit("(")?;
                self.expr(e1)?;
                self.lit(",")?;
                self.space()?;
                self.expr(e2)?;
                self.lit(")")?;
            }
            Expr::Assign(_, place, e) => {
                self.lit("assign")?;
                self.lit("(")?;
                self.place(place)?;
                self.lit(",")?;
                self.space()?;
                self.expr(e)?;
                self.lit(")")?;
            }
            Expr::Place(_, p) => {
                self.place(p)?;
            }
            Expr::Add(_, e1, e2) => {
                self.lit("add")?;
                self.lit("(")?;
                self.expr(e1)?;
                self.lit(",")?;
                self.space()?;
                self.expr(e2)?;
                self.lit(")")?;
            }
            Expr::Int(_, i) => {
                self.lit(i)?;
            }
            Expr::Bool(_, b) => {
                self.lit(b)?;
            }
            Expr::String(_, s) => {
                self.lit(&s)?;
            }
            Expr::Block(_, b) => {
                self.ast_block(b)?;
            }
            Expr::Unit(_) => {
                self.lit("()")?;
            }
        }
        if self.verbose {
            self.lit(")")?;
            self.lit(":")?;
            self.ty(&e.ty())?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ast::Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).ast_function(self)
    }
}

impl std::fmt::Display for crate::mir::Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).mir_function(self)
    }
}

impl<'a> std::fmt::Display for Loan {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).loan(self)
    }
}

impl<'a> std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).place(self)
    }
}

pub struct Verbose<T>(T);

impl ast::Function {
    pub fn verbose(&self) -> Verbose<&ast::Function> {
        Verbose(&self)
    }
}

impl crate::mir::Function {
    pub fn verbose(&self) -> Verbose<&mir::Function> {
        Verbose(&self)
    }
}

impl Loan {
    pub fn verbose(&self) -> Verbose<&Loan> {
        Verbose(&self)
    }
}

impl Place {
    pub fn verbose(&self) -> Verbose<&Place> {
        Verbose(&self)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a ast::Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.ast_function(&self.0)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a mir::Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.mir_function(&self.0)
    }
}

impl std::fmt::Display for Verbose<&Loan> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.loan(&self.0)
    }
}

impl std::fmt::Display for Verbose<&Place> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut printer = Printer::new(f);
        printer.verbose = true;
        printer.place(&self.0)
    }
}