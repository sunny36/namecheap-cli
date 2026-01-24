pub mod json;
pub mod table;

use crate::cli::GlobalOpts;

pub fn is_json(global: &GlobalOpts) -> bool {
    global.json
}
