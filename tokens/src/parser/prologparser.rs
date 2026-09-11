// Generated from prolog.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
use super::prologlistener::*;
use super::prologvisitor::*;
use antlr4rust::PredictionContextCache;
use antlr4rust::TokenSource;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::error_strategy::{DefaultErrorStrategy, ErrorStrategy};
use antlr4rust::errors::*;
use antlr4rust::int_stream::EOF;
use antlr4rust::parser::{BaseParser, Parser, ParserNodeType, ParserRecog};
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext, cast, cast_mut};
use antlr4rust::recognizer::{Actions, Recognizer};
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::token::{OwningToken, TOKEN_EOF, Token};
use antlr4rust::token_factory::{CommonTokenFactory, TokenAware, TokenFactory};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::tree::*;
use antlr4rust::vocabulary::{Vocabulary, VocabularyImpl};

use antlr4rust::lazy_static;
use antlr4rust::{TidAble, TidExt};

use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

pub const prolog_COLON_DASH: i32 = 1;
pub const prolog_IF_THEN: i32 = 2;
pub const prolog_NOT: i32 = 3;
pub const prolog_NOT_IDENTICAL: i32 = 4;
pub const prolog_IDENTICAL: i32 = 5;
pub const prolog_NOT_UNIFIABLE: i32 = 6;
pub const prolog_ARITH_NE: i32 = 7;
pub const prolog_ARITH_EQ: i32 = 8;
pub const prolog_UNIV: i32 = 9;
pub const prolog_TERM_LE: i32 = 10;
pub const prolog_TERM_GE: i32 = 11;
pub const prolog_TERM_LT: i32 = 12;
pub const prolog_TERM_GT: i32 = 13;
pub const prolog_LE: i32 = 14;
pub const prolog_GE: i32 = 15;
pub const prolog_LT: i32 = 16;
pub const prolog_GT: i32 = 17;
pub const prolog_UNIFY: i32 = 18;
pub const prolog_IS: i32 = 19;
pub const prolog_INT_DIV: i32 = 20;
pub const prolog_DIV: i32 = 21;
pub const prolog_MOD: i32 = 22;
pub const prolog_REM: i32 = 23;
pub const prolog_POW_XFX: i32 = 24;
pub const prolog_POW_XFY: i32 = 25;
pub const prolog_SHL: i32 = 26;
pub const prolog_SHR: i32 = 27;
pub const prolog_BIT_AND: i32 = 28;
pub const prolog_BIT_OR: i32 = 29;
pub const prolog_BIT_NOT: i32 = 30;
pub const prolog_PLUS: i32 = 31;
pub const prolog_MINUS: i32 = 32;
pub const prolog_STAR: i32 = 33;
pub const prolog_SLASH: i32 = 34;
pub const prolog_SEMI: i32 = 35;
pub const prolog_COMMA: i32 = 36;
pub const prolog_COLON: i32 = 37;
pub const prolog_DOT: i32 = 38;
pub const prolog_LPAREN: i32 = 39;
pub const prolog_RPAREN: i32 = 40;
pub const prolog_LBRACK: i32 = 41;
pub const prolog_RBRACK: i32 = 42;
pub const prolog_LBRACE: i32 = 43;
pub const prolog_RBRACE: i32 = 44;
pub const prolog_BAR: i32 = 45;
pub const prolog_CUT: i32 = 46;
pub const prolog_FLOAT: i32 = 47;
pub const prolog_BINARY: i32 = 48;
pub const prolog_OCTAL: i32 = 49;
pub const prolog_HEX: i32 = 50;
pub const prolog_DECIMAL: i32 = 51;
pub const prolog_CHARACTER_CODE_CONSTANT: i32 = 52;
pub const prolog_VARIABLE: i32 = 53;
pub const prolog_ATOM_ID: i32 = 54;
pub const prolog_QUOTED_ATOM: i32 = 55;
pub const prolog_DOUBLE_QUOTED_STRING: i32 = 56;
pub const prolog_BACK_QUOTED_STRING: i32 = 57;
pub const prolog_GRAPHIC_ATOM: i32 = 58;
pub const prolog_WS: i32 = 59;
pub const prolog_COMMENT: i32 = 60;
pub const prolog_BLOCK_COMMENT: i32 = 61;
pub const prolog_ERROR_CHAR: i32 = 62;
pub const prolog_EOF: i32 = EOF;
pub const RULE_p_text: usize = 0;
pub const RULE_source_item: usize = 1;
pub const RULE_directive: usize = 2;
pub const RULE_clause: usize = 3;
pub const RULE_callable: usize = 4;
pub const RULE_body: usize = 5;
pub const RULE_disjunction: usize = 6;
pub const RULE_if_then: usize = 7;
pub const RULE_conjunction: usize = 8;
pub const RULE_term999: usize = 9;
pub const RULE_relation_expr: usize = 10;
pub const RULE_relation_op: usize = 11;
pub const RULE_qualified_expr: usize = 12;
pub const RULE_additive_expr: usize = 13;
pub const RULE_multiplicative_expr: usize = 14;
pub const RULE_power_expr: usize = 15;
pub const RULE_unary_expr: usize = 16;
pub const RULE_argument_list: usize = 17;
pub const RULE_primary: usize = 18;
pub const RULE_list: usize = 19;
pub const RULE_list_items: usize = 20;
pub const RULE_atom: usize = 21;
pub const RULE_integer: usize = 22;
pub const ruleNames: [&'static str; 23] = [
    "p_text",
    "source_item",
    "directive",
    "clause",
    "callable",
    "body",
    "disjunction",
    "if_then",
    "conjunction",
    "term999",
    "relation_expr",
    "relation_op",
    "qualified_expr",
    "additive_expr",
    "multiplicative_expr",
    "power_expr",
    "unary_expr",
    "argument_list",
    "primary",
    "list",
    "list_items",
    "atom",
    "integer",
];

pub const _LITERAL_NAMES: [Option<&'static str>; 47] = [
    None,
    Some("':-'"),
    Some("'->'"),
    Some("'\\+'"),
    Some("'\\=='"),
    Some("'=='"),
    Some("'\\='"),
    Some("'=\\='"),
    Some("'=:='"),
    Some("'=..'"),
    Some("'@=<'"),
    Some("'@>='"),
    Some("'@<'"),
    Some("'@>'"),
    Some("'=<'"),
    Some("'>='"),
    Some("'<'"),
    Some("'>'"),
    Some("'='"),
    Some("'is'"),
    Some("'//'"),
    Some("'div'"),
    Some("'mod'"),
    Some("'rem'"),
    Some("'**'"),
    Some("'^'"),
    Some("'<<'"),
    Some("'>>'"),
    Some("'/\\'"),
    Some("'\\/'"),
    Some("'\\'"),
    Some("'+'"),
    Some("'-'"),
    Some("'*'"),
    Some("'/'"),
    Some("';'"),
    Some("','"),
    Some("':'"),
    Some("'.'"),
    Some("'('"),
    Some("')'"),
    Some("'['"),
    Some("']'"),
    Some("'{'"),
    Some("'}'"),
    Some("'|'"),
    Some("'!'"),
];
pub const _SYMBOLIC_NAMES: [Option<&'static str>; 63] = [
    None,
    Some("COLON_DASH"),
    Some("IF_THEN"),
    Some("NOT"),
    Some("NOT_IDENTICAL"),
    Some("IDENTICAL"),
    Some("NOT_UNIFIABLE"),
    Some("ARITH_NE"),
    Some("ARITH_EQ"),
    Some("UNIV"),
    Some("TERM_LE"),
    Some("TERM_GE"),
    Some("TERM_LT"),
    Some("TERM_GT"),
    Some("LE"),
    Some("GE"),
    Some("LT"),
    Some("GT"),
    Some("UNIFY"),
    Some("IS"),
    Some("INT_DIV"),
    Some("DIV"),
    Some("MOD"),
    Some("REM"),
    Some("POW_XFX"),
    Some("POW_XFY"),
    Some("SHL"),
    Some("SHR"),
    Some("BIT_AND"),
    Some("BIT_OR"),
    Some("BIT_NOT"),
    Some("PLUS"),
    Some("MINUS"),
    Some("STAR"),
    Some("SLASH"),
    Some("SEMI"),
    Some("COMMA"),
    Some("COLON"),
    Some("DOT"),
    Some("LPAREN"),
    Some("RPAREN"),
    Some("LBRACK"),
    Some("RBRACK"),
    Some("LBRACE"),
    Some("RBRACE"),
    Some("BAR"),
    Some("CUT"),
    Some("FLOAT"),
    Some("BINARY"),
    Some("OCTAL"),
    Some("HEX"),
    Some("DECIMAL"),
    Some("CHARACTER_CODE_CONSTANT"),
    Some("VARIABLE"),
    Some("ATOM_ID"),
    Some("QUOTED_ATOM"),
    Some("DOUBLE_QUOTED_STRING"),
    Some("BACK_QUOTED_STRING"),
    Some("GRAPHIC_ATOM"),
    Some("WS"),
    Some("COMMENT"),
    Some("BLOCK_COMMENT"),
    Some("ERROR_CHAR"),
];
lazy_static! {
    static ref _shared_context_cache: Arc<PredictionContextCache> =
        Arc::new(PredictionContextCache::new());
    static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(
        _LITERAL_NAMES.iter(),
        _SYMBOLIC_NAMES.iter(),
        None
    ));
}

type BaseParserType<'input, I> = BaseParser<
    'input,
    prologParserExt<'input>,
    I,
    prologParserContextType,
    dyn prologListener<'input> + 'input,
>;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type prologTreeWalker<'input, 'a> =
    ParseTreeWalker<'input, 'a, prologParserContextType, dyn prologListener<'input> + 'a>;

/// Parser for prolog grammar
pub struct prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    base: BaseParserType<'input, I>,
    interpreter: Arc<ParserATNSimulator>,
    _shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn set_error_strategy(
        &mut self,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) {
        self.err_handler = strategy
    }

    pub fn with_strategy(
        input: I,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) -> Self {
        antlr4rust::recognizer::check_version("0", "5");
        let interpreter = Arc::new(ParserATNSimulator::new(
            _ATN.clone(),
            _decision_to_DFA.clone(),
            _shared_context_cache.clone(),
        ));
        Self {
            base: BaseParser::new_base_parser(
                input,
                Arc::clone(&interpreter),
                prologParserExt {
                    _pd: Default::default(),
                },
            ),
            interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }
}

type DynStrategy<'input, I> = Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>> + 'input>;

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn new(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for prologParser
pub trait prologParserContext<'input>:
    for<'x> Listenable<dyn prologListener<'input> + 'x>
    + for<'x> Visitable<dyn prologVisitor<'input> + 'x>
    + ParserRuleContext<'input, TF = LocalTokenFactory<'input>, Ctx = prologParserContextType>
{
}

antlr4rust::coerce_from! { 'input : prologParserContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn prologParserContext<'input> + 'input
where
    T: prologVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn prologVisitor<'input> + 'x))
    }
}

