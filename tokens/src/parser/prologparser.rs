// Generated from prolog.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
use antlr4rust::PredictionContextCache;
use antlr4rust::parser::{Parser, BaseParser, ParserRecog, ParserNodeType};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::TokenSource;
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::errors::*;
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::recognizer::{Recognizer,Actions};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::error_strategy::{ErrorStrategy, DefaultErrorStrategy};
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext,cast,cast_mut};
use antlr4rust::tree::*;
use antlr4rust::token::{TOKEN_EOF,OwningToken,Token};
use antlr4rust::int_stream::EOF;
use antlr4rust::vocabulary::{Vocabulary,VocabularyImpl};
use antlr4rust::token_factory::{CommonTokenFactory,TokenFactory, TokenAware};
use super::prologlistener::*;
use super::prologvisitor::*;

use antlr4rust::lazy_static;
use antlr4rust::{TidAble,TidExt};

use std::marker::PhantomData;
use std::sync::Arc;
use std::rc::Rc;
use std::convert::TryFrom;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use std::borrow::{Borrow,BorrowMut};
use std::any::{Any,TypeId};

		pub const prolog_T__0:i32=1; 
		pub const prolog_T__1:i32=2; 
		pub const prolog_T__2:i32=3; 
		pub const prolog_T__3:i32=4; 
		pub const prolog_T__4:i32=5; 
		pub const prolog_T__5:i32=6; 
		pub const prolog_T__6:i32=7; 
		pub const prolog_T__7:i32=8; 
		pub const prolog_T__8:i32=9; 
		pub const prolog_T__9:i32=10; 
		pub const prolog_T__10:i32=11; 
		pub const prolog_T__11:i32=12; 
		pub const prolog_T__12:i32=13; 
		pub const prolog_T__13:i32=14; 
		pub const prolog_T__14:i32=15; 
		pub const prolog_T__15:i32=16; 
		pub const prolog_T__16:i32=17; 
		pub const prolog_T__17:i32=18; 
		pub const prolog_T__18:i32=19; 
		pub const prolog_T__19:i32=20; 
		pub const prolog_T__20:i32=21; 
		pub const prolog_T__21:i32=22; 
		pub const prolog_T__22:i32=23; 
		pub const prolog_T__23:i32=24; 
		pub const prolog_T__24:i32=25; 
		pub const prolog_T__25:i32=26; 
		pub const prolog_T__26:i32=27; 
		pub const prolog_T__27:i32=28; 
		pub const prolog_T__28:i32=29; 
		pub const prolog_T__29:i32=30; 
		pub const prolog_T__30:i32=31; 
		pub const prolog_T__31:i32=32; 
		pub const prolog_T__32:i32=33; 
		pub const prolog_T__33:i32=34; 
		pub const prolog_T__34:i32=35; 
		pub const prolog_T__35:i32=36; 
		pub const prolog_T__36:i32=37; 
		pub const prolog_T__37:i32=38; 
		pub const prolog_T__38:i32=39; 
		pub const prolog_T__39:i32=40; 
		pub const prolog_T__40:i32=41; 
		pub const prolog_T__41:i32=42; 
		pub const prolog_T__42:i32=43; 
		pub const prolog_T__43:i32=44; 
		pub const prolog_T__44:i32=45; 
		pub const prolog_T__45:i32=46; 
		pub const prolog_T__46:i32=47; 
		pub const prolog_T__47:i32=48; 
		pub const prolog_T__48:i32=49; 
		pub const prolog_T__49:i32=50; 
		pub const prolog_T__50:i32=51; 
		pub const prolog_T__51:i32=52; 
		pub const prolog_LETTER_DIGIT:i32=53; 
		pub const prolog_VARIABLE:i32=54; 
		pub const prolog_DECIMAL:i32=55; 
		pub const prolog_BINARY:i32=56; 
		pub const prolog_OCTAL:i32=57; 
		pub const prolog_HEX:i32=58; 
		pub const prolog_CHARACTER_CODE_CONSTANT:i32=59; 
		pub const prolog_FLOAT:i32=60; 
		pub const prolog_GRAPHIC_TOKEN:i32=61; 
		pub const prolog_QUOTED:i32=62; 
		pub const prolog_DOUBLE_QUOTED_LIST:i32=63; 
		pub const prolog_BACK_QUOTED_STRING:i32=64; 
		pub const prolog_WS:i32=65; 
		pub const prolog_COMMENT:i32=66; 
		pub const prolog_MULTILINE_COMMENT:i32=67;
	pub const prolog_EOF:i32=EOF;
	pub const RULE_p_text:usize = 0; 
	pub const RULE_directive:usize = 1; 
	pub const RULE_clause:usize = 2; 
	pub const RULE_fact:usize = 3; 
	pub const RULE_rule_:usize = 4; 
	pub const RULE_head:usize = 5; 
	pub const RULE_body:usize = 6; 
	pub const RULE_termlist:usize = 7; 
	pub const RULE_term:usize = 8; 
	pub const RULE_operator_:usize = 9; 
	pub const RULE_atom:usize = 10; 
	pub const RULE_integer:usize = 11;
	pub const ruleNames: [&'static str; 12] =  [
		"p_text", "directive", "clause", "fact", "rule_", "head", "body", "termlist", 
		"term", "operator_", "atom", "integer"
	];


	pub const _LITERAL_NAMES: [Option<&'static str>;53] = [
		None, Some("':-'"), Some("'.'"), Some("';'"), Some("','"), Some("'('"), 
		Some("')'"), Some("'-'"), Some("'['"), Some("'|'"), Some("']'"), Some("'{'"), 
		Some("'}'"), Some("'-->'"), Some("'?-'"), Some("'dynamic'"), Some("'multifile'"), 
		Some("'discontiguous'"), Some("'public'"), Some("'->'"), Some("'div'"), 
		Some("'\\+'"), Some("'='"), Some("'\\='"), Some("'=='"), Some("'\\=='"), 
		Some("'@<'"), Some("'@=<'"), Some("'@>'"), Some("'@>='"), Some("'=..'"), 
		Some("'is'"), Some("'=:='"), Some("'=\\='"), Some("'<'"), Some("'=<'"), 
		Some("'>'"), Some("'>='"), Some("':'"), Some("'+'"), Some("'/\\'"), Some("'\\/'"), 
		Some("'*'"), Some("'/'"), Some("'//'"), Some("'rem'"), Some("'mod'"), 
		Some("'<<'"), Some("'>>'"), Some("'**'"), Some("'^'"), Some("'\\'"), Some("'!'")
	];
	pub const _SYMBOLIC_NAMES: [Option<&'static str>;68]  = [
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, Some("LETTER_DIGIT"), Some("VARIABLE"), 
		Some("DECIMAL"), Some("BINARY"), Some("OCTAL"), Some("HEX"), Some("CHARACTER_CODE_CONSTANT"), 
		Some("FLOAT"), Some("GRAPHIC_TOKEN"), Some("QUOTED"), Some("DOUBLE_QUOTED_LIST"), 
		Some("BACK_QUOTED_STRING"), Some("WS"), Some("COMMENT"), Some("MULTILINE_COMMENT")
	];
	lazy_static!{
	    static ref _shared_context_cache: Arc<PredictionContextCache> = Arc::new(PredictionContextCache::new());
		static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(_LITERAL_NAMES.iter(), _SYMBOLIC_NAMES.iter(), None));
	}


type BaseParserType<'input, I> =
	BaseParser<'input,prologParserExt<'input>, I, prologParserContextType , dyn prologListener<'input> + 'input >;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type prologTreeWalker<'input,'a> =
	ParseTreeWalker<'input, 'a, prologParserContextType , dyn prologListener<'input> + 'a>;

/// Parser for prolog grammar
pub struct prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	base:BaseParserType<'input,I>,
	interpreter:Arc<ParserATNSimulator>,
	_shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >,
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn set_error_strategy(&mut self, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) {
        self.err_handler = strategy
    }

    pub fn with_strategy(input: I, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) -> Self {
		antlr4rust::recognizer::check_version("0","5");
		let interpreter = Arc::new(ParserATNSimulator::new(
			_ATN.clone(),
			_decision_to_DFA.clone(),
			_shared_context_cache.clone(),
		));
		Self {
			base: BaseParser::new_base_parser(
				input,
				Arc::clone(&interpreter),
				prologParserExt{
					_pd: Default::default(),
				}
			),
			interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }

}

