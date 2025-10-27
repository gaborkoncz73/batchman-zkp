#![allow(nonstandard_style)]
// Generated from prolog.g4 by ANTLR 4.13.2
use antlr4rust::tree::ParseTreeListener;
use super::prologparser::*;

pub trait prologListener<'input> : ParseTreeListener<'input,prologParserContextType>{
/**
 * Enter a parse tree produced by {@link prologParser#p_text}.
 * @param ctx the parse tree
 */
fn enter_p_text(&mut self, _ctx: &P_textContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#p_text}.
 * @param ctx the parse tree
 */
fn exit_p_text(&mut self, _ctx: &P_textContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#directive}.
 * @param ctx the parse tree
 */
fn enter_directive(&mut self, _ctx: &DirectiveContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#directive}.
 * @param ctx the parse tree
 */
fn exit_directive(&mut self, _ctx: &DirectiveContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#clause}.
 * @param ctx the parse tree
 */
fn enter_clause(&mut self, _ctx: &ClauseContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#clause}.
 * @param ctx the parse tree
 */
fn exit_clause(&mut self, _ctx: &ClauseContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#fact}.
 * @param ctx the parse tree
 */
fn enter_fact(&mut self, _ctx: &FactContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#fact}.
 * @param ctx the parse tree
 */
fn exit_fact(&mut self, _ctx: &FactContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#rule_}.
 * @param ctx the parse tree
 */
fn enter_rule_(&mut self, _ctx: &Rule_Context<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#rule_}.
 * @param ctx the parse tree
 */
fn exit_rule_(&mut self, _ctx: &Rule_Context<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#head}.
 * @param ctx the parse tree
 */
fn enter_head(&mut self, _ctx: &HeadContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#head}.
 * @param ctx the parse tree
 */
fn exit_head(&mut self, _ctx: &HeadContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#body}.
 * @param ctx the parse tree
 */
fn enter_body(&mut self, _ctx: &BodyContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#body}.
 * @param ctx the parse tree
 */
fn exit_body(&mut self, _ctx: &BodyContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#termlist}.
 * @param ctx the parse tree
 */
fn enter_termlist(&mut self, _ctx: &TermlistContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#termlist}.
 * @param ctx the parse tree
 */
fn exit_termlist(&mut self, _ctx: &TermlistContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code atom_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_atom_term(&mut self, _ctx: &Atom_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code atom_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_atom_term(&mut self, _ctx: &Atom_termContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code binary_operator}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_binary_operator(&mut self, _ctx: &Binary_operatorContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code binary_operator}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_binary_operator(&mut self, _ctx: &Binary_operatorContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code unary_operator}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_unary_operator(&mut self, _ctx: &Unary_operatorContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code unary_operator}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_unary_operator(&mut self, _ctx: &Unary_operatorContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code braced_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_braced_term(&mut self, _ctx: &Braced_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code braced_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_braced_term(&mut self, _ctx: &Braced_termContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code list_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_list_term(&mut self, _ctx: &List_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code list_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_list_term(&mut self, _ctx: &List_termContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code variable}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_variable(&mut self, _ctx: &VariableContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code variable}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_variable(&mut self, _ctx: &VariableContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code float}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_float(&mut self, _ctx: &FloatContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code float}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_float(&mut self, _ctx: &FloatContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code compound_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_compound_term(&mut self, _ctx: &Compound_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code compound_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_compound_term(&mut self, _ctx: &Compound_termContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code integer_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_integer_term(&mut self, _ctx: &Integer_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code integer_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_integer_term(&mut self, _ctx: &Integer_termContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code curly_bracketed_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn enter_curly_bracketed_term(&mut self, _ctx: &Curly_bracketed_termContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code curly_bracketed_term}
 * labeled alternative in {@link prologParser#term}.
 * @param ctx the parse tree
 */
fn exit_curly_bracketed_term(&mut self, _ctx: &Curly_bracketed_termContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#operator_}.
 * @param ctx the parse tree
 */
fn enter_operator_(&mut self, _ctx: &Operator_Context<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#operator_}.
 * @param ctx the parse tree
 */
fn exit_operator_(&mut self, _ctx: &Operator_Context<'input>) { }
/**
 * Enter a parse tree produced by the {@code empty_list}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_empty_list(&mut self, _ctx: &Empty_listContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code empty_list}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_empty_list(&mut self, _ctx: &Empty_listContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code empty_braces}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_empty_braces(&mut self, _ctx: &Empty_bracesContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code empty_braces}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_empty_braces(&mut self, _ctx: &Empty_bracesContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code name}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_name(&mut self, _ctx: &NameContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code name}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_name(&mut self, _ctx: &NameContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code graphic}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_graphic(&mut self, _ctx: &GraphicContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code graphic}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_graphic(&mut self, _ctx: &GraphicContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code quoted_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_quoted_string(&mut self, _ctx: &Quoted_stringContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code quoted_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_quoted_string(&mut self, _ctx: &Quoted_stringContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code dq_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_dq_string(&mut self, _ctx: &Dq_stringContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code dq_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_dq_string(&mut self, _ctx: &Dq_stringContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code backq_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_backq_string(&mut self, _ctx: &Backq_stringContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code backq_string}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_backq_string(&mut self, _ctx: &Backq_stringContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code semicolon}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_semicolon(&mut self, _ctx: &SemicolonContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code semicolon}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_semicolon(&mut self, _ctx: &SemicolonContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code cut}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn enter_cut(&mut self, _ctx: &CutContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code cut}
 * labeled alternative in {@link prologParser#atom}.
 * @param ctx the parse tree
 */
fn exit_cut(&mut self, _ctx: &CutContext<'input>) { }
/**
 * Enter a parse tree produced by {@link prologParser#integer}.
 * @param ctx the parse tree
 */
fn enter_integer(&mut self, _ctx: &IntegerContext<'input>) { }
/**
 * Exit a parse tree produced by {@link prologParser#integer}.
 * @param ctx the parse tree
 */
fn exit_integer(&mut self, _ctx: &IntegerContext<'input>) { }

}

antlr4rust::coerce_from!{ 'input : prologListener<'input> }


