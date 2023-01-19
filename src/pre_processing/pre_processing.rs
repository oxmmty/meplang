use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bytes::{Bytes, BytesMut};

use crate::ast::attribute::WithAttributes;
use crate::ast::block::{RBlock};
use crate::ast::constant::RConstant;
use crate::ast::contract::RContract;
use crate::parser::error::{new_error_from_located, new_generic_error};
use crate::{parser::parser::Located, ast::file::RFile};
use crate::parser::parser::Rule;

use super::attribute::Attribute;
use super::queue::DedupQueue;

#[derive(Clone, Default, Debug)]
pub struct Contract {
    pub dependencies: HashSet<usize>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Default, Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Clone, Debug)]
pub struct BlockItem {
    pub assumes: HashMap<Bytes, u8>,
    pub content: BlockItemContent,
}

#[derive(Clone, Debug)]
pub enum BlockItemContent {
    Bytes(Bytes),
    Contract(usize),
    Block(usize),
    Push(Push),
}

#[derive(Clone, Debug)]
pub enum Push {
    Constant(Bytes),
    ContractPc(usize),
    ContractSize(usize),
    BlockSize(usize),
    BlockPc(usize),
}

pub fn pre_process(
    input: &str,
    file: RFile,
    contract_name: String,
) -> Result<Vec<Contract>, pest::error::Error<Rule>> {
    let mut main_index: Option<usize> = None;
    let mut contract_names = HashMap::<String, usize>::new();
    let mut contract_attributes = Vec::<Vec<Attribute>>::with_capacity(file.0.len());

    for r_contract_with_attr in &file.0 {
        contract_attributes.push(Vec::with_capacity(r_contract_with_attr.attributes.len()));
        for r_attribute in &r_contract_with_attr.attributes {
            let attribute = Attribute::from_r_attribute(input, r_attribute)?;
            if attribute.is_contract_attribute() {
                contract_attributes.last_mut().unwrap().push(
                    Attribute::from_r_attribute(input, r_attribute)?
                );
            } else {
                return Err(new_error_from_located(input, r_attribute, "Invalid contract argument"));
            }
        }

        let r_contract = &r_contract_with_attr.inner.inner;
        let name = r_contract.name_str();
        if contract_names.insert(name.to_owned(), contract_names.len()).is_some() {
            return Err(new_error_from_located(
                input,
                &r_contract.name,
                &format!("Name `{}` already used", name)
            ));
        }
        if name == &contract_name {
            main_index = Some(contract_names.len() - 1);
        }
    }
    dbg!(&contract_attributes);

    let Some(main_index) = main_index else {
        return Err(new_generic_error(
            format!("Contract `{}` not found", contract_name)
        ));
    };

    let mut contracts = HashMap::<usize, Contract>::new();
    let mut contracts_queue = DedupQueue::<usize>::default();
    contracts_queue.insert_if_needed(main_index);

    while let Some(index_to_process) = contracts_queue.pop() {
        let contract = pre_process_contract(
            input,
            &file.0[index_to_process],
            &contract_names,
        )?;

        for dependency in &contract.dependencies {
            contracts_queue.insert_if_needed(*dependency);
        }

        contracts.insert(
            index_to_process,
            contract,
        );
    }

    // for index in 0..file.0.len() {
    //     if contract_remapping_queue.remapping(&index).is_none() {
    //         log::warn!("{}", new_error_from_located(
    //             code,
    //             file.0[index].inner_located(),
    //             &format!("Unused contract `{}`", file.0[index].inner().name())
    //         ));
    //     }
    // }

    Ok(contracts.into_iter().map(|(_, c)| c).collect())
}

pub fn extract_constants(
    input: &str,
    r_constants: &Vec<Located<RConstant>>,
    contract_names: &HashMap<String, usize>,
) -> Result<HashMap<String, Bytes>, pest::error::Error<Rule>> {
    let mut constants = HashMap::<String, Bytes>::new();

    for r_constant in r_constants {
        let constant_name = r_constant.name_str();
        let value = r_constant.value.inner.clone().0;
        if value.len() >= 32 {
            return Err(new_error_from_located(
                input,
                &r_constant.value,
                &format!("Constants cannot exceed 32 bytes")
            ));
        }
        if 
            contract_names.contains_key(constant_name) ||
            constants.insert(constant_name.to_owned(), value.clone()).is_some()
        {
            return Err(new_error_from_located(
                input,
                &r_constant.name,
                &format!("Name {} already used", r_constant.name.0)
            ));
        }
    }
    Ok(constants)
}

