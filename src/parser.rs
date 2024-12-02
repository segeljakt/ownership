use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Function;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Stmt;
use crate::ast::Type;
use std::rc::Rc;

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.skip_whitespace();
        if self.at("if") {
            self.parse_ifelse()
        } else if self.at("while") {
            self.parse_while()
        } else if self.at("add") {
            self.parse_add()
        } else if self.at("assign") {
            self.consume("assign(")?;
            let place = self.parse_place()?;
            self.consume(",")?;
            let value = self.parse_expr()?;
            self.consume(")")?;
            Some(Expr::Assign(Type::Unknown, place, Rc::new(value)))
        } else if self.at("(") {
            self.parse_tuple_expr()
        } else if self.consume("&mut").is_some() {
            let place = self.parse_place()?;
            Some(Expr::RefMut(Type::Unknown, place))
        } else if self.consume("&").is_some() {
            let place = self.parse_place()?;
            Some(Expr::Ref(Type::Unknown, place))
        } else if self.consume("seq(").is_some() {
            let first = self.parse_expr()?;
            self.consume(",")?;
            let second = self.parse_expr()?;
            self.consume(")")?;
            Some(Expr::Seq(Type::Unknown, Rc::new(first), Rc::new(second)))
        } else if self.consume("assign(").is_some() {
            let place = self.parse_place()?;
            self.consume(",")?;
            let value = self.parse_expr()?;
            self.consume(")")?;
            Some(Expr::Assign(Type::Unknown, place, Rc::new(value)))
        } else if self.at("deref(") || self.at("index(") {
            let place = self.parse_place()?;
            Some(Expr::Place(Type::Unknown, place))
        } else {
            if let Some(place) = self.parse_place() {
                Some(Expr::Place(Type::Unknown, place))
            } else {
                self.parse_literal()
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        self.consume("let")?;
        self.skip_whitespace();
        let mutable = self.consume("mut").is_some();
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.consume(":")?;
        let ty = self.parse_type()?;
        self.consume("=")?;
        let expr = self.parse_expr()?;
        let local = Local {
            id: name,
            ty,
            mutable,
        };
        Some(Stmt::Let(local, expr))
    }

    fn parse_ifelse(&mut self) -> Option<Expr> {
        self.consume("if")?;
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        self.consume("else")?;
        let else_branch = self.parse_block()?;
        Some(Expr::IfElse(
            Type::Unknown,
            Rc::new(cond),
            Rc::new(then_branch),
            Rc::new(else_branch),
        ))
    }

    fn parse_while(&mut self) -> Option<Expr> {
        self.consume("while")?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Some(Expr::While(Type::Unknown, Rc::new(cond), Rc::new(body)))
    }

    fn parse_add(&mut self) -> Option<Expr> {
        self.consume("add")?;
        self.consume("(")?;
        let lhs = self.parse_expr()?;
        self.consume(",")?;
        let rhs = self.parse_expr()?;
        self.consume(")")?;
        Some(Expr::Add(Type::Unknown, Rc::new(lhs), Rc::new(rhs)))
    }

    fn parse_place(&mut self) -> Option<Place> {
        self.skip_whitespace();

        let name = self.parse_identifier()?;
        let mut elems = Vec::new();

        loop {
            if self.consume(".deref").is_some() {
                elems.push(PlaceElem::Deref);
            } else if self.consume(".index(").is_some() {
                let index = self.parse_int()? as usize;
                self.consume(")")?;
                elems.push(PlaceElem::Index(index));
            } else {
                // Base case: a simple name
                let local = Local {
                    id: name,
                    ty: Type::Unknown,
                    mutable: false,
                };
                return Some(Place { local, elems });
            }
        }
    }

    fn parse_literal(&mut self) -> Option<Expr> {
        if let Some(value) = self.parse_int() {
            Some(Expr::Int(Type::Int, value))
        } else if let Some(value) = self.parse_bool() {
            Some(Expr::Bool(Type::Bool, value))
        } else if let Some(value) = self.parse_string() {
            Some(Expr::String(Type::Unknown, value))
        } else {
            None
        }
    }

    pub fn parse_function(&mut self) -> Option<Function> {
        self.consume("fn")?;
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.consume("(")?;
        let mut params = Vec::new();
        while self.consume(")").is_none() {
            let mutable = self.consume("mut").is_some();
            let id = self.parse_identifier()?;
            self.consume(":")?;
            let ty = self.parse_type()?;
            params.push(Local { id, ty, mutable });
            if self.consume(",").is_none() {
                self.consume(")")?;
                break;
            }
        }
        self.consume("->")?;
        let ty = self.parse_type()?;
        let body = self.parse_block()?;
        Some(Function {
            id: name,
            params,
            ty,
            body,
        })
    }

    fn parse_int(&mut self) -> Option<i32> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].parse().ok()
    }

    fn parse_bool(&mut self) -> Option<bool> {
        if self.consume("true").is_some() {
            Some(true)
        } else if self.consume("false").is_some() {
            Some(false)
        } else {
            None
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        if self.peek_char()? == '"' {
            self.advance();
            let start = self.pos;
            while let Some(c) = self.peek_char() {
                if c == '"' {
                    let value = self.input[start..self.pos].to_string();
                    self.advance(); // Consume closing quote
                    return Some(value);
                }
                self.advance();
            }
        }
        None
    }

    fn parse_tuple_expr(&mut self) -> Option<Expr> {
        self.consume("(")?; // Consume the opening parenthesis
        let mut elements = Vec::new();

        // Parse the first element
        if let Some(first) = self.parse_expr() {
            elements.push(first);

            // Parse additional elements if commas are found
            while self.consume(",").is_some() {
                if let Some(next) = self.parse_expr() {
                    elements.push(next);
                } else {
                    return None; // Invalid syntax
                }
            }
        }

        self.consume(")")?; // Consume the closing parenthesis

        if elements.len() == 1 {
            // A single element in parentheses is not a tuple, just return the inner expression
            Some(elements.into_iter().next().unwrap())
        } else {
            // Return a tuple expression
            Some(Expr::Tuple(
                Type::Tuple(vec![Type::Unknown; elements.len()]),
                elements,
            ))
        }
    }

    fn parse_type(&mut self) -> Option<Type> {
        self.skip_whitespace();

        // Check for reference types (& or &mut)
        if self.consume("&{").is_some() {
            let mut loans = Vec::new();

            loop {
                self.skip_whitespace();

                if self.consume("mut(").is_some() {
                    // Parse a mutable loan
                    let place = self.parse_place()?;
                    self.consume(")")?;
                    loans.push(Loan {
                        place,
                        mutable: true,
                    });
                } else if self.consume("shared(").is_some() {
                    // Parse a shared loan
                    let place = self.parse_place()?;
                    self.consume(")")?;
                    loans.push(Loan {
                        place,
                        mutable: false,
                    });
                } else {
                    break; // End of loan list
                }

                self.skip_whitespace();
                if !self.consume(",").is_some() {
                    break; // No more loans
                }
            }

            self.consume("}")?; // End of loan list
            self.skip_whitespace();

            // Determine if it's a mutable or shared reference
            if self.consume("mut").is_some() {
                let t = Rc::new(self.parse_type()?);
                Some(Type::RefMut(loans, t))
            } else {
                let t = Rc::new(self.parse_type()?);
                Some(Type::Ref(loans, t))
            }
        } else if self.consume("i32").is_some() {
            Some(Type::Int)
        } else if self.consume("String").is_some() {
            Some(Type::String)
        } else if self.consume("bool").is_some() {
            Some(Type::Bool)
        } else if self.consume("()").is_some() {
            Some(Type::Unit)
        } else if self.consume("(").is_some() {
            // Parse tuple types
            let mut types = Vec::new();
            if let Some(first) = self.parse_type() {
                types.push(first);
                while self.consume(",").is_some() {
                    if let Some(next) = self.parse_type() {
                        types.push(next);
                    } else {
                        return None; // Invalid tuple type
                    }
                }
            }
            self.consume(")")?;
            Some(Type::Tuple(types))
        } else {
            None
        }
    }

    fn parse_identifier(&mut self) -> Option<String> {
        self.skip_whitespace();
        if let Some(c) = self.peek_char() {
            if !(c.is_alphabetic() || c == '_') {
                return None;
            }
        }
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if start == self.pos {
            None
        } else {
            Some(self.input[start..self.pos].to_string())
        }
    }

    fn parse_block(&mut self) -> Option<Block> {
        self.consume("{")?;
        let mut stmts = Vec::new();
        while !self.at("}") {
            if self.at("let") {
                stmts.push(self.parse_let_stmt()?);
            } else {
                let expr = self.parse_expr()?;
                if self.consume("}").is_some() {
                    return Some(Block { stmts, expr });
                } else {
                    stmts.push(Stmt::Expr(expr));
                }
            }
            self.consume(";")?;
        }
        self.consume("}")?;
        let expr = Expr::Unit(Type::Unknown);
        Some(Block { stmts, expr })
    }

    fn at(&self, expected: &str) -> bool {
        self.input[self.pos..].starts_with(expected)
    }

    fn consume(&mut self, expected: &str) -> Option<()> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Some(())
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos += c.len_utf8();
        }
    }
}

impl Function {
    pub fn parse(input: &str) -> Option<Self> {
        let mut parser = Parser::new(input);
        parser.parse_function()
    }
}
