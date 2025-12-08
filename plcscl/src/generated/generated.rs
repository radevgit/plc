// Generated SCL parser modules

pub mod lexer {
    include!(concat!(env!("OUT_DIR"), "/generated_lexer.rs"));
}

pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/generated_ast.rs"));
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/generated_parser.rs"));
}

pub use ast::*;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ParserLimits, ParseError, ParseErrorKind};
