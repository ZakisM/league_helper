use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunePage {
    pub runes: Vec<isize>,
    pub primary_tree: isize,
    pub secondary_tree: isize,
}
