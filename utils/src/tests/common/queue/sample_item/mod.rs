use serde::{Deserialize, Serialize};

use crate::simplequeue::traits::Item;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]

pub struct FakeItem {
    id: u64,
    content: String,
}

impl Item for FakeItem {}
