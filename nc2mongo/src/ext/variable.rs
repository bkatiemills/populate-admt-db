use netcdf::{Extent, Extents};

use crate::error::Result;
use crate::ext::ArrayBaseExt;

pub trait VariableExt {
    fn to_value(&self) -> Result<serde_json::Value>;
}

impl<'a> VariableExt for netcdf::Variable<'a> {
    fn to_value(&self) -> Result<serde_json::Value> {
        let dimensions = Extents::from(
            self.dimensions()
                .iter()
                .map(|dim| Extent::from(..dim.len()))
                .collect::<Vec<_>>(),
        );

        use netcdf::types::{BasicType::*, VariableType::*};
        use std::string::String;

        match self.vartype() {
            Basic(Ubyte) => self.get::<u8, _>(dimensions)?.to_value(),
            Basic(Ushort) => self.get::<u16, _>(dimensions)?.to_value(),
            Basic(Uint) => self.get::<u32, _>(dimensions)?.to_value(),
            Basic(Uint64) => self.get::<u64, _>(dimensions)?.to_value(),
            Basic(Byte) => self.get::<i8, _>(dimensions)?.to_value(),
            Basic(Short) => self.get::<i16, _>(dimensions)?.to_value(),
            Basic(Int) => self.get::<i32, _>(dimensions)?.to_value(),
            Basic(Int64) => self.get::<i64, _>(dimensions)?.to_value(),
            Basic(Float) => self.get::<f32, _>(dimensions)?.to_value(),
            Basic(Double) => self.get::<f64, _>(dimensions)?.to_value(),
            Basic(Char) => {
                let shape = self
                    .dimensions()
                    .iter()
                    .map(|dim| dim.len())
                    .collect::<Vec<_>>();

                let span = self
                    .dimensions()
                    .iter()
                    .map(netcdf::Dimension::len)
                    .reduce(std::ops::Mul::mul)
                    .unwrap_or(0);

                let mut buffer = vec![0u8; span];

                self.get_raw_values(&mut buffer, dimensions)?;

                let dontsplit = vec!["STRING2", "STRING4", "STRING8", "STRING16", "STRING32", "STRING64", "STRING256", "DATE_TIME"];
                if self.dimensions().len() == 1 && dontsplit.contains(&self.dimensions()[0].name().as_str()) {
                    let y: String = buffer.iter().map(|&c| c as char).collect::<String>().trim().to_owned();
                    Ok(serde_json::Value::from(y))
                } else if  self.dimensions().len() == 1 {
                    let y: Vec<String> = buffer.iter().map(|&c| (c as char).to_string()).collect();
                    Ok(serde_json::Value::from(y))
                } else if self.dimensions().len() == 2 {
                    let y: Vec<String> = buffer.chunks(shape[shape.len() - 1]).map(|chunk| chunk.iter().map(|&c| c as char).collect::<String>().trim().to_owned()).collect();
                    Ok(serde_json::Value::from(y))
                } else if self.dimensions().len() == 3 {
                    let y: Vec<Vec<String>> = buffer.chunks(shape[shape.len() - 1] * shape[shape.len() - 2]).map(|chunk| chunk.chunks(shape[shape.len() - 1]).map(|subchunk| subchunk.iter().map(|&c| c as char).collect::<String>().trim().to_owned()).collect()).collect();
                    Ok(serde_json::Value::from(y))
                } else {
                    let y: Vec<Vec<Vec<String>>> = buffer.chunks(shape[shape.len() - 1] * shape[shape.len() - 2] * shape[shape.len() - 3]).map(|chunk| chunk.chunks(shape[shape.len() - 1] * shape[shape.len() - 2]).map(|subchunk| subchunk.chunks(shape[shape.len() - 1]).map(|subsubchunk| subsubchunk.iter().map(|&c| c as char).collect::<String>().trim().to_owned()).collect()).collect()).collect();
                    Ok(serde_json::Value::from(y))
                }
            }

            _ => Ok(serde_json::Value::Null),
        }
    }
}
