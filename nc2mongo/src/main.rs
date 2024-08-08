use argo_data::VariableExt;

pub fn main() -> argo_data::error::Result<()> {
    let file = netcdf::open("/home/zellio/repos/bkatiemills/ifremer/aoml/1900167/1900167_prof.nc")?;

    let map = file
        .variables()
        .map(|variable| {
            variable
                .try_into_json()
                .map(|json_var| (variable.name(), json_var))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(serde_json::Map::from_iter)?;

    println!("{}", serde_json::to_string(&map)?);

    Ok(())
}