pub fn pre_process_contract(
    code: &str,
    r_contract_with_attr: &Located<WithAttributes<RContract>>,
    contract_names: &HashMap<String, usize>,
) -> Result<Contract, pest::error::Error<Rule>> {
    let r_contract = &r_contract_with_attr.inner.inner;

    let constants = extract_constants(
        code,
        &r_contract.constants,
        contract_names,
    )?;

    let mut main_index: Option<usize> = None;
    let mut block_names = HashMap::<String, usize>::new();

    for r_block_with_attr in &r_contract.blocks {
        let r_block = r_block_with_attr.inner_located();
        let name = r_block.inner.name_str();
        if 
            contract_names.contains_key(name)||
            constants.contains_key(name) ||
            block_names.insert(name.to_owned(), block_names.len()).is_some()
        {
            return Err(new_error_from_located(
                code,
                &r_contract.name,
                &format!("Name `{}` already used", name)
            ));
        }
        if name == "main" {
            main_index = Some(block_names.len() - 1);
        }
    }

    let Some(main_index) = main_index else {
        return Err(new_error_from_located(
            code,
            &r_contract,
            &format!("Block `main` not found in contract `{}`", r_contract.name_str())
        ));
    };

    let mut blocks = Vec::<Block>::new();
    let mut dependencies = HashSet::<usize>::new();

    let mut blocks_queue = DedupQueue::<usize>::default();
    blocks_queue.insert_if_needed(main_index);

    while let Some(index_to_process) = blocks_queue.pop() {
        // let block = pre_process_block(
        //     code,
        //     &r_contract.blocks[index_to_process],
        //     &mut blocks_queue,
        //     &constants,
        //     contract_names,
        //     blocks_queue,
        // )?;

        // blocks.insert(index_to_process, block);
    }

    // for index in 0..r_contract.blocks.len() {
    //     if blocks_queue.remapping(&index).is_none() {
    //         log::warn!("{}", new_error_from_located(
    //             code,
    //             r_contract.blocks[index].inner_located(),
    //             &format!("Unused contract `{}`", r_contract.blocks[index].inner().name_str())
    //         ));
    //     }
    // }

    Ok(Contract { blocks, dependencies })
}

// pub fn pre_process_block(
//     code: &str,
//     r_block_with_attr: &Located<WithAttributes<RBlock>>,
//     blocks_remapping_queue: &mut RemappingQueue<usize>,
//     constants: &HashMap<String, Bytes>,
//     contract_names: &HashMap<String, usize>,
//     contract_remapping_queue: &mut RemappingQueue<usize>,
// ) -> Result<Block, pest::error::Error<Rule>> {
//     let r_block = r_block_with_attr.inner();

//     let mut items = Vec::<BlockItem>::new();
//     let mut current_bytes: Option<BytesMut> = None;
//     for line in &r_block.lines {
//         match &line.inner {
//             RBlockLine::Var(var) => {
//                 let name = var.name();
//                 if let Some(bytes) = constants.get(name) {
//                     if let Some(c_bytes) = current_bytes.as_mut() {
//                         c_bytes.extend_from_slice(bytes);
//                     } else {
//                         current_bytes = Some(bytes[..].into());
//                     }
//                 } else {
//                     if let Some(bytes) = current_bytes.take() {
//                         items.push(BlockItem {
//                             assumes: HashMap::new(),
//                             content: BlockItemContent::Bytes(bytes.into()),
//                         })
//                     }
//                     if let Some(contract_old_index) = contract_names.get(name) {
//                         contract_remapping_queue.insert_if_needed(*contract_old_index);
//                         let contract_new_index = contract_remapping_queue.remapping(contract_old_index).unwrap();
//                         items.push(BlockItem {
//                             assumes: HashMap::new(),
//                             content: BlockItemContent::Contract(contract_new_index),
//                         })
//                     }
//                 }
//             },
//             RBlockLine::Function(_) => (),
//             RBlockLine::Bytes(_) => (),
//         }
//     }

//     Ok(Block { items })
// }