type DynStrategy<'input,I> = Box<dyn ErrorStrategy<'input,BaseParserType<'input,I>> + 'input>;

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn new(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for prologParser
pub trait prologParserContext<'input>:
	for<'x> Listenable<dyn prologListener<'input> + 'x > + 
	for<'x> Visitable<dyn prologVisitor<'input> + 'x > + 
	ParserRuleContext<'input, TF=LocalTokenFactory<'input>, Ctx=prologParserContextType>
{}

antlr4rust::coerce_from!{ 'input : prologParserContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn prologParserContext<'input> + 'input
where
    T: prologVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn prologVisitor<'input> + 'x))
    }
}

impl<'input> prologParserContext<'input> for TerminalNode<'input,prologParserContextType> {}
impl<'input> prologParserContext<'input> for ErrorNode<'input,prologParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn prologParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn prologListener<'input> + 'input }

pub struct prologParserContextType;
antlr4rust::tid!{prologParserContextType}

impl<'input> ParserNodeType<'input> for prologParserContextType{
	type TF = LocalTokenFactory<'input>;
	type Type = dyn prologParserContext<'input> + 'input;
}

impl<'input, I> Deref for prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    type Target = BaseParserType<'input,I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct prologParserExt<'input>{
	_pd: PhantomData<&'input str>,
}

impl<'input> prologParserExt<'input>{
}
antlr4rust::tid! { prologParserExt<'a> }

impl<'input> TokenAware<'input> for prologParserExt<'input>{
	type TF = LocalTokenFactory<'input>;
}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> ParserRecog<'input, BaseParserType<'input,I>> for prologParserExt<'input>{}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> Actions<'input, BaseParserType<'input,I>> for prologParserExt<'input>{
	fn get_grammar_file_name(&self) -> & str{ "prolog.g4"}

   	fn get_rule_names(&self) -> &[& str] {&ruleNames}

   	fn get_vocabulary(&self) -> &dyn Vocabulary { &**VOCABULARY }
	fn sempred(_localctx: Option<&(dyn prologParserContext<'input> + 'input)>, rule_index: i32, pred_index: i32,
			   recog:&mut BaseParserType<'input,I>
	)->bool{
		match rule_index {
					8 => prologParser::<'input,I>::term_sempred(_localctx.and_then(|x|x.downcast_ref()), pred_index, recog),
			_ => true
		}
	}
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	fn term_sempred(_localctx: Option<&TermContext<'input>>, pred_index:i32,
						recog:&mut <Self as Deref>::Target
		) -> bool {
		match pred_index {
				0=>{
					recog.precpred(None, 5)
				}
			_ => true
		}
	}
}
//------------------- p_text ----------------
pub type P_textContextAll<'input> = P_textContext<'input>;


pub type P_textContext<'input> = BaseParserRuleContext<'input,P_textContextExt<'input>>;

#[derive(Clone)]
pub struct P_textContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for P_textContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for P_textContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_p_text(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_p_text(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for P_textContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_p_text(self);
	}
}

