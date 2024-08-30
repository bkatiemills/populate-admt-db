#![allow(nonstandard_style)]
use netcdf;
use tokio;
use std::error::Error;
use std::env;
use mongodb::bson::{doc};
use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// helper functions ///////////////////////////////////////////

fn trim_null_bytes(input: String) -> String {
    input.trim().trim_end_matches('\0').to_string()
}

fn unpack_string(name: &str, buflen: usize, extents: netcdf::Extents, file: &netcdf::File) -> String {
    let mut dump = vec![0_u8; buflen];
    if let Some(variable) = file.variable(name) {
        if let Ok(_) = variable.get_raw_values(&mut dump, extents) {
            if let Ok(string) = String::from_utf8(dump) {
                return trim_null_bytes(string);
            }
        }
    }
    String::new()
}

fn unpack_string_array(name: &str, buflen: usize, arraydim: usize, extents: netcdf::Extents, file: &netcdf::File) -> Vec<String> {
    let mut dump = vec![0_u8; buflen * arraydim];
    if let Some(variable) = file.variable(name) {
        if let Ok(_) = variable.get_raw_values(&mut dump, extents) {
            let strings: Vec<String> = dump
                .chunks_exact(buflen)
                .map(|chunk| {
                    let string: String = String::from_utf8_lossy(chunk).into_owned().parse().unwrap_or_default();
                    string.trim().to_string(); // Strip leading and trailing whitespace
                    trim_null_bytes(string)
                })
                .collect();
            return strings;
        }
    }
    vec![String::new(); arraydim]
}

fn split_string(input: String, separator: char) -> Vec<String> {
    input.split(separator).map(|s| s.trim().to_string()).collect()
}

