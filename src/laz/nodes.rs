use crate::laz::types::LazValue;
use std::borrow::Cow;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone, Copy, Hash, Debug, Default, PartialEq, Eq)]
pub struct ID(pub u64);

#[derive(Clone, Copy, Hash, Debug)]
pub struct OutputID { pub node: ID, pub outport: usize }

#[derive(Clone, Copy, Hash, Debug)]
pub struct InputID { pub node: ID, pub inport: usize }

#[derive(Clone, Debug)]
pub enum LazError {
    InvalidInputType { from: OutputID, expected: String },
    NoSuchNode(ID),
    NoSuchOutport(OutputID),
    Other(String),
}

pub struct NodeInputs<'a> {
    inputs: Vec<Cow<'a, OutputID>>,
}

pub struct IODescription {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

/// Invariant: NodeInputs.inputs.len() == IODescription.inputs.len()
pub trait LazNode {
    fn inputs<'a>(&'a self) -> Vec<&'a OutputID>;
    fn inputs_muts<'a>(&'a mut self) -> Vec<&'a mut OutputID>;

    fn io_description(&self) -> IODescription;

    fn evaluate_for(&mut self, inputs: Vec<(OutputID, LazValue)>) -> Result<Vec<LazValue>, LazError>;
}

pub struct ConstantNode {
    pub value: LazValue,
}

impl LazNode for ConstantNode {
    fn inputs<'a>(&'a self) -> Vec<&'a OutputID> {
        vec![]
    }
    fn inputs_muts<'a>(&'a mut self) -> Vec<&'a mut OutputID> {
        vec![]
    }
    fn io_description(&self) -> IODescription {
        IODescription {
            inputs: vec![],
            outputs: vec![ "Value".into() ],
        }
    }

    fn evaluate_for(&mut self, inputs: Vec<(OutputID, LazValue)>) -> Result<Vec<LazValue>, LazError> {
        assert!(inputs.is_empty());

        Ok(vec![self.value.clone()])
    }
}

pub struct ReadFileNode {
    pub file_name: OutputID,
    file_cache: Option<(PathBuf, Box<[u8]>)>,
}

impl ReadFileNode {
    pub fn new(file_name: OutputID) -> ReadFileNode {
        ReadFileNode {
            file_name,
            file_cache: None,
        }
    }
}

impl LazNode for ReadFileNode {
    fn inputs<'a>(&'a self) -> Vec<&'a OutputID> {
        vec![ &self.file_name ]
    }
    fn inputs_muts<'a>(&'a mut self) -> Vec<&'a mut OutputID> {
        vec![ &mut self.file_name ]
    }
    fn io_description(&self) -> IODescription {
        IODescription {
            inputs: vec![ "File name <string>".into() ],
            outputs: vec![ "Contents [byte]".into() ],
        }
    }

    fn evaluate_for(&mut self, inputs: Vec<(OutputID, LazValue)>) -> Result<Vec<LazValue>, LazError> {
        assert_eq!(inputs.len(), 1);

        let path = if let LazValue::String(ref path) = inputs[0].1 {
            path
        } else {
            Err(LazError::InvalidInputType { from: inputs[0].0, expected: "String".into() })?
        };

        let path = PathBuf::from(path);
        if let Some((cache_path, cont)) = &self.file_cache {
            if &path == cache_path {
                return Ok(vec![LazValue::Array(cont.into_iter().map(|&x| LazValue::Byte(x)).collect())]);
            }
        }

        let mut f = std::fs::File::open(path.clone()).map_err(|_| LazError::Other("couldn't open file :(:( :(".into()))?;

        let mut content = Vec::new();
        f.read_to_end(&mut content).map_err(|_| LazError::Other("when the file reading is sus".into()))?;
        self.file_cache = Some((path, content.clone().into_boxed_slice()));
        Ok(vec![LazValue::Array(content.into_iter().map(|x| LazValue::Byte(x)).collect())])
    }
}

pub struct SumNode {
    pub input_list: OutputID,
}

impl LazNode for SumNode {
    fn inputs<'a>(&'a self) -> Vec<&'a OutputID> {
        vec![ &self.input_list ]
    }
    fn inputs_muts<'a>(&'a mut self) -> Vec<&'a mut OutputID> {
        vec![ &mut self.input_list ]
    }
    fn io_description(&self) -> IODescription {
        IODescription {
            inputs: vec![ "List of numbers [{num}]".into() ],
            outputs: vec![ "Sum {num}".into() ],
        }
    }

    fn evaluate_for(&mut self, inputs: Vec<(OutputID, LazValue)>) -> Result<Vec<LazValue>, LazError> {
        assert_eq!(inputs.len(), 1);

        let data = if let LazValue::Array(ref data) = inputs[0].1 {
            data
        } else {
            Err(LazError::InvalidInputType { from: inputs[0].0, expected: "[{num}]".into() })?
        };

        let sum_bytes: Option<u8> =
                data.iter()
                .map(|x| if let LazValue::Byte(b) = x { Some(b) } else { None } )
                .fold(Some(0u8), |acc, b| Some(acc?.overflowing_add(*b?).0));
        if let Some(sum) = sum_bytes {
            return Ok(vec![LazValue::Byte(sum)]);
        }

        let sum_unsigned: Option<u64> =
                data.iter()
                .map(|x| if let LazValue::Unsigned(b) = x { Some(b) } else { None } )
                .fold(Some(0), |acc, b| Some(acc? + b?));
        if let Some(sum) = sum_unsigned {
            return Ok(vec![LazValue::Unsigned(sum)]);
        }

        let sum_signed: Option<i64> =
                data.iter()
                .map(|x| if let LazValue::Signed(b) = x { Some(b) } else { None } )
                .fold(Some(0), |acc, b| Some(acc? + b?));
        if let Some(sum) = sum_signed {
            return Ok(vec![LazValue::Signed(sum)]);
        }

        Err(LazError::InvalidInputType { from: inputs[0].0, expected: "Could not sum".into() })
    }
}
