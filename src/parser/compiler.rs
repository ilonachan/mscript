/// probably an older version of crate::interpreter::ast

use std::{collections::HashMap, ops::Deref};

use antlr_rust::{
    common_token_stream::CommonTokenStream,
    int_stream::IntStream,
    parser_rule_context::ParserRuleContext,
    token::{Token, TOKEN_EOF},
    token_stream::{TokenStream, UnbufferedTokenStream},
    tree::{ParseTree, ParseTreeVisitor, Visitable, Tree},
    InputStream, Lexer, rule_context::RuleContext,
};

use crate::interpreter::{mshlexer::{DEC_INT, HEX_INT, BIN_INT}, mshparser::{NumIntContext, NumFloatContext}};

use super::{
    mshlexer::{MshLexer, _SYMBOLIC_NAMES},
    mshparser::{ExecLineContext, MshParser, MshParserContextType, ExprContext, ListInitContext, MshParserContext},
    mshvisitor::MshVisitor,
};

static DATA: &str = "#!exec ./lmao.m\nconst test: int = 0x42; export test2 = -4.32e36\nlocal hi = true";

/// Currently each tree node is identified by its rule id and token interval.
/// The corresponding type is specified here for clarity, and can later be used more abstractly.
type RuleContextKey = (usize, isize, isize);
/**
A `HashMap` which uses the `antlr-rust` parse tree nodes as a key.
Because these nodes aren't assigned a unique ID, this map internally
uses the token range (which should hopefully be unique).
 */
pub struct TreeAttributeMap<V> {
    base: HashMap<RuleContextKey, V>,
}
impl<V> TreeAttributeMap<V> {
    pub fn new() -> Self {
        TreeAttributeMap {
            base: HashMap::<RuleContextKey, V>::new(),
        }
    }

    /// calculate the unique key for a parse tree node (currently uses the token range)
    fn key_for_ctx<'i, Ctx>(ctx: &Ctx) -> RuleContextKey where Ctx: ParseTree<'i> {
        let si = ctx.get_source_interval();
        (ctx.get_rule_index(), si.a, si.b)
    }

    /// calculate the unique key for a parse tree node (currently uses the token range)
    fn key_for_generic<'i, Ref>(ctx: Ref) -> RuleContextKey where Ref: Deref<Target = dyn MshParserContext<'i>>{
        let si = ctx.get_source_interval();
        (ctx.get_rule_index(), si.a, si.b)
    }

    /// get the value for a specified node context
    pub fn get<'i, Ctx>(&self, ctx: &Ctx) -> Option<&V> where Ctx: ParseTree<'i> {
        self.base.get(&Self::key_for_ctx(ctx))
    }
    /// get the value for a specified node context
    pub fn get_generic<'i, Ref>(&self, ctx: Ref) -> Option<&V> where Ref: Deref<Target = dyn MshParserContext<'i>> {
        self.base.get(&Self::key_for_generic(ctx))
    }
    /// get the value for a specified node context mutably
    pub fn get_mut<'i, Ctx>(&mut self, ctx: &Ctx) -> Option<&mut V> where Ctx: ParseTree<'i> {
        self.base.get_mut(&Self::key_for_ctx(ctx))
    }
    /// get the value for a specified node context
    pub fn get_mut_generic<'i, Ref>(&mut self, ctx: Ref) -> Option<&mut V> where Ref: Deref<Target = dyn MshParserContext<'i>> {
        self.base.get_mut(&Self::key_for_generic(ctx))
    }
    /// get the value for a specified node context mutably
    pub fn remove<'i, Ctx>(&mut self, ctx: &Ctx) -> Option<V> where Ctx: ParseTree<'i> {
        self.base.remove(&Self::key_for_ctx(ctx))
    }
    /// get the value for a specified node context
    pub fn remove_generic<'i, Ref>(&mut self, ctx: Ref) -> Option<V> where Ref: Deref<Target = dyn MshParserContext<'i>> {
        self.base.remove(&Self::key_for_generic(ctx))
    }

