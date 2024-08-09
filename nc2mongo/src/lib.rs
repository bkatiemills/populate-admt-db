mod ext;
pub use ext::*;
pub mod error;

/* katie's notes
I might just do:

pub mod ext;
pub mod error;

everything is still accessible publicly, but you have to be explicit about getting stuff from the 
ext module
*/