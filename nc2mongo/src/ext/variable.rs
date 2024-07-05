use crate::error::{Error, Result};

pub trait VariableExt {
    fn try_into_json(&self) -> Result<serde_json::Value>;
}

impl<'a> VariableExt for netcdf::Variable<'a> {
    fn try_into_json(&self) -> Result<serde_json::Value> {
        match self.vartype() {
            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Byte) => {
                Ok(self.get_values::<i8, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Char) => {
               
                let dimension_sizes: Vec<_> = self
                    .dimensions()
                    .iter()
                    .map(|d| d.len())
                    .collect();
                
                let final_dimension = self
                    .dimensions()
                    .iter()
                    .last()
                    .ok_or(Error::NoNetCDFDimensions)?;

                let var_len = final_dimension.len();

                let mut buffer = vec![0u8; self.len()];

                self.get_raw_values(&mut buffer, netcdf::Extents::All)
                    .unwrap();

                let mut strings = buffer
                    .chunks(var_len)
                    .map(|s| String::from_utf8(s.into()).unwrap())
                    .collect::<Vec<_>>();

                for string in strings.iter_mut() {
                    *string = string.trim().to_string();
                }

                let mut xx = Vec::new();
                if dimension_sizes.len() > 2 {
                    xx = strings
                        .chunks(dimension_sizes[dimension_sizes.len() - 2])
                        .map(|s| s.to_vec())
                        .collect::<Vec<_>>();
                }

                let mut yy = Vec::new();
                if dimension_sizes.len() > 3 {
                    yy = xx
                        .chunks(dimension_sizes[dimension_sizes.len() - 3])
                        .map(|s| s.to_vec())
                        .collect::<Vec<_>>();
                }

                if dimension_sizes.len() < 3 {
                    if dimension_sizes.len() == 1 {
                        if self.dimensions()[0].name() == "N_PROF"  {
                            let chars: Vec<String> = strings[0].chars().map(|c| c.to_string()).collect();
                            Ok(serde_json::Value::from(chars))
                        } else { 
                            Ok(serde_json::Value::from(strings[0].clone()))
                        }
                    } else if dimension_sizes.len() == 2 && self.dimensions()[0].name() == "N_PROF" && self.dimensions()[1].name() == "N_LEVELS"{
                        let mut zz = Vec::new();
                        for string in strings.iter_mut() {
                            let chars: Vec<String> = string.chars().map(|c| c.to_string()).collect();
                            zz.push(chars);
                        }
                        Ok(serde_json::Value::from(zz))
                    } else {
                        Ok(serde_json::Value::from(strings))
                    }
                } else if dimension_sizes.len() == 3 {
                    Ok(serde_json::Value::from(xx))
                } else {
                    Ok(serde_json::Value::from(yy))
                }
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Ubyte) => {
                Ok(self.get_values::<u8, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Short) => {
                Ok(self.get_values::<i16, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Ushort) => {
                Ok(self.get_values::<u16, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Int) => {
                Ok(self.get_values::<i32, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Uint) => {
                Ok(self.get_values::<u32, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Int64) => {
                Ok(self.get_values::<i64, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Uint64) => {
                Ok(self.get_values::<u64, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Float) => {
                //Ok(self.get_values::<f32, _>(netcdf::Extents::All)?.into())
                let values = self.get_values::<f32, _>(netcdf::Extents::All)?;

                let dimension_sizes: Vec<_> = self
                    .dimensions()
                    .iter()
                    .map(|d| d.len())
                    .collect();

                let mut xx = Vec::new();
                if dimension_sizes.len() == 2 {
                    xx = values
                        .chunks(dimension_sizes[dimension_sizes.len() - 1])
                        .map(|s| s.to_vec())
                        .collect::<Vec<_>>();
                }

                if dimension_sizes.len() == 1 {
                    Ok(values.into())
                } else {
                    Ok(serde_json::Value::from(xx))
                } 
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Double) => {
                Ok(self.get_values::<f64, _>(netcdf::Extents::All)?.into())
            }

            // Only basic types supported
            _ => todo!(),
        }
    }
}