impl<'input> prologParserContext<'input> for TerminalNode<'input, prologParserContextType> {}
impl<'input> prologParserContext<'input> for ErrorNode<'input, prologParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn prologParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn prologListener<'input> + 'input }

pub struct prologParserContextType;
antlr4rust::tid! {prologParserContextType}

impl<'input> ParserNodeType<'input> for prologParserContextType {
    type TF = LocalTokenFactory<'input>;
    type Type = dyn prologParserContext<'input> + 'input;
}

impl<'input, I> Deref for prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    type Target = BaseParserType<'input, I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct prologParserExt<'input> {
    _pd: PhantomData<&'input str>,
}

impl<'input> prologParserExt<'input> {}
antlr4rust::tid! { prologParserExt<'a> }

impl<'input> TokenAware<'input> for prologParserExt<'input> {
    type TF = LocalTokenFactory<'input>;
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    ParserRecog<'input, BaseParserType<'input, I>> for prologParserExt<'input>
{
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    Actions<'input, BaseParserType<'input, I>> for prologParserExt<'input>
{
    fn get_grammar_file_name(&self) -> &str {
        "prolog.g4"
    }

    fn get_rule_names(&self) -> &[&str] {
        &ruleNames
    }

    fn get_vocabulary(&self) -> &dyn Vocabulary {
        &**VOCABULARY
    }
}
//------------------- p_text ----------------
pub type P_textContextAll<'input> = P_textContext<'input>;

pub type P_textContext<'input> = BaseParserRuleContext<'input, P_textContextExt<'input>>;

#[derive(Clone)]
pub struct P_textContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for P_textContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for P_textContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_p_text(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_p_text(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for P_textContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_p_text(self);
    }
}

impl<'input> CustomRuleContext<'input> for P_textContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_p_text
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_p_text }
}
antlr4rust::tid! {P_textContextExt<'a>}

impl<'input> P_textContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<P_textContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            P_textContextExt { ph: PhantomData },
        ))
    }
}

pub trait P_textContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<P_textContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token EOF
    /// Returns `None` if there is no child corresponding to token EOF
    fn EOF(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_EOF, 0)
    }
    fn source_item_all(&self) -> Vec<Rc<Source_itemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn source_item(&self, i: usize) -> Option<Rc<Source_itemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> P_textContextAttrs<'input> for P_textContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn p_text(&mut self) -> Result<Rc<P_textContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = P_textContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_p_text);
        let mut _localctx: Rc<P_textContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(49);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == prolog_COLON_DASH
                    || (((_la - 54) & !0x3f) == 0 && ((1usize << (_la - 54)) & 19) != 0)
                {
                    {
                        {
                            /*InvokeRule source_item*/
                            recog.base.set_state(46);
                            recog.source_item()?;
                        }
                    }
                    recog.base.set_state(51);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(52);
                recog.base.match_token(prolog_EOF, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- source_item ----------------
pub type Source_itemContextAll<'input> = Source_itemContext<'input>;

pub type Source_itemContext<'input> = BaseParserRuleContext<'input, Source_itemContextExt<'input>>;

#[derive(Clone)]
pub struct Source_itemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Source_itemContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Source_itemContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_source_item(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_source_item(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Source_itemContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_source_item(self);
    }
}

impl<'input> CustomRuleContext<'input> for Source_itemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_source_item
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_source_item }
}
antlr4rust::tid! {Source_itemContextExt<'a>}

impl<'input> Source_itemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Source_itemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Source_itemContextExt { ph: PhantomData },
        ))
    }
}

pub trait Source_itemContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Source_itemContextExt<'input>>
{
    fn directive(&self) -> Option<Rc<DirectiveContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn clause(&self) -> Option<Rc<ClauseContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> Source_itemContextAttrs<'input> for Source_itemContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn source_item(&mut self) -> Result<Rc<Source_itemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = Source_itemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 2, RULE_source_item);
        let mut _localctx: Rc<Source_itemContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(56);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                prolog_COLON_DASH => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule directive*/
                        recog.base.set_state(54);
                        recog.directive()?;
                    }
                }

                prolog_ATOM_ID | prolog_QUOTED_ATOM | prolog_GRAPHIC_ATOM => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule clause*/
                        recog.base.set_state(55);
                        recog.clause()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- directive ----------------
pub type DirectiveContextAll<'input> = DirectiveContext<'input>;

pub type DirectiveContext<'input> = BaseParserRuleContext<'input, DirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct DirectiveContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for DirectiveContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for DirectiveContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_directive(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_directive(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for DirectiveContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_directive(self);
    }
}

impl<'input> CustomRuleContext<'input> for DirectiveContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_directive
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_directive }
}
antlr4rust::tid! {DirectiveContextExt<'a>}

impl<'input> DirectiveContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<DirectiveContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            DirectiveContextExt { ph: PhantomData },
        ))
    }
}

pub trait DirectiveContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<DirectiveContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token COLON_DASH
    /// Returns `None` if there is no child corresponding to token COLON_DASH
    fn COLON_DASH(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COLON_DASH, 0)
    }
    fn body(&self) -> Option<Rc<BodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token DOT
    /// Returns `None` if there is no child corresponding to token DOT
    fn DOT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_DOT, 0)
    }
}

impl<'input> DirectiveContextAttrs<'input> for DirectiveContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn directive(&mut self) -> Result<Rc<DirectiveContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = DirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_directive);
        let mut _localctx: Rc<DirectiveContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(58);
                recog
                    .base
                    .match_token(prolog_COLON_DASH, &mut recog.err_handler)?;

                /*InvokeRule body*/
                recog.base.set_state(59);
                recog.body()?;

                recog.base.set_state(60);
                recog.base.match_token(prolog_DOT, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- clause ----------------
pub type ClauseContextAll<'input> = ClauseContext<'input>;

pub type ClauseContext<'input> = BaseParserRuleContext<'input, ClauseContextExt<'input>>;

#[derive(Clone)]
pub struct ClauseContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for ClauseContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ClauseContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_clause(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_clause(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ClauseContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_clause(self);
    }
}

impl<'input> CustomRuleContext<'input> for ClauseContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_clause
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_clause }
}
antlr4rust::tid! {ClauseContextExt<'a>}

impl<'input> ClauseContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ClauseContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ClauseContextExt { ph: PhantomData },
        ))
    }
}

pub trait ClauseContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<ClauseContextExt<'input>>
{
    fn callable(&self) -> Option<Rc<CallableContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token DOT
    /// Returns `None` if there is no child corresponding to token DOT
    fn DOT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_DOT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token COLON_DASH
    /// Returns `None` if there is no child corresponding to token COLON_DASH
    fn COLON_DASH(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COLON_DASH, 0)
    }
    fn body(&self) -> Option<Rc<BodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ClauseContextAttrs<'input> for ClauseContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn clause(&mut self) -> Result<Rc<ClauseContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ClauseContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_clause);
        let mut _localctx: Rc<ClauseContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule callable*/
                recog.base.set_state(62);
                recog.callable()?;

                recog.base.set_state(65);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_COLON_DASH {
                    {
                        recog.base.set_state(63);
                        recog
                            .base
                            .match_token(prolog_COLON_DASH, &mut recog.err_handler)?;

                        /*InvokeRule body*/
                        recog.base.set_state(64);
                        recog.body()?;
                    }
                }

                recog.base.set_state(67);
                recog.base.match_token(prolog_DOT, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- callable ----------------
pub type CallableContextAll<'input> = CallableContext<'input>;

pub type CallableContext<'input> = BaseParserRuleContext<'input, CallableContextExt<'input>>;

#[derive(Clone)]
pub struct CallableContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for CallableContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for CallableContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_callable(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_callable(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for CallableContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_callable(self);
    }
}

impl<'input> CustomRuleContext<'input> for CallableContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_callable
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_callable }
}
antlr4rust::tid! {CallableContextExt<'a>}

impl<'input> CallableContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CallableContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CallableContextExt { ph: PhantomData },
        ))
    }
}

pub trait CallableContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<CallableContextExt<'input>>
{
    fn atom(&self) -> Option<Rc<AtomContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LPAREN, 0)
    }
    fn argument_list(&self) -> Option<Rc<Argument_listContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RPAREN, 0)
    }
}

impl<'input> CallableContextAttrs<'input> for CallableContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn callable(&mut self) -> Result<Rc<CallableContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = CallableContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_callable);
        let mut _localctx: Rc<CallableContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule atom*/
                recog.base.set_state(69);
                recog.atom()?;

                recog.base.set_state(74);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_LPAREN {
                    {
                        recog.base.set_state(70);
                        recog
                            .base
                            .match_token(prolog_LPAREN, &mut recog.err_handler)?;

                        /*InvokeRule argument_list*/
                        recog.base.set_state(71);
                        recog.argument_list()?;

                        recog.base.set_state(72);
                        recog
                            .base
                            .match_token(prolog_RPAREN, &mut recog.err_handler)?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- body ----------------
pub type BodyContextAll<'input> = BodyContext<'input>;

pub type BodyContext<'input> = BaseParserRuleContext<'input, BodyContextExt<'input>>;

#[derive(Clone)]
pub struct BodyContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for BodyContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for BodyContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_body(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_body(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for BodyContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_body(self);
    }
}

impl<'input> CustomRuleContext<'input> for BodyContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_body
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_body }
}
antlr4rust::tid! {BodyContextExt<'a>}

impl<'input> BodyContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<BodyContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            BodyContextExt { ph: PhantomData },
        ))
    }
}

pub trait BodyContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<BodyContextExt<'input>>
{
    fn disjunction(&self) -> Option<Rc<DisjunctionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> BodyContextAttrs<'input> for BodyContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn body(&mut self) -> Result<Rc<BodyContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = BodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_body);
        let mut _localctx: Rc<BodyContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule disjunction*/
                recog.base.set_state(76);
                recog.disjunction()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- disjunction ----------------
pub type DisjunctionContextAll<'input> = DisjunctionContext<'input>;

pub type DisjunctionContext<'input> = BaseParserRuleContext<'input, DisjunctionContextExt<'input>>;

#[derive(Clone)]
pub struct DisjunctionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for DisjunctionContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for DisjunctionContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_disjunction(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_disjunction(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for DisjunctionContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_disjunction(self);
    }
}

impl<'input> CustomRuleContext<'input> for DisjunctionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_disjunction
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_disjunction }
}
antlr4rust::tid! {DisjunctionContextExt<'a>}

impl<'input> DisjunctionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<DisjunctionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            DisjunctionContextExt { ph: PhantomData },
        ))
    }
}

