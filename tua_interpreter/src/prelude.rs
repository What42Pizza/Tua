pub use crate::{*, compiler_mod::*, logger::*, additions::*, //fns::*,
    data_mod::{data::*, errors::*}
};

pub use std::{fs,
    io::Error as IoError,
    fmt::Error as FmtError,
    fmt::{self, Display},
    path::{PathBuf, Path},
};

//pub use rayon::prelude::*;
//pub use hashbrown::HashMap;
pub use array_init::array_init;
//pub use regex::Regex;
//pub use derive_is_enum_variant::is_enum_variant;
