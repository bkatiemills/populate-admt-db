#[derive(Debug)]                        // there is a list of a couple dozen traits that can be derived in this way; they are all rust builtins.
pub enum Error {
    NetCDF(netcdf::Error),              // the name 'NetCDF' is arbitrary, but 'netcdf' became a reserved word when we added it under [dependencies] in Cargo.toml
    NoNetCDFDimensions,
    SerdeJson(serde_json::Error),
    Unknown(String),                    // best practice to have a shrug variant
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { //boilerplaty
        match self {    // this is the last statement, so it's the return value for the fmt function
            Self::NetCDF(err) => write!(f, "NetCDF Error: {}", err),
            Self::NoNetCDFDimensions => write!(f, "No NetCDF dimension found."),
            Self::SerdeJson(err) => write!(f, "Serde Error: {}", err),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for Error {} // std::error::Error only requires the Debug and Display traits be implemented, which are done above, so nothing else needs be implemented here; all other trait methods have default implementations.

pub type Result<T> = core::result::Result<T, Error>; // here's the custom Result type we're returning in main()

impl From<netcdf::Error> for Error {                // these are about the most generic From trait implementations possible
    fn from(value: netcdf::Error) -> Self {
        Self::NetCDF(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}