pub trait DisjunctionContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<DisjunctionContextExt<'input>>
{
    fn if_then(&self) -> Option<Rc<If_thenContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token SEMI
    /// Returns `None` if there is no child corresponding to token SEMI
    fn SEMI(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_SEMI, 0)
    }
    fn disjunction(&self) -> Option<Rc<DisjunctionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> DisjunctionContextAttrs<'input> for DisjunctionContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn disjunction(&mut self) -> Result<Rc<DisjunctionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = DisjunctionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 12, RULE_disjunction);
        let mut _localctx: Rc<DisjunctionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule if_then*/
                recog.base.set_state(78);
                recog.if_then()?;

                recog.base.set_state(81);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_SEMI {
                    {
                        recog.base.set_state(79);
                        recog
                            .base
                            .match_token(prolog_SEMI, &mut recog.err_handler)?;

                        /*InvokeRule disjunction*/
                        recog.base.set_state(80);
                        recog.disjunction()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- if_then ----------------
pub type If_thenContextAll<'input> = If_thenContext<'input>;

pub type If_thenContext<'input> = BaseParserRuleContext<'input, If_thenContextExt<'input>>;

#[derive(Clone)]
pub struct If_thenContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for If_thenContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for If_thenContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_if_then(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_if_then(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for If_thenContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_if_then(self);
    }
}

impl<'input> CustomRuleContext<'input> for If_thenContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_if_then
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_if_then }
}
antlr4rust::tid! {If_thenContextExt<'a>}

impl<'input> If_thenContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<If_thenContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            If_thenContextExt { ph: PhantomData },
        ))
    }
}

pub trait If_thenContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<If_thenContextExt<'input>>
{
    fn conjunction(&self) -> Option<Rc<ConjunctionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token IF_THEN
    /// Returns `None` if there is no child corresponding to token IF_THEN
    fn IF_THEN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_IF_THEN, 0)
    }
    fn if_then(&self) -> Option<Rc<If_thenContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> If_thenContextAttrs<'input> for If_thenContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn if_then(&mut self) -> Result<Rc<If_thenContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = If_thenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_if_then);
        let mut _localctx: Rc<If_thenContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule conjunction*/
                recog.base.set_state(83);
                recog.conjunction()?;

                recog.base.set_state(86);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_IF_THEN {
                    {
                        recog.base.set_state(84);
                        recog
                            .base
                            .match_token(prolog_IF_THEN, &mut recog.err_handler)?;

                        /*InvokeRule if_then*/
                        recog.base.set_state(85);
                        recog.if_then()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- conjunction ----------------
pub type ConjunctionContextAll<'input> = ConjunctionContext<'input>;

pub type ConjunctionContext<'input> = BaseParserRuleContext<'input, ConjunctionContextExt<'input>>;

#[derive(Clone)]
pub struct ConjunctionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for ConjunctionContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ConjunctionContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_conjunction(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_conjunction(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ConjunctionContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_conjunction(self);
    }
}

impl<'input> CustomRuleContext<'input> for ConjunctionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_conjunction
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_conjunction }
}
antlr4rust::tid! {ConjunctionContextExt<'a>}

impl<'input> ConjunctionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ConjunctionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ConjunctionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ConjunctionContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<ConjunctionContextExt<'input>>
{
    fn term999(&self) -> Option<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token COMMA
    /// Returns `None` if there is no child corresponding to token COMMA
    fn COMMA(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COMMA, 0)
    }
    fn conjunction(&self) -> Option<Rc<ConjunctionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ConjunctionContextAttrs<'input> for ConjunctionContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn conjunction(&mut self) -> Result<Rc<ConjunctionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ConjunctionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 16, RULE_conjunction);
        let mut _localctx: Rc<ConjunctionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule term999*/
                recog.base.set_state(88);
                recog.term999()?;

                recog.base.set_state(91);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_COMMA {
                    {
                        recog.base.set_state(89);
                        recog
                            .base
                            .match_token(prolog_COMMA, &mut recog.err_handler)?;

                        /*InvokeRule conjunction*/
                        recog.base.set_state(90);
                        recog.conjunction()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- term999 ----------------
pub type Term999ContextAll<'input> = Term999Context<'input>;

pub type Term999Context<'input> = BaseParserRuleContext<'input, Term999ContextExt<'input>>;

#[derive(Clone)]
pub struct Term999ContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Term999Context<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Term999Context<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_term999(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_term999(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Term999Context<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_term999(self);
    }
}

impl<'input> CustomRuleContext<'input> for Term999ContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_term999
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_term999 }
}
antlr4rust::tid! {Term999ContextExt<'a>}

impl<'input> Term999ContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Term999ContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Term999ContextExt { ph: PhantomData },
        ))
    }
}

pub trait Term999ContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Term999ContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token NOT
    /// Returns `None` if there is no child corresponding to token NOT
    fn NOT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_NOT, 0)
    }
    fn term999(&self) -> Option<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn relation_expr(&self) -> Option<Rc<Relation_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> Term999ContextAttrs<'input> for Term999Context<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn term999(&mut self) -> Result<Rc<Term999ContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = Term999ContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_term999);
        let mut _localctx: Rc<Term999ContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(96);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                prolog_NOT => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(93);
                        recog.base.match_token(prolog_NOT, &mut recog.err_handler)?;

                        /*InvokeRule term999*/
                        recog.base.set_state(94);
                        recog.term999()?;
                    }
                }

                prolog_BIT_NOT
                | prolog_PLUS
                | prolog_MINUS
                | prolog_LPAREN
                | prolog_LBRACK
                | prolog_LBRACE
                | prolog_CUT
                | prolog_FLOAT
                | prolog_BINARY
                | prolog_OCTAL
                | prolog_HEX
                | prolog_DECIMAL
                | prolog_CHARACTER_CODE_CONSTANT
                | prolog_VARIABLE
                | prolog_ATOM_ID
                | prolog_QUOTED_ATOM
                | prolog_DOUBLE_QUOTED_STRING
                | prolog_BACK_QUOTED_STRING
                | prolog_GRAPHIC_ATOM => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule relation_expr*/
                        recog.base.set_state(95);
                        recog.relation_expr()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relation_expr ----------------
pub type Relation_exprContextAll<'input> = Relation_exprContext<'input>;

pub type Relation_exprContext<'input> =
    BaseParserRuleContext<'input, Relation_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Relation_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Relation_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Relation_exprContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relation_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_relation_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Relation_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_relation_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Relation_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relation_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relation_expr }
}
antlr4rust::tid! {Relation_exprContextExt<'a>}

impl<'input> Relation_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Relation_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Relation_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Relation_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Relation_exprContextExt<'input>>
{
    fn qualified_expr_all(&self) -> Vec<Rc<Qualified_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn qualified_expr(&self, i: usize) -> Option<Rc<Qualified_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn relation_op(&self) -> Option<Rc<Relation_opContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> Relation_exprContextAttrs<'input> for Relation_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relation_expr(&mut self) -> Result<Rc<Relation_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            Relation_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 20, RULE_relation_expr);
        let mut _localctx: Rc<Relation_exprContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule qualified_expr*/
                recog.base.set_state(98);
                recog.qualified_expr()?;

                recog.base.set_state(102);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 1048560) != 0) {
                    {
                        /*InvokeRule relation_op*/
                        recog.base.set_state(99);
                        recog.relation_op()?;

                        /*InvokeRule qualified_expr*/
                        recog.base.set_state(100);
                        recog.qualified_expr()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relation_op ----------------
pub type Relation_opContextAll<'input> = Relation_opContext<'input>;

pub type Relation_opContext<'input> = BaseParserRuleContext<'input, Relation_opContextExt<'input>>;

#[derive(Clone)]
pub struct Relation_opContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Relation_opContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Relation_opContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relation_op(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_relation_op(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Relation_opContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_relation_op(self);
    }
}

impl<'input> CustomRuleContext<'input> for Relation_opContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relation_op
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relation_op }
}
antlr4rust::tid! {Relation_opContextExt<'a>}

impl<'input> Relation_opContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Relation_opContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Relation_opContextExt { ph: PhantomData },
        ))
    }
}

