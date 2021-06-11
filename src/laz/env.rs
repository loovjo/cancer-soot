use std::collections::HashMap;

use crate::laz::types::LazValue;
use crate::laz::nodes::{LazNode, ID, OutputID, InputID, LazError};

#[derive(Default)]
pub struct LazEnv {
    nodes: HashMap<ID, Box<dyn LazNode>>,
    connections: HashMap<OutputID, InputID>,

    smallest_unused_id: ID, // Invariant: !self.nodes.contains(self.smallest_unused_id)

    selected: Option<ID>,
}

impl LazEnv {
    pub fn add_node(&mut self, node: Box<dyn LazNode>) -> ID {
        let id = self.smallest_unused_id.clone();
        self.nodes.insert(id, node);
        self.smallest_unused_id.0 += 1;

        id
    }

    pub fn get_node(&self, id: ID) -> Option<&dyn LazNode> {
        self.nodes.get(&id).map(|x| x.as_ref())
    }

    pub fn get_node_mut(&mut self, id: ID) -> Option<&mut (dyn LazNode + 'static)> {
        self.nodes.get_mut(&id).map(|x| x.as_mut())
    }

    fn inputs_for(&self, id: ID) -> Result<Vec<OutputID>, LazError> {
        Ok(
            self.nodes.get(&id).ok_or(LazError::NoSuchNode(id))?.inputs()
            .into_iter().cloned().collect::<Vec<_>>()
        )
    }

    pub fn evaluate_node(&mut self, id: ID) -> Result<Vec<LazValue>, LazError> {
        let input_refs = self.nodes.get(&id).ok_or(LazError::NoSuchNode(id))?.inputs();
        // We need to clone each element because the recursive evaluate_node call might modify this
        // node's input refs
        let input_ids = input_refs.into_iter().cloned().collect::<Vec<_>>();

        let mut values = Vec::with_capacity(input_ids.len());

        for input_id in input_ids {
            let outputs: Vec<LazValue> = self.evaluate_node(input_id.node)?;
            let value = outputs.into_iter().nth(input_id.outport).ok_or(LazError::NoSuchOutport(input_id))?;
            values.push((input_id, value));
        }

        let node = self.nodes.get_mut(&id).ok_or(LazError::NoSuchNode(id))?;
        node.evaluate_for(values)
    }
}