impl<'input> CustomRuleContext<'input> for P_textContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_p_text }
	//fn type_rule_index() -> usize where Self: Sized { RULE_p_text }
}
antlr4rust::tid!{P_textContextExt<'a>}

impl<'input> P_textContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<P_textContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,P_textContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait P_textContextAttrs<'input>: prologParserContext<'input> + BorrowMut<P_textContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_EOF, 0)
}
fn directive_all(&self) ->  Vec<Rc<DirectiveContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn directive(&self, i: usize) -> Option<Rc<DirectiveContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn clause_all(&self) ->  Vec<Rc<ClauseContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn clause(&self, i: usize) -> Option<Rc<ClauseContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> P_textContextAttrs<'input> for P_textContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn p_text(&mut self,)
	-> Result<Rc<P_textContextAll<'input>>,ANTLRError> {
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
			recog.base.set_state(28);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 1)) & !0x3f) == 0 && ((1usize << (_la - 1)) & 4294964437) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4294967295) != 0) {
				{
				recog.base.set_state(26);
				recog.err_handler.sync(&mut recog.base)?;
				match recog.base.input.la(1) {
				prolog_T__0 
					=> {
						{
						/*InvokeRule directive*/
						recog.base.set_state(24);
						recog.directive()?;

						}
					}

				prolog_T__2 |prolog_T__4 |prolog_T__6 |prolog_T__7 |prolog_T__10 |prolog_T__12 |
				prolog_T__13 |prolog_T__14 |prolog_T__15 |prolog_T__16 |prolog_T__17 |
				prolog_T__18 |prolog_T__19 |prolog_T__20 |prolog_T__21 |prolog_T__22 |
				prolog_T__23 |prolog_T__24 |prolog_T__25 |prolog_T__26 |prolog_T__27 |
				prolog_T__28 |prolog_T__29 |prolog_T__30 |prolog_T__31 |prolog_T__32 |
				prolog_T__33 |prolog_T__34 |prolog_T__35 |prolog_T__36 |prolog_T__37 |
				prolog_T__38 |prolog_T__39 |prolog_T__40 |prolog_T__41 |prolog_T__42 |
				prolog_T__43 |prolog_T__44 |prolog_T__45 |prolog_T__46 |prolog_T__47 |
				prolog_T__48 |prolog_T__49 |prolog_T__50 |prolog_T__51 |prolog_LETTER_DIGIT |
				prolog_VARIABLE |prolog_DECIMAL |prolog_BINARY |prolog_OCTAL |prolog_HEX |
				prolog_CHARACTER_CODE_CONSTANT |prolog_FLOAT |prolog_GRAPHIC_TOKEN |
				prolog_QUOTED |prolog_DOUBLE_QUOTED_LIST |prolog_BACK_QUOTED_STRING 
					=> {
						{
						/*InvokeRule clause*/
						recog.base.set_state(25);
						recog.clause()?;

						}
					}

					_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
				}
				}
				recog.base.set_state(30);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(31);
			recog.base.match_token(prolog_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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


pub type DirectiveContext<'input> = BaseParserRuleContext<'input,DirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct DirectiveContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for DirectiveContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for DirectiveContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_directive(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_directive(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for DirectiveContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_directive(self);
	}
}

impl<'input> CustomRuleContext<'input> for DirectiveContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_directive }
	//fn type_rule_index() -> usize where Self: Sized { RULE_directive }
}
antlr4rust::tid!{DirectiveContextExt<'a>}

impl<'input> DirectiveContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<DirectiveContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,DirectiveContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait DirectiveContextAttrs<'input>: prologParserContext<'input> + BorrowMut<DirectiveContextExt<'input>>{

fn term(&self) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> DirectiveContextAttrs<'input> for DirectiveContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn directive(&mut self,)
	-> Result<Rc<DirectiveContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = DirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_directive);
        let mut _localctx: Rc<DirectiveContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(33);
			recog.base.match_token(prolog_T__0,&mut recog.err_handler)?;

			/*InvokeRule term*/
			recog.base.set_state(34);
			recog.term_rec(0)?;

			recog.base.set_state(35);
			recog.base.match_token(prolog_T__1,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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


pub type ClauseContext<'input> = BaseParserRuleContext<'input,ClauseContextExt<'input>>;

#[derive(Clone)]
pub struct ClauseContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for ClauseContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for ClauseContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_clause(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_clause(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for ClauseContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_clause(self);
	}
}

impl<'input> CustomRuleContext<'input> for ClauseContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_clause }
	//fn type_rule_index() -> usize where Self: Sized { RULE_clause }
}
antlr4rust::tid!{ClauseContextExt<'a>}

impl<'input> ClauseContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ClauseContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ClauseContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ClauseContextAttrs<'input>: prologParserContext<'input> + BorrowMut<ClauseContextExt<'input>>{

fn fact(&self) -> Option<Rc<FactContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn rule_(&self) -> Option<Rc<Rule_ContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ClauseContextAttrs<'input> for ClauseContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn clause(&mut self,)
	-> Result<Rc<ClauseContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ClauseContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_clause);
        let mut _localctx: Rc<ClauseContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(39);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(2,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule fact*/
					recog.base.set_state(37);
					recog.fact()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule rule_*/
					recog.base.set_state(38);
					recog.rule_()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
//------------------- fact ----------------
pub type FactContextAll<'input> = FactContext<'input>;


pub type FactContext<'input> = BaseParserRuleContext<'input,FactContextExt<'input>>;

#[derive(Clone)]
pub struct FactContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for FactContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for FactContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_fact(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_fact(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for FactContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_fact(self);
	}
}

impl<'input> CustomRuleContext<'input> for FactContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_fact }
	//fn type_rule_index() -> usize where Self: Sized { RULE_fact }
}
antlr4rust::tid!{FactContextExt<'a>}

impl<'input> FactContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<FactContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FactContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait FactContextAttrs<'input>: prologParserContext<'input> + BorrowMut<FactContextExt<'input>>{

fn term(&self) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> FactContextAttrs<'input> for FactContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn fact(&mut self,)
	-> Result<Rc<FactContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FactContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_fact);
        let mut _localctx: Rc<FactContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule term*/
			recog.base.set_state(41);
			recog.term_rec(0)?;

			recog.base.set_state(42);
			recog.base.match_token(prolog_T__1,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
//------------------- rule_ ----------------
pub type Rule_ContextAll<'input> = Rule_Context<'input>;


pub type Rule_Context<'input> = BaseParserRuleContext<'input,Rule_ContextExt<'input>>;

#[derive(Clone)]
pub struct Rule_ContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for Rule_Context<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Rule_Context<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_rule_(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_rule_(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Rule_Context<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_rule_(self);
	}
}

impl<'input> CustomRuleContext<'input> for Rule_ContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_rule_ }
	//fn type_rule_index() -> usize where Self: Sized { RULE_rule_ }
}
antlr4rust::tid!{Rule_ContextExt<'a>}

impl<'input> Rule_ContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<Rule_ContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,Rule_ContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait Rule_ContextAttrs<'input>: prologParserContext<'input> + BorrowMut<Rule_ContextExt<'input>>{

fn head(&self) -> Option<Rc<HeadContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn body(&self) -> Option<Rc<BodyContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> Rule_ContextAttrs<'input> for Rule_Context<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn rule_(&mut self,)
	-> Result<Rc<Rule_ContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = Rule_ContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_rule_);
        let mut _localctx: Rc<Rule_ContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule head*/
			recog.base.set_state(44);
			recog.head()?;

			recog.base.set_state(45);
			recog.base.match_token(prolog_T__0,&mut recog.err_handler)?;

			/*InvokeRule body*/
			recog.base.set_state(46);
			recog.body()?;

			recog.base.set_state(47);
			recog.base.match_token(prolog_T__1,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
//------------------- head ----------------
pub type HeadContextAll<'input> = HeadContext<'input>;


pub type HeadContext<'input> = BaseParserRuleContext<'input,HeadContextExt<'input>>;

#[derive(Clone)]
pub struct HeadContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for HeadContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for HeadContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_head(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_head(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for HeadContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_head(self);
	}
}

impl<'input> CustomRuleContext<'input> for HeadContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_head }
	//fn type_rule_index() -> usize where Self: Sized { RULE_head }
}
antlr4rust::tid!{HeadContextExt<'a>}

impl<'input> HeadContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<HeadContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,HeadContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait HeadContextAttrs<'input>: prologParserContext<'input> + BorrowMut<HeadContextExt<'input>>{

fn term(&self) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> HeadContextAttrs<'input> for HeadContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn head(&mut self,)
	-> Result<Rc<HeadContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = HeadContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_head);
        let mut _localctx: Rc<HeadContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule term*/
			recog.base.set_state(49);
			recog.term_rec(0)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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


pub type BodyContext<'input> = BaseParserRuleContext<'input,BodyContextExt<'input>>;

#[derive(Clone)]
pub struct BodyContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for BodyContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for BodyContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_body(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_body(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for BodyContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_body(self);
	}
}

impl<'input> CustomRuleContext<'input> for BodyContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_body }
	//fn type_rule_index() -> usize where Self: Sized { RULE_body }
}
antlr4rust::tid!{BodyContextExt<'a>}

impl<'input> BodyContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<BodyContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BodyContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BodyContextAttrs<'input>: prologParserContext<'input> + BorrowMut<BodyContextExt<'input>>{

fn termlist_all(&self) ->  Vec<Rc<TermlistContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn termlist(&self, i: usize) -> Option<Rc<TermlistContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> BodyContextAttrs<'input> for BodyContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn body(&mut self,)
	-> Result<Rc<BodyContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 12, RULE_body);
        let mut _localctx: Rc<BodyContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule termlist*/
			recog.base.set_state(51);
			recog.termlist()?;

			recog.base.set_state(56);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==prolog_T__2 {
				{
				{
				recog.base.set_state(52);
				recog.base.match_token(prolog_T__2,&mut recog.err_handler)?;

				/*InvokeRule termlist*/
				recog.base.set_state(53);
				recog.termlist()?;

				}
				}
				recog.base.set_state(58);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
//------------------- termlist ----------------
pub type TermlistContextAll<'input> = TermlistContext<'input>;


pub type TermlistContext<'input> = BaseParserRuleContext<'input,TermlistContextExt<'input>>;

#[derive(Clone)]
pub struct TermlistContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for TermlistContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for TermlistContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_termlist(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_termlist(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for TermlistContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_termlist(self);
	}
}

impl<'input> CustomRuleContext<'input> for TermlistContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_termlist }
	//fn type_rule_index() -> usize where Self: Sized { RULE_termlist }
}
antlr4rust::tid!{TermlistContextExt<'a>}

impl<'input> TermlistContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<TermlistContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,TermlistContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait TermlistContextAttrs<'input>: prologParserContext<'input> + BorrowMut<TermlistContextExt<'input>>{

fn term_all(&self) ->  Vec<Rc<TermContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn term(&self, i: usize) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> TermlistContextAttrs<'input> for TermlistContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn termlist(&mut self,)
	-> Result<Rc<TermlistContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = TermlistContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_termlist);
        let mut _localctx: Rc<TermlistContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule term*/
			recog.base.set_state(59);
			recog.term_rec(0)?;

			recog.base.set_state(64);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==prolog_T__3 {
				{
				{
				recog.base.set_state(60);
				recog.base.match_token(prolog_T__3,&mut recog.err_handler)?;

				/*InvokeRule term*/
				recog.base.set_state(61);
				recog.term_rec(0)?;

				}
				}
				recog.base.set_state(66);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
//------------------- term ----------------
#[derive(Debug)]
pub enum TermContextAll<'input>{
	Atom_termContext(Atom_termContext<'input>),
	Binary_operatorContext(Binary_operatorContext<'input>),
	Unary_operatorContext(Unary_operatorContext<'input>),
	Braced_termContext(Braced_termContext<'input>),
	List_termContext(List_termContext<'input>),
	VariableContext(VariableContext<'input>),
	FloatContext(FloatContext<'input>),
	Compound_termContext(Compound_termContext<'input>),
	Integer_termContext(Integer_termContext<'input>),
	Curly_bracketed_termContext(Curly_bracketed_termContext<'input>),
Error(TermContext<'input>)
}
antlr4rust::tid!{TermContextAll<'a>}

impl<'input> antlr4rust::parser_rule_context::DerefSeal for TermContextAll<'input>{}

impl<'input> prologParserContext<'input> for TermContextAll<'input>{}

impl<'input> Deref for TermContextAll<'input>{
	type Target = dyn TermContextAttrs<'input> + 'input;
	fn deref(&self) -> &Self::Target{
		use TermContextAll::*;
		match self{
			Atom_termContext(inner) => inner,
			Binary_operatorContext(inner) => inner,
			Unary_operatorContext(inner) => inner,
			Braced_termContext(inner) => inner,
			List_termContext(inner) => inner,
			VariableContext(inner) => inner,
			FloatContext(inner) => inner,
			Compound_termContext(inner) => inner,
			Integer_termContext(inner) => inner,
			Curly_bracketed_termContext(inner) => inner,
Error(inner) => inner
		}
	}
}
impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for TermContextAll<'input>{
	fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) { self.deref().accept(visitor) }
}
impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for TermContextAll<'input>{
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> { self.deref().enter(listener) }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> { self.deref().exit(listener) }
}



pub type TermContext<'input> = BaseParserRuleContext<'input,TermContextExt<'input>>;

#[derive(Clone)]
pub struct TermContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for TermContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for TermContext<'input>{
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for TermContext<'input>{
}

impl<'input> CustomRuleContext<'input> for TermContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}
antlr4rust::tid!{TermContextExt<'a>}

impl<'input> TermContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<TermContextAll<'input>> {
		Rc::new(
		TermContextAll::Error(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,TermContextExt{

				ph:PhantomData
			}),
		)
		)
	}
}

pub trait TermContextAttrs<'input>: prologParserContext<'input> + BorrowMut<TermContextExt<'input>>{


}

impl<'input> TermContextAttrs<'input> for TermContext<'input>{}

pub type Atom_termContext<'input> = BaseParserRuleContext<'input,Atom_termContextExt<'input>>;

pub trait Atom_termContextAttrs<'input>: prologParserContext<'input>{
	fn atom(&self) -> Option<Rc<AtomContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Atom_termContextAttrs<'input> for Atom_termContext<'input>{}

pub struct Atom_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Atom_termContextExt<'a>}

impl<'input> prologParserContext<'input> for Atom_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Atom_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_atom_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_atom_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Atom_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_atom_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for Atom_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Atom_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Atom_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Atom_termContext<'input> {}

impl<'input> Atom_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Atom_termContext(
				BaseParserRuleContext::copy_from(ctx,Atom_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Binary_operatorContext<'input> = BaseParserRuleContext<'input,Binary_operatorContextExt<'input>>;

pub trait Binary_operatorContextAttrs<'input>: prologParserContext<'input>{
	fn term_all(&self) ->  Vec<Rc<TermContextAll<'input>>> where Self:Sized{
		self.children_of_type()
	}
	fn term(&self, i: usize) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
		self.child_of_type(i)
	}
	fn operator_(&self) -> Option<Rc<Operator_ContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Binary_operatorContextAttrs<'input> for Binary_operatorContext<'input>{}

pub struct Binary_operatorContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Binary_operatorContextExt<'a>}

impl<'input> prologParserContext<'input> for Binary_operatorContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Binary_operatorContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_binary_operator(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_binary_operator(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Binary_operatorContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_binary_operator(self);
	}
}

impl<'input> CustomRuleContext<'input> for Binary_operatorContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Binary_operatorContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Binary_operatorContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Binary_operatorContext<'input> {}

impl<'input> Binary_operatorContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Binary_operatorContext(
				BaseParserRuleContext::copy_from(ctx,Binary_operatorContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Unary_operatorContext<'input> = BaseParserRuleContext<'input,Unary_operatorContextExt<'input>>;

pub trait Unary_operatorContextAttrs<'input>: prologParserContext<'input>{
	fn operator_(&self) -> Option<Rc<Operator_ContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
	fn term(&self) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Unary_operatorContextAttrs<'input> for Unary_operatorContext<'input>{}

pub struct Unary_operatorContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Unary_operatorContextExt<'a>}

impl<'input> prologParserContext<'input> for Unary_operatorContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Unary_operatorContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_unary_operator(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_unary_operator(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Unary_operatorContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_unary_operator(self);
	}
}

impl<'input> CustomRuleContext<'input> for Unary_operatorContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Unary_operatorContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Unary_operatorContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Unary_operatorContext<'input> {}

impl<'input> Unary_operatorContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Unary_operatorContext(
				BaseParserRuleContext::copy_from(ctx,Unary_operatorContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Braced_termContext<'input> = BaseParserRuleContext<'input,Braced_termContextExt<'input>>;

pub trait Braced_termContextAttrs<'input>: prologParserContext<'input>{
	fn termlist(&self) -> Option<Rc<TermlistContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Braced_termContextAttrs<'input> for Braced_termContext<'input>{}

pub struct Braced_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Braced_termContextExt<'a>}

impl<'input> prologParserContext<'input> for Braced_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Braced_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_braced_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_braced_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Braced_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_braced_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for Braced_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Braced_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Braced_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Braced_termContext<'input> {}

impl<'input> Braced_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Braced_termContext(
				BaseParserRuleContext::copy_from(ctx,Braced_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type List_termContext<'input> = BaseParserRuleContext<'input,List_termContextExt<'input>>;

pub trait List_termContextAttrs<'input>: prologParserContext<'input>{
	fn termlist(&self) -> Option<Rc<TermlistContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
	fn term(&self) -> Option<Rc<TermContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> List_termContextAttrs<'input> for List_termContext<'input>{}

pub struct List_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{List_termContextExt<'a>}

impl<'input> prologParserContext<'input> for List_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for List_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_list_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_list_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for List_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_list_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for List_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for List_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for List_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for List_termContext<'input> {}

impl<'input> List_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::List_termContext(
				BaseParserRuleContext::copy_from(ctx,List_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type VariableContext<'input> = BaseParserRuleContext<'input,VariableContextExt<'input>>;

pub trait VariableContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token VARIABLE
	/// Returns `None` if there is no child corresponding to token VARIABLE
	fn VARIABLE(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_VARIABLE, 0)
	}
}

impl<'input> VariableContextAttrs<'input> for VariableContext<'input>{}

pub struct VariableContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{VariableContextExt<'a>}

impl<'input> prologParserContext<'input> for VariableContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for VariableContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_variable(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_variable(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for VariableContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_variable(self);
	}
}

impl<'input> CustomRuleContext<'input> for VariableContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for VariableContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for VariableContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for VariableContext<'input> {}

impl<'input> VariableContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::VariableContext(
				BaseParserRuleContext::copy_from(ctx,VariableContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type FloatContext<'input> = BaseParserRuleContext<'input,FloatContextExt<'input>>;

pub trait FloatContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token FLOAT
	/// Returns `None` if there is no child corresponding to token FLOAT
	fn FLOAT(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_FLOAT, 0)
	}
}

impl<'input> FloatContextAttrs<'input> for FloatContext<'input>{}

pub struct FloatContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{FloatContextExt<'a>}

impl<'input> prologParserContext<'input> for FloatContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for FloatContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_float(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_float(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for FloatContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_float(self);
	}
}

impl<'input> CustomRuleContext<'input> for FloatContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for FloatContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for FloatContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for FloatContext<'input> {}

impl<'input> FloatContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::FloatContext(
				BaseParserRuleContext::copy_from(ctx,FloatContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Compound_termContext<'input> = BaseParserRuleContext<'input,Compound_termContextExt<'input>>;

pub trait Compound_termContextAttrs<'input>: prologParserContext<'input>{
	fn atom(&self) -> Option<Rc<AtomContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
	fn termlist(&self) -> Option<Rc<TermlistContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Compound_termContextAttrs<'input> for Compound_termContext<'input>{}

pub struct Compound_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Compound_termContextExt<'a>}

impl<'input> prologParserContext<'input> for Compound_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Compound_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_compound_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_compound_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Compound_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_compound_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for Compound_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Compound_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Compound_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Compound_termContext<'input> {}

impl<'input> Compound_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Compound_termContext(
				BaseParserRuleContext::copy_from(ctx,Compound_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Integer_termContext<'input> = BaseParserRuleContext<'input,Integer_termContextExt<'input>>;

pub trait Integer_termContextAttrs<'input>: prologParserContext<'input>{
	fn integer(&self) -> Option<Rc<IntegerContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Integer_termContextAttrs<'input> for Integer_termContext<'input>{}

pub struct Integer_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Integer_termContextExt<'a>}

impl<'input> prologParserContext<'input> for Integer_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Integer_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_integer_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_integer_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Integer_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_integer_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for Integer_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Integer_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Integer_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Integer_termContext<'input> {}

impl<'input> Integer_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Integer_termContext(
				BaseParserRuleContext::copy_from(ctx,Integer_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Curly_bracketed_termContext<'input> = BaseParserRuleContext<'input,Curly_bracketed_termContextExt<'input>>;

pub trait Curly_bracketed_termContextAttrs<'input>: prologParserContext<'input>{
	fn termlist(&self) -> Option<Rc<TermlistContextAll<'input>>> where Self:Sized{
		self.child_of_type(0)
	}
}

impl<'input> Curly_bracketed_termContextAttrs<'input> for Curly_bracketed_termContext<'input>{}

pub struct Curly_bracketed_termContextExt<'input>{
	base:TermContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Curly_bracketed_termContextExt<'a>}

impl<'input> prologParserContext<'input> for Curly_bracketed_termContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Curly_bracketed_termContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_curly_bracketed_term(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_curly_bracketed_term(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Curly_bracketed_termContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_curly_bracketed_term(self);
	}
}

impl<'input> CustomRuleContext<'input> for Curly_bracketed_termContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_term }
	//fn type_rule_index() -> usize where Self: Sized { RULE_term }
}

impl<'input> Borrow<TermContextExt<'input>> for Curly_bracketed_termContext<'input>{
	fn borrow(&self) -> &TermContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<TermContextExt<'input>> for Curly_bracketed_termContext<'input>{
	fn borrow_mut(&mut self) -> &mut TermContextExt<'input> { &mut self.base }
}

impl<'input> TermContextAttrs<'input> for Curly_bracketed_termContext<'input> {}

impl<'input> Curly_bracketed_termContextExt<'input>{
	fn new(ctx: &dyn TermContextAttrs<'input>) -> Rc<TermContextAll<'input>>  {
		Rc::new(
			TermContextAll::Curly_bracketed_termContext(
				BaseParserRuleContext::copy_from(ctx,Curly_bracketed_termContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn  term(&mut self,)
	-> Result<Rc<TermContextAll<'input>>,ANTLRError> {
		self.term_rec(0)
	}

	fn term_rec(&mut self, _p: i32)
	-> Result<Rc<TermContextAll<'input>>,ANTLRError> {
		let recog = self;
		let _parentctx = recog.ctx.take();
		let _parentState = recog.base.get_state();
		let mut _localctx = TermContextExt::new(_parentctx.clone(), recog.base.get_state());
		recog.base.enter_recursion_rule(_localctx.clone(), 16, RULE_term, _p);
	    let mut _localctx: Rc<TermContextAll> = _localctx;
        let mut _prevctx = _localctx.clone();
		let _startState = 16;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {
			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(102);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(8,&mut recog.base)? {
				1 =>{
					{
					let mut tmp = VariableContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();

					recog.base.set_state(68);
					recog.base.match_token(prolog_VARIABLE,&mut recog.err_handler)?;

					}
				}
			,
				2 =>{
					{
					let mut tmp = Braced_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					recog.base.set_state(69);
					recog.base.match_token(prolog_T__4,&mut recog.err_handler)?;

					/*InvokeRule termlist*/
					recog.base.set_state(70);
					recog.termlist()?;

					recog.base.set_state(71);
					recog.base.match_token(prolog_T__5,&mut recog.err_handler)?;

					}
				}
			,
				3 =>{
					{
					let mut tmp = Integer_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					recog.base.set_state(74);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==prolog_T__6 {
						{
						recog.base.set_state(73);
						recog.base.match_token(prolog_T__6,&mut recog.err_handler)?;

						}
					}

					/*InvokeRule integer*/
					recog.base.set_state(76);
					recog.integer()?;

					}
				}
			,
				4 =>{
					{
					let mut tmp = FloatContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					recog.base.set_state(78);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==prolog_T__6 {
						{
						recog.base.set_state(77);
						recog.base.match_token(prolog_T__6,&mut recog.err_handler)?;

						}
					}

					recog.base.set_state(80);
					recog.base.match_token(prolog_FLOAT,&mut recog.err_handler)?;

					}
				}
			,
				5 =>{
					{
					let mut tmp = Compound_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					/*InvokeRule atom*/
					recog.base.set_state(81);
					recog.atom()?;

					recog.base.set_state(82);
					recog.base.match_token(prolog_T__4,&mut recog.err_handler)?;

					/*InvokeRule termlist*/
					recog.base.set_state(83);
					recog.termlist()?;

					recog.base.set_state(84);
					recog.base.match_token(prolog_T__5,&mut recog.err_handler)?;

					}
				}
			,
				6 =>{
					{
					let mut tmp = Unary_operatorContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					/*InvokeRule operator_*/
					recog.base.set_state(86);
					recog.operator_()?;

					/*InvokeRule term*/
					recog.base.set_state(87);
					recog.term_rec(4)?;

					}
				}
			,
				7 =>{
					{
					let mut tmp = List_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					recog.base.set_state(89);
					recog.base.match_token(prolog_T__7,&mut recog.err_handler)?;

					/*InvokeRule termlist*/
					recog.base.set_state(90);
					recog.termlist()?;

					recog.base.set_state(93);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if _la==prolog_T__8 {
						{
						recog.base.set_state(91);
						recog.base.match_token(prolog_T__8,&mut recog.err_handler)?;

						/*InvokeRule term*/
						recog.base.set_state(92);
						recog.term_rec(0)?;

						}
					}

					recog.base.set_state(95);
					recog.base.match_token(prolog_T__9,&mut recog.err_handler)?;

					}
				}
			,
				8 =>{
					{
					let mut tmp = Curly_bracketed_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					recog.base.set_state(97);
					recog.base.match_token(prolog_T__10,&mut recog.err_handler)?;

					/*InvokeRule termlist*/
					recog.base.set_state(98);
					recog.termlist()?;

					recog.base.set_state(99);
					recog.base.match_token(prolog_T__11,&mut recog.err_handler)?;

					}
				}
			,
				9 =>{
					{
					let mut tmp = Atom_termContextExt::new(&**_localctx);
					recog.ctx = Some(tmp.clone());
					_localctx = tmp;
					_prevctx = _localctx.clone();
					/*InvokeRule atom*/
					recog.base.set_state(101);
					recog.atom()?;

					}
				}

				_ => {}
			}
			let tmp = recog.input.lt(-1).cloned();
			recog.ctx.as_ref().unwrap().set_stop(tmp);
			recog.base.set_state(110);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(9,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					recog.trigger_exit_rule_event()?;
					_prevctx = _localctx.clone();
					{
					{
					/*recRuleLabeledAltStartAction*/
					let mut tmp = Binary_operatorContextExt::new(&**TermContextExt::new(_parentctx.clone(), _parentState));
					recog.push_new_recursion_context(tmp.clone(), _startState, RULE_term)?;
					_localctx = tmp;
					recog.base.set_state(104);
					if !({let _localctx = Some(_localctx.clone());
					recog.precpred(None, 5)}) {
						Err(FailedPredicateError::new(&mut recog.base, Some("recog.precpred(None, 5)".to_owned()), None))?;
					}
					/*InvokeRule operator_*/
					recog.base.set_state(105);
					recog.operator_()?;

					/*InvokeRule term*/
					recog.base.set_state(106);
					recog.term_rec(5)?;

					}
					} 
				}
				recog.base.set_state(112);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(9,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_) => {},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re)=>{
			//_localctx.exception = re;
			recog.err_handler.report_error(&mut recog.base, re);
	        recog.err_handler.recover(&mut recog.base, re)?;}
		}
		recog.base.unroll_recursion_context(_parentctx)?;

		Ok(_localctx)
	}
}
//------------------- operator_ ----------------
pub type Operator_ContextAll<'input> = Operator_Context<'input>;


pub type Operator_Context<'input> = BaseParserRuleContext<'input,Operator_ContextExt<'input>>;

#[derive(Clone)]
pub struct Operator_ContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for Operator_Context<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Operator_Context<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_operator_(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_operator_(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Operator_Context<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_operator_(self);
	}
}

impl<'input> CustomRuleContext<'input> for Operator_ContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_operator_ }
	//fn type_rule_index() -> usize where Self: Sized { RULE_operator_ }
}
antlr4rust::tid!{Operator_ContextExt<'a>}

impl<'input> Operator_ContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<Operator_ContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,Operator_ContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait Operator_ContextAttrs<'input>: prologParserContext<'input> + BorrowMut<Operator_ContextExt<'input>>{


}

impl<'input> Operator_ContextAttrs<'input> for Operator_Context<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn operator_(&mut self,)
	-> Result<Rc<Operator_ContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = Operator_ContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_operator_);
        let mut _localctx: Rc<Operator_ContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(113);
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 4294959232) != 0) || ((((_la - 32)) & !0x3f) == 0 && ((1usize << (_la - 32)) & 1048575) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
#[derive(Debug)]
pub enum AtomContextAll<'input>{
	Backq_stringContext(Backq_stringContext<'input>),
	CutContext(CutContext<'input>),
	Empty_bracesContext(Empty_bracesContext<'input>),
	Dq_stringContext(Dq_stringContext<'input>),
	NameContext(NameContext<'input>),
	Quoted_stringContext(Quoted_stringContext<'input>),
	Empty_listContext(Empty_listContext<'input>),
	GraphicContext(GraphicContext<'input>),
	SemicolonContext(SemicolonContext<'input>),
Error(AtomContext<'input>)
}
antlr4rust::tid!{AtomContextAll<'a>}

impl<'input> antlr4rust::parser_rule_context::DerefSeal for AtomContextAll<'input>{}

impl<'input> prologParserContext<'input> for AtomContextAll<'input>{}

impl<'input> Deref for AtomContextAll<'input>{
	type Target = dyn AtomContextAttrs<'input> + 'input;
	fn deref(&self) -> &Self::Target{
		use AtomContextAll::*;
		match self{
			Backq_stringContext(inner) => inner,
			CutContext(inner) => inner,
			Empty_bracesContext(inner) => inner,
			Dq_stringContext(inner) => inner,
			NameContext(inner) => inner,
			Quoted_stringContext(inner) => inner,
			Empty_listContext(inner) => inner,
			GraphicContext(inner) => inner,
			SemicolonContext(inner) => inner,
Error(inner) => inner
		}
	}
}
impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for AtomContextAll<'input>{
	fn accept(&self, visitor: &mut (dyn prologVisitor<'input> + 'a)) { self.deref().accept(visitor) }
}
impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for AtomContextAll<'input>{
    fn enter(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> { self.deref().enter(listener) }
    fn exit(&self, listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> { self.deref().exit(listener) }
}



pub type AtomContext<'input> = BaseParserRuleContext<'input,AtomContextExt<'input>>;

#[derive(Clone)]
pub struct AtomContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for AtomContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for AtomContext<'input>{
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for AtomContext<'input>{
}

impl<'input> CustomRuleContext<'input> for AtomContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}
antlr4rust::tid!{AtomContextExt<'a>}

impl<'input> AtomContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<AtomContextAll<'input>> {
		Rc::new(
		AtomContextAll::Error(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AtomContextExt{

				ph:PhantomData
			}),
		)
		)
	}
}

pub trait AtomContextAttrs<'input>: prologParserContext<'input> + BorrowMut<AtomContextExt<'input>>{


}

impl<'input> AtomContextAttrs<'input> for AtomContext<'input>{}

pub type Backq_stringContext<'input> = BaseParserRuleContext<'input,Backq_stringContextExt<'input>>;

pub trait Backq_stringContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token BACK_QUOTED_STRING
	/// Returns `None` if there is no child corresponding to token BACK_QUOTED_STRING
	fn BACK_QUOTED_STRING(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_BACK_QUOTED_STRING, 0)
	}
}

impl<'input> Backq_stringContextAttrs<'input> for Backq_stringContext<'input>{}

pub struct Backq_stringContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Backq_stringContextExt<'a>}

impl<'input> prologParserContext<'input> for Backq_stringContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Backq_stringContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_backq_string(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_backq_string(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Backq_stringContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_backq_string(self);
	}
}

impl<'input> CustomRuleContext<'input> for Backq_stringContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for Backq_stringContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for Backq_stringContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for Backq_stringContext<'input> {}

impl<'input> Backq_stringContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::Backq_stringContext(
				BaseParserRuleContext::copy_from(ctx,Backq_stringContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type CutContext<'input> = BaseParserRuleContext<'input,CutContextExt<'input>>;

pub trait CutContextAttrs<'input>: prologParserContext<'input>{
}

impl<'input> CutContextAttrs<'input> for CutContext<'input>{}

pub struct CutContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{CutContextExt<'a>}

impl<'input> prologParserContext<'input> for CutContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for CutContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_cut(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_cut(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for CutContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_cut(self);
	}
}

impl<'input> CustomRuleContext<'input> for CutContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for CutContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for CutContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for CutContext<'input> {}

impl<'input> CutContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::CutContext(
				BaseParserRuleContext::copy_from(ctx,CutContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Empty_bracesContext<'input> = BaseParserRuleContext<'input,Empty_bracesContextExt<'input>>;

pub trait Empty_bracesContextAttrs<'input>: prologParserContext<'input>{
}

impl<'input> Empty_bracesContextAttrs<'input> for Empty_bracesContext<'input>{}

pub struct Empty_bracesContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Empty_bracesContextExt<'a>}

impl<'input> prologParserContext<'input> for Empty_bracesContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Empty_bracesContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_empty_braces(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_empty_braces(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Empty_bracesContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_empty_braces(self);
	}
}

impl<'input> CustomRuleContext<'input> for Empty_bracesContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for Empty_bracesContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for Empty_bracesContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for Empty_bracesContext<'input> {}

impl<'input> Empty_bracesContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::Empty_bracesContext(
				BaseParserRuleContext::copy_from(ctx,Empty_bracesContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Dq_stringContext<'input> = BaseParserRuleContext<'input,Dq_stringContextExt<'input>>;

pub trait Dq_stringContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token DOUBLE_QUOTED_LIST
	/// Returns `None` if there is no child corresponding to token DOUBLE_QUOTED_LIST
	fn DOUBLE_QUOTED_LIST(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_DOUBLE_QUOTED_LIST, 0)
	}
}

impl<'input> Dq_stringContextAttrs<'input> for Dq_stringContext<'input>{}

pub struct Dq_stringContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Dq_stringContextExt<'a>}

impl<'input> prologParserContext<'input> for Dq_stringContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Dq_stringContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_dq_string(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_dq_string(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Dq_stringContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_dq_string(self);
	}
}

impl<'input> CustomRuleContext<'input> for Dq_stringContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for Dq_stringContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for Dq_stringContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for Dq_stringContext<'input> {}

impl<'input> Dq_stringContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::Dq_stringContext(
				BaseParserRuleContext::copy_from(ctx,Dq_stringContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type NameContext<'input> = BaseParserRuleContext<'input,NameContextExt<'input>>;

pub trait NameContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token LETTER_DIGIT
	/// Returns `None` if there is no child corresponding to token LETTER_DIGIT
	fn LETTER_DIGIT(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_LETTER_DIGIT, 0)
	}
}

impl<'input> NameContextAttrs<'input> for NameContext<'input>{}

pub struct NameContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{NameContextExt<'a>}

impl<'input> prologParserContext<'input> for NameContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for NameContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_name(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_name(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for NameContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_name(self);
	}
}

impl<'input> CustomRuleContext<'input> for NameContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for NameContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for NameContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for NameContext<'input> {}

impl<'input> NameContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::NameContext(
				BaseParserRuleContext::copy_from(ctx,NameContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Quoted_stringContext<'input> = BaseParserRuleContext<'input,Quoted_stringContextExt<'input>>;

pub trait Quoted_stringContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token QUOTED
	/// Returns `None` if there is no child corresponding to token QUOTED
	fn QUOTED(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_QUOTED, 0)
	}
}

impl<'input> Quoted_stringContextAttrs<'input> for Quoted_stringContext<'input>{}

pub struct Quoted_stringContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Quoted_stringContextExt<'a>}

impl<'input> prologParserContext<'input> for Quoted_stringContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Quoted_stringContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_quoted_string(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_quoted_string(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Quoted_stringContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_quoted_string(self);
	}
}

impl<'input> CustomRuleContext<'input> for Quoted_stringContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for Quoted_stringContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for Quoted_stringContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for Quoted_stringContext<'input> {}

impl<'input> Quoted_stringContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::Quoted_stringContext(
				BaseParserRuleContext::copy_from(ctx,Quoted_stringContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type Empty_listContext<'input> = BaseParserRuleContext<'input,Empty_listContextExt<'input>>;

pub trait Empty_listContextAttrs<'input>: prologParserContext<'input>{
}

impl<'input> Empty_listContextAttrs<'input> for Empty_listContext<'input>{}

pub struct Empty_listContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{Empty_listContextExt<'a>}

impl<'input> prologParserContext<'input> for Empty_listContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for Empty_listContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_empty_list(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_empty_list(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for Empty_listContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_empty_list(self);
	}
}

impl<'input> CustomRuleContext<'input> for Empty_listContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for Empty_listContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for Empty_listContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for Empty_listContext<'input> {}

impl<'input> Empty_listContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::Empty_listContext(
				BaseParserRuleContext::copy_from(ctx,Empty_listContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type GraphicContext<'input> = BaseParserRuleContext<'input,GraphicContextExt<'input>>;

pub trait GraphicContextAttrs<'input>: prologParserContext<'input>{
	/// Retrieves first TerminalNode corresponding to token GRAPHIC_TOKEN
	/// Returns `None` if there is no child corresponding to token GRAPHIC_TOKEN
	fn GRAPHIC_TOKEN(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
		self.get_token(prolog_GRAPHIC_TOKEN, 0)
	}
}

impl<'input> GraphicContextAttrs<'input> for GraphicContext<'input>{}

pub struct GraphicContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{GraphicContextExt<'a>}

impl<'input> prologParserContext<'input> for GraphicContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for GraphicContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_graphic(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_graphic(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for GraphicContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_graphic(self);
	}
}

impl<'input> CustomRuleContext<'input> for GraphicContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for GraphicContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for GraphicContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for GraphicContext<'input> {}

impl<'input> GraphicContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::GraphicContext(
				BaseParserRuleContext::copy_from(ctx,GraphicContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

pub type SemicolonContext<'input> = BaseParserRuleContext<'input,SemicolonContextExt<'input>>;

pub trait SemicolonContextAttrs<'input>: prologParserContext<'input>{
}

impl<'input> SemicolonContextAttrs<'input> for SemicolonContext<'input>{}

pub struct SemicolonContextExt<'input>{
	base:AtomContextExt<'input>,
	ph:PhantomData<&'input str>
}

antlr4rust::tid!{SemicolonContextExt<'a>}

impl<'input> prologParserContext<'input> for SemicolonContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for SemicolonContext<'input>{
	fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.enter_every_rule(self)?;
		listener.enter_semicolon(self);
		Ok(())
	}
	fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
		listener.exit_semicolon(self);
		listener.exit_every_rule(self)?;
		Ok(())
	}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for SemicolonContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_semicolon(self);
	}
}

impl<'input> CustomRuleContext<'input> for SemicolonContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_atom }
	//fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}

impl<'input> Borrow<AtomContextExt<'input>> for SemicolonContext<'input>{
	fn borrow(&self) -> &AtomContextExt<'input> { &self.base }
}
impl<'input> BorrowMut<AtomContextExt<'input>> for SemicolonContext<'input>{
	fn borrow_mut(&mut self) -> &mut AtomContextExt<'input> { &mut self.base }
}

impl<'input> AtomContextAttrs<'input> for SemicolonContext<'input> {}

impl<'input> SemicolonContextExt<'input>{
	fn new(ctx: &dyn AtomContextAttrs<'input>) -> Rc<AtomContextAll<'input>>  {
		Rc::new(
			AtomContextAll::SemicolonContext(
				BaseParserRuleContext::copy_from(ctx,SemicolonContextExt{
        			base: ctx.borrow().clone(),
        			ph:PhantomData
				})
			)
		)
	}
}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn atom(&mut self,)
	-> Result<Rc<AtomContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AtomContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_atom);
        let mut _localctx: Rc<AtomContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(126);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			prolog_T__7 
				=> {
					let tmp = Empty_listContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 1)?;
					_localctx = tmp;
					{
					recog.base.set_state(115);
					recog.base.match_token(prolog_T__7,&mut recog.err_handler)?;

					recog.base.set_state(116);
					recog.base.match_token(prolog_T__9,&mut recog.err_handler)?;

					}
				}

			prolog_T__10 
				=> {
					let tmp = Empty_bracesContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 2)?;
					_localctx = tmp;
					{
					recog.base.set_state(117);
					recog.base.match_token(prolog_T__10,&mut recog.err_handler)?;

					recog.base.set_state(118);
					recog.base.match_token(prolog_T__11,&mut recog.err_handler)?;

					}
				}

			prolog_LETTER_DIGIT 
				=> {
					let tmp = NameContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 3)?;
					_localctx = tmp;
					{
					recog.base.set_state(119);
					recog.base.match_token(prolog_LETTER_DIGIT,&mut recog.err_handler)?;

					}
				}

			prolog_GRAPHIC_TOKEN 
				=> {
					let tmp = GraphicContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 4)?;
					_localctx = tmp;
					{
					recog.base.set_state(120);
					recog.base.match_token(prolog_GRAPHIC_TOKEN,&mut recog.err_handler)?;

					}
				}

			prolog_QUOTED 
				=> {
					let tmp = Quoted_stringContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 5)?;
					_localctx = tmp;
					{
					recog.base.set_state(121);
					recog.base.match_token(prolog_QUOTED,&mut recog.err_handler)?;

					}
				}

			prolog_DOUBLE_QUOTED_LIST 
				=> {
					let tmp = Dq_stringContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 6)?;
					_localctx = tmp;
					{
					recog.base.set_state(122);
					recog.base.match_token(prolog_DOUBLE_QUOTED_LIST,&mut recog.err_handler)?;

					}
				}

			prolog_BACK_QUOTED_STRING 
				=> {
					let tmp = Backq_stringContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 7)?;
					_localctx = tmp;
					{
					recog.base.set_state(123);
					recog.base.match_token(prolog_BACK_QUOTED_STRING,&mut recog.err_handler)?;

					}
				}

			prolog_T__2 
				=> {
					let tmp = SemicolonContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 8)?;
					_localctx = tmp;
					{
					recog.base.set_state(124);
					recog.base.match_token(prolog_T__2,&mut recog.err_handler)?;

					}
				}

			prolog_T__51 
				=> {
					let tmp = CutContextExt::new(&**_localctx);
					recog.base.enter_outer_alt(Some(tmp.clone()), 9)?;
					_localctx = tmp;
					{
					recog.base.set_state(125);
					recog.base.match_token(prolog_T__51,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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


pub type IntegerContext<'input> = BaseParserRuleContext<'input,IntegerContextExt<'input>>;

#[derive(Clone)]
pub struct IntegerContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> prologParserContext<'input> for IntegerContext<'input>{}

impl<'input,'a> Listenable<dyn prologListener<'input> + 'a> for IntegerContext<'input>{
		fn enter(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_integer(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn prologListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_integer(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn prologVisitor<'input> + 'a> for IntegerContext<'input>{
	fn accept(&self,visitor: &mut (dyn prologVisitor<'input> + 'a)) {
		visitor.visit_integer(self);
	}
}

impl<'input> CustomRuleContext<'input> for IntegerContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = prologParserContextType;
	fn get_rule_index(&self) -> usize { RULE_integer }
	//fn type_rule_index() -> usize where Self: Sized { RULE_integer }
}
antlr4rust::tid!{IntegerContextExt<'a>}

impl<'input> IntegerContextExt<'input>{
	fn new(parent: Option<Rc<dyn prologParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<IntegerContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,IntegerContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait IntegerContextAttrs<'input>: prologParserContext<'input> + BorrowMut<IntegerContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token DECIMAL
/// Returns `None` if there is no child corresponding to token DECIMAL
fn DECIMAL(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_DECIMAL, 0)
}
/// Retrieves first TerminalNode corresponding to token CHARACTER_CODE_CONSTANT
/// Returns `None` if there is no child corresponding to token CHARACTER_CODE_CONSTANT
fn CHARACTER_CODE_CONSTANT(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_CHARACTER_CODE_CONSTANT, 0)
}
/// Retrieves first TerminalNode corresponding to token BINARY
/// Returns `None` if there is no child corresponding to token BINARY
fn BINARY(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_BINARY, 0)
}
/// Retrieves first TerminalNode corresponding to token OCTAL
/// Returns `None` if there is no child corresponding to token OCTAL
fn OCTAL(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_OCTAL, 0)
}
/// Retrieves first TerminalNode corresponding to token HEX
/// Returns `None` if there is no child corresponding to token HEX
fn HEX(&self) -> Option<Rc<TerminalNode<'input,prologParserContextType>>> where Self:Sized{
	self.get_token(prolog_HEX, 0)
}

}

impl<'input> IntegerContextAttrs<'input> for IntegerContext<'input>{}

impl<'input, I> prologParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn integer(&mut self,)
	-> Result<Rc<IntegerContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = IntegerContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_integer);
        let mut _localctx: Rc<IntegerContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(128);
			_la = recog.base.input.la(1);
			if { !(((((_la - 55)) & !0x3f) == 0 && ((1usize << (_la - 55)) & 31) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
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
	lazy_static!{
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(
                _ATN.clone(),
                _ATN.get_decision_state(i),
                i,
            ).into())
        }
        Arc::new(dfa)
    };
	static ref _serializedATN: Vec<i32> = vec![
		4, 1, 67, 131, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 
		4, 2, 5, 7, 5, 2, 6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 
		7, 10, 2, 11, 7, 11, 1, 0, 1, 0, 5, 0, 27, 8, 0, 10, 0, 12, 0, 30, 9, 
		0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 3, 2, 40, 8, 2, 1, 
		3, 1, 3, 1, 3, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 5, 1, 5, 1, 6, 1, 6, 1, 
		6, 5, 6, 55, 8, 6, 10, 6, 12, 6, 58, 9, 6, 1, 7, 1, 7, 1, 7, 5, 7, 63, 
		8, 7, 10, 7, 12, 7, 66, 9, 7, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 
		3, 8, 75, 8, 8, 1, 8, 1, 8, 3, 8, 79, 8, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 
		8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 3, 8, 94, 8, 8, 1, 
		8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 3, 8, 103, 8, 8, 1, 8, 1, 8, 1, 
		8, 1, 8, 5, 8, 109, 8, 8, 10, 8, 12, 8, 112, 9, 8, 1, 9, 1, 9, 1, 10, 
		1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 
		3, 10, 127, 8, 10, 1, 11, 1, 11, 1, 11, 0, 1, 16, 12, 0, 2, 4, 6, 8, 10, 
		12, 14, 16, 18, 20, 22, 0, 2, 2, 0, 7, 7, 13, 51, 1, 0, 55, 59, 143, 0, 
		28, 1, 0, 0, 0, 2, 33, 1, 0, 0, 0, 4, 39, 1, 0, 0, 0, 6, 41, 1, 0, 0, 
		0, 8, 44, 1, 0, 0, 0, 10, 49, 1, 0, 0, 0, 12, 51, 1, 0, 0, 0, 14, 59, 
		1, 0, 0, 0, 16, 102, 1, 0, 0, 0, 18, 113, 1, 0, 0, 0, 20, 126, 1, 0, 0, 
		0, 22, 128, 1, 0, 0, 0, 24, 27, 3, 2, 1, 0, 25, 27, 3, 4, 2, 0, 26, 24, 
		1, 0, 0, 0, 26, 25, 1, 0, 0, 0, 27, 30, 1, 0, 0, 0, 28, 26, 1, 0, 0, 0, 
		28, 29, 1, 0, 0, 0, 29, 31, 1, 0, 0, 0, 30, 28, 1, 0, 0, 0, 31, 32, 5, 
		0, 0, 1, 32, 1, 1, 0, 0, 0, 33, 34, 5, 1, 0, 0, 34, 35, 3, 16, 8, 0, 35, 
		36, 5, 2, 0, 0, 36, 3, 1, 0, 0, 0, 37, 40, 3, 6, 3, 0, 38, 40, 3, 8, 4, 
		0, 39, 37, 1, 0, 0, 0, 39, 38, 1, 0, 0, 0, 40, 5, 1, 0, 0, 0, 41, 42, 
		3, 16, 8, 0, 42, 43, 5, 2, 0, 0, 43, 7, 1, 0, 0, 0, 44, 45, 3, 10, 5, 
		0, 45, 46, 5, 1, 0, 0, 46, 47, 3, 12, 6, 0, 47, 48, 5, 2, 0, 0, 48, 9, 
		1, 0, 0, 0, 49, 50, 3, 16, 8, 0, 50, 11, 1, 0, 0, 0, 51, 56, 3, 14, 7, 
		0, 52, 53, 5, 3, 0, 0, 53, 55, 3, 14, 7, 0, 54, 52, 1, 0, 0, 0, 55, 58, 
		1, 0, 0, 0, 56, 54, 1, 0, 0, 0, 56, 57, 1, 0, 0, 0, 57, 13, 1, 0, 0, 0, 
		58, 56, 1, 0, 0, 0, 59, 64, 3, 16, 8, 0, 60, 61, 5, 4, 0, 0, 61, 63, 3, 
		16, 8, 0, 62, 60, 1, 0, 0, 0, 63, 66, 1, 0, 0, 0, 64, 62, 1, 0, 0, 0, 
		64, 65, 1, 0, 0, 0, 65, 15, 1, 0, 0, 0, 66, 64, 1, 0, 0, 0, 67, 68, 6, 
		8, -1, 0, 68, 103, 5, 54, 0, 0, 69, 70, 5, 5, 0, 0, 70, 71, 3, 14, 7, 
		0, 71, 72, 5, 6, 0, 0, 72, 103, 1, 0, 0, 0, 73, 75, 5, 7, 0, 0, 74, 73, 
		1, 0, 0, 0, 74, 75, 1, 0, 0, 0, 75, 76, 1, 0, 0, 0, 76, 103, 3, 22, 11, 
		0, 77, 79, 5, 7, 0, 0, 78, 77, 1, 0, 0, 0, 78, 79, 1, 0, 0, 0, 79, 80, 
		1, 0, 0, 0, 80, 103, 5, 60, 0, 0, 81, 82, 3, 20, 10, 0, 82, 83, 5, 5, 
		0, 0, 83, 84, 3, 14, 7, 0, 84, 85, 5, 6, 0, 0, 85, 103, 1, 0, 0, 0, 86, 
		87, 3, 18, 9, 0, 87, 88, 3, 16, 8, 4, 88, 103, 1, 0, 0, 0, 89, 90, 5, 
		8, 0, 0, 90, 93, 3, 14, 7, 0, 91, 92, 5, 9, 0, 0, 92, 94, 3, 16, 8, 0, 
		93, 91, 1, 0, 0, 0, 93, 94, 1, 0, 0, 0, 94, 95, 1, 0, 0, 0, 95, 96, 5, 
		10, 0, 0, 96, 103, 1, 0, 0, 0, 97, 98, 5, 11, 0, 0, 98, 99, 3, 14, 7, 
		0, 99, 100, 5, 12, 0, 0, 100, 103, 1, 0, 0, 0, 101, 103, 3, 20, 10, 0, 
		102, 67, 1, 0, 0, 0, 102, 69, 1, 0, 0, 0, 102, 74, 1, 0, 0, 0, 102, 78, 
		1, 0, 0, 0, 102, 81, 1, 0, 0, 0, 102, 86, 1, 0, 0, 0, 102, 89, 1, 0, 0, 
		0, 102, 97, 1, 0, 0, 0, 102, 101, 1, 0, 0, 0, 103, 110, 1, 0, 0, 0, 104, 
		105, 10, 5, 0, 0, 105, 106, 3, 18, 9, 0, 106, 107, 3, 16, 8, 5, 107, 109, 
		1, 0, 0, 0, 108, 104, 1, 0, 0, 0, 109, 112, 1, 0, 0, 0, 110, 108, 1, 0, 
		0, 0, 110, 111, 1, 0, 0, 0, 111, 17, 1, 0, 0, 0, 112, 110, 1, 0, 0, 0, 
		113, 114, 7, 0, 0, 0, 114, 19, 1, 0, 0, 0, 115, 116, 5, 8, 0, 0, 116, 
		127, 5, 10, 0, 0, 117, 118, 5, 11, 0, 0, 118, 127, 5, 12, 0, 0, 119, 127, 
		5, 53, 0, 0, 120, 127, 5, 61, 0, 0, 121, 127, 5, 62, 0, 0, 122, 127, 5, 
		63, 0, 0, 123, 127, 5, 64, 0, 0, 124, 127, 5, 3, 0, 0, 125, 127, 5, 52, 
		0, 0, 126, 115, 1, 0, 0, 0, 126, 117, 1, 0, 0, 0, 126, 119, 1, 0, 0, 0, 
		126, 120, 1, 0, 0, 0, 126, 121, 1, 0, 0, 0, 126, 122, 1, 0, 0, 0, 126, 
		123, 1, 0, 0, 0, 126, 124, 1, 0, 0, 0, 126, 125, 1, 0, 0, 0, 127, 21, 
		1, 0, 0, 0, 128, 129, 7, 1, 0, 0, 129, 23, 1, 0, 0, 0, 11, 26, 28, 39, 
		56, 64, 74, 78, 93, 102, 110, 126
	];
}