pub trait Relation_opContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Relation_opContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token UNIFY
    /// Returns `None` if there is no child corresponding to token UNIFY
    fn UNIFY(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_UNIFY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NOT_UNIFIABLE
    /// Returns `None` if there is no child corresponding to token NOT_UNIFIABLE
    fn NOT_UNIFIABLE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_NOT_UNIFIABLE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token IDENTICAL
    /// Returns `None` if there is no child corresponding to token IDENTICAL
    fn IDENTICAL(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_IDENTICAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NOT_IDENTICAL
    /// Returns `None` if there is no child corresponding to token NOT_IDENTICAL
    fn NOT_IDENTICAL(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_NOT_IDENTICAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token TERM_LT
    /// Returns `None` if there is no child corresponding to token TERM_LT
    fn TERM_LT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_TERM_LT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token TERM_LE
    /// Returns `None` if there is no child corresponding to token TERM_LE
    fn TERM_LE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_TERM_LE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token TERM_GT
    /// Returns `None` if there is no child corresponding to token TERM_GT
    fn TERM_GT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_TERM_GT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token TERM_GE
    /// Returns `None` if there is no child corresponding to token TERM_GE
    fn TERM_GE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_TERM_GE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token UNIV
    /// Returns `None` if there is no child corresponding to token UNIV
    fn UNIV(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_UNIV, 0)
    }
    /// Retrieves first TerminalNode corresponding to token IS
    /// Returns `None` if there is no child corresponding to token IS
    fn IS(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_IS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ARITH_EQ
    /// Returns `None` if there is no child corresponding to token ARITH_EQ
    fn ARITH_EQ(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_ARITH_EQ, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ARITH_NE
    /// Returns `None` if there is no child corresponding to token ARITH_NE
    fn ARITH_NE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_ARITH_NE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LT
    /// Returns `None` if there is no child corresponding to token LT
    fn LT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LE
    /// Returns `None` if there is no child corresponding to token LE
    fn LE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token GT
    /// Returns `None` if there is no child corresponding to token GT
    fn GT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_GT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token GE
    /// Returns `None` if there is no child corresponding to token GE
    fn GE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_GE, 0)
    }
}

impl<'input> Relation_opContextAttrs<'input> for Relation_opContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relation_op(&mut self) -> Result<Rc<Relation_opContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = Relation_opContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 22, RULE_relation_op);
        let mut _localctx: Rc<Relation_opContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(104);
                _la = recog.base.input.la(1);
                if { !(((_la) & !0x3f) == 0 && ((1usize << _la) & 1048560) != 0) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- qualified_expr ----------------
pub type Qualified_exprContextAll<'input> = Qualified_exprContext<'input>;

pub type Qualified_exprContext<'input> =
    BaseParserRuleContext<'input, Qualified_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Qualified_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Qualified_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Qualified_exprContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_qualified_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_qualified_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Qualified_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_qualified_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Qualified_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_qualified_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_qualified_expr }
}
antlr4rust::tid! {Qualified_exprContextExt<'a>}

impl<'input> Qualified_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Qualified_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Qualified_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Qualified_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Qualified_exprContextExt<'input>>
{
    fn additive_expr(&self) -> Option<Rc<Additive_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token COLON
    /// Returns `None` if there is no child corresponding to token COLON
    fn COLON(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COLON, 0)
    }
    fn qualified_expr(&self) -> Option<Rc<Qualified_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> Qualified_exprContextAttrs<'input> for Qualified_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn qualified_expr(&mut self) -> Result<Rc<Qualified_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            Qualified_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 24, RULE_qualified_expr);
        let mut _localctx: Rc<Qualified_exprContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule additive_expr*/
                recog.base.set_state(106);
                recog.additive_expr()?;

                recog.base.set_state(109);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == prolog_COLON {
                    {
                        recog.base.set_state(107);
                        recog
                            .base
                            .match_token(prolog_COLON, &mut recog.err_handler)?;

                        /*InvokeRule qualified_expr*/
                        recog.base.set_state(108);
                        recog.qualified_expr()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- additive_expr ----------------
pub type Additive_exprContextAll<'input> = Additive_exprContext<'input>;

pub type Additive_exprContext<'input> =
    BaseParserRuleContext<'input, Additive_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Additive_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Additive_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Additive_exprContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_additive_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_additive_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Additive_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_additive_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Additive_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_additive_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_additive_expr }
}
antlr4rust::tid! {Additive_exprContextExt<'a>}

impl<'input> Additive_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Additive_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Additive_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Additive_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Additive_exprContextExt<'input>>
{
    fn multiplicative_expr_all(&self) -> Vec<Rc<Multiplicative_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn multiplicative_expr(&self, i: usize) -> Option<Rc<Multiplicative_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token PLUS in current rule
    fn PLUS_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token PLUS, starting from 0.
    /// Returns `None` if number of children corresponding to token PLUS is less or equal than `i`.
    fn PLUS(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_PLUS, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token MINUS in current rule
    fn MINUS_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token MINUS, starting from 0.
    /// Returns `None` if number of children corresponding to token MINUS is less or equal than `i`.
    fn MINUS(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_MINUS, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token BIT_AND in current rule
    fn BIT_AND_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token BIT_AND, starting from 0.
    /// Returns `None` if number of children corresponding to token BIT_AND is less or equal than `i`.
    fn BIT_AND(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BIT_AND, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token BIT_OR in current rule
    fn BIT_OR_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token BIT_OR, starting from 0.
    /// Returns `None` if number of children corresponding to token BIT_OR is less or equal than `i`.
    fn BIT_OR(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BIT_OR, i)
    }
}

impl<'input> Additive_exprContextAttrs<'input> for Additive_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn additive_expr(&mut self) -> Result<Rc<Additive_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            Additive_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 26, RULE_additive_expr);
        let mut _localctx: Rc<Additive_exprContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule multiplicative_expr*/
                recog.base.set_state(111);
                recog.multiplicative_expr()?;

                recog.base.set_state(116);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la - 28) & !0x3f) == 0 && ((1usize << (_la - 28)) & 27) != 0) {
                    {
                        {
                            recog.base.set_state(112);
                            _la = recog.base.input.la(1);
                            if {
                                !(((_la - 28) & !0x3f) == 0 && ((1usize << (_la - 28)) & 27) != 0)
                            } {
                                recog.err_handler.recover_inline(&mut recog.base)?;
                            } else {
                                if recog.base.input.la(1) == TOKEN_EOF {
                                    recog.base.matched_eof = true
                                };
                                recog.err_handler.report_match(&mut recog.base);
                                recog.base.consume(&mut recog.err_handler);
                            }
                            /*InvokeRule multiplicative_expr*/
                            recog.base.set_state(113);
                            recog.multiplicative_expr()?;
                        }
                    }
                    recog.base.set_state(118);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- multiplicative_expr ----------------
pub type Multiplicative_exprContextAll<'input> = Multiplicative_exprContext<'input>;

pub type Multiplicative_exprContext<'input> =
    BaseParserRuleContext<'input, Multiplicative_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Multiplicative_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Multiplicative_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a>
    for Multiplicative_exprContext<'input>
{
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_multiplicative_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_multiplicative_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Multiplicative_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_multiplicative_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Multiplicative_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_multiplicative_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_multiplicative_expr }
}
antlr4rust::tid! {Multiplicative_exprContextExt<'a>}

impl<'input> Multiplicative_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Multiplicative_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Multiplicative_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Multiplicative_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Multiplicative_exprContextExt<'input>>
{
    fn power_expr_all(&self) -> Vec<Rc<Power_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn power_expr(&self, i: usize) -> Option<Rc<Power_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token STAR in current rule
    fn STAR_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token STAR, starting from 0.
    /// Returns `None` if number of children corresponding to token STAR is less or equal than `i`.
    fn STAR(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_STAR, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SLASH in current rule
    fn SLASH_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SLASH, starting from 0.
    /// Returns `None` if number of children corresponding to token SLASH is less or equal than `i`.
    fn SLASH(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_SLASH, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token INT_DIV in current rule
    fn INT_DIV_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token INT_DIV, starting from 0.
    /// Returns `None` if number of children corresponding to token INT_DIV is less or equal than `i`.
    fn INT_DIV(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_INT_DIV, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token DIV in current rule
    fn DIV_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token DIV, starting from 0.
    /// Returns `None` if number of children corresponding to token DIV is less or equal than `i`.
    fn DIV(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_DIV, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token MOD in current rule
    fn MOD_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token MOD, starting from 0.
    /// Returns `None` if number of children corresponding to token MOD is less or equal than `i`.
    fn MOD(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_MOD, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token REM in current rule
    fn REM_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token REM, starting from 0.
    /// Returns `None` if number of children corresponding to token REM is less or equal than `i`.
    fn REM(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_REM, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SHL in current rule
    fn SHL_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SHL, starting from 0.
    /// Returns `None` if number of children corresponding to token SHL is less or equal than `i`.
    fn SHL(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_SHL, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SHR in current rule
    fn SHR_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SHR, starting from 0.
    /// Returns `None` if number of children corresponding to token SHR is less or equal than `i`.
    fn SHR(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_SHR, i)
    }
}

impl<'input> Multiplicative_exprContextAttrs<'input> for Multiplicative_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn multiplicative_expr(
        &mut self,
    ) -> Result<Rc<Multiplicative_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            Multiplicative_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 28, RULE_multiplicative_expr);
        let mut _localctx: Rc<Multiplicative_exprContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule power_expr*/
                recog.base.set_state(119);
                recog.power_expr()?;

                recog.base.set_state(124);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la - 20) & !0x3f) == 0 && ((1usize << (_la - 20)) & 24783) != 0) {
                    {
                        {
                            recog.base.set_state(120);
                            _la = recog.base.input.la(1);
                            if {
                                !(((_la - 20) & !0x3f) == 0
                                    && ((1usize << (_la - 20)) & 24783) != 0)
                            } {
                                recog.err_handler.recover_inline(&mut recog.base)?;
                            } else {
                                if recog.base.input.la(1) == TOKEN_EOF {
                                    recog.base.matched_eof = true
                                };
                                recog.err_handler.report_match(&mut recog.base);
                                recog.base.consume(&mut recog.err_handler);
                            }
                            /*InvokeRule power_expr*/
                            recog.base.set_state(121);
                            recog.power_expr()?;
                        }
                    }
                    recog.base.set_state(126);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- power_expr ----------------
pub type Power_exprContextAll<'input> = Power_exprContext<'input>;

pub type Power_exprContext<'input> = BaseParserRuleContext<'input, Power_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Power_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Power_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Power_exprContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_power_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_power_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Power_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_power_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Power_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_power_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_power_expr }
}
antlr4rust::tid! {Power_exprContextExt<'a>}

impl<'input> Power_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Power_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Power_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Power_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Power_exprContextExt<'input>>
{
    fn unary_expr(&self) -> Option<Rc<Unary_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> Power_exprContextAttrs<'input> for Power_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn power_expr(&mut self) -> Result<Rc<Power_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = Power_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 30, RULE_power_expr);
        let mut _localctx: Rc<Power_exprContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule unary_expr*/
                recog.base.set_state(127);
                recog.unary_expr()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- unary_expr ----------------
pub type Unary_exprContextAll<'input> = Unary_exprContext<'input>;

pub type Unary_exprContext<'input> = BaseParserRuleContext<'input, Unary_exprContextExt<'input>>;

#[derive(Clone)]
pub struct Unary_exprContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Unary_exprContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Unary_exprContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_unary_expr(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_unary_expr(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Unary_exprContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_unary_expr(self);
    }
}

impl<'input> CustomRuleContext<'input> for Unary_exprContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_unary_expr
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_unary_expr }
}
antlr4rust::tid! {Unary_exprContextExt<'a>}

impl<'input> Unary_exprContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Unary_exprContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Unary_exprContextExt { ph: PhantomData },
        ))
    }
}

pub trait Unary_exprContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Unary_exprContextExt<'input>>
{
    fn power_expr(&self) -> Option<Rc<Power_exprContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token PLUS
    /// Returns `None` if there is no child corresponding to token PLUS
    fn PLUS(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_PLUS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MINUS
    /// Returns `None` if there is no child corresponding to token MINUS
    fn MINUS(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_MINUS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token BIT_NOT
    /// Returns `None` if there is no child corresponding to token BIT_NOT
    fn BIT_NOT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BIT_NOT, 0)
    }
    fn primary_all(&self) -> Vec<Rc<PrimaryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn primary(&self, i: usize) -> Option<Rc<PrimaryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token POW_XFX
    /// Returns `None` if there is no child corresponding to token POW_XFX
    fn POW_XFX(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_POW_XFX, 0)
    }
    /// Retrieves first TerminalNode corresponding to token POW_XFY
    /// Returns `None` if there is no child corresponding to token POW_XFY
    fn POW_XFY(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_POW_XFY, 0)
    }
}

