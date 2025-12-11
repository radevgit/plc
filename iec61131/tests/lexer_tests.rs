use iec61131::{Lexer, Token};

#[test]
fn test_dotdot_token() {
    let code = "0..9";
    let mut lexer = Lexer::new(code);
    
    let t1 = lexer.next_token();
    println!("Token 1: {:?}", t1);
    assert!(matches!(t1.token, Token::IntLiteral(_)));
    
    let t2 = lexer.next_token();
    println!("Token 2: {:?}", t2);
    assert!(matches!(t2.token, Token::DotDot), "Expected DotDot, got {:?}", t2.token);
    
    let t3 = lexer.next_token();
    println!("Token 3: {:?}", t3);
    assert!(matches!(t3.token, Token::IntLiteral(_)));
}