////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Read the command line argument as file name of interest
    let filename = std::env::args().nth(1).expect("Missing data directory argument");

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
    let argo = client.database("argo").collection::<DataSchema>("argo");
    let argo_search = client.database("argo").collection::<DataSchema>("argo_search");

    // structs to describe documents //////////////////////////////

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct GeoJSONPoint {
        #[serde(rename = "type")]
        location_type: String,
        coordinates: [f64; 2],
    } 

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct DataInfo {
        DATA_MODE: String,
        UNITS: String,
        LONG_NAME: String,
        PROFILE_PARAMETER_QC: String,
    } 

    // todo: add timestamp added as a simple versioning mechanism
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct DataSchema {
        _id: String,
        geolocation: GeoJSONPoint,
        CYCLE_NUMBER: i32,
        DIRECTION: String,
        DATA_STATE_INDICATOR: String,
        DATA_MODE: String,
        DATE_CREATION: String,
        DATE_UPDATE: String,
        DC_REFERENCE: String,
        JULD: f64,
        JULD_QC: String,
        JULD_LOCATION: f64,
        POSITION_QC: String,
        VERTICAL_SAMPLING_SCHEME: String,
        CONFIG_MISSION_NUMBER: i32,
        STATION_PARAMETERS: Vec<String>,
        realtime_data: Option<HashMap<String, Vec<f64>>>,
        adjusted_data: Option<HashMap<String, Vec<f64>>>,
        data_info: Option<HashMap<String, DataInfo>>,
        level_qc: Option<HashMap<String, Vec<String>>>,
        adjusted_level_qc: Option<HashMap<String, Vec<String>>>,
        DATA_TYPE: String,
        FORMAT_VERSION: String,
        HANDBOOK_VERSION: String,
        REFERENCE_DATE_TIME: String,
        PROJECT_NAME: String,   
        PI_NAME: Vec<String>,
        DATA_CENTRE: String,
        PLATFORM_TYPE: String,
        PLATFORM_NUMBER: String,
        FLOAT_SERIAL_NO: String,
        FIRMWARE_VERSION: String,
        WMO_INST_TYPE: String,
        POSITIONING_SYSTEM: String,
        source_file: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct MapSchema {
        _id: String,
        geolocation: GeoJSONPoint,
        JULD: f64,
        STATION_PARAMETERS: Vec<String>,
        source_file: String,
    }
    // construct link to upstream netcdf file
    let parts: Vec<&str> = filename.split("ifremer/").collect();
    let source_file = format!("ftp://ftp.ifremer.fr/ifremer/argo/dac/{}", parts.get(1).unwrap());

    // remove previous content from this file
    // todo: surely there is a better way to do this; at least skip this via env variable when doing full rebuild
    argo.delete_many(doc! { "source_file": source_file.clone() }, None).await?;
    argo_search.delete_many(doc! { "source_file": source_file.clone() }, None).await?;

    // open the file or inform the user the profile has been dropped
    println!("Processing file: {}", filename.clone());
    let id = filename
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".nc"))
        .unwrap_or("");
    let file = match netcdf::open(&filename.clone()) {
        Ok(file) => file,
        Err(_e) => {
            eprintln!("Deleted contents of file: {}", source_file);
            std::process::exit(1);
        }
    };

    // loop over internal profiles
    let N_PROF: usize = file.dimension("N_PROF").unwrap().len();
    for pfl in 0..N_PROF {

        // data unpacking /////////////////////////////////////////////
        let pindex = 0; // just use the first profile for now
        let STRING1: usize = 1;
        let STRING2: usize = 2;
        let STRING4: usize = 4;
        let STRING8: usize = 8;
        let STRING16: usize = 16;
        let STRING32: usize = 32;
        let STRING64: usize = 64;
        let STRING256: usize = 256;
        let DATE_TIME: usize = 14;
        
        let N_PARAM: usize = file.dimension("N_PARAM").unwrap().len();
        let N_LEVELS: usize = file.dimension("N_LEVELS").unwrap().len();
        //let N_CALIB: usize = file.dimension("N_CALIB").unwrap().len();
        //let N_HISTORY: usize = file.dimension("N_HISTORY").unwrap().len();
    
        let DATA_TYPE: String = unpack_string("DATA_TYPE", STRING16, [..16].into(), &file);
        let FORMAT_VERSION: String = unpack_string("FORMAT_VERSION", STRING4, [..4].into(), &file);
        let HANDBOOK_VERSION: String = unpack_string("HANDBOOK_VERSION", STRING4, [..4].into(), &file);
        let REFERENCE_DATE_TIME: String = unpack_string("REFERENCE_DATE_TIME", DATE_TIME, [..14].into(), &file);
        let DATE_CREATION: String = unpack_string("DATE_CREATION", DATE_TIME, [..14].into(), &file);
        let DATE_UPDATE: String = unpack_string("DATE_UPDATE", DATE_TIME, [..14].into(), &file);
        let PLATFORM_NUMBER: String = unpack_string("PLATFORM_NUMBER", STRING8, [pfl..(pfl+1), 0..8].into(), &file);
        let PROJECT_NAME: String = unpack_string("PROJECT_NAME", STRING64, [pfl..(pfl+1), 0..64].into(), &file);
        let PI_NAME: String = unpack_string("PI_NAME", STRING64, [pfl..(pfl+1), 0..64].into(), &file);
        let namesize: usize = file.variable("STATION_PARAMETERS").unwrap().dimensions()[2].len();
        let STATION_PARAMETERS: Vec<String> = unpack_string_array(
            "STATION_PARAMETERS",
            match namesize {
                1 => STRING1,
                2 => STRING2,
                4 => STRING4,
                8 => STRING8,
                16 => STRING16,
                32 => STRING32,
                64 => STRING64,
                256 => STRING256,
                _ => panic!("Unsupported namesize: {}", namesize),
            },
            N_PARAM,
            [pfl..(pfl+1), 0..N_PARAM, 0..namesize].into(),
            &file,
        );
        let CYCLE_NUMBER: i32 = file.variable("CYCLE_NUMBER").map(|var| var.get_value([pindex]).unwrap_or(99999)).unwrap_or(99999);
        let DIRECTION: String = unpack_string("DIRECTION", STRING1, [pfl..(pfl+1)].into(), &file);
        let DATA_CENTRE: String = unpack_string("DATA_CENTRE", STRING2, [pfl..(pfl+1), 0..2].into(), &file);
        let DC_REFERENCE: String = unpack_string("DC_REFERENCE", STRING32, [pfl..(pfl+1), 0..32].into(), &file);
        let DATA_STATE_INDICATOR: String = unpack_string("DATA_STATE_INDICATOR", STRING4, [pfl..(pfl+1), 0..4].into(), &file);
        let DATA_MODE: String = unpack_string("DATA_MODE", STRING1, [pfl..(pfl+1)].into(), &file);
        let PLATFORM_TYPE: String = unpack_string("PLATFORM_TYPE", STRING32, [pfl..(pfl+1), 0..32].into(), &file);
        let FLOAT_SERIAL_NO: String = unpack_string("FLOAT_SERIAL_NO", STRING32, [pfl..(pfl+1), 0..32].into(), &file);
        let FIRMWARE_VERSION: String = unpack_string("FIRMWARE_VERSION", STRING32, [pfl..(pfl+1), 0..32].into(), &file);
        let WMO_INST_TYPE: String = unpack_string("WMO_INST_TYPE", STRING4, [pfl..(pfl+1), 0..4].into(), &file);
        let JULD: f64 = file.variable("JULD").map(|var| var.get_value([pindex]).unwrap_or(999999.0)).unwrap_or(999999.0);
        let JULD_QC: String = unpack_string("JULD_QC", STRING1, [pfl..(pfl+1)].into(), &file);
        let JULD_LOCATION: f64 = file.variable("JULD_LOCATION").map(|var| var.get_value([pindex]).unwrap_or(999999.0)).unwrap_or(999999.0);
        let mut LATITUDE: f64 = file.variable("LATITUDE").map(|var| var.get_value([pindex]).unwrap_or(99999.0)).unwrap_or(99999.0);
        let mut LONGITUDE: f64 = file.variable("LONGITUDE").map(|var| var.get_value([pindex]).unwrap_or(99999.0)).unwrap_or(99999.0);
        let latitude_fills = [99999.0, -99.999, -999.0];
        let longitude_fills = [99999.0, -999.999, -999.0]; 
        if latitude_fills.contains(&LATITUDE) || longitude_fills.contains(&LONGITUDE) || LATITUDE.is_nan() || LONGITUDE.is_nan() {
            LATITUDE = -90.0;
            LONGITUDE = 0.0;
        }
        LONGITUDE = if LONGITUDE > 180.0 {
            LONGITUDE - 360.0
        } else if LONGITUDE < -180.0 {
            LONGITUDE + 360.0
        } else {
            LONGITUDE
        };
        let POSITION_QC: String = unpack_string("POSITION_QC", STRING1, [pfl..(pfl+1)].into(), &file);
        let POSITIONING_SYSTEM: String = unpack_string("POSITIONING_SYSTEM", STRING8, [pfl..(pfl+1), 0..8].into(), &file);
        let VERTICAL_SAMPLING_SCHEME: String = unpack_string("VERTICAL_SAMPLING_SCHEME", STRING256, [pfl..(pfl+1), 0..256].into(), &file);
        let CONFIG_MISSION_NUMBER: i32 = file.variable("CONFIG_MISSION_NUMBER").map(|var| var.get_value([pindex]).unwrap_or(99999)).unwrap_or(99999);

        let PARAMETER_DATA_MODE: Vec<String> = if let Some(_variable) = file.variable("PARAMETER_DATA_MODE") {
            unpack_string_array("PARAMETER_DATA_MODE", STRING1, N_PARAM, [pfl..(pfl+1), 0..N_PARAM].into(), &file)
        } else {
            vec![DATA_MODE.clone(); STATION_PARAMETERS.len()]
        };
        
        let mut realtime_data: Option<HashMap<String, Vec<f64>>> = STATION_PARAMETERS.iter()
            .map(|param| {
                if param.is_empty() {
                    Ok((param.clone(), vec![]))
                } else {
                    match file.variable(param) {
                        Some(variable) => {
                            let mut data: Vec<f64> = variable.get_values([pfl..(pfl+1), 0..N_LEVELS])?;
                            if let Some(pos) = data.iter().rposition(|&x| x != 99999.0) {
                                data.truncate(pos + 1);
                            }
                            Ok((param.clone(), data))
                        },
                        None => Ok((param.clone(), vec![])),
                    }
                }
            })
            .collect::<Result<_, Box<dyn Error>>>()
            .map(Some)
            .unwrap_or(None);
        if let Some(realtime_data) = &mut realtime_data {
            realtime_data.retain(|_, v| !v.is_empty());
        }

        let mut adjusted_data: Option<HashMap<String, Vec<f64>>> = STATION_PARAMETERS.iter()
            .enumerate()
            .map(|(i, param)| {
                if param.is_empty() {
                    Ok((param.clone(), vec![]))
                } else {
                    let data_mode = PARAMETER_DATA_MODE.get(i).cloned().unwrap_or(DATA_MODE.clone());
                    if data_mode == "R" || param == "NB_SAMPLE_CTD" {
                        Ok((param.clone(), vec![]))
                    } else {
                        let adjusted_variable_name = format!("{}_ADJUSTED", param);
                        match file.variable(&adjusted_variable_name) {
                            Some(variable) => {
                                let mut data: Vec<f64> = variable.get_values([pfl..(pfl+1), 0..N_LEVELS])?;
                                if let Some(pos) = data.iter().rposition(|&x| x != 99999.0) {
                                    data.truncate(pos + 1);
                                }
                                Ok((param.clone(), data))
                            },
                            None => Ok((param.clone(), vec![])),
                        }                    
                    }
                }
            })
            .collect::<Result<_, Box<dyn Error>>>()
            .map(Some)
            .unwrap_or(None);
        if let Some(adjusted_data) = &mut adjusted_data {
            adjusted_data.retain(|_, v| !v.is_empty());
        }

        let mut level_qc: Option<HashMap<String, Vec<String>>> = STATION_PARAMETERS.iter()
            .map(|param| {
                if param.is_empty() {
                    Ok((param.clone(), vec![]))
                } else {
                    let qc_variable_name = format!("{}_QC", param);
                    let mut qc_vec = unpack_string_array(&qc_variable_name, STRING1, N_LEVELS, [pfl..(pfl+1), 0..N_LEVELS].into(), &file);
                    if let Some(pos) = qc_vec.iter().rposition(|x| x != "") {
                        qc_vec.truncate(pos + 1);
                    }
                    Ok((param.clone(), qc_vec))
                }
            })
            .collect::<Result<_, Box<dyn Error>>>()
            .map(Some)
            .unwrap_or(None);
        if let Some(level_qc) = &mut level_qc {
            level_qc.retain(|_, v| !v.is_empty() && !v.iter().all(|x| x == ""));
        }
            
        let mut adjusted_level_qc: Option<HashMap<String, Vec<String>>> = STATION_PARAMETERS.iter()
            .enumerate()
            .map(|(i, param)| {
                if param.is_empty() {
                    Ok((param.clone(), vec![]))
                } else {
                    let data_mode = PARAMETER_DATA_MODE.get(i).cloned().unwrap_or(DATA_MODE.clone());
                    if data_mode == "R" || param == "NB_SAMPLE_CTD" {
                        Ok((param.clone(), vec![]))
                    } else {
                        let qc_variable_name = format!("{}_ADJUSTED_QC", param);
                        let mut qc_vec = unpack_string_array(&qc_variable_name, STRING1, N_LEVELS, [pfl..(pfl+1), 0..N_LEVELS].into(), &file);
                        if let Some(pos) = qc_vec.iter().rposition(|x| x != "") {
                            qc_vec.truncate(pos + 1);
                        }
                        Ok((param.clone(), qc_vec))
                    }
                }
            })
            .collect::<Result<_, Box<dyn Error>>>()
            .map(Some)
            .unwrap_or(None);
        if let Some(adjusted_level_qc) = &mut adjusted_level_qc {
            adjusted_level_qc.retain(|_, v| !v.is_empty());
        }

        // make sure we didn't truncate too many fill values
        // Find the maximum length among all vectors in the HashMaps
        let max_len = realtime_data.as_ref().map_or(0, |m| m.values().map(|v| v.len()).max().unwrap_or(0))
        .max(adjusted_data.as_ref().map_or(0, |m| m.values().map(|v| v.len()).max().unwrap_or(0)))
        .max(level_qc.as_ref().map_or(0, |m| m.values().map(|v| v.len()).max().unwrap_or(0)))
        .max(adjusted_level_qc.as_ref().map_or(0, |m| m.values().map(|v| v.len()).max().unwrap_or(0)));
        // Pad vectors in realtime_data and adjusted_data with 99999.0
        if let Some(realtime_data) = &mut realtime_data {
            for vec in realtime_data.values_mut() {
                vec.resize(max_len, 99999.0);
            }
        }
        if let Some(adjusted_data) = &mut adjusted_data {
            for vec in adjusted_data.values_mut() {
                vec.resize(max_len, 99999.0);
            }
        }
        // Pad vectors in level_qc and adjusted_level_qc with ""
        if let Some(level_qc) = &mut level_qc {
            for vec in level_qc.values_mut() {
                vec.resize(max_len, "".to_string());
            }
        }
        if let Some(adjusted_level_qc) = &mut adjusted_level_qc {
            for vec in adjusted_level_qc.values_mut() {
                vec.resize(max_len, "".to_string());
            }
        }

        let mut data_info: Option<HashMap<String, DataInfo>> = STATION_PARAMETERS.iter()
            .enumerate()
            .map(|(i, param)| {
                if param.is_empty() || param == "NB_SAMPLE_CTD" {
                    Ok((param.clone(), DataInfo {
                        DATA_MODE: "".to_string(),
                        UNITS: "".to_string(),
                        LONG_NAME: "".to_string(),
                        PROFILE_PARAMETER_QC: "".to_string(),
                    }))
                } else {
                    // assumption: if PARAMETER_DATA_MODE exists, it should be used in lieu of DATA_MODE
                    let data_mode = PARAMETER_DATA_MODE.get(i).cloned().unwrap_or(DATA_MODE.clone());
                    match file.variable(param) {
                        Some(variable) => {
                            //let data_mode = PARAMETER_DATA_MODE.get(i).cloned().unwrap_or(DATA_MODE.clone());
                            let units = variable.attribute_value("units").unwrap()?;
                            let long_name = variable.attribute_value("long_name").unwrap()?;
                            let qc_variable_name = format!("PROFILE_{}_QC", param);
                            let qc_value = unpack_string(&qc_variable_name, STRING1, [pfl..(pfl+1)].into(), &file);
                            if let netcdf::AttributeValue::Str(u) = units {
                                if let netcdf::AttributeValue::Str(l) = long_name {
                                    Ok((param.clone(), DataInfo {
                                        DATA_MODE: data_mode,
                                        UNITS: u.to_string(),
                                        LONG_NAME: l.to_string(),
                                        PROFILE_PARAMETER_QC: qc_value,
                                    }))
                                } else {
                                    Err("Could not extract long_name attribute".into())
                                }
                            } else {
                                Err("Could not extract units attribute".into())
                            } 
                        },
                        None => Ok((param.clone(), DataInfo {
                            DATA_MODE: "".to_string(),
                            UNITS: "".to_string(),
                            LONG_NAME: "".to_string(),
                            PROFILE_PARAMETER_QC: "".to_string(),
                        })),
                    }   
                }
            })
            .collect::<Result<_, Box<dyn Error>>>()
            .map(Some)
            .unwrap_or(None);
        if let Some(data_info) = &mut data_info {
            data_info.remove("");
        }

        // let adjusted_level_error: HashMap<String, Vec<f64>> = STATION_PARAMETERS.iter()
        //     .map(|param| {
        //         let adjusted_variable_name = format!("{}_ADJUSTED_ERROR", param);
        //         let variable = file.variable(&adjusted_variable_name).expect(&format!("Could not find variable '{}'", adjusted_variable_name));
        //         let data: Vec<f64> = variable.get_values([..1, ..N_LEVELS])?;
        //         Ok((param.clone(), data))
        //     })
        //     .collect::<Result<_, Box<dyn Error>>>()?;
        
        // construct the structs for this file ///////////////////////////////
    
        let data_object = DataSchema {
            _id: format!("{}_{}", id, pfl),
            geolocation: GeoJSONPoint {
                location_type: "Point".to_string(),
                coordinates: [LONGITUDE, LATITUDE],
            },
            CYCLE_NUMBER: CYCLE_NUMBER,
            DIRECTION: DIRECTION,
            DATA_STATE_INDICATOR: DATA_STATE_INDICATOR,
            DATA_MODE: DATA_MODE,
            DATE_CREATION: DATE_CREATION,
            DATE_UPDATE: DATE_UPDATE,
            DC_REFERENCE: DC_REFERENCE,
            JULD: JULD,
            JULD_QC: JULD_QC,
            JULD_LOCATION: JULD_LOCATION,
            POSITION_QC: POSITION_QC,
            VERTICAL_SAMPLING_SCHEME: VERTICAL_SAMPLING_SCHEME,
            CONFIG_MISSION_NUMBER: CONFIG_MISSION_NUMBER,
            STATION_PARAMETERS: STATION_PARAMETERS.clone(),
            realtime_data: realtime_data,
            adjusted_data: adjusted_data,
            data_info: data_info,
            level_qc: level_qc,
            adjusted_level_qc: adjusted_level_qc,
            DATA_TYPE: DATA_TYPE,
            FORMAT_VERSION: FORMAT_VERSION,
            HANDBOOK_VERSION: HANDBOOK_VERSION,
            REFERENCE_DATE_TIME: REFERENCE_DATE_TIME,
            PROJECT_NAME: PROJECT_NAME,
            PI_NAME: split_string(PI_NAME, ','),
            DATA_CENTRE: DATA_CENTRE,
            PLATFORM_TYPE: PLATFORM_TYPE,
            PLATFORM_NUMBER: PLATFORM_NUMBER,
            FLOAT_SERIAL_NO: FLOAT_SERIAL_NO,
            FIRMWARE_VERSION: FIRMWARE_VERSION,
            WMO_INST_TYPE: WMO_INST_TYPE,
            POSITIONING_SYSTEM: POSITIONING_SYSTEM,
            source_file: source_file.clone(),
        };

        let map_object = MapSchema {
            _id: format!("{}_{}", id, pfl),
            geolocation: GeoJSONPoint {
                location_type: "Point".to_string(),
                coordinates: [LONGITUDE, LATITUDE],
            },
            JULD: JULD,
            STATION_PARAMETERS: STATION_PARAMETERS,
            source_file: source_file.clone(),
        };

        //println!("{:?}", data_object);
    
        // insert the structs into the database ////////////////////////////
        let filter = doc! {
            "_id": data_object._id.clone(),
        };
        let update = doc! {
            "$set": bson::to_bson(&data_object)?,
        };
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        argo.update_one(filter, update, options).await?;

        let map_filter = doc! {
            "_id": map_object._id.clone(),
        };
        let map_update = doc! {
            "$set": bson::to_bson(&map_object)?,
        };
        let map_options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        argo_search.update_one(map_filter, map_update, map_options).await?;
    }
    
    Ok(())
}