    /// set the value for a specified node context
    pub fn set<'i, Ctx>(&mut self, ctx: &Ctx, value: V)
    where
        Ctx: ParseTree<'i>,
    {
        self.base.insert(Self::key_for_ctx(ctx), value);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MshValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<MshValue>),
    Dict(HashMap<String, MshValue>),
    None,
}

pub struct VarScope {
    parent: Box<Option<Self>>,
    values: HashMap<String, MshValue>,
}
impl VarScope {
    pub fn new_global() -> Self {
        VarScope {
            parent: Box::new(None),
            values: HashMap::new(),
        }
    }
    pub fn new_local(parent: Self) -> Self {
        VarScope {
            parent: Box::new(Some(parent)),
            values: HashMap::new(),
        }
    }
}

pub struct InterpretingVisitor {
    expr_value: TreeAttributeMap<MshValue>,
    scopes: TreeAttributeMap<VarScope>,
    global_scope: VarScope,
}
impl InterpretingVisitor {
    fn new(global_scope: Option<VarScope>) -> Self {
        InterpretingVisitor {
            expr_value: TreeAttributeMap::new(),
            scopes: TreeAttributeMap::new(),
            global_scope: global_scope.unwrap_or_else(|| VarScope::new_global()),
        }
    }

    /* fn parse_float_literal(content: &str) -> f64 {
        let sign: i8 = if content.starts_with("-") {-1} else {1};
        let nosign = if content.starts_with("+") || content.starts_with("-") {&content[1..]} else {content};

        let mut mantisse = nosign;
        let mut exponent = "";

        let split_scientific = nosign.to_lowercase();
        if let Some((a,b)) = split_scientific.split_once("p") {
            mantisse = a;
            exponent = b;
        } else if !nosign.starts_with("0x") {
            if let Some((a,b)) = split_scientific.split_once("e") {
                mantisse = a;
                exponent = b;
            }
        }
        
        
        let exponent = if exponent == "" {0} else {
            let fact: i8 = if exponent.starts_with("-") {-1} else {1};
            let nosign = if exponent.starts_with("+") || exponent.starts_with("-")
                {exponent[1..].to_owned()} else {exponent};

            if nosign.starts_with("0x") {
                i64::from_str_radix(&nosign[2..], 16)
            } else if nosign.starts_with("0b") {
                i64::from_str_radix(&nosign[2..], 2)
            } else {
                i64::from_str_radix(&nosign, 10)
            }.expect("float: invalid exponent format") * fact as i64
        };
        
        let nosign = if mantisse.starts_with("+") || mantisse.starts_with("-")
            {mantisse[1..].to_owned()} else {mantisse};


        let content = ctx.get_text().replace("_", "");
        let val = match ctx.start().get_token_type() {
            DEC_FLOAT => i64::from_str_radix(&content, 10),
            HEX_FLOAT => i64::from_str_radix(&content[2..], 16),
            BIN_FLOAT => i64::from_str_radix(&content[2..], 2),
            _ => panic!("incorrect token type")
        }.expect("invalid float format");
    } */
}

impl<'input> ParseTreeVisitor<'input, MshParserContextType> for InterpretingVisitor {}

impl<'input> MshVisitor<'input> for InterpretingVisitor {
    fn visit_file(&mut self, ctx: &super::mshparser::FileContext<'input>) {
        self.visit_children(ctx);
        let exec_line_ctx = ctx.child_of_type::<ExecLineContext>(0).unwrap();
        if let Some(val) = self.expr_value.get(exec_line_ctx.as_ref()) {
            println!("value was correctly stored: {:?}", val);
        }
        if let None = self.expr_value.get(ctx) {
            println!("value was not present for file node (correct)");
        }
    }

    fn visit_execLine(&mut self, ctx: &ExecLineContext<'input>) {
        self.expr_value.set(ctx, MshValue::Int(42));
        println!("#!exec {:?}", ctx.get_text());
    }

