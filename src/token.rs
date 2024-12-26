#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Equal,          // =
    Colon,          // :
    SemiColon,      // ;
    LeftBrace,      // {
    RightBrace,     // }
    LeftParen,      // (
    RightParen,     // )
    Ident,          // [a-zA-Z_][a-zA-Z0-9_]*
    String,         // "..."
    Number,         // 123
    Eof,            // End of file
    Mut,            // mut
    Fn,             // fn
    Let,            // let
    If,             // if
    Else,           // else
    While,          // while
    Return,         // return
    Ampersand,      // &
    Plus,           // +
    Minus,          // -
    Slash,          // /
    Star,           // *
    Dot,            // .
    Label,
    Err,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub data: T,
    pub start: usize,
    pub end: usize,
}

impl<T> Spanned<T> {
    pub fn new(data: T, span: std::ops::Range<usize>) -> Self {
        Spanned { data, start: span.start, end: span.end }
    }

    pub fn span(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}
