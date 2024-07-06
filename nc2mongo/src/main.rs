use argo_data::VariableExt;
use std::env;

pub fn main() -> argo_data::error::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let file = netcdf::open(file_path)?;

    let map = file
        .variables()
        .map(|variable| {
            variable
                .to_value()
                .map(|json_var| (variable.name(), json_var))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(serde_json::Map::from_iter)?;

    println!("{}", serde_json::to_string(&map)?);

    Ok(())
}
