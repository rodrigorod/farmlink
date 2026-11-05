//! Farming Simulator 25 filesystem, mod packaging, and metadata integration.

pub mod mod_desc;
pub mod scanner;

pub use mod_desc::{parse_mod_desc, ModDesc};
pub use scanner::{calculate_file_sha256, extract_mod_desc_from_zip, LocalMod, ModScanner};
