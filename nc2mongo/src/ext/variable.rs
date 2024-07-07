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
                let span = self
                    .dimensions()
                    .iter()
                    .map(netcdf::Dimension::len)
                    .reduce(std::ops::Mul::mul)
                    .unwrap_or(0);

                let mut buffer = vec![0u8; span];

                self.get_raw_values(&mut buffer, dimensions)?;

                match self.dimensions() {
                    [] => Ok(serde_json::Value::Array(vec![])),
                    [rest @ .., tail] => if tail.name().starts_with("STRING") || tail.name() == "DATE_TIME" {
                        let raw_values = buffer
                            .chunks(tail.len())
                            .map(|data| {
                                serde_json::Value::String(unsafe {
                                    std::string::String::from_utf8_unchecked(data.to_vec())
                                        .trim()
                                        .to_string()
                                })
                            })
                            .collect::<Vec<_>>();

                        ndarray::Array::from_shape_vec(
                            rest.iter().map(|dim| dim.len()).collect::<Vec<_>>(),
                            raw_values,
                        )
                    } else {
                        ndarray::Array::from_shape_vec(
                            self.dimensions()
                                .iter()
                                .map(|dim| dim.len())
                                .collect::<Vec<_>>(),
                            buffer
                                .iter()
                                .map(|c| serde_json::Value::String((*c as char).to_string()))
                                .collect(),
                        )
                    }?
                    .to_value(),
                }
            }

            _ => Ok(serde_json::Value::Null),
        }
    }
}
