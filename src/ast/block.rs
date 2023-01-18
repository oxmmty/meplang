use bytes::Bytes;
use pest::iterators::Pair;
use crate::{ast::attribute::RAttribute, parser::parser::FromPair};
use crate::parser::parser::{Rule, map_unique_child, get_next, Located};

use super::contract::VarName;

#[derive(Debug, Clone)]
pub struct RBlock {
    name: Located<VarName>,
    lines: Vec<Located<BlockLine>>,
}

#[derive(Debug, Clone)]
pub enum BlockLine {
    Var(String),
    Function(Function),
    Bytes(Bytes),
}

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    arg: Argument,
}

#[derive(Debug, Clone)]
pub enum Argument {
    Var(VariableField),
    Bytes(Bytes),
}

#[derive(Debug, Clone)]
pub struct VariableField {
    pub variable: String,
    pub field: String,
}

impl FromPair for RBlock {
    fn from_pair(block_decl: pest::iterators::Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(block_decl.as_rule() == Rule::block_decl);

        let mut block_decl_inner = block_decl.into_inner();

        _ = get_next(&mut block_decl_inner, Rule::block_keyword);

        let name = Located::<VarName>::from_pair(
            get_next(&mut block_decl_inner, Rule::var_name)
        )?;

        let mut lines = Vec::<Located<BlockLine>>::new();
        while let Some(block_content) = block_decl_inner.next() {
            assert!(block_content.as_rule() == Rule::block_content);
            lines.push(
                map_unique_child(block_content, |block_line| {
                    assert!(block_line.as_rule() == Rule::block_line);
                    Located::<BlockLine>::from_pair(block_line)
                })?
            );
        }

        Ok(RBlock { name, lines })
    }
}

impl FromPair for BlockLine {
    fn from_pair(block_line: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(block_line.as_rule() == Rule::block_line);

        map_unique_child(block_line, |child| {
            match child.as_rule() {
                Rule::hex_litteral => Ok(BlockLine::Bytes(Bytes::from_pair(child)?)),
                Rule::function => Ok(BlockLine::Function(Function::from_pair(child)?)),
                Rule::var_name => Ok(BlockLine::Var(child.as_str().to_owned())),
                _ => unreachable!(),
            }
        })
    }
}

impl FromPair for Function {
    fn from_pair(function: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(function.as_rule() == Rule::function);

        let mut function_inner = function.into_inner();

        let name = get_next(&mut function_inner, Rule::var_name).as_str().to_owned();

        let arg = function_inner.next().unwrap();
        let arg = match arg.as_rule() {
            Rule::hex_litteral => Argument::Bytes(Bytes::from_pair(arg)?),
            Rule::var_field => Argument::Var(VariableField::from_pair(arg)?),
            _ => unreachable!(),
        };

        assert!(function_inner.next() == None);

        Ok(Function { name, arg })

    }
}

impl FromPair for VariableField {
    fn from_pair(pair: Pair<Rule>) -> Result<Self, pest::error::Error<Rule>> {
        assert!(pair.as_rule() == Rule::var_field);
        let mut pair_inner = pair.into_inner();

        let variable = get_next(&mut pair_inner, Rule::var_name).as_str().to_owned();
        _ = get_next(&mut pair_inner, Rule::dot);
        let field = get_next(&mut pair_inner, Rule::var_name).as_str().to_owned();
        assert!(pair_inner.next() == None);

        Ok(VariableField { variable, field })
    }
}
