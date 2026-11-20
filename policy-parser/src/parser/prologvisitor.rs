#![allow(nonstandard_style)]
// Generated from prolog.g4 by ANTLR 4.13.2
use super::prologparser::*;
use antlr4rust::tree::{ParseTreeVisitor, ParseTreeVisitorCompat};

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link prologParser}.
 */
pub trait prologVisitor<'input>: ParseTreeVisitor<'input, prologParserContextType> {
    /**
     * Visit a parse tree produced by {@link prologParser#p_text}.
     * @param ctx the parse tree
     */
    fn visit_p_text(&mut self, ctx: &P_textContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#source_item}.
     * @param ctx the parse tree
     */
    fn visit_source_item(&mut self, ctx: &Source_itemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#directive}.
     * @param ctx the parse tree
     */
    fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#clause}.
     * @param ctx the parse tree
     */
    fn visit_clause(&mut self, ctx: &ClauseContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#callable}.
     * @param ctx the parse tree
     */
    fn visit_callable(&mut self, ctx: &CallableContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#body}.
     * @param ctx the parse tree
     */
    fn visit_body(&mut self, ctx: &BodyContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#if_then}.
     * @param ctx the parse tree
     */
    fn visit_if_then(&mut self, ctx: &If_thenContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#conjunction}.
     * @param ctx the parse tree
     */
    fn visit_conjunction(&mut self, ctx: &ConjunctionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#term999}.
     * @param ctx the parse tree
     */
    fn visit_term999(&mut self, ctx: &Term999Context<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#relation_expr}.
     * @param ctx the parse tree
     */
    fn visit_relation_expr(&mut self, ctx: &Relation_exprContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#relation_op}.
     * @param ctx the parse tree
     */
    fn visit_relation_op(&mut self, ctx: &Relation_opContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#qualified_expr}.
     * @param ctx the parse tree
     */
    fn visit_qualified_expr(&mut self, ctx: &Qualified_exprContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#additive_expr}.
     * @param ctx the parse tree
     */
    fn visit_additive_expr(&mut self, ctx: &Additive_exprContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#multiplicative_expr}.
     * @param ctx the parse tree
     */
    fn visit_multiplicative_expr(&mut self, ctx: &Multiplicative_exprContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#unary_expr}.
     * @param ctx the parse tree
     */
    fn visit_unary_expr(&mut self, ctx: &Unary_exprContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#power_operand}.
     * @param ctx the parse tree
     */
    fn visit_power_operand(&mut self, ctx: &Power_operandContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#term_list}.
     * @param ctx the parse tree
     */
    fn visit_term_list(&mut self, ctx: &Term_listContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code compoundTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_compoundTerm(&mut self, ctx: &CompoundTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code variableTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_variableTerm(&mut self, ctx: &VariableTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code integerTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_integerTerm(&mut self, ctx: &IntegerTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code floatTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_floatTerm(&mut self, ctx: &FloatTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code atomTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_atomTerm(&mut self, ctx: &AtomTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code stringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_stringTerm(&mut self, ctx: &StringTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code backStringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_backStringTerm(&mut self, ctx: &BackStringTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code listTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_listTerm(&mut self, ctx: &ListTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code parenthesizedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_parenthesizedTerm(&mut self, ctx: &ParenthesizedTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code bracedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_bracedTerm(&mut self, ctx: &BracedTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code cutTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_cutTerm(&mut self, ctx: &CutTermContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#functor}.
     * @param ctx the parse tree
     */
    fn visit_functor(&mut self, ctx: &FunctorContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#operator_name}.
     * @param ctx the parse tree
     */
    fn visit_operator_name(&mut self, ctx: &Operator_nameContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#number}.
     * @param ctx the parse tree
     */
    fn visit_number(&mut self, ctx: &NumberContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code emptyList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_emptyList(&mut self, ctx: &EmptyListContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code properList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_properList(&mut self, ctx: &ProperListContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code improperList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_improperList(&mut self, ctx: &ImproperListContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#atom}.
     * @param ctx the parse tree
     */
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#integer}.
     * @param ctx the parse tree
     */
    fn visit_integer(&mut self, ctx: &IntegerContext<'input>) {
        self.visit_children(ctx)
    }
}

pub trait prologVisitorCompat<'input>:
    ParseTreeVisitorCompat<'input, Node = prologParserContextType>
{
    /**
     * Visit a parse tree produced by {@link prologParser#p_text}.
     * @param ctx the parse tree
     */
    fn visit_p_text(&mut self, ctx: &P_textContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#source_item}.
     * @param ctx the parse tree
     */
    fn visit_source_item(&mut self, ctx: &Source_itemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#directive}.
     * @param ctx the parse tree
     */
    fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#clause}.
     * @param ctx the parse tree
     */
    fn visit_clause(&mut self, ctx: &ClauseContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#callable}.
     * @param ctx the parse tree
     */
    fn visit_callable(&mut self, ctx: &CallableContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#body}.
     * @param ctx the parse tree
     */
    fn visit_body(&mut self, ctx: &BodyContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#if_then}.
     * @param ctx the parse tree
     */
    fn visit_if_then(&mut self, ctx: &If_thenContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#conjunction}.
     * @param ctx the parse tree
     */
    fn visit_conjunction(&mut self, ctx: &ConjunctionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#term999}.
     * @param ctx the parse tree
     */
    fn visit_term999(&mut self, ctx: &Term999Context<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#relation_expr}.
     * @param ctx the parse tree
     */
    fn visit_relation_expr(&mut self, ctx: &Relation_exprContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#relation_op}.
     * @param ctx the parse tree
     */
    fn visit_relation_op(&mut self, ctx: &Relation_opContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#qualified_expr}.
     * @param ctx the parse tree
     */
    fn visit_qualified_expr(&mut self, ctx: &Qualified_exprContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#additive_expr}.
     * @param ctx the parse tree
     */
    fn visit_additive_expr(&mut self, ctx: &Additive_exprContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#multiplicative_expr}.
     * @param ctx the parse tree
     */
    fn visit_multiplicative_expr(
        &mut self,
        ctx: &Multiplicative_exprContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#unary_expr}.
     * @param ctx the parse tree
     */
    fn visit_unary_expr(&mut self, ctx: &Unary_exprContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#power_operand}.
     * @param ctx the parse tree
     */
    fn visit_power_operand(&mut self, ctx: &Power_operandContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#term_list}.
     * @param ctx the parse tree
     */
    fn visit_term_list(&mut self, ctx: &Term_listContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code compoundTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_compoundTerm(&mut self, ctx: &CompoundTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code variableTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_variableTerm(&mut self, ctx: &VariableTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code integerTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_integerTerm(&mut self, ctx: &IntegerTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code floatTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_floatTerm(&mut self, ctx: &FloatTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code atomTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_atomTerm(&mut self, ctx: &AtomTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code stringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_stringTerm(&mut self, ctx: &StringTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code backStringTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_backStringTerm(&mut self, ctx: &BackStringTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code listTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_listTerm(&mut self, ctx: &ListTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code parenthesizedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_parenthesizedTerm(&mut self, ctx: &ParenthesizedTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code bracedTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_bracedTerm(&mut self, ctx: &BracedTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code cutTerm}
     * labeled alternative in {@link prologParser#primary}.
     * @param ctx the parse tree
     */
    fn visit_cutTerm(&mut self, ctx: &CutTermContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#functor}.
     * @param ctx the parse tree
     */
    fn visit_functor(&mut self, ctx: &FunctorContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#operator_name}.
     * @param ctx the parse tree
     */
    fn visit_operator_name(&mut self, ctx: &Operator_nameContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#number}.
     * @param ctx the parse tree
     */
    fn visit_number(&mut self, ctx: &NumberContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code emptyList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_emptyList(&mut self, ctx: &EmptyListContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code properList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_properList(&mut self, ctx: &ProperListContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by the {@code improperList}
     * labeled alternative in {@link prologParser#list}.
     * @param ctx the parse tree
     */
    fn visit_improperList(&mut self, ctx: &ImproperListContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#atom}.
     * @param ctx the parse tree
     */
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link prologParser#integer}.
     * @param ctx the parse tree
     */
    fn visit_integer(&mut self, ctx: &IntegerContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }
}

impl<'input, T> prologVisitor<'input> for T
where
    T: prologVisitorCompat<'input>,
{
    fn visit_p_text(&mut self, ctx: &P_textContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_p_text(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_source_item(&mut self, ctx: &Source_itemContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_source_item(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_directive(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_clause(&mut self, ctx: &ClauseContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_clause(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_callable(&mut self, ctx: &CallableContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_callable(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_body(&mut self, ctx: &BodyContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_body(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_if_then(&mut self, ctx: &If_thenContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_if_then(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_conjunction(&mut self, ctx: &ConjunctionContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_conjunction(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_term999(&mut self, ctx: &Term999Context<'input>) {
        let result = <Self as prologVisitorCompat>::visit_term999(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relation_expr(&mut self, ctx: &Relation_exprContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_relation_expr(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relation_op(&mut self, ctx: &Relation_opContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_relation_op(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_qualified_expr(&mut self, ctx: &Qualified_exprContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_qualified_expr(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_additive_expr(&mut self, ctx: &Additive_exprContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_additive_expr(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_multiplicative_expr(&mut self, ctx: &Multiplicative_exprContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_multiplicative_expr(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_unary_expr(&mut self, ctx: &Unary_exprContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_unary_expr(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_power_operand(&mut self, ctx: &Power_operandContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_power_operand(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_term_list(&mut self, ctx: &Term_listContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_term_list(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_compoundTerm(&mut self, ctx: &CompoundTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_compoundTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_variableTerm(&mut self, ctx: &VariableTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_variableTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_integerTerm(&mut self, ctx: &IntegerTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_integerTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_floatTerm(&mut self, ctx: &FloatTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_floatTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_atomTerm(&mut self, ctx: &AtomTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_atomTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_stringTerm(&mut self, ctx: &StringTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_stringTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_backStringTerm(&mut self, ctx: &BackStringTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_backStringTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_listTerm(&mut self, ctx: &ListTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_listTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_parenthesizedTerm(&mut self, ctx: &ParenthesizedTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_parenthesizedTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_bracedTerm(&mut self, ctx: &BracedTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_bracedTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_cutTerm(&mut self, ctx: &CutTermContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_cutTerm(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_functor(&mut self, ctx: &FunctorContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_functor(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_operator_name(&mut self, ctx: &Operator_nameContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_operator_name(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_number(&mut self, ctx: &NumberContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_number(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_emptyList(&mut self, ctx: &EmptyListContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_emptyList(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_properList(&mut self, ctx: &ProperListContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_properList(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_improperList(&mut self, ctx: &ImproperListContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_improperList(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_atom(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_integer(&mut self, ctx: &IntegerContext<'input>) {
        let result = <Self as prologVisitorCompat>::visit_integer(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }
}