impl<'input> Unary_exprContextAttrs<'input> for Unary_exprContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn unary_expr(&mut self) -> Result<Rc<Unary_exprContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = Unary_exprContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 32, RULE_unary_expr);
        let mut _localctx: Rc<Unary_exprContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(138);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                prolog_BIT_NOT | prolog_PLUS | prolog_MINUS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(129);
                        _la = recog.base.input.la(1);
                        if { !(((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 7) != 0) } {
                            recog.err_handler.recover_inline(&mut recog.base)?;
                        } else {
                            if recog.base.input.la(1) == TOKEN_EOF {
                                recog.base.matched_eof = true
                            };
                            recog.err_handler.report_match(&mut recog.base);
                            recog.base.consume(&mut recog.err_handler);
                        }
                        /*InvokeRule power_expr*/
                        recog.base.set_state(130);
                        recog.power_expr()?;
                    }
                }

                prolog_LPAREN
                | prolog_LBRACK
                | prolog_LBRACE
                | prolog_CUT
                | prolog_FLOAT
                | prolog_BINARY
                | prolog_OCTAL
                | prolog_HEX
                | prolog_DECIMAL
                | prolog_CHARACTER_CODE_CONSTANT
                | prolog_VARIABLE
                | prolog_ATOM_ID
                | prolog_QUOTED_ATOM
                | prolog_DOUBLE_QUOTED_STRING
                | prolog_BACK_QUOTED_STRING
                | prolog_GRAPHIC_ATOM => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule primary*/
                        recog.base.set_state(131);
                        recog.primary()?;

                        recog.base.set_state(136);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.base.input.la(1) {
                            prolog_POW_XFX => {
                                {
                                    recog.base.set_state(132);
                                    recog
                                        .base
                                        .match_token(prolog_POW_XFX, &mut recog.err_handler)?;

                                    /*InvokeRule primary*/
                                    recog.base.set_state(133);
                                    recog.primary()?;
                                }
                            }

                            prolog_POW_XFY => {
                                {
                                    recog.base.set_state(134);
                                    recog
                                        .base
                                        .match_token(prolog_POW_XFY, &mut recog.err_handler)?;

                                    /*InvokeRule power_expr*/
                                    recog.base.set_state(135);
                                    recog.power_expr()?;
                                }
                            }

                            prolog_IF_THEN | prolog_NOT_IDENTICAL | prolog_IDENTICAL
                            | prolog_NOT_UNIFIABLE | prolog_ARITH_NE | prolog_ARITH_EQ
                            | prolog_UNIV | prolog_TERM_LE | prolog_TERM_GE | prolog_TERM_LT
                            | prolog_TERM_GT | prolog_LE | prolog_GE | prolog_LT | prolog_GT
                            | prolog_UNIFY | prolog_IS | prolog_INT_DIV | prolog_DIV
                            | prolog_MOD | prolog_REM | prolog_SHL | prolog_SHR
                            | prolog_BIT_AND | prolog_BIT_OR | prolog_PLUS | prolog_MINUS
                            | prolog_STAR | prolog_SLASH | prolog_SEMI | prolog_COMMA
                            | prolog_COLON | prolog_DOT | prolog_RPAREN | prolog_RBRACK
                            | prolog_RBRACE | prolog_BAR => {}

                            _ => {}
                        }
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- argument_list ----------------
pub type Argument_listContextAll<'input> = Argument_listContext<'input>;

pub type Argument_listContext<'input> =
    BaseParserRuleContext<'input, Argument_listContextExt<'input>>;

#[derive(Clone)]
pub struct Argument_listContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for Argument_listContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for Argument_listContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_argument_list(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_argument_list(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for Argument_listContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_argument_list(self);
    }
}

impl<'input> CustomRuleContext<'input> for Argument_listContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_argument_list
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_argument_list }
}
antlr4rust::tid! {Argument_listContextExt<'a>}

impl<'input> Argument_listContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<Argument_listContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            Argument_listContextExt { ph: PhantomData },
        ))
    }
}

pub trait Argument_listContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<Argument_listContextExt<'input>>
{
    fn term999_all(&self) -> Vec<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn term999(&self, i: usize) -> Option<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COMMA, i)
    }
}

impl<'input> Argument_listContextAttrs<'input> for Argument_listContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn argument_list(&mut self) -> Result<Rc<Argument_listContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            Argument_listContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 34, RULE_argument_list);
        let mut _localctx: Rc<Argument_listContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule term999*/
                recog.base.set_state(140);
                recog.term999()?;

                recog.base.set_state(145);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == prolog_COMMA {
                    {
                        {
                            recog.base.set_state(141);
                            recog
                                .base
                                .match_token(prolog_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule term999*/
                            recog.base.set_state(142);
                            recog.term999()?;
                        }
                    }
                    recog.base.set_state(147);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- primary ----------------
#[derive(Debug)]
pub enum PrimaryContextAll<'input> {
    BackStringTermContext(BackStringTermContext<'input>),
    StringTermContext(StringTermContext<'input>),
    IntegerTermContext(IntegerTermContext<'input>),
    CutTermContext(CutTermContext<'input>),
    ParenthesizedTermContext(ParenthesizedTermContext<'input>),
    BracedTermContext(BracedTermContext<'input>),
    AtomTermContext(AtomTermContext<'input>),
    CompoundTermContext(CompoundTermContext<'input>),
    VariableTermContext(VariableTermContext<'input>),
    ListTermContext(ListTermContext<'input>),
    FloatTermContext(FloatTermContext<'input>),
    Error(PrimaryContext<'input>),
}
antlr4rust::tid! {PrimaryContextAll<'a>}

impl<'input> antlr4rust::parser_rule_context::DerefSeal for PrimaryContextAll<'input> {}

impl<'input> prologParserContext<'input> for PrimaryContextAll<'input> {}

impl<'input> Deref for PrimaryContextAll<'input> {
    type Target = dyn PrimaryContextAttrs<'input> + 'input;
    fn deref(&self) -> &Self::Target {
        use PrimaryContextAll::*;
        match self {
            BackStringTermContext(inner) => inner,
            StringTermContext(inner) => inner,
            IntegerTermContext(inner) => inner,
            CutTermContext(inner) => inner,
            ParenthesizedTermContext(inner) => inner,
            BracedTermContext(inner) => inner,
            AtomTermContext(inner) => inner,
            CompoundTermContext(inner) => inner,
            VariableTermContext(inner) => inner,
            ListTermContext(inner) => inner,
            FloatTermContext(inner) => inner,
            Error(inner) => inner,
        }
    }
}
impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for PrimaryContextAll<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        self.deref().accept(visitor)
    }
}
impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for PrimaryContextAll<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().enter(listener)
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().exit(listener)
    }
}

pub type PrimaryContext<'input> = BaseParserRuleContext<'input, PrimaryContextExt<'input>>;

#[derive(Clone)]
pub struct PrimaryContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for PrimaryContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for PrimaryContext<'input> {}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for PrimaryContext<'input> {}

impl<'input> CustomRuleContext<'input> for PrimaryContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}
antlr4rust::tid! {PrimaryContextExt<'a>}

impl<'input> PrimaryContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::Error(
            BaseParserRuleContext::new_parser_ctx(
                parent,
                invoking_state,
                PrimaryContextExt { ph: PhantomData },
            ),
        ))
    }
}

pub trait PrimaryContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<PrimaryContextExt<'input>>
{
}

impl<'input> PrimaryContextAttrs<'input> for PrimaryContext<'input> {}

pub type BackStringTermContext<'input> =
    BaseParserRuleContext<'input, BackStringTermContextExt<'input>>;

pub trait BackStringTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token BACK_QUOTED_STRING
    /// Returns `None` if there is no child corresponding to token BACK_QUOTED_STRING
    fn BACK_QUOTED_STRING(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BACK_QUOTED_STRING, 0)
    }
}

impl<'input> BackStringTermContextAttrs<'input> for BackStringTermContext<'input> {}

pub struct BackStringTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {BackStringTermContextExt<'a>}

impl<'input> prologParserContext<'input> for BackStringTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for BackStringTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_backStringTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_backStringTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for BackStringTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_backStringTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for BackStringTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for BackStringTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for BackStringTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for BackStringTermContext<'input> {}

impl<'input> BackStringTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::BackStringTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                BackStringTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type StringTermContext<'input> = BaseParserRuleContext<'input, StringTermContextExt<'input>>;

pub trait StringTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token DOUBLE_QUOTED_STRING
    /// Returns `None` if there is no child corresponding to token DOUBLE_QUOTED_STRING
    fn DOUBLE_QUOTED_STRING(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_DOUBLE_QUOTED_STRING, 0)
    }
}

impl<'input> StringTermContextAttrs<'input> for StringTermContext<'input> {}

pub struct StringTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {StringTermContextExt<'a>}

impl<'input> prologParserContext<'input> for StringTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for StringTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_stringTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_stringTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for StringTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_stringTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for StringTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for StringTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for StringTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for StringTermContext<'input> {}

impl<'input> StringTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::StringTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                StringTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type IntegerTermContext<'input> = BaseParserRuleContext<'input, IntegerTermContextExt<'input>>;

pub trait IntegerTermContextAttrs<'input>: prologParserContext<'input> {
    fn integer(&self) -> Option<Rc<IntegerContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> IntegerTermContextAttrs<'input> for IntegerTermContext<'input> {}

pub struct IntegerTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {IntegerTermContextExt<'a>}

impl<'input> prologParserContext<'input> for IntegerTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for IntegerTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_integerTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_integerTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for IntegerTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_integerTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for IntegerTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for IntegerTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for IntegerTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for IntegerTermContext<'input> {}

impl<'input> IntegerTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::IntegerTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                IntegerTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type CutTermContext<'input> = BaseParserRuleContext<'input, CutTermContextExt<'input>>;

pub trait CutTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token CUT
    /// Returns `None` if there is no child corresponding to token CUT
    fn CUT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_CUT, 0)
    }
}

impl<'input> CutTermContextAttrs<'input> for CutTermContext<'input> {}

pub struct CutTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {CutTermContextExt<'a>}

impl<'input> prologParserContext<'input> for CutTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for CutTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_cutTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_cutTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for CutTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_cutTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for CutTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for CutTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for CutTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for CutTermContext<'input> {}

impl<'input> CutTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::CutTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                CutTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type ParenthesizedTermContext<'input> =
    BaseParserRuleContext<'input, ParenthesizedTermContextExt<'input>>;

pub trait ParenthesizedTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LPAREN, 0)
    }
    fn body(&self) -> Option<Rc<BodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RPAREN, 0)
    }
}

impl<'input> ParenthesizedTermContextAttrs<'input> for ParenthesizedTermContext<'input> {}

pub struct ParenthesizedTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {ParenthesizedTermContextExt<'a>}

impl<'input> prologParserContext<'input> for ParenthesizedTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ParenthesizedTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_parenthesizedTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_parenthesizedTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ParenthesizedTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_parenthesizedTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for ParenthesizedTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for ParenthesizedTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for ParenthesizedTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for ParenthesizedTermContext<'input> {}

impl<'input> ParenthesizedTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::ParenthesizedTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                ParenthesizedTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type BracedTermContext<'input> = BaseParserRuleContext<'input, BracedTermContextExt<'input>>;

pub trait BracedTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token LBRACE
    /// Returns `None` if there is no child corresponding to token LBRACE
    fn LBRACE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LBRACE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACE
    /// Returns `None` if there is no child corresponding to token RBRACE
    fn RBRACE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RBRACE, 0)
    }
    fn body(&self) -> Option<Rc<BodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> BracedTermContextAttrs<'input> for BracedTermContext<'input> {}

