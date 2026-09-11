#![allow(nonstandard_style)]
// Generated from prolog.g4 by ANTLR 4.13.2
use super::prologparser::*;
use antlr4rust::tree::ParseTreeListener;

pub trait prologListener<'input>: ParseTreeListener<'input, prologParserContextType> {
    /**
     * Enter a parse tree produced by {@link prologParser#p_text}.
     * @param ctx the parse tree
     */
    fn enter_p_text(&mut self, _ctx: &P_textContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#p_text}.
     * @param ctx the parse tree
     */
    fn exit_p_text(&mut self, _ctx: &P_textContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#source_item}.
     * @param ctx the parse tree
     */
    fn enter_source_item(&mut self, _ctx: &Source_itemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#source_item}.
     * @param ctx the parse tree
     */
    fn exit_source_item(&mut self, _ctx: &Source_itemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#directive}.
     * @param ctx the parse tree
     */
    fn enter_directive(&mut self, _ctx: &DirectiveContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#directive}.
     * @param ctx the parse tree
     */
    fn exit_directive(&mut self, _ctx: &DirectiveContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#clause}.
     * @param ctx the parse tree
     */
    fn enter_clause(&mut self, _ctx: &ClauseContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#clause}.
     * @param ctx the parse tree
     */
    fn exit_clause(&mut self, _ctx: &ClauseContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#callable}.
     * @param ctx the parse tree
     */
    fn enter_callable(&mut self, _ctx: &CallableContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#callable}.
     * @param ctx the parse tree
     */
    fn exit_callable(&mut self, _ctx: &CallableContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#body}.
     * @param ctx the parse tree
     */
    fn enter_body(&mut self, _ctx: &BodyContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#body}.
     * @param ctx the parse tree
     */
    fn exit_body(&mut self, _ctx: &BodyContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#disjunction}.
     * @param ctx the parse tree
     */
    fn enter_disjunction(&mut self, _ctx: &DisjunctionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#disjunction}.
     * @param ctx the parse tree
     */
    fn exit_disjunction(&mut self, _ctx: &DisjunctionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#if_then}.
     * @param ctx the parse tree
     */
    fn enter_if_then(&mut self, _ctx: &If_thenContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#if_then}.
     * @param ctx the parse tree
     */
    fn exit_if_then(&mut self, _ctx: &If_thenContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#conjunction}.
     * @param ctx the parse tree
     */
    fn enter_conjunction(&mut self, _ctx: &ConjunctionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#conjunction}.
     * @param ctx the parse tree
     */
    fn exit_conjunction(&mut self, _ctx: &ConjunctionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#term999}.
     * @param ctx the parse tree
     */
    fn enter_term999(&mut self, _ctx: &Term999Context<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#term999}.
     * @param ctx the parse tree
     */
    fn exit_term999(&mut self, _ctx: &Term999Context<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#relation_expr}.
     * @param ctx the parse tree
     */
    fn enter_relation_expr(&mut self, _ctx: &Relation_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#relation_expr}.
     * @param ctx the parse tree
     */
    fn exit_relation_expr(&mut self, _ctx: &Relation_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#relation_op}.
     * @param ctx the parse tree
     */
    fn enter_relation_op(&mut self, _ctx: &Relation_opContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#relation_op}.
     * @param ctx the parse tree
     */
    fn exit_relation_op(&mut self, _ctx: &Relation_opContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#qualified_expr}.
     * @param ctx the parse tree
     */
    fn enter_qualified_expr(&mut self, _ctx: &Qualified_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#qualified_expr}.
     * @param ctx the parse tree
     */
    fn exit_qualified_expr(&mut self, _ctx: &Qualified_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#additive_expr}.
     * @param ctx the parse tree
     */
    fn enter_additive_expr(&mut self, _ctx: &Additive_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#additive_expr}.
     * @param ctx the parse tree
     */
    fn exit_additive_expr(&mut self, _ctx: &Additive_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#multiplicative_expr}.
     * @param ctx the parse tree
     */
    fn enter_multiplicative_expr(&mut self, _ctx: &Multiplicative_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#multiplicative_expr}.
     * @param ctx the parse tree
     */
    fn exit_multiplicative_expr(&mut self, _ctx: &Multiplicative_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#power_expr}.
     * @param ctx the parse tree
     */
    fn enter_power_expr(&mut self, _ctx: &Power_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#power_expr}.
     * @param ctx the parse tree
     */
    fn exit_power_expr(&mut self, _ctx: &Power_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#unary_expr}.
     * @param ctx the parse tree
     */
    fn enter_unary_expr(&mut self, _ctx: &Unary_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#unary_expr}.
     * @param ctx the parse tree
     */
    fn exit_unary_expr(&mut self, _ctx: &Unary_exprContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#argument_list}.
     * @param ctx the parse tree
     */
    fn enter_argument_list(&mut self, _ctx: &Argument_listContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#argument_list}.
     * @param ctx the parse tree
     */
    fn exit_argument_list(&mut self, _ctx: &Argument_listContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code compoundTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_compoundTerm(&mut self, _ctx: &CompoundTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code compoundTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_compoundTerm(&mut self, _ctx: &CompoundTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code variableTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_variableTerm(&mut self, _ctx: &VariableTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code variableTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_variableTerm(&mut self, _ctx: &VariableTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code integerTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_integerTerm(&mut self, _ctx: &IntegerTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code integerTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_integerTerm(&mut self, _ctx: &IntegerTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code floatTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_floatTerm(&mut self, _ctx: &FloatTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code floatTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_floatTerm(&mut self, _ctx: &FloatTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code atomTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_atomTerm(&mut self, _ctx: &AtomTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code atomTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_atomTerm(&mut self, _ctx: &AtomTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code stringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_stringTerm(&mut self, _ctx: &StringTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code stringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_stringTerm(&mut self, _ctx: &StringTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code backStringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_backStringTerm(&mut self, _ctx: &BackStringTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code backStringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_backStringTerm(&mut self, _ctx: &BackStringTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code listTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_listTerm(&mut self, _ctx: &ListTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code listTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_listTerm(&mut self, _ctx: &ListTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code parenthesizedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_parenthesizedTerm(&mut self, _ctx: &ParenthesizedTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code parenthesizedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_parenthesizedTerm(&mut self, _ctx: &ParenthesizedTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code bracedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_bracedTerm(&mut self, _ctx: &BracedTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code bracedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_bracedTerm(&mut self, _ctx: &BracedTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code cutTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn enter_cutTerm(&mut self, _ctx: &CutTermContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code cutTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn exit_cutTerm(&mut self, _ctx: &CutTermContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code emptyList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn enter_emptyList(&mut self, _ctx: &EmptyListContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code emptyList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn exit_emptyList(&mut self, _ctx: &EmptyListContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code properList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn enter_properList(&mut self, _ctx: &ProperListContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code properList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn exit_properList(&mut self, _ctx: &ProperListContext<'input>) {}
    /**
     * Enter a parse tree produced by the {@code improperList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn enter_improperList(&mut self, _ctx: &ImproperListContext<'input>) {}
    /**
     * Exit a parse tree produced by the {@code improperList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn exit_improperList(&mut self, _ctx: &ImproperListContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#list_items}.
     * @param ctx the parse tree
     */
    fn enter_list_items(&mut self, _ctx: &List_itemsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#list_items}.
     * @param ctx the parse tree
     */
    fn exit_list_items(&mut self, _ctx: &List_itemsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#atom}.
     * @param ctx the parse tree
     */
    fn enter_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#atom}.
     * @param ctx the parse tree
     */
    fn exit_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link prologParser#integer}.
     * @param ctx the parse tree
     */
    fn enter_integer(&mut self, _ctx: &IntegerContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link prologParser#integer}.
     * @param ctx the parse tree
     */
    fn exit_integer(&mut self, _ctx: &IntegerContext<'input>) {}
}

antlr4rust::coerce_from! { 'input : prologListener<'input> }
