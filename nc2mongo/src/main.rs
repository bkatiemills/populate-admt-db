use std::collections::HashSet;
use argo_data::VariableExt;
use std::env;
use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
use tokio;

#[tokio::main]
pub async fn main() -> argo_data::error::Result<()> {
    // mongodb setup ///////////////////////////////////////////
    // Load the MongoDB connection string from an environment variable:
    let client_uri =
       env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!"); 
    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
       ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
          .await?;
    let client = Client::with_options(options)?; 
    let argo = client.database("argo").collection::<mongodb::bson::Document>("argo");

    // get command line parameters
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let file = netcdf::open(file_path)?;
    println!("{}", file_path);

    // chew up that data
    let mut map = file
        .variables()
        .map(|variable| {
            variable
                .to_value()
                .map(|json_var| (variable.name(), json_var))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(serde_json::Map::from_iter)?;

    // append custom parameters
    map.insert("gdac_file".to_string(), serde_json::Value::from(file_path.clone().replace("/bulk/ifremer/", "ftp://ftp.ifremer.fr/ifremer/argo/dac/")));
    
    let _id = file_path.split('/').last().unwrap_or("").trim_end_matches(".nc");
    map.insert("_id".to_string(), serde_json::Value::from(_id.clone()));
    
    if let Some(serde_json::Value::Array(station_parameters)) = map.get("STATION_PARAMETERS") {
        let info_rows: HashSet<_> = station_parameters
            .iter()
            .filter_map(|value| value.as_array())
            .flatten()
            .filter_map(|value| value.as_str())
            .filter(|&s| !s.is_empty())
            .collect();
        let info_rows: Vec<_> = info_rows.into_iter().collect();
        map.insert("info_rows".to_string(), serde_json::Value::from(info_rows));
    }
    
    if let (Some(serde_json::Value::Array(latitude)), Some(serde_json::Value::Array(longitude))) = (map.get("LATITUDE"), map.get("LONGITUDE")) {
        if let (Some(serde_json::Value::Number(lat)), Some(serde_json::Value::Number(lon))) = (latitude.first(), longitude.first()) {
            if let (Some(lat), Some(mut lon)) = (lat.as_f64(), lon.as_f64()) {
                while lon > 180.0 {
                    lon -= 360.0;
                }
                while lon < -180.0 {
                    lon += 360.0;
                }
                let geojson_point = serde_json::json!({
                    "type": "Point",
                    "coordinates": [lon, lat]
                });
                map.insert("geolocation".to_string(), geojson_point);
            }
        }
    }
    map.insert("info_cols".to_string(), serde_json::Value::from(vec!["units"]));

    let mut data_info = Vec::new();
    if let Some(serde_json::Value::Array(info_rows)) = map.get("info_rows") {
    for value in info_rows {
        if let serde_json::Value::String(key) = value {
            if let Some(variable) = file.variable(&key) {
                if let Some(units) = variable.attribute("units") {
                    match units.value() {
                        Ok(netcdf::AttributeValue::Str(units_str)) => {
                            data_info.push(vec![units_str]);
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
    map.insert("data_info".to_string(), serde_json::Value::from(data_info));
    println!("{}", serde_json::to_string(&map)?);

    let filter = mongodb::bson::doc! { "_id": _id };
    let update = mongodb::bson::doc! { "$set": mongodb::bson::to_bson(&map)? };
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    argo.update_one(filter, update, options).await?;

    Ok(())
}
