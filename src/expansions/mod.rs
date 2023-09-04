//! expansions
//!
//! It is recommended to keep a dummy expansion such as yasb's "looseships" list
//! for ships that have been unreleased for 2.0.
//!
//! ```json
//! {
//!  {
//!    "name": "T-70 X-Wing Expansion Pack",
//!    "sku": "swz25",
//!    "contents": [
//!    {
//!        "count": 1,
//!        "type": "ship",
//!        "xws": "t70xwing"
//!      },
//!      {
//!        "count": 1,
//!        "type": "pilot",
//!        "xws": "poedameron"
//!      },
//!      {
//!        "count": 1,
//!        "type": "upgrade",
//!        "xws": "blackone"
//!      },
//!      {
//!        "count": 1,
//!        "type": "upgrade",
//!        "xws": "bb8"
//!      }
//!    ]
//!  },
//!  {
//!    "name": "Unreleased for 2nd Edition",
//!    "sku": "swzunreleased"
//!   }
//! }
//! ```
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
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
#[derive(Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
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
    Ok(expansions)
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
            for item_count in &e.contents {
                if known_missing(&item_count.item.xws) {
                    continue;
                }
                let result = match item_count.item.r#type {
                    ItemType::Ship => d.get_ship(&item_count.item.xws).is_some(),
                    ItemType::Pilot => d.get_pilot(&item_count.item.xws).is_some(),
                    ItemType::Upgrade => d.get_upgrade(&item_count.item.xws).is_some(),
                    _ => continue,
                };

                println!("{:?}", item_count);
                assert!(result, "missing expansion item");

                io::stdout().flush().unwrap();
            }
        }
    }
}
