grammar Msh;

// keep track of nesting levels to determine correct newline behavior
@lexer::fields {
  nesting: usize,
  bracket_stack: Vec<usize>,
  strict_dolstr: bool
}
@lexer::init { nesting: 0, bracket_stack: vec![], strict_dolstr: false}


file: (STATIC_EXEC execLine NL)? instructions EOF;

execLine: FILE_PATH;

instructions: (tlstat ((SEMICOLON | SEMICOLON? NL) tlstat?)*)?;
tlstat: staticInst
      | argdecl
      | exportRunBlock
      | stat;

exportRunBlock: (EXPORT | RUN) block;

stat: block
    | funcdef
    | vardecl
    | expr
    | BREAK
    | CONTINUE
    | IF expr NL? THEN stat NL? (ELSE stat)?
    | LOOP stat WHILE expr
    | WHILE expr LOOP stat
    ;

staticInst: STATIC_INST ID ~NL* NL ;

expr: number                                                  # num
    | LITERAL                                                 # literal
    | bool                                                    # boolean
    | ID                                                      # identifier
    | LBRACK (listEntry (COMMA listEntry)* COMMA)? RBRACK   # listInit
    | LBRACE (dictEntry (COMMA dictEntry)* COMMA)? RBRACE   # dictInit
    | LPAREN expr RPAREN                                      # brackets
    | expr LBRACK index=expr RBRACK                           # index
    | importStmt                                              # inlineImport
    | GLOBAL ID                                               # inlineGlobal
    | expr LPAREN funcArgs RPAREN                             # functionCall
// logic operators
    | NOT expr                                                # not
    | expr AND expr                                           # and
    | expr OR expr                                            # or
    | BITNOT expr                                             # bitnot
    | expr BITAND expr                                        # bitand
    | expr XOR expr                                           # bitxor
    | expr BITOR expr                                         # bitor
// math operators
    | INC expr                                                # preInc
    | DEC expr                                                # preDec
    | expr INC                                                # postInc
    | expr DEC                                                # postDec
    | expr ATOP expr                                          # atOperator
    | expr TWOSTAR expr                                       # pow
    | expr STAR expr                                          # mul
    | expr SLASH expr                                         # div
    | expr MOD expr                                           # mod
    | expr PLUS expr                                          # plus
    | expr MINUS expr                                         # minus

    | expr DOT ID                                             # dotaccess
    | expr AS typedef                                         # typecast
    ;

vardecl: (EXPORT? (LOCAL|CONST) | EXPORT) ID (COLON typedef)? (EQ expr)?;
argdecl: ARG ID (COLON typedef)? (EQ expr)?;

funcdef: EXPORT? FUNC ID LPAREN funcFormalArgs? RPAREN (RARROW typedef) block;

funcFormalArgs: funcFormalArg (COMMA funcFormalArg)* COMMA?;

funcFormalArg: ID (COLON typedef)? (EQ expr);

block: LBRACE instructions RBRACE;

typedef: ID;

listEntry: STAR expr | expr;
dictEntry: TWOSTAR expr | ID | expr COLON expr;

funcArgs: posArgs COMMA? | (posArgs COMMA)? kwArgs COMMA?;
posArgs: expr (COMMA expr)*;
kwArgs: ID EQ expr;

// TODO more functionality for import targets
importStmt: IMPORT (ID EQ)? importSource
           | IMPORT (STAR | importSelector (COMMA importSelector)*) FROM importSource;
importSource: expr;
importSelector: STAR | (ID EQ)? ID;

number: numInt | numFloat;
numInt: DEC_INT | HEX_INT | BIN_INT;
// numFloat: DEC_FLOAT | HEX_FLOAT | BIN_FLOAT;
numFloat: DEC_FLOAT;

bool: TRUE | FALSE;



/// lexer

//keywords
TRUE: 'true';
FALSE: 'false';
LOCAL: 'local';
GLOBAL: 'global';
FUNC: 'func';
IMPORT: 'import';
FROM: 'from';
AS: 'as';
ARG: 'arg';
RUN: 'run';
EXPORT: 'export';
CONST: 'const';
IF: 'if';
THEN: 'then';
ELSE: 'else';
LOOP: 'loop';
WHILE: 'while';
BREAK: 'break';
CONTINUE: 'continue';

// identifiers (makes sense right)
ID: ID_LETTER (ID_LETTER | DEC_DIGIT) *;
fragment ID_LETTER: [a-zA-Z_$];