    fn visit_listInit(&mut self, ctx: &super::mshparser::ListInitContext<'input>) {
        let mut values = Vec::new();
        for child in ctx.children_of_type::<ExprContext>() {
            values.push(self.expr_value.get(child.as_ref()).unwrap().clone());
        }
        self.expr_value.set(ctx, MshValue::List(values));
    }

    fn visit_list_entry(&mut self, ctx: &super::mshparser::List_entryContext<'input>) {
        let value = self.expr_value.get_generic(ctx.get_child(0).unwrap()).unwrap().clone();
        let list = match self.expr_value.get_mut_generic(ctx.get_parent_ctx().unwrap()).unwrap() {
            MshValue::List(list) => list,
            _ => panic!("this should not happen")
        };
        list.push(value);
    }

    fn visit_dictInit(&mut self, ctx: &super::mshparser::DictInitContext<'input>) {
        
    }

    fn visit_brackets(&mut self, ctx: &super::mshparser::BracketsContext<'input>) {
        let child = ctx.child_of_type::<ExprContext>(0).unwrap();
        child.accept(self);
        self.expr_value.set(ctx, self.expr_value.get(child.as_ref()).unwrap().clone());
    }

    fn visit_number(&mut self, ctx: &super::mshparser::NumberContext<'input>) {
        self.visit_children(ctx);
        if let Some(child) = ctx.child_of_type::<NumIntContext>(0) {
            self.expr_value.set(ctx, self.expr_value.get(child.as_ref()).unwrap().clone());
        }
        if let Some(child) = ctx.child_of_type::<NumFloatContext>(0) {
            self.expr_value.set(ctx, self.expr_value.get(child.as_ref()).unwrap().clone());
        }
    }
    fn visit_numInt(&mut self, ctx: &super::mshparser::NumIntContext<'input>) {
        let content = ctx.get_text().replace("_", "");

        let val = match ctx.start().get_token_type() {
            DEC_INT => i64::from_str_radix(&content, 10),
            HEX_INT => i64::from_str_radix(&content[2..], 16),
            BIN_INT => i64::from_str_radix(&content[2..], 2),
            _ => panic!("incorrect token type")
        }.expect("invalid int format");
        self.expr_value.set(ctx, MshValue::Int(val))
    }
    fn visit_numFloat(&mut self, ctx: &super::mshparser::NumFloatContext<'input>) {
        self.expr_value.set(ctx, MshValue::Float(ctx.get_text().parse::<f64>().expect("invalid float format")));
    }
    fn visit_bool(&mut self, ctx: &super::mshparser::BoolContext<'input>) {
        self.expr_value.set(ctx, MshValue::Bool(ctx.get_text() == "true"));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_lexer() {
        let mut lexer = MshLexer::new(InputStream::new(DATA.into()));
        // let token_src = CommonTokenStream::new(lexer);
    
        let mut string = String::new();
        {
            let mut token_source = UnbufferedTokenStream::new_unbuffered(&mut lexer);
            while token_source.la(1) != TOKEN_EOF {
                {
                    let token = token_source.lt(1).unwrap();
    
                    let len = token.get_stop() as usize + 1 - token.get_start() as usize;
                    string.extend(
                        format!(
                            "{},len {}:\n{}\n",
                            _SYMBOLIC_NAMES[token.get_token_type() as usize]
                                .unwrap_or(&format!("{}", token.get_token_type())),
                            len,
                            String::from_iter(DATA.chars().skip(token.get_start() as usize).take(len))
                        )
                        .chars(),
                    );
                }
                token_source.consume();
            }
        }
        println!("{}", string);
        println!(
            "{}",
            lexer
                .get_interpreter()
                .unwrap()
                .get_dfa()
                .read()
                .to_lexer_string()
        );
    }

    #[test]
    pub fn test_parser() {
        let lexer = MshLexer::new(InputStream::new(DATA.into()));
        let token_src = CommonTokenStream::new(lexer);
    
        let mut parser = MshParser::new(token_src);
        let result = parser.file().expect("parse unsuccessful");
    
        let mut visitor = InterpretingVisitor::new(None);
        result.accept(&mut visitor);
    }
}