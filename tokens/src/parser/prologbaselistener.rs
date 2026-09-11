// Generated from prolog.g4 by ANTLR 4.13.2

use super::prologparser::*;
use antlr4rust::tree::ParseTreeListener;

// A complete Visitor for a parse tree produced by prologParser.

pub trait prologBaseListener<'input>: ParseTreeListener<'input, prologParserContextType> {
    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_p_text(&mut self, _ctx: &P_textContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_p_text(&mut self, _ctx: &P_textContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_source_item(&mut self, _ctx: &Source_itemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_source_item(&mut self, _ctx: &Source_itemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_directive(&mut self, _ctx: &DirectiveContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_directive(&mut self, _ctx: &DirectiveContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_clause(&mut self, _ctx: &ClauseContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_clause(&mut self, _ctx: &ClauseContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_callable(&mut self, _ctx: &CallableContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_callable(&mut self, _ctx: &CallableContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_body(&mut self, _ctx: &BodyContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_body(&mut self, _ctx: &BodyContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_disjunction(&mut self, _ctx: &DisjunctionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_disjunction(&mut self, _ctx: &DisjunctionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_if_then(&mut self, _ctx: &If_thenContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_if_then(&mut self, _ctx: &If_thenContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_conjunction(&mut self, _ctx: &ConjunctionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_conjunction(&mut self, _ctx: &ConjunctionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_term999(&mut self, _ctx: &Term999Context<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_term999(&mut self, _ctx: &Term999Context<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relation_expr(&mut self, _ctx: &Relation_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relation_expr(&mut self, _ctx: &Relation_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relation_op(&mut self, _ctx: &Relation_opContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relation_op(&mut self, _ctx: &Relation_opContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_qualified_expr(&mut self, _ctx: &Qualified_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_qualified_expr(&mut self, _ctx: &Qualified_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_additive_expr(&mut self, _ctx: &Additive_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_additive_expr(&mut self, _ctx: &Additive_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_multiplicative_expr(&mut self, _ctx: &Multiplicative_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_multiplicative_expr(&mut self, _ctx: &Multiplicative_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_power_expr(&mut self, _ctx: &Power_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_power_expr(&mut self, _ctx: &Power_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unary_expr(&mut self, _ctx: &Unary_exprContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unary_expr(&mut self, _ctx: &Unary_exprContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_argument_list(&mut self, _ctx: &Argument_listContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_argument_list(&mut self, _ctx: &Argument_listContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_compoundterm(&mut self, _ctx: &CompoundTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_compoundterm(&mut self, _ctx: &CompoundTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_variableterm(&mut self, _ctx: &VariableTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_variableterm(&mut self, _ctx: &VariableTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_integerterm(&mut self, _ctx: &IntegerTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_integerterm(&mut self, _ctx: &IntegerTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_floatterm(&mut self, _ctx: &FloatTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_floatterm(&mut self, _ctx: &FloatTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_atomterm(&mut self, _ctx: &AtomTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_atomterm(&mut self, _ctx: &AtomTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_stringterm(&mut self, _ctx: &StringTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_stringterm(&mut self, _ctx: &StringTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_backstringterm(&mut self, _ctx: &BackStringTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_backstringterm(&mut self, _ctx: &BackStringTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_listterm(&mut self, _ctx: &ListTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_listterm(&mut self, _ctx: &ListTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_parenthesizedterm(&mut self, _ctx: &ParenthesizedTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_parenthesizedterm(&mut self, _ctx: &ParenthesizedTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_bracedterm(&mut self, _ctx: &BracedTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_bracedterm(&mut self, _ctx: &BracedTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_cutterm(&mut self, _ctx: &CutTermContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_cutterm(&mut self, _ctx: &CutTermContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_emptylist(&mut self, _ctx: &EmptyListContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_emptylist(&mut self, _ctx: &EmptyListContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_properlist(&mut self, _ctx: &ProperListContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_properlist(&mut self, _ctx: &ProperListContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_improperlist(&mut self, _ctx: &ImproperListContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_improperlist(&mut self, _ctx: &ImproperListContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_list_items(&mut self, _ctx: &List_itemsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_list_items(&mut self, _ctx: &List_itemsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_atom(&mut self, _ctx: &AtomContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_integer(&mut self, _ctx: &IntegerContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  prologBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_integer(&mut self, _ctx: &IntegerContext<'input>) {}
}
