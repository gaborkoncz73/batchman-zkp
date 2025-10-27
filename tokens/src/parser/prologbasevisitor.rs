
// Generated from prolog.g4 by ANTLR 4.13.2

use antlr4rust::tree::ParseTreeVisitor;
use super::prologparser::*;

// A complete Visitor for a parse tree produced by prologParser.

pub trait prologBaseVisitor<'input>:
    ParseTreeVisitor<'input, prologParserContextType> {
	// Visit a parse tree produced by prologParser#p_text.
	fn visit_p_text(&mut self, ctx: &P_textContext<'input>) {
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

	// Visit a parse tree produced by prologParser#fact.
	fn visit_fact(&mut self, ctx: &FactContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#rule_.
	fn visit_rule_(&mut self, ctx: &Rule_Context<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#head.
	fn visit_head(&mut self, ctx: &HeadContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#body.
	fn visit_body(&mut self, ctx: &BodyContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#termlist.
	fn visit_termlist(&mut self, ctx: &TermlistContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#atom_term.
	fn visit_atom_term(&mut self, ctx: &Atom_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#binary_operator.
	fn visit_binary_operator(&mut self, ctx: &Binary_operatorContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#unary_operator.
	fn visit_unary_operator(&mut self, ctx: &Unary_operatorContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#braced_term.
	fn visit_braced_term(&mut self, ctx: &Braced_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#list_term.
	fn visit_list_term(&mut self, ctx: &List_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#variable.
	fn visit_variable(&mut self, ctx: &VariableContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#float.
	fn visit_float(&mut self, ctx: &FloatContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#compound_term.
	fn visit_compound_term(&mut self, ctx: &Compound_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#integer_term.
	fn visit_integer_term(&mut self, ctx: &Integer_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#curly_bracketed_term.
	fn visit_curly_bracketed_term(&mut self, ctx: &Curly_bracketed_termContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#operator_.
	fn visit_operator_(&mut self, ctx: &Operator_Context<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#empty_list.
	fn visit_empty_list(&mut self, ctx: &Empty_listContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#empty_braces.
	fn visit_empty_braces(&mut self, ctx: &Empty_bracesContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#name.
	fn visit_name(&mut self, ctx: &NameContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#graphic.
	fn visit_graphic(&mut self, ctx: &GraphicContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#quoted_string.
	fn visit_quoted_string(&mut self, ctx: &Quoted_stringContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#dq_string.
	fn visit_dq_string(&mut self, ctx: &Dq_stringContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#backq_string.
	fn visit_backq_string(&mut self, ctx: &Backq_stringContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#semicolon.
	fn visit_semicolon(&mut self, ctx: &SemicolonContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#cut.
	fn visit_cut(&mut self, ctx: &CutContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by prologParser#integer.
	fn visit_integer(&mut self, ctx: &IntegerContext<'input>) {
            self.visit_children(ctx)
        }

}