pub struct BracedTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {BracedTermContextExt<'a>}

impl<'input> prologParserContext<'input> for BracedTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for BracedTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_bracedTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_bracedTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for BracedTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_bracedTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for BracedTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for BracedTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for BracedTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for BracedTermContext<'input> {}

impl<'input> BracedTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::BracedTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                BracedTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type AtomTermContext<'input> = BaseParserRuleContext<'input, AtomTermContextExt<'input>>;

pub trait AtomTermContextAttrs<'input>: prologParserContext<'input> {
    fn atom(&self) -> Option<Rc<AtomContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> AtomTermContextAttrs<'input> for AtomTermContext<'input> {}

pub struct AtomTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {AtomTermContextExt<'a>}

impl<'input> prologParserContext<'input> for AtomTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for AtomTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_atomTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_atomTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for AtomTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_atomTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for AtomTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for AtomTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for AtomTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for AtomTermContext<'input> {}

impl<'input> AtomTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::AtomTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                AtomTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type CompoundTermContext<'input> =
    BaseParserRuleContext<'input, CompoundTermContextExt<'input>>;

pub trait CompoundTermContextAttrs<'input>: prologParserContext<'input> {
    fn atom(&self) -> Option<Rc<AtomContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LPAREN, 0)
    }
    fn argument_list(&self) -> Option<Rc<Argument_listContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RPAREN, 0)
    }
}

impl<'input> CompoundTermContextAttrs<'input> for CompoundTermContext<'input> {}

pub struct CompoundTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {CompoundTermContextExt<'a>}

impl<'input> prologParserContext<'input> for CompoundTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for CompoundTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_compoundTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_compoundTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for CompoundTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_compoundTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for CompoundTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for CompoundTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for CompoundTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for CompoundTermContext<'input> {}

impl<'input> CompoundTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::CompoundTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                CompoundTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type VariableTermContext<'input> =
    BaseParserRuleContext<'input, VariableTermContextExt<'input>>;

pub trait VariableTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token VARIABLE
    /// Returns `None` if there is no child corresponding to token VARIABLE
    fn VARIABLE(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_VARIABLE, 0)
    }
}

impl<'input> VariableTermContextAttrs<'input> for VariableTermContext<'input> {}

pub struct VariableTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {VariableTermContextExt<'a>}

impl<'input> prologParserContext<'input> for VariableTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for VariableTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_variableTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_variableTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for VariableTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_variableTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for VariableTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for VariableTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for VariableTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for VariableTermContext<'input> {}

impl<'input> VariableTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::VariableTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                VariableTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type ListTermContext<'input> = BaseParserRuleContext<'input, ListTermContextExt<'input>>;

pub trait ListTermContextAttrs<'input>: prologParserContext<'input> {
    fn list(&self) -> Option<Rc<ListContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ListTermContextAttrs<'input> for ListTermContext<'input> {}

pub struct ListTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {ListTermContextExt<'a>}

impl<'input> prologParserContext<'input> for ListTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ListTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_listTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_listTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ListTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_listTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for ListTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for ListTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for ListTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for ListTermContext<'input> {}

impl<'input> ListTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::ListTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                ListTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type FloatTermContext<'input> = BaseParserRuleContext<'input, FloatTermContextExt<'input>>;

pub trait FloatTermContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token FLOAT
    /// Returns `None` if there is no child corresponding to token FLOAT
    fn FLOAT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_FLOAT, 0)
    }
}

impl<'input> FloatTermContextAttrs<'input> for FloatTermContext<'input> {}

pub struct FloatTermContextExt<'input> {
    base: PrimaryContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {FloatTermContextExt<'a>}

impl<'input> prologParserContext<'input> for FloatTermContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for FloatTermContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_floatTerm(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_floatTerm(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for FloatTermContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_floatTerm(self);
    }
}

impl<'input> CustomRuleContext<'input> for FloatTermContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_primary
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_primary }
}

impl<'input> Borrow<PrimaryContextExt<'input>> for FloatTermContext<'input> {
    fn borrow(&self) -> &PrimaryContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<PrimaryContextExt<'input>> for FloatTermContext<'input> {
    fn borrow_mut(&mut self) -> &mut PrimaryContextExt<'input> {
        &mut self.base
    }
}

impl<'input> PrimaryContextAttrs<'input> for FloatTermContext<'input> {}

impl<'input> FloatTermContextExt<'input> {
    fn new(ctx: &dyn PrimaryContextAttrs<'input>) -> Rc<PrimaryContextAll<'input>> {
        Rc::new(PrimaryContextAll::FloatTermContext(
            BaseParserRuleContext::copy_from(
                ctx,
                FloatTermContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn primary(&mut self) -> Result<Rc<PrimaryContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PrimaryContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 36, RULE_primary);
        let mut _localctx: Rc<PrimaryContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(170);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(16, &mut recog.base)? {
                1 => {
                    let tmp = CompoundTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 1)?;
                    _localctx = tmp;
                    {
                        /*InvokeRule atom*/
                        recog.base.set_state(148);
                        recog.atom()?;

                        recog.base.set_state(149);
                        recog
                            .base
                            .match_token(prolog_LPAREN, &mut recog.err_handler)?;

                        /*InvokeRule argument_list*/
                        recog.base.set_state(150);
                        recog.argument_list()?;

                        recog.base.set_state(151);
                        recog
                            .base
                            .match_token(prolog_RPAREN, &mut recog.err_handler)?;
                    }
                }
                2 => {
                    let tmp = VariableTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 2)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(153);
                        recog
                            .base
                            .match_token(prolog_VARIABLE, &mut recog.err_handler)?;
                    }
                }
                3 => {
                    let tmp = IntegerTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 3)?;
                    _localctx = tmp;
                    {
                        /*InvokeRule integer*/
                        recog.base.set_state(154);
                        recog.integer()?;
                    }
                }
                4 => {
                    let tmp = FloatTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 4)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(155);
                        recog
                            .base
                            .match_token(prolog_FLOAT, &mut recog.err_handler)?;
                    }
                }
                5 => {
                    let tmp = AtomTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 5)?;
                    _localctx = tmp;
                    {
                        /*InvokeRule atom*/
                        recog.base.set_state(156);
                        recog.atom()?;
                    }
                }
                6 => {
                    let tmp = StringTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 6)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(157);
                        recog
                            .base
                            .match_token(prolog_DOUBLE_QUOTED_STRING, &mut recog.err_handler)?;
                    }
                }
                7 => {
                    let tmp = BackStringTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 7)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(158);
                        recog
                            .base
                            .match_token(prolog_BACK_QUOTED_STRING, &mut recog.err_handler)?;
                    }
                }
                8 => {
                    let tmp = ListTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 8)?;
                    _localctx = tmp;
                    {
                        /*InvokeRule list*/
                        recog.base.set_state(159);
                        recog.list()?;
                    }
                }
                9 => {
                    let tmp = ParenthesizedTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 9)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(160);
                        recog
                            .base
                            .match_token(prolog_LPAREN, &mut recog.err_handler)?;

                        /*InvokeRule body*/
                        recog.base.set_state(161);
                        recog.body()?;

                        recog.base.set_state(162);
                        recog
                            .base
                            .match_token(prolog_RPAREN, &mut recog.err_handler)?;
                    }
                }
                10 => {
                    let tmp = BracedTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 10)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(164);
                        recog
                            .base
                            .match_token(prolog_LBRACE, &mut recog.err_handler)?;

                        recog.base.set_state(166);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3221225480) != 0)
                            || (((_la - 32) & !0x3f) == 0
                                && ((1usize << (_la - 32)) & 134204033) != 0)
                        {
                            {
                                /*InvokeRule body*/
                                recog.base.set_state(165);
                                recog.body()?;
                            }
                        }

                        recog.base.set_state(168);
                        recog
                            .base
                            .match_token(prolog_RBRACE, &mut recog.err_handler)?;
                    }
                }
                11 => {
                    let tmp = CutTermContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 11)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(169);
                        recog.base.match_token(prolog_CUT, &mut recog.err_handler)?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- list ----------------
#[derive(Debug)]
pub enum ListContextAll<'input> {
    EmptyListContext(EmptyListContext<'input>),
    ProperListContext(ProperListContext<'input>),
    ImproperListContext(ImproperListContext<'input>),
    Error(ListContext<'input>),
}
antlr4rust::tid! {ListContextAll<'a>}

impl<'input> antlr4rust::parser_rule_context::DerefSeal for ListContextAll<'input> {}

impl<'input> prologParserContext<'input> for ListContextAll<'input> {}

impl<'input> Deref for ListContextAll<'input> {
    type Target = dyn ListContextAttrs<'input> + 'input;
    fn deref(&self) -> &Self::Target {
        use ListContextAll::*;
        match self {
            EmptyListContext(inner) => inner,
            ProperListContext(inner) => inner,
            ImproperListContext(inner) => inner,
            Error(inner) => inner,
        }
    }
}
impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ListContextAll<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        self.deref().accept(visitor)
    }
}
impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ListContextAll<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().enter(listener)
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().exit(listener)
    }
}

pub type ListContext<'input> = BaseParserRuleContext<'input, ListContextExt<'input>>;

#[derive(Clone)]
pub struct ListContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for ListContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ListContext<'input> {}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ListContext<'input> {}

impl<'input> CustomRuleContext<'input> for ListContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_list
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_list }
}
antlr4rust::tid! {ListContextExt<'a>}

impl<'input> ListContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ListContextAll<'input>> {
        Rc::new(ListContextAll::Error(
            BaseParserRuleContext::new_parser_ctx(
                parent,
                invoking_state,
                ListContextExt { ph: PhantomData },
            ),
        ))
    }
}

pub trait ListContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<ListContextExt<'input>>
{
}

impl<'input> ListContextAttrs<'input> for ListContext<'input> {}

pub type EmptyListContext<'input> = BaseParserRuleContext<'input, EmptyListContextExt<'input>>;

pub trait EmptyListContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RBRACK, 0)
    }
}

impl<'input> EmptyListContextAttrs<'input> for EmptyListContext<'input> {}

pub struct EmptyListContextExt<'input> {
    base: ListContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {EmptyListContextExt<'a>}

impl<'input> prologParserContext<'input> for EmptyListContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for EmptyListContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_emptyList(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_emptyList(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for EmptyListContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_emptyList(self);
    }
}

impl<'input> CustomRuleContext<'input> for EmptyListContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_list
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_list }
}

impl<'input> Borrow<ListContextExt<'input>> for EmptyListContext<'input> {
    fn borrow(&self) -> &ListContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<ListContextExt<'input>> for EmptyListContext<'input> {
    fn borrow_mut(&mut self) -> &mut ListContextExt<'input> {
        &mut self.base
    }
}

impl<'input> ListContextAttrs<'input> for EmptyListContext<'input> {}

impl<'input> EmptyListContextExt<'input> {
    fn new(ctx: &dyn ListContextAttrs<'input>) -> Rc<ListContextAll<'input>> {
        Rc::new(ListContextAll::EmptyListContext(
            BaseParserRuleContext::copy_from(
                ctx,
                EmptyListContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type ProperListContext<'input> = BaseParserRuleContext<'input, ProperListContextExt<'input>>;

pub trait ProperListContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LBRACK, 0)
    }
    fn list_items(&self) -> Option<Rc<List_itemsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RBRACK, 0)
    }
}

impl<'input> ProperListContextAttrs<'input> for ProperListContext<'input> {}

pub struct ProperListContextExt<'input> {
    base: ListContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {ProperListContextExt<'a>}

impl<'input> prologParserContext<'input> for ProperListContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ProperListContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_properList(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_properList(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ProperListContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_properList(self);
    }
}

impl<'input> CustomRuleContext<'input> for ProperListContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_list
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_list }
}

