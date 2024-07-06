#[derive(Debug)]
pub enum Error {
    NetCDF(netcdf::Error),
    NoNetCDFDimensions,
    SerdeJson(serde_json::Error),
    Unknown(String),
    Ndarray(ndarray::ShapeError),
    Mongodb(mongodb::error::Error),
    Mongobson(mongodb::bson::ser::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetCDF(err) => write!(f, "NetCDF Error: {err}"),
            Self::NoNetCDFDimensions => write!(f, "No NetCDF dimension found."),
            Self::SerdeJson(err) => write!(f, "Serde Error: {err}"),
            Self::Ndarray(err) => write!(f, "ndarray Error: {err}"),
            Self::Mongodb(err) => write!(f, "Mongodb Error: {err}"),
            Self::Mongobson(err) => write!(f, "Mongobson Error: {err}"),
            Self::Unknown(msg) => write!(f, "Unknown error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

impl From<netcdf::Error> for Error {
    fn from(value: netcdf::Error) -> Self {
        Self::NetCDF(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl From<ndarray::ShapeError> for Error {
    fn from(value: ndarray::ShapeError) -> Self {
        Self::Ndarray(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Mongodb(value)
    }
}

impl From<mongodb::bson::ser::Error> for Error {
    fn from(value: mongodb::bson::ser::Error) -> Self {
        Self::Mongobson(value)
    }
}