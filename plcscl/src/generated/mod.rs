// Generated SCL parser modules

pub mod lexer {
    include!("generated_lexer.rs");
}

pub mod ast {
    include!("generated_ast.rs");
}

pub mod parser {
    include!("generated_parser.rs");
}

pub use ast::*;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ParserLimits, ParseError};

