//! expansions
//!
//! This module supports an expansion file format originally based on yasb's,
//! but uses only xws ids for the individual items.
//!
//! Currently, expansions names _should_ use the xws-ification of their names as used by
//! yasb for compatibility.
//!
//! It is recommended to keep a dummy expansion such as yasb's "looseships" list
//! for ships that have been unreleased for 2.0.
//!
//! ```json
//! {
//!     "firsteditionvt49decimatorexpansionpack": [
//!    {
//!      "xws": "vt49decimator",
//!      "type": "ship",
//!      "count": 1
//!    },
//!    {
//!      "xws": "vt49decimatordebris0",
//!      "type": "obstacle",
//!      "count": 1
//!    }
//!    ],
//!    "looseships": [
//!       "xws": "ewing",
//!       "type": "ship",
//!       "count": 1,
//!     ]
//! }
//! ```
use std::{fs, io};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ItemType {
    #[serde(alias = "ship")]
    Ship,
    #[serde(alias = "obstacle")]
    Obstacle,
    #[serde(alias = "pilot")]
    Pilot,
    #[serde(alias = "upgrade")]
    Upgrade,
    #[serde(alias = "damage")]
    Damage,
}

/// Item are the unique type and xws ID combos that _must_ be unique according
/// to spec.
#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(tag = "type")]
pub struct Item {
    pub r#type: ItemType,
    pub xws: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub struct ItemCount {
    #[serde(flatten)]
    pub item: Item,
    pub count: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub struct Expansion {
    pub sku: String,
    pub name: String,
    pub contents: Vec<ItemCount>,
}

impl Expansion {
    /// edition returns 1, 2, or 0 if unknown
    pub fn edition(&self) -> u8 {
        match self.sku.chars().nth(2) {
            Some('z') | Some('Z') => 2,
            Some('x') | Some('X') => 1,
            Some(_) | None => 0,
        }
    }
}

pub type Expansions = Vec<Expansion>;

/// TODO: Should be a real impl?
pub fn has_item(expansions: &Expansions, item: &Item) -> bool {
    for e in expansions.iter() {
        for i in &e.contents {
            if &i.item == item {
                return true;
            }
        }
    }
    false
}

/// Loads a yasb based expansion content list from embedded file.
pub fn load_expansions() -> Result<Expansions, io::Error> {
    //TODO: embed with rust-embed or include_bytes! or something
    let buffer = fs::read_to_string("./src/expansions/expansions.json")?;

    let expansions = serde_json::from_str(&buffer)?;
    return Ok(expansions);
}

#[cfg(test)]
mod test {
    use std::{io::Write, path::Path};

    use super::*;
    use crate::xwingdata2;
    use crate::xwingdata2::known_missing;

    #[test]
    fn test_valid_xws() {
        // checks if all the contents are valid xwsdata
        let r = load_expansions().unwrap();

        let d = xwingdata2::load_from_manifest(Path::new("xwing-data2")).unwrap();

        for e in r.iter() {
            if e.edition() != 2 {
                continue;
            }
            for item_count in &e.contents {
                let result = match item_count {
                    ItemCount {
                        item:
                            Item {
                                r#type: ItemType::Ship,
                                xws,
                            },
                        ..
                    } => d.get_ship(&xws).is_some() || known_missing(&xws),
                    ItemCount {
                        item:
                            Item {
                                r#type: ItemType::Pilot,
                                xws,
                            },
                        ..
                    } => d.get_pilot(&xws).is_some() || known_missing(&xws),
                    ItemCount {
                        item:
                            Item {
                                r#type: ItemType::Upgrade,
                                xws,
                            },
                        ..
                    } => d.get_upgrade(&xws).is_some() || known_missing(&xws),
                    _ => continue,
                };

                println!("{:?}", item_count);
                assert!(result, "missing expansion item");

                io::stdout().flush().unwrap();
            }
        }
    }
}
