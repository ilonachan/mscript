use parce::prelude::*;

/// One of my experiments with other parser generators for Rust.
/// In the end I got Antlr4 to work properly, and I believe
/// it will be the best option going forward.

#[lexer(MshLexer)]
enum MshLexemes {
  Bool = " 'true' | 'false' ", // match string literals, use | for multiple possible patterns

  #[frag] NumSign = "[+\\-]",
  #[frag] DecDigit = "[0-9]",
  #[frag] HexDigit = "DecDigit | [A-Fa-f]",

  DecInt = " NumSign? ('0' | [1-9] DecDigit*)",
  HexInt = " NumSign? '0x' HexDigit+",
  BinInt = " NumSign? '0b' [01]+",

  DecFloat = "DecInt '.' DecDigit* | NumSign? '.' DecDigit+ ",
  HexFloat = "HexInt '.' HexDigit* | NumSign? '0x.' HexDigit+",
  // BinFloat = "NumSign? '0b.' [01]+ | BinInt '.' [01]*",

  And = '&', // can omit double quotes if pattern is a single character
  #[skip] Whitespace = "[ \n\r\t]" // skippable lexemes
}

pub fn test_lexer() {
  let mshlex = MshLexer::default();
  println!("{:?}", mshlex.lex("123").unwrap());
}
