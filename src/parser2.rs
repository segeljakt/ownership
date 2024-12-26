use crate::cst::Block;
use crate::cst::Expr;
use crate::cst::Function;
use crate::cst::Loan;
use crate::cst::Local;
use crate::cst::Place;
use crate::cst::PlaceElem;
use crate::cst::Stmt;
use crate::cst::Type;
use crate::lexer::Lexer;
use crate::token::Spanned;
use crate::token::Token;
use std::rc::Rc;

impl Token {
    fn bp(self, prefix: bool) -> Option<(u8, u8)> {
        let res = match self {
            Token::Number | Token::Ident => (99, 100),
            Token::LeftParen => (99, 0),
            Token::RightParen => (0, 100),
            Token::Equal => (2, 1),
            Token::Plus | Token::Minus | Token::Star if prefix => (99, 9),
            Token::Plus | Token::Minus => (5, 6),
            Token::Star | Token::Slash => (7, 8),
            _ => return None,
        };
        Some(res)
    }
}

struct Frame {
    min_bp: u8,
    lhs: Option<Expr>,
    token: Option<Token>,
}

pub struct Parser<'a, I: Iterator<Item = Spanned<Token>>> {
    input: &'a str,
    lexer: std::iter::Peekable<I>,
    pos: usize,
}

impl<'a, I: Iterator<Item = Spanned<Token>>> Parser<'a, I> {
    pub fn new(input: &'a str, iter: I) -> Self {
        Self {
            input,
            lexer: iter.peekable(),
            pos: 0,
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let mut top = Frame {
            min_bp: 0,
            lhs: None,
            token: None,
        };
        let mut stack = Vec::new();
        loop {
            let token = self.lexer.next();
            let r_bp = loop {
                if let Some(ref token) = token {
                    match token.data.bp(top.lhs.is_none()) {
                        Some((lbp, rbp)) if top.min_bp <= lbp => break rbp,
                        _ => {}
                    }
                    let res = top;
                    top = match stack.pop() {
                        Some(top) => top,
                        None => return res.lhs,
                    };
                    match top.token.unwrap() {
                        Token::Number => {
                            let value = self.input[token.span()].parse().unwrap();
                            top.lhs = Some(Expr::Int(Type::Int, value));
                        }
                        Token::Ident => {
                            let id = &self.input[token.span()];
                            top.lhs = Some(Expr::Var(Type::Unknown, id.to_string()));
                        }
                        Token::String => {
                            let text = &self.input[token.span()];
                            top.lhs = Some(Expr::String(Type::Unknown, text.to_string()));
                        }
                        Token::LeftParen => {
                            let rhs = res.lhs?;
                            top.lhs = Some(rhs);
                        }
                        _ => return None,
                    }
                }
            };
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        self.consume("let")?;
        self.skip_whitespace();
        let mutable = self.consume("mut").is_some();
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        let ty = if self.at(":") {
            self.consume(":")?;
            self.parse_type()?
        } else {
            Type::Unknown
        };
        self.consume("=")?;
        let expr = self.parse_expr()?;
        let local = Local {
            id: name,
            ty,
            mutable,
        };
        Some(Stmt::Let(local, Some(expr)))
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
        let ty = if self.at("->") {
            self.consume("->")?;
            self.parse_type()?
        } else {
            Type::Unit
        };
        let block = self.parse_block()?;
        Some(Function {
            id: name,
            params,
            ty,
            block,
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
        if self.consume("&").is_some() {
            let mut loans = Vec::new();

            loop {
                if self.consume("mut").is_some() {
                    self.consume("(")?;
                    let place = self.parse_place()?;
                    self.consume(")")?;
                    loans.push(Loan {
                        place,
                        mutable: true,
                    });
                } else if self.consume("shared").is_some() {
                    self.consume("(")?;
                    let place = self.parse_place()?;
                    self.consume(")")?;
                    loans.push(Loan {
                        place,
                        mutable: false,
                    });
                } else {
                    break;
                }

                if !self.consume(",").is_some() {
                    break; // No more loans
                }
            }

            self.consume("}")?; // End of loan list

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
            if types.is_empty() {
                Some(Type::Unit)
            } else if types.len() == 1 {
                types.into_iter().next()
            } else {
                Some(Type::Tuple(types))
            }
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
                    return Some(Block {
                        stmts,
                        expr: Some(expr),
                    });
                } else {
                    stmts.push(Stmt::Expr(expr));
                }
            }
            self.consume(";")?;
        }
        self.consume("}")?;
        Some(Block { stmts, expr: None })
    }

    fn peek(&mut self) -> Option<&Spanned<Token>> {
        self.lexer.peek()
    }

    fn next(&mut self) -> Option<Spanned<Token>> {
        self.lexer.next()
    }

    fn at(&mut self, expected: Token) -> bool {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: Token) -> Option<Spanned<Token>> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Some(())
        } else {
            None
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
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(input, lexer);
        parser.parse_function()
    }
}