impl<'input> Borrow<ListContextExt<'input>> for ProperListContext<'input> {
    fn borrow(&self) -> &ListContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<ListContextExt<'input>> for ProperListContext<'input> {
    fn borrow_mut(&mut self) -> &mut ListContextExt<'input> {
        &mut self.base
    }
}

impl<'input> ListContextAttrs<'input> for ProperListContext<'input> {}

impl<'input> ProperListContextExt<'input> {
    fn new(ctx: &dyn ListContextAttrs<'input>) -> Rc<ListContextAll<'input>> {
        Rc::new(ListContextAll::ProperListContext(
            BaseParserRuleContext::copy_from(
                ctx,
                ProperListContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type ImproperListContext<'input> =
    BaseParserRuleContext<'input, ImproperListContextExt<'input>>;

pub trait ImproperListContextAttrs<'input>: prologParserContext<'input> {
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_LBRACK, 0)
    }
    fn list_items(&self) -> Option<Rc<List_itemsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token BAR
    /// Returns `None` if there is no child corresponding to token BAR
    fn BAR(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BAR, 0)
    }
    fn term999(&self) -> Option<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_RBRACK, 0)
    }
}

impl<'input> ImproperListContextAttrs<'input> for ImproperListContext<'input> {}

pub struct ImproperListContextExt<'input> {
    base: ListContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {ImproperListContextExt<'a>}

impl<'input> prologParserContext<'input> for ImproperListContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for ImproperListContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_improperList(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_improperList(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for ImproperListContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_improperList(self);
    }
}

impl<'input> CustomRuleContext<'input> for ImproperListContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_list
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_list }
}

impl<'input> Borrow<ListContextExt<'input>> for ImproperListContext<'input> {
    fn borrow(&self) -> &ListContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<ListContextExt<'input>> for ImproperListContext<'input> {
    fn borrow_mut(&mut self) -> &mut ListContextExt<'input> {
        &mut self.base
    }
}

impl<'input> ListContextAttrs<'input> for ImproperListContext<'input> {}

impl<'input> ImproperListContextExt<'input> {
    fn new(ctx: &dyn ListContextAttrs<'input>) -> Rc<ListContextAll<'input>> {
        Rc::new(ListContextAll::ImproperListContext(
            BaseParserRuleContext::copy_from(
                ctx,
                ImproperListContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn list(&mut self) -> Result<Rc<ListContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ListContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 38, RULE_list);
        let mut _localctx: Rc<ListContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(184);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(17, &mut recog.base)? {
                1 => {
                    let tmp = EmptyListContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 1)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(172);
                        recog
                            .base
                            .match_token(prolog_LBRACK, &mut recog.err_handler)?;

                        recog.base.set_state(173);
                        recog
                            .base
                            .match_token(prolog_RBRACK, &mut recog.err_handler)?;
                    }
                }
                2 => {
                    let tmp = ProperListContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 2)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(174);
                        recog
                            .base
                            .match_token(prolog_LBRACK, &mut recog.err_handler)?;

                        /*InvokeRule list_items*/
                        recog.base.set_state(175);
                        recog.list_items()?;

                        recog.base.set_state(176);
                        recog
                            .base
                            .match_token(prolog_RBRACK, &mut recog.err_handler)?;
                    }
                }
                3 => {
                    let tmp = ImproperListContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 3)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(178);
                        recog
                            .base
                            .match_token(prolog_LBRACK, &mut recog.err_handler)?;

                        /*InvokeRule list_items*/
                        recog.base.set_state(179);
                        recog.list_items()?;

                        recog.base.set_state(180);
                        recog.base.match_token(prolog_BAR, &mut recog.err_handler)?;

                        /*InvokeRule term999*/
                        recog.base.set_state(181);
                        recog.term999()?;

                        recog.base.set_state(182);
                        recog
                            .base
                            .match_token(prolog_RBRACK, &mut recog.err_handler)?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- list_items ----------------
pub type List_itemsContextAll<'input> = List_itemsContext<'input>;

pub type List_itemsContext<'input> = BaseParserRuleContext<'input, List_itemsContextExt<'input>>;

#[derive(Clone)]
pub struct List_itemsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for List_itemsContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for List_itemsContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_list_items(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_list_items(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for List_itemsContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_list_items(self);
    }
}

impl<'input> CustomRuleContext<'input> for List_itemsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_list_items
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_list_items }
}
antlr4rust::tid! {List_itemsContextExt<'a>}

impl<'input> List_itemsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<List_itemsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            List_itemsContextExt { ph: PhantomData },
        ))
    }
}

pub trait List_itemsContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<List_itemsContextExt<'input>>
{
    fn term999_all(&self) -> Vec<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn term999(&self, i: usize) -> Option<Rc<Term999ContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_COMMA, i)
    }
}

impl<'input> List_itemsContextAttrs<'input> for List_itemsContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn list_items(&mut self) -> Result<Rc<List_itemsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = List_itemsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 40, RULE_list_items);
        let mut _localctx: Rc<List_itemsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule term999*/
                recog.base.set_state(186);
                recog.term999()?;

                recog.base.set_state(191);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == prolog_COMMA {
                    {
                        {
                            recog.base.set_state(187);
                            recog
                                .base
                                .match_token(prolog_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule term999*/
                            recog.base.set_state(188);
                            recog.term999()?;
                        }
                    }
                    recog.base.set_state(193);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- atom ----------------
pub type AtomContextAll<'input> = AtomContext<'input>;

pub type AtomContext<'input> = BaseParserRuleContext<'input, AtomContextExt<'input>>;

#[derive(Clone)]
pub struct AtomContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for AtomContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for AtomContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_atom(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_atom(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for AtomContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_atom(self);
    }
}

impl<'input> CustomRuleContext<'input> for AtomContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_atom
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}
antlr4rust::tid! {AtomContextExt<'a>}

impl<'input> AtomContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<AtomContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            AtomContextExt { ph: PhantomData },
        ))
    }
}

pub trait AtomContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<AtomContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ATOM_ID
    /// Returns `None` if there is no child corresponding to token ATOM_ID
    fn ATOM_ID(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_ATOM_ID, 0)
    }
    /// Retrieves first TerminalNode corresponding to token QUOTED_ATOM
    /// Returns `None` if there is no child corresponding to token QUOTED_ATOM
    fn QUOTED_ATOM(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_QUOTED_ATOM, 0)
    }
    /// Retrieves first TerminalNode corresponding to token GRAPHIC_ATOM
    /// Returns `None` if there is no child corresponding to token GRAPHIC_ATOM
    fn GRAPHIC_ATOM(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_GRAPHIC_ATOM, 0)
    }
}

impl<'input> AtomContextAttrs<'input> for AtomContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn atom(&mut self) -> Result<Rc<AtomContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = AtomContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 42, RULE_atom);
        let mut _localctx: Rc<AtomContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(194);
                _la = recog.base.input.la(1);
                if { !(((_la - 54) & !0x3f) == 0 && ((1usize << (_la - 54)) & 19) != 0) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- integer ----------------
pub type IntegerContextAll<'input> = IntegerContext<'input>;

pub type IntegerContext<'input> = BaseParserRuleContext<'input, IntegerContextExt<'input>>;

#[derive(Clone)]
pub struct IntegerContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> prologParserContext<'input> for IntegerContext<'input> {}

impl<'input, 'a> Listenable<dyn prologListener<'input> + 'a> for IntegerContext<'input> {
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_integer(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_integer(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn prologVisitor<'input> + 'a> for IntegerContext<'input> {
    fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) {
        visitor.visit_integer(self);
    }
}

impl<'input> CustomRuleContext<'input> for IntegerContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = prologParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_integer
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_integer }
}
antlr4rust::tid! {IntegerContextExt<'a>}

impl<'input> IntegerContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn prologParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<IntegerContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            IntegerContextExt { ph: PhantomData },
        ))
    }
}

