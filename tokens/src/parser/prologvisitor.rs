#![allow(nonstandard_style)]
// Generated from prolog.g4 by ANTLR 4.13.2
use antlr4rust::tree::{ParseTreeVisitor,ParseTreeVisitorCompat};
use super::prologparser::*;

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link prologParser}.
 */
pub trait prologVisitor<'input>: ParseTreeVisitor<'input,prologParserContextType>{
	/**
	 * Visit a parse tree produced by {@link prologParser#p_text}.
	 * @param ctx the parse tree
	 */
	fn visit_p_text(&mut self, ctx: &P_textContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#directive}.
	 * @param ctx the parse tree
	 */
	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#clause}.
	 * @param ctx the parse tree
	 */
	fn visit_clause(&mut self, ctx: &ClauseContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#fact}.
	 * @param ctx the parse tree
	 */
	fn visit_fact(&mut self, ctx: &FactContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#rule_}.
	 * @param ctx the parse tree
	 */
	fn visit_rule_(&mut self, ctx: &Rule_Context<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#head}.
	 * @param ctx the parse tree
	 */
	fn visit_head(&mut self, ctx: &HeadContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#body}.
	 * @param ctx the parse tree
	 */
	fn visit_body(&mut self, ctx: &BodyContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#termlist}.
	 * @param ctx the parse tree
	 */
	fn visit_termlist(&mut self, ctx: &TermlistContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code atom_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_atom_term(&mut self, ctx: &Atom_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code binary_operator}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_binary_operator(&mut self, ctx: &Binary_operatorContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code unary_operator}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_unary_operator(&mut self, ctx: &Unary_operatorContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code braced_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_braced_term(&mut self, ctx: &Braced_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code list_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_list_term(&mut self, ctx: &List_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code variable}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_variable(&mut self, ctx: &VariableContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code float}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_float(&mut self, ctx: &FloatContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code compound_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_compound_term(&mut self, ctx: &Compound_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code integer_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_integer_term(&mut self, ctx: &Integer_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code curly_bracketed_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
	fn visit_curly_bracketed_term(&mut self, ctx: &Curly_bracketed_termContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#operator_}.
	 * @param ctx the parse tree
	 */
	fn visit_operator_(&mut self, ctx: &Operator_Context<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code empty_list}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_empty_list(&mut self, ctx: &Empty_listContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code empty_braces}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_empty_braces(&mut self, ctx: &Empty_bracesContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code name}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_name(&mut self, ctx: &NameContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code graphic}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_graphic(&mut self, ctx: &GraphicContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code quoted_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_quoted_string(&mut self, ctx: &Quoted_stringContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code dq_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_dq_string(&mut self, ctx: &Dq_stringContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code backq_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_backq_string(&mut self, ctx: &Backq_stringContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code semicolon}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_semicolon(&mut self, ctx: &SemicolonContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by the {@code cut}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
	fn visit_cut(&mut self, ctx: &CutContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link prologParser#integer}.
	 * @param ctx the parse tree
	 */
	fn visit_integer(&mut self, ctx: &IntegerContext<'input>) { self.visit_children(ctx) }

}

pub trait prologVisitorCompat<'input>:ParseTreeVisitorCompat<'input, Node= prologParserContextType>{
	/**
	 * Visit a parse tree produced by {@link prologParser#p_text}.
	 * @param ctx the parse tree
	 */
		fn visit_p_text(&mut self, ctx: &P_textContext<'input>) -> Self::Return {
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
	 * Visit a parse tree produced by {@link prologParser#fact}.
	 * @param ctx the parse tree
	 */
		fn visit_fact(&mut self, ctx: &FactContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link prologParser#rule_}.
	 * @param ctx the parse tree
	 */
		fn visit_rule_(&mut self, ctx: &Rule_Context<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link prologParser#head}.
	 * @param ctx the parse tree
	 */
		fn visit_head(&mut self, ctx: &HeadContext<'input>) -> Self::Return {
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
	 * Visit a parse tree produced by {@link prologParser#termlist}.
	 * @param ctx the parse tree
	 */
		fn visit_termlist(&mut self, ctx: &TermlistContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code atom_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_atom_term(&mut self, ctx: &Atom_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code binary_operator}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_binary_operator(&mut self, ctx: &Binary_operatorContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code unary_operator}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_unary_operator(&mut self, ctx: &Unary_operatorContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code braced_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_braced_term(&mut self, ctx: &Braced_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code list_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_list_term(&mut self, ctx: &List_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code variable}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_variable(&mut self, ctx: &VariableContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code float}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_float(&mut self, ctx: &FloatContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code compound_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_compound_term(&mut self, ctx: &Compound_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code integer_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_integer_term(&mut self, ctx: &Integer_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code curly_bracketed_term}
	 * labeled alternative in {@link prologParser#term}.
	 * @param ctx the parse tree
	 */
		fn visit_curly_bracketed_term(&mut self, ctx: &Curly_bracketed_termContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link prologParser#operator_}.
	 * @param ctx the parse tree
	 */
		fn visit_operator_(&mut self, ctx: &Operator_Context<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code empty_list}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_empty_list(&mut self, ctx: &Empty_listContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code empty_braces}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_empty_braces(&mut self, ctx: &Empty_bracesContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code name}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_name(&mut self, ctx: &NameContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code graphic}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_graphic(&mut self, ctx: &GraphicContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code quoted_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_quoted_string(&mut self, ctx: &Quoted_stringContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code dq_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_dq_string(&mut self, ctx: &Dq_stringContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code backq_string}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_backq_string(&mut self, ctx: &Backq_stringContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code semicolon}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_semicolon(&mut self, ctx: &SemicolonContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by the {@code cut}
	 * labeled alternative in {@link prologParser#atom}.
	 * @param ctx the parse tree
	 */
		fn visit_cut(&mut self, ctx: &CutContext<'input>) -> Self::Return {
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

impl<'input,T> prologVisitor<'input> for T
where
	T: prologVisitorCompat<'input>
{
	fn visit_p_text(&mut self, ctx: &P_textContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_p_text(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_directive(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_clause(&mut self, ctx: &ClauseContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_clause(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_fact(&mut self, ctx: &FactContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_fact(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_rule_(&mut self, ctx: &Rule_Context<'input>){
		let result = <Self as prologVisitorCompat>::visit_rule_(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_head(&mut self, ctx: &HeadContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_head(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_body(&mut self, ctx: &BodyContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_body(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_termlist(&mut self, ctx: &TermlistContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_termlist(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_atom_term(&mut self, ctx: &Atom_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_atom_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_binary_operator(&mut self, ctx: &Binary_operatorContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_binary_operator(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_unary_operator(&mut self, ctx: &Unary_operatorContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_unary_operator(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_braced_term(&mut self, ctx: &Braced_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_braced_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_list_term(&mut self, ctx: &List_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_list_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_variable(&mut self, ctx: &VariableContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_variable(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_float(&mut self, ctx: &FloatContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_float(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_compound_term(&mut self, ctx: &Compound_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_compound_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_integer_term(&mut self, ctx: &Integer_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_integer_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_curly_bracketed_term(&mut self, ctx: &Curly_bracketed_termContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_curly_bracketed_term(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_operator_(&mut self, ctx: &Operator_Context<'input>){
		let result = <Self as prologVisitorCompat>::visit_operator_(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_empty_list(&mut self, ctx: &Empty_listContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_empty_list(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_empty_braces(&mut self, ctx: &Empty_bracesContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_empty_braces(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_name(&mut self, ctx: &NameContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_name(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_graphic(&mut self, ctx: &GraphicContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_graphic(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_quoted_string(&mut self, ctx: &Quoted_stringContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_quoted_string(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_dq_string(&mut self, ctx: &Dq_stringContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_dq_string(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_backq_string(&mut self, ctx: &Backq_stringContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_backq_string(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_semicolon(&mut self, ctx: &SemicolonContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_semicolon(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_cut(&mut self, ctx: &CutContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_cut(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_integer(&mut self, ctx: &IntegerContext<'input>){
		let result = <Self as prologVisitorCompat>::visit_integer(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

}