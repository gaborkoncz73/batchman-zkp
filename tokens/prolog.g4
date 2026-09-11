grammar prolog;

// Batchman policy-Prolog dialect.
//
// This grammar deliberately models a closed, pure policy subset instead of
// pretending to accept every SWI-Prolog extension. Operator precedence and
// associativity match the constructs supported by the policy compiler.

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
    : atom (LPAREN argument_list RPAREN)?
    ;

// Control operators. The recursive side encodes the Prolog associativity.
body
    : disjunction
    ;

disjunction
    : if_then (SEMI disjunction)?
    ;

if_then
    : conjunction (IF_THEN if_then)?
    ;

conjunction
    : term999 (COMMA conjunction)?
    ;

term999
    : NOT term999
    | relation_expr
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
      ((PLUS | MINUS | BIT_AND | BIT_OR) multiplicative_expr)*
    ;

// yfx priority 400.
multiplicative_expr
    : power_expr
      ((STAR | SLASH | INT_DIV | DIV | MOD | REM | SHL | SHR) power_expr)*
    ;

// Priority 200 needs special care: prefix fy operators accept a priority-200
// operand, while **/^ require a lower-priority left operand. Consequently
// -A^B is -(A^B), but (-A)^B remains possible through a parenthesized primary.
power_expr
    : unary_expr
    ;

unary_expr
    : (PLUS | MINUS | BIT_NOT) power_expr
    | primary (POW_XFX primary | POW_XFY power_expr)?
    ;

// Arguments and list items are term(999): an unparenthesized comma separates
// arguments/items, while a parenthesized comma remains a ','/2 term.
argument_list
    : term999 (COMMA term999)*
    ;

primary
    : atom LPAREN argument_list RPAREN  # compoundTerm
    | VARIABLE                          # variableTerm
    | integer                           # integerTerm
    | FLOAT                             # floatTerm
    | atom                              # atomTerm
    | DOUBLE_QUOTED_STRING              # stringTerm
    | BACK_QUOTED_STRING                # backStringTerm
    | list                              # listTerm
    | LPAREN body RPAREN                # parenthesizedTerm
    | LBRACE body? RBRACE               # bracedTerm
    | CUT                               # cutTerm
    ;

list
    : LBRACK RBRACK                                      # emptyList
    | LBRACK list_items RBRACK                           # properList
    | LBRACK list_items BAR term999 RBRACK               # improperList
    ;

list_items
    : term999 (COMMA term999)*
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
LPAREN        : '(';
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

GRAPHIC_ATOM
    : [#$&*+./:<=>?@^~\\-]+
    ;

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
COMMENT       : '%' ~[\r\n]* -> channel(HIDDEN);
BLOCK_COMMENT : '/*' .*? '*/' -> channel(HIDDEN);

// Keep unknown input visible to the parser, which then reports a hard error.
ERROR_CHAR : .;
