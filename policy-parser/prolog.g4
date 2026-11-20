grammar prolog;

// Batchman policy-Prolog dialect.
//
// This grammar deliberately models a closed, pure policy subset instead of
// pretending to accept every SWI-Prolog extension. Operator precedence and
// associativity match the constructs supported by the policy compiler.
//
// Layout is significant in the same places as in SWI-Prolog:
// * A functor must be followed immediately by '(' (ISO open_ct). '(' after
//   layout is LAYOUT_LPAREN, which only opens a parenthesized term, so
//   `- (1,2)` is -(','(1,2)) while `-(1,2)` is -(1,2), and `foo (a)` is an
//   error. The AST builder also rejects comments between functor and '('.
// * '-' immediately followed by a number is a negative number literal. The
//   AST builder decides this from token positions, see `terms.rs`.
// * The end '.' must be followed by layout, '%' or end of file.

p_text
    : source_item* EOF
    ;

source_item
    : directive
    | clause
    ;

directive
    : COLON_DASH body DOT
    ;

clause
    : callable (COLON_DASH body)? DOT
    ;

callable
    : atom (LPAREN term_list RPAREN)?
    ;

// Control operators. The recursive side encodes the Prolog associativity;
// body itself is the ';' disjunction.
body
    : if_then (SEMI body)?
    ;

if_then
    : conjunction (IF_THEN if_then)?
    ;

conjunction
    : term999 (COMMA conjunction)?
    ;

// relation_expr comes first: `\+(a) = b` is functional notation, =(\+(a), b),
// and ANTLR resolves the ambiguity with `\+ ((a) = b)` to the first alternative.
term999
    : relation_expr
    | NOT term999
    ;

// xfx priority 700: chaining, such as A < B < C, is intentionally rejected.
relation_expr
    : qualified_expr (relation_op qualified_expr)?
    ;

relation_op
    : UNIFY
    | NOT_UNIFIABLE
    | IDENTICAL
    | NOT_IDENTICAL
    | TERM_LT
    | TERM_LE
    | TERM_GT
    | TERM_GE
    | UNIV
    | IS
    | ARITH_EQ
    | ARITH_NE
    | LT
    | LE
    | GT
    | GE
    ;

qualified_expr
    : additive_expr (COLON qualified_expr)?
    ;

// yfx priority 500.
additive_expr
    : multiplicative_expr
      (ops+=(PLUS | MINUS | BIT_AND | BIT_OR) multiplicative_expr)*
    ;

// yfx priority 400.
multiplicative_expr
    : unary_expr
      (ops+=(STAR | SLASH | INT_DIV | DIV | MOD | REM | SHL | SHR) unary_expr)*
    ;

// Priority 200 needs special care: prefix fy operators accept a priority-200
// operand, while **/^ require a lower-priority left operand. Consequently
// -A^B is -(A^B), but (-A)^B remains possible through a parenthesized primary.
// The primary alternative comes first so that functional notation such as
// -(1)^2 = ^(-(1), 2) wins over the prefix reading -((1)^2).
unary_expr
    : primary (POW_XFX power_operand | POW_XFY unary_expr)?
    | op=(PLUS | MINUS | BIT_NOT) unary_expr
    ;

// The xfx right operand must have priority < 200; a negative number literal
// (priority 0) qualifies, `- 1` (prefix operator, priority 200) does not.
power_operand
    : primary
    | MINUS number
    ;

// Arguments and list items are term(999): an unparenthesized comma separates
// arguments/items, while a parenthesized comma remains a ','/2 term.
term_list
    : term999 (COMMA term999)*
    ;

primary
    : functor LPAREN term_list RPAREN               # compoundTerm
    | VARIABLE                                      # variableTerm
    | integer                                       # integerTerm
    | FLOAT                                         # floatTerm
    | atom                                          # atomTerm
    | DOUBLE_QUOTED_STRING                          # stringTerm
    | BACK_QUOTED_STRING                            # backStringTerm
    | list                                          # listTerm
    | (LPAREN | LAYOUT_LPAREN) body RPAREN          # parenthesizedTerm
    | LBRACE body? RBRACE                           # bracedTerm
    | CUT                                           # cutTerm
    ;

// Operator names are ordinary functors in functional notation: -(1,2),
// =(A,B), mod(7,2), \+(G).
functor
    : atom
    | operator_name
    ;

operator_name
    : relation_op
    | NOT | IF_THEN | SEMI | COLON
    | PLUS | MINUS | BIT_AND | BIT_OR
    | STAR | SLASH | INT_DIV | DIV | MOD | REM | SHL | SHR
    | POW_XFX | POW_XFY | BIT_NOT
    ;

number
    : integer
    | FLOAT
    ;

list
    : LBRACK RBRACK                                      # emptyList
    | LBRACK term_list RBRACK                            # properList
    | LBRACK term_list BAR term999 RBRACK                # improperList
    ;

atom
    : ATOM_ID
    | QUOTED_ATOM
    | GRAPHIC_ATOM
    ;

integer
    : DECIMAL
    | BINARY
    | OCTAL
    | HEX
    | CHARACTER_CODE_CONSTANT
    ;

// Comments come first: on equal-length matches ANTLR picks the earliest
// rule, and `/**/` would otherwise lex as a graphic atom.
COMMENT       : '%' ~[\r\n]* -> channel(HIDDEN);
BLOCK_COMMENT : '/*' .*? '*/' -> channel(HIDDEN);