// strings; TODO: add better support for the lexing of dolstrings
LITERAL : '\'' (~['$] | {recog.strict_dolstr}? '$' | ESCAPE_CHARS )* '\'';
DOLSTRING : ('$\'' | {!recog.strict_dolstr}? '\'') (~['$] | ESCAPE_CHARS )* (('$' ID | '{' DOLSTR_NESTED '}') (~['$] | ESCAPE_CHARS )*)+ '\'';
fragment DOLSTR_NESTED: ~[{}]* ('{' DOLSTR_NESTED '}' ~[{}]*)*;
fragment ESCAPE_CHARS : '\\' ([$'bnrt\\] | 'x' HEX_DIGIT HEX_DIGIT | 'u' HEX_DIGIT HEX_DIGIT HEX_DIGIT HEX_DIGIT);

// file paths: either it's obvious that we have a path, or we explicitly denote it with ~
// TODO: allow variables to be entered
FILE_PATH: '~'? '.'? '.'? '/' (FILE_PATH_SEGMENT ('/' FILE_PATH_SEGMENT)* '/'?)?
         | '~' FILE_PATH_SEGMENT ('/' FILE_PATH_SEGMENT)* '/'?;
fragment FILE_PATH_SEGMENT: FILE_PATH_CHAR+
                          | '\'' (~'\'' | ESCAPE_CHARS) '\'';
fragment FILE_PATH_CHAR: [a-zA-Z0-9_\-+?"~%.];

// integers and floating point numbers
fragment NUM_SIGN : [+\-];
fragment DEC_DIGIT: [0-9];
fragment HEX_DIGIT: [0-9A-Fa-f];
fragment OCT_DIGIT: [0-7];

DEC_INT : NUM_SIGN? ('0' | [1-9] ('_'? DEC_DIGIT)*) ;
HEX_INT : NUM_SIGN? '0x' HEX_DIGIT ('_'? HEX_DIGIT)* ;
BIN_INT : NUM_SIGN? '0b' [01] ('_'? [01])* ;

fragment EXPONENT: DEC_INT;
// fragment EXPONENT: DEC_INT|HEX_INT|BIN_INT;
DEC_FLOAT : (DEC_INT '.' (DEC_DIGIT ('_'? DEC_DIGIT)*)? | NUM_SIGN? '.' DEC_DIGIT ('_'? DEC_DIGIT)*) ([eEpP] EXPONENT)? ;
// HEX_FLOAT : (HEX_INT '.' (HEX_DIGIT ('_'? HEX_DIGIT)*)? | NUM_SIGN? '0x.' HEX_DIGIT ('_'? HEX_DIGIT)*) ([pP] EXPONENT)?;
// BIN_FLOAT : (NUM_SIGN? '0b.' [01] ('_'? [01])* | BIN_INT '.' ([01] ('_'? [01])*)?) ([eEpP] EXPONENT)?;

// operators
AND : '&&';
BITAND: '&';
OR : '||';
BITOR : '|';
NOT : '!';
BITNOT: '!!';
XOR: '^';
PLUS: '+';
MINUS: '-';
STAR: '*';
TWOSTAR: '**';
SLASH: '/';
MOD: '%';
ATOP: '@';

BITANDEQ : '&=';
BITOREQ : '|=';
XOREQ: '^=';
PLUSEQ: '+=';
MINUSEQ: '-=';
MULEQ: '*=';
POWEQ: '**=';
DIVEQ: '/=';
MODEQ: '%=';
ATOPEQ: '@=';

INC: '++';
DEC: '--';

// misc characters
DOT: '.';
COMMA: ',';
COLON: ':';
SEMICOLON: ';';
EQ: '=';
GT: '>';
GEQ: '>=';
LT: '<';
LEQ: '<=';
RARROW: '->';

STATIC_INST: '#!';
STATIC_EXEC: '#!exec';


// documentation & general comments
DOCBCOMMENT: '##<' (BCOMMENT | DOCBCOMMENT | ~'>' | '>' ~'#')* '>##' -> skip;
DOCCOMMENT: '##' ~'\n'* '\n' -> skip;
BCOMMENT: '#<' (BCOMMENT | DOCBCOMMENT | ~'>' | '>' ~'#')* '>#' -> skip;
COMMENT : '#' (~'!' ~'\n'*)? '\n' -> skip;


// bracket level influences newline parsing
LPAREN : '(' {recog.nesting+=1;} ;
RPAREN : ')' {recog.nesting-=1;} ;
LBRACK : '[' {recog.nesting+=1;} ;
RBRACK : ']' {recog.nesting-=1;} ;
LBRACE : '{' {
  let nesting = recog.nesting;
  recog.bracket_stack.push(nesting);
  recog.nesting = 0;
} ;
RBRACE : '{' {
  if recog.nesting != 0 {
    // TODO: throw a normal exception instead
    panic!("nesting was bad");
  }
  recog.nesting = recog.bracket_stack.pop().expect("bracket illegaly closed");
} ;

// how to work with whitespace & newlines
WS : [ \t] -> skip;
LINE_ESCAPE: '\\' NL -> skip;
IGNORE_NEWLINE
:
 '\r'? '\n' {recog.nesting > 0}? -> skip
;
NL : '\r'?'\n';