pub trait IntegerContextAttrs<'input>:
    prologParserContext<'input> + BorrowMut<IntegerContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token DECIMAL
    /// Returns `None` if there is no child corresponding to token DECIMAL
    fn DECIMAL(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_DECIMAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token BINARY
    /// Returns `None` if there is no child corresponding to token BINARY
    fn BINARY(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_BINARY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token OCTAL
    /// Returns `None` if there is no child corresponding to token OCTAL
    fn OCTAL(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_OCTAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token HEX
    /// Returns `None` if there is no child corresponding to token HEX
    fn HEX(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_HEX, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CHARACTER_CODE_CONSTANT
    /// Returns `None` if there is no child corresponding to token CHARACTER_CODE_CONSTANT
    fn CHARACTER_CODE_CONSTANT(&self) -> Option<Rc<TerminalNode<'input, prologParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(prolog_CHARACTER_CODE_CONSTANT, 0)
    }
}

impl<'input> IntegerContextAttrs<'input> for IntegerContext<'input> {}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn integer(&mut self) -> Result<Rc<IntegerContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = IntegerContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 44, RULE_integer);
        let mut _localctx: Rc<IntegerContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(196);
                _la = recog.base.input.la(1);
                if { !(((_la - 48) & !0x3f) == 0 && ((1usize << (_la - 48)) & 31) != 0) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
lazy_static! {
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(_ATN.clone(), _ATN.get_decision_state(i), i).into())
        }
        Arc::new(dfa)
    };
    static ref _serializedATN: Vec<i32> = vec![
        4, 1, 62, 199, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 4, 2, 5, 7, 5, 2,
        6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2,
        13, 7, 13, 2, 14, 7, 14, 2, 15, 7, 15, 2, 16, 7, 16, 2, 17, 7, 17, 2, 18, 7, 18, 2, 19, 7,
        19, 2, 20, 7, 20, 2, 21, 7, 21, 2, 22, 7, 22, 1, 0, 5, 0, 48, 8, 0, 10, 0, 12, 0, 51, 9, 0,
        1, 0, 1, 0, 1, 1, 1, 1, 3, 1, 57, 8, 1, 1, 2, 1, 2, 1, 2, 1, 2, 1, 3, 1, 3, 1, 3, 3, 3, 66,
        8, 3, 1, 3, 1, 3, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 3, 4, 75, 8, 4, 1, 5, 1, 5, 1, 6, 1, 6, 1,
        6, 3, 6, 82, 8, 6, 1, 7, 1, 7, 1, 7, 3, 7, 87, 8, 7, 1, 8, 1, 8, 1, 8, 3, 8, 92, 8, 8, 1,
        9, 1, 9, 1, 9, 3, 9, 97, 8, 9, 1, 10, 1, 10, 1, 10, 1, 10, 3, 10, 103, 8, 10, 1, 11, 1, 11,
        1, 12, 1, 12, 1, 12, 3, 12, 110, 8, 12, 1, 13, 1, 13, 1, 13, 5, 13, 115, 8, 13, 10, 13, 12,
        13, 118, 9, 13, 1, 14, 1, 14, 1, 14, 5, 14, 123, 8, 14, 10, 14, 12, 14, 126, 9, 14, 1, 15,
        1, 15, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 3, 16, 137, 8, 16, 3, 16, 139, 8,
        16, 1, 17, 1, 17, 1, 17, 5, 17, 144, 8, 17, 10, 17, 12, 17, 147, 9, 17, 1, 18, 1, 18, 1,
        18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1, 18, 1,
        18, 1, 18, 1, 18, 3, 18, 167, 8, 18, 1, 18, 1, 18, 3, 18, 171, 8, 18, 1, 19, 1, 19, 1, 19,
        1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 3, 19, 185, 8, 19, 1, 20, 1,
        20, 1, 20, 5, 20, 190, 8, 20, 10, 20, 12, 20, 193, 9, 20, 1, 21, 1, 21, 1, 22, 1, 22, 1,
        22, 0, 0, 23, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38,
        40, 42, 44, 0, 6, 1, 0, 4, 19, 2, 0, 28, 29, 31, 32, 3, 0, 20, 23, 26, 27, 33, 34, 1, 0,
        30, 32, 2, 0, 54, 55, 58, 58, 1, 0, 48, 52, 205, 0, 49, 1, 0, 0, 0, 2, 56, 1, 0, 0, 0, 4,
        58, 1, 0, 0, 0, 6, 62, 1, 0, 0, 0, 8, 69, 1, 0, 0, 0, 10, 76, 1, 0, 0, 0, 12, 78, 1, 0, 0,
        0, 14, 83, 1, 0, 0, 0, 16, 88, 1, 0, 0, 0, 18, 96, 1, 0, 0, 0, 20, 98, 1, 0, 0, 0, 22, 104,
        1, 0, 0, 0, 24, 106, 1, 0, 0, 0, 26, 111, 1, 0, 0, 0, 28, 119, 1, 0, 0, 0, 30, 127, 1, 0,
        0, 0, 32, 138, 1, 0, 0, 0, 34, 140, 1, 0, 0, 0, 36, 170, 1, 0, 0, 0, 38, 184, 1, 0, 0, 0,
        40, 186, 1, 0, 0, 0, 42, 194, 1, 0, 0, 0, 44, 196, 1, 0, 0, 0, 46, 48, 3, 2, 1, 0, 47, 46,
        1, 0, 0, 0, 48, 51, 1, 0, 0, 0, 49, 47, 1, 0, 0, 0, 49, 50, 1, 0, 0, 0, 50, 52, 1, 0, 0, 0,
        51, 49, 1, 0, 0, 0, 52, 53, 5, 0, 0, 1, 53, 1, 1, 0, 0, 0, 54, 57, 3, 4, 2, 0, 55, 57, 3,
        6, 3, 0, 56, 54, 1, 0, 0, 0, 56, 55, 1, 0, 0, 0, 57, 3, 1, 0, 0, 0, 58, 59, 5, 1, 0, 0, 59,
        60, 3, 10, 5, 0, 60, 61, 5, 38, 0, 0, 61, 5, 1, 0, 0, 0, 62, 65, 3, 8, 4, 0, 63, 64, 5, 1,
        0, 0, 64, 66, 3, 10, 5, 0, 65, 63, 1, 0, 0, 0, 65, 66, 1, 0, 0, 0, 66, 67, 1, 0, 0, 0, 67,
        68, 5, 38, 0, 0, 68, 7, 1, 0, 0, 0, 69, 74, 3, 42, 21, 0, 70, 71, 5, 39, 0, 0, 71, 72, 3,
        34, 17, 0, 72, 73, 5, 40, 0, 0, 73, 75, 1, 0, 0, 0, 74, 70, 1, 0, 0, 0, 74, 75, 1, 0, 0, 0,
        75, 9, 1, 0, 0, 0, 76, 77, 3, 12, 6, 0, 77, 11, 1, 0, 0, 0, 78, 81, 3, 14, 7, 0, 79, 80, 5,
        35, 0, 0, 80, 82, 3, 12, 6, 0, 81, 79, 1, 0, 0, 0, 81, 82, 1, 0, 0, 0, 82, 13, 1, 0, 0, 0,
        83, 86, 3, 16, 8, 0, 84, 85, 5, 2, 0, 0, 85, 87, 3, 14, 7, 0, 86, 84, 1, 0, 0, 0, 86, 87,
        1, 0, 0, 0, 87, 15, 1, 0, 0, 0, 88, 91, 3, 18, 9, 0, 89, 90, 5, 36, 0, 0, 90, 92, 3, 16, 8,
        0, 91, 89, 1, 0, 0, 0, 91, 92, 1, 0, 0, 0, 92, 17, 1, 0, 0, 0, 93, 94, 5, 3, 0, 0, 94, 97,
        3, 18, 9, 0, 95, 97, 3, 20, 10, 0, 96, 93, 1, 0, 0, 0, 96, 95, 1, 0, 0, 0, 97, 19, 1, 0, 0,
        0, 98, 102, 3, 24, 12, 0, 99, 100, 3, 22, 11, 0, 100, 101, 3, 24, 12, 0, 101, 103, 1, 0, 0,
        0, 102, 99, 1, 0, 0, 0, 102, 103, 1, 0, 0, 0, 103, 21, 1, 0, 0, 0, 104, 105, 7, 0, 0, 0,
        105, 23, 1, 0, 0, 0, 106, 109, 3, 26, 13, 0, 107, 108, 5, 37, 0, 0, 108, 110, 3, 24, 12, 0,
        109, 107, 1, 0, 0, 0, 109, 110, 1, 0, 0, 0, 110, 25, 1, 0, 0, 0, 111, 116, 3, 28, 14, 0,
        112, 113, 7, 1, 0, 0, 113, 115, 3, 28, 14, 0, 114, 112, 1, 0, 0, 0, 115, 118, 1, 0, 0, 0,
        116, 114, 1, 0, 0, 0, 116, 117, 1, 0, 0, 0, 117, 27, 1, 0, 0, 0, 118, 116, 1, 0, 0, 0, 119,
        124, 3, 30, 15, 0, 120, 121, 7, 2, 0, 0, 121, 123, 3, 30, 15, 0, 122, 120, 1, 0, 0, 0, 123,
        126, 1, 0, 0, 0, 124, 122, 1, 0, 0, 0, 124, 125, 1, 0, 0, 0, 125, 29, 1, 0, 0, 0, 126, 124,
        1, 0, 0, 0, 127, 128, 3, 32, 16, 0, 128, 31, 1, 0, 0, 0, 129, 130, 7, 3, 0, 0, 130, 139, 3,
        30, 15, 0, 131, 136, 3, 36, 18, 0, 132, 133, 5, 24, 0, 0, 133, 137, 3, 36, 18, 0, 134, 135,
        5, 25, 0, 0, 135, 137, 3, 30, 15, 0, 136, 132, 1, 0, 0, 0, 136, 134, 1, 0, 0, 0, 136, 137,
        1, 0, 0, 0, 137, 139, 1, 0, 0, 0, 138, 129, 1, 0, 0, 0, 138, 131, 1, 0, 0, 0, 139, 33, 1,
        0, 0, 0, 140, 145, 3, 18, 9, 0, 141, 142, 5, 36, 0, 0, 142, 144, 3, 18, 9, 0, 143, 141, 1,
        0, 0, 0, 144, 147, 1, 0, 0, 0, 145, 143, 1, 0, 0, 0, 145, 146, 1, 0, 0, 0, 146, 35, 1, 0,
        0, 0, 147, 145, 1, 0, 0, 0, 148, 149, 3, 42, 21, 0, 149, 150, 5, 39, 0, 0, 150, 151, 3, 34,
        17, 0, 151, 152, 5, 40, 0, 0, 152, 171, 1, 0, 0, 0, 153, 171, 5, 53, 0, 0, 154, 171, 3, 44,
        22, 0, 155, 171, 5, 47, 0, 0, 156, 171, 3, 42, 21, 0, 157, 171, 5, 56, 0, 0, 158, 171, 5,
        57, 0, 0, 159, 171, 3, 38, 19, 0, 160, 161, 5, 39, 0, 0, 161, 162, 3, 10, 5, 0, 162, 163,
        5, 40, 0, 0, 163, 171, 1, 0, 0, 0, 164, 166, 5, 43, 0, 0, 165, 167, 3, 10, 5, 0, 166, 165,
        1, 0, 0, 0, 166, 167, 1, 0, 0, 0, 167, 168, 1, 0, 0, 0, 168, 171, 5, 44, 0, 0, 169, 171, 5,
        46, 0, 0, 170, 148, 1, 0, 0, 0, 170, 153, 1, 0, 0, 0, 170, 154, 1, 0, 0, 0, 170, 155, 1, 0,
        0, 0, 170, 156, 1, 0, 0, 0, 170, 157, 1, 0, 0, 0, 170, 158, 1, 0, 0, 0, 170, 159, 1, 0, 0,
        0, 170, 160, 1, 0, 0, 0, 170, 164, 1, 0, 0, 0, 170, 169, 1, 0, 0, 0, 171, 37, 1, 0, 0, 0,
        172, 173, 5, 41, 0, 0, 173, 185, 5, 42, 0, 0, 174, 175, 5, 41, 0, 0, 175, 176, 3, 40, 20,
        0, 176, 177, 5, 42, 0, 0, 177, 185, 1, 0, 0, 0, 178, 179, 5, 41, 0, 0, 179, 180, 3, 40, 20,
        0, 180, 181, 5, 45, 0, 0, 181, 182, 3, 18, 9, 0, 182, 183, 5, 42, 0, 0, 183, 185, 1, 0, 0,
        0, 184, 172, 1, 0, 0, 0, 184, 174, 1, 0, 0, 0, 184, 178, 1, 0, 0, 0, 185, 39, 1, 0, 0, 0,
        186, 191, 3, 18, 9, 0, 187, 188, 5, 36, 0, 0, 188, 190, 3, 18, 9, 0, 189, 187, 1, 0, 0, 0,
        190, 193, 1, 0, 0, 0, 191, 189, 1, 0, 0, 0, 191, 192, 1, 0, 0, 0, 192, 41, 1, 0, 0, 0, 193,
        191, 1, 0, 0, 0, 194, 195, 7, 4, 0, 0, 195, 43, 1, 0, 0, 0, 196, 197, 7, 5, 0, 0, 197, 45,
        1, 0, 0, 0, 19, 49, 56, 65, 74, 81, 86, 91, 96, 102, 109, 116, 124, 136, 138, 145, 166,
        170, 184, 191
    ];
}
