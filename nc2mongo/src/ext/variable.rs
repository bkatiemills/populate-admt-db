// crate in this context is a pointer to ourself,
// so error:: is the error.rs module in the same directory as lib.rs
use crate::error::{Error, Result};

pub trait VariableExt {
    fn try_into_json(&self) -> Result<serde_json::Value>; // &self is kind of like 'this', it'll refer to the object that's getting the trait implemented on it
}

impl<'a> VariableExt for netcdf::Variable<'a> { // "lifetime parameters" <'a> tbd, I don't get it
    fn try_into_json(&self) -> Result<serde_json::Value> {
        match self.vartype() {
            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Byte) => { // note there are two enums here, VariableType, one variant of which is Basic, which is itself an enum with a bunch of its own variants
                // get_values requires an argument of type Extents; we use the All variant to get everything
                // get_values is a generic function, so we need the turbofish to establish some concrete types:
                // i8 is the type of data we're going to pack into the Vec we return,
                // and _ is indicating that the type of the Extents is inferred, in this case from the argument to the function.
                // finally, the .into() is taking the resulting Vec<i8> and trying to coerce it into what we're actually supposed to return, which is a Result<serde_json::Value>
                // this works since serde_json::Value implements From<Vec<i8>>
                Ok(self.get_values::<i8, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Char) => {
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

                if strings.len() == 1 {
                    Ok(serde_json::Value::from(strings[0].clone()))
                } else {
                    Ok(serde_json::Value::from(strings))
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
                Ok(self.get_values::<f32, _>(netcdf::Extents::All)?.into())
            }

            netcdf::types::VariableType::Basic(netcdf::types::BasicType::Double) => {
                Ok(self.get_values::<f64, _>(netcdf::Extents::All)?.into())
            }

            // Only basic types supported
            _ => todo!(),
        }
    }
}