// Explicit operators must precede the generic graphic-atom rule.
COLON_DASH    : ':-';
IF_THEN       : '->';
NOT           : '\\+';

NOT_IDENTICAL : '\\==';
IDENTICAL     : '==';
NOT_UNIFIABLE : '\\=';
ARITH_NE      : '=\\=';
ARITH_EQ      : '=:=';
UNIV          : '=..';
TERM_LE       : '@=<';
TERM_GE       : '@>=';
TERM_LT       : '@<';
TERM_GT       : '@>';
LE            : '=<';
GE            : '>=';
LT            : '<';
GT            : '>';
UNIFY         : '=';
IS            : 'is';

INT_DIV       : '//';
DIV           : 'div';
MOD           : 'mod';
REM           : 'rem';
POW_XFX       : '**';
POW_XFY       : '^';
SHL           : '<<';
SHR           : '>>';
BIT_AND       : '/\\';
BIT_OR        : '\\/';
BIT_NOT       : '\\';
PLUS          : '+';
MINUS         : '-';
STAR          : '*';
SLASH         : '/';

SEMI          : ';';
COMMA         : ',';
COLON         : ':';
DOT           : '.';
// '.' glued to a following character is not an end token (`p(1).q(1).`).
GLUED_DOT     : '.' ~[ \t\r\n%];
LPAREN        : '(';
LAYOUT_LPAREN : [ \t\r\n]+ '(';
RPAREN        : ')';
LBRACK        : '[';
RBRACK        : ']';
LBRACE        : '{';
RBRACE        : '}';
BAR           : '|';
CUT           : '!';

FLOAT
    : DECIMAL_FRAGMENT '.' DIGIT+ ([eE] [+-]? DIGIT+)?
    ;

BINARY  : '0b' [01]+;
OCTAL   : '0o' [0-7]+;
HEX     : '0x' HEX_DIGIT+;
DECIMAL : DECIMAL_FRAGMENT;

CHARACTER_CODE_CONSTANT
    : '0' '\'' (META_ESCAPE | CONTROL_ESCAPE | OCTAL_ESCAPE | HEX_ESCAPE | .)
    ;

VARIABLE : [A-Z_] [a-zA-Z0-9_]*;
ATOM_ID  : [a-z] [a-zA-Z0-9_]*;

QUOTED_ATOM
    : '\'' (CONTINUATION_ESCAPE | SINGLE_QUOTED_CHARACTER)* '\''
    ;

DOUBLE_QUOTED_STRING
    : '"' (CONTINUATION_ESCAPE | DOUBLE_QUOTED_CHARACTER)* '"'
    ;

BACK_QUOTED_STRING
    : '`' (CONTINUATION_ESCAPE | BACK_QUOTED_CHARACTER)* '`'
    ;

// As in SWI-Prolog, '/*' at the start of a token always opens a comment, while
// inside a symbol-char sequence it does not: `a/**/= b` is a = b, `a =/**/ b`
// is an error.
GRAPHIC_ATOM
    : '/' (GRAPHIC_CHAR_NOT_STAR GRAPHIC_CHAR*)?
    | GRAPHIC_CHAR_NOT_SLASH GRAPHIC_CHAR*
    ;

fragment GRAPHIC_CHAR          : [#$&*+./:<=>?@^~\\-];
fragment GRAPHIC_CHAR_NOT_STAR : [#$&+./:<=>?@^~\\-];
fragment GRAPHIC_CHAR_NOT_SLASH : [#$&*+.:<=>?@^~\\-];

fragment DECIMAL_FRAGMENT : DIGIT+;
fragment DIGIT             : [0-9];
fragment HEX_DIGIT         : [0-9a-fA-F];

fragment SINGLE_QUOTED_CHARACTER
    : NON_QUOTE_CHARACTER
    | '\'\''
    | '"'
    | '`'
    ;

fragment DOUBLE_QUOTED_CHARACTER
    : NON_DOUBLE_QUOTE_CHARACTER
    | '""'
    ;

fragment BACK_QUOTED_CHARACTER
    : NON_BACK_QUOTE_CHARACTER
    | '``'
    ;

fragment NON_QUOTE_CHARACTER
    : ~['\\\r\n]
    | META_ESCAPE
    | CONTROL_ESCAPE
    | OCTAL_ESCAPE
    | HEX_ESCAPE
    ;

fragment NON_DOUBLE_QUOTE_CHARACTER
    : ~["\\\r\n]
    | META_ESCAPE
    | CONTROL_ESCAPE
    | OCTAL_ESCAPE
    | HEX_ESCAPE
    ;

fragment NON_BACK_QUOTE_CHARACTER
    : ~[`\\\r\n]
    | META_ESCAPE
    | CONTROL_ESCAPE
    | OCTAL_ESCAPE
    | HEX_ESCAPE
    ;

fragment META_ESCAPE         : '\\' [\\'"`];
fragment CONTROL_ESCAPE      : '\\' [abrftnv];
fragment OCTAL_ESCAPE        : '\\' [0-7]+ '\\';
fragment HEX_ESCAPE          : '\\x' HEX_DIGIT+ '\\';
fragment CONTINUATION_ESCAPE : '\\' '\r'? '\n';

WS            : [ \t\r\n]+ -> skip;
// Keep unknown input visible to the parser, which then reports a hard error.
ERROR_CHAR : .;
