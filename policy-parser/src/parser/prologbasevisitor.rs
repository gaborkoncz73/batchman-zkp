// Generated from prolog.g4 by ANTLR 4.13.2

use super::prologparser::*;
use antlr4rust::tree::ParseTreeVisitor;

// A complete Visitor for a parse tree produced by prologParser.

pub trait prologBaseVisitor<'input>: ParseTreeVisitor<'input, prologParserContextType> {
    // Visit a parse tree produced by prologParser#p_text.
    fn visit_p_text(&mut self, ctx: &P_textContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#source_item.
    fn visit_source_item(&mut self, ctx: &Source_itemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#directive.
    fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#clause.
    fn visit_clause(&mut self, ctx: &ClauseContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#callable.
    fn visit_callable(&mut self, ctx: &CallableContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#body.
    fn visit_body(&mut self, ctx: &BodyContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#if_then.
    fn visit_if_then(&mut self, ctx: &If_thenContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#conjunction.
    fn visit_conjunction(&mut self, ctx: &ConjunctionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#term999.
    fn visit_term999(&mut self, ctx: &Term999Context<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#relation_expr.
    fn visit_relation_expr(&mut self, ctx: &Relation_exprContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#relation_op.
    fn visit_relation_op(&mut self, ctx: &Relation_opContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#qualified_expr.
    fn visit_qualified_expr(&mut self, ctx: &Qualified_exprContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#additive_expr.
    fn visit_additive_expr(&mut self, ctx: &Additive_exprContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#multiplicative_expr.
    fn visit_multiplicative_expr(&mut self, ctx: &Multiplicative_exprContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#unary_expr.
    fn visit_unary_expr(&mut self, ctx: &Unary_exprContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#power_operand.
    fn visit_power_operand(&mut self, ctx: &Power_operandContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#term_list.
    fn visit_term_list(&mut self, ctx: &Term_listContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#compoundTerm.
    fn visit_compoundterm(&mut self, ctx: &CompoundTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#variableTerm.
    fn visit_variableterm(&mut self, ctx: &VariableTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#integerTerm.
    fn visit_integerterm(&mut self, ctx: &IntegerTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#floatTerm.
    fn visit_floatterm(&mut self, ctx: &FloatTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#atomTerm.
    fn visit_atomterm(&mut self, ctx: &AtomTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#stringTerm.
    fn visit_stringterm(&mut self, ctx: &StringTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#backStringTerm.
    fn visit_backstringterm(&mut self, ctx: &BackStringTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#listTerm.
    fn visit_listterm(&mut self, ctx: &ListTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#parenthesizedTerm.
    fn visit_parenthesizedterm(&mut self, ctx: &ParenthesizedTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#bracedTerm.
    fn visit_bracedterm(&mut self, ctx: &BracedTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#cutTerm.
    fn visit_cutterm(&mut self, ctx: &CutTermContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#functor.
    fn visit_functor(&mut self, ctx: &FunctorContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#operator_name.
    fn visit_operator_name(&mut self, ctx: &Operator_nameContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#number.
    fn visit_number(&mut self, ctx: &NumberContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#emptyList.
    fn visit_emptylist(&mut self, ctx: &EmptyListContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#properList.
    fn visit_properlist(&mut self, ctx: &ProperListContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#improperList.
    fn visit_improperlist(&mut self, ctx: &ImproperListContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#atom.
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by prologParser#integer.
    fn visit_integer(&mut self, ctx: &IntegerContext<'input>) {
        self.visit_children(ctx)
    }
}
