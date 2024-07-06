use crate::error::Result;
use ndarray::{ArrayBase, Data, DataOwned, Dimension, RawData, RemoveAxis};

pub trait ArrayBaseExt {
    fn to_value(&self) -> Result<serde_json::Value>;
}

impl<S, D> ArrayBaseExt for ArrayBase<S, D>
where
    S: RawData + Data + DataOwned,
    <S as RawData>::Elem: Clone,
    D: Dimension + RemoveAxis,
    serde_json::Value: From<<S as RawData>::Elem>,
{
    fn to_value(&self) -> Result<serde_json::Value> {
        match self.shape() {
            [] => Ok(serde_json::Value::Array(vec![])),

            [_] => Ok(serde_json::Value::Array(
                self.as_slice()
                    .unwrap()
                    .iter()
                    .map(|value| serde_json::Value::from(value.clone()))
                    .collect::<Vec<_>>(),
            )),

            [_, ..] => {
                let mut values = vec![];

                for subview in self.outer_iter() {
                    let arr: ArrayBase<S, _> = ArrayBase::from_shape_vec(
                        subview.shape(),
                        subview.as_slice().unwrap().to_vec(),
                    )?;
                    values.push(arr.to_value()?);
                }

                Ok(serde_json::Value::Array(values))
            }
        }
    }
}
