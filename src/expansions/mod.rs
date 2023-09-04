//! A (US) SKU and xws based list of expansions and their contents.
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
//!```
//!
//!```rust
//!use xwingtmg2_inventory_rs::expansions::Catalog;
//!
//!let catalog = Catalog::load().unwrap();
//!let core = catalog.expansions.get("swz01").unwrap();
//!```
//!
//! NOTES:
//!
//! - Even though sku's are unique enough for this to be a map, storing as a
//!   list makes it easier to keep sorted in the json.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io};

/// Type literals used in the serialized format.
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

/// Syntactic sugar for knowing when an xws id is intended to be used.
pub type XWS = String;

/// Item are the unique type and xws ID combos that _must_ be unique according
/// to spec.
#[derive(Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(tag = "type")]
pub struct Item {
    pub r#type: ItemType,
    pub xws: XWS,
}

/// An association between an Item and it's count that is mostly useful for
/// de/serialization.
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub struct ItemCount {
    #[serde(flatten)]
    pub item: Item,
    pub count: u32,
}

/// The (US) SKU is used to refer to expansions because it really isn't part
/// of the XWS specification or data, and the names are open to
/// interpretation, duplicative, etc., so don't make good ids.
pub type SKU = String;

/// Basic expansion metadata
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub struct Expansion {
    pub sku: SKU,
    pub name: String,
    pub contents: Vec<ItemCount>,
}

/// A catalog is the list from an `expansions.json` processed into some useful
/// maps.
#[derive(Default)]
pub struct Catalog {
    /// A map of SKU to expansion contents and other metadata.
    pub expansions: BTreeMap<SKU, Expansion>,
    /// A lookup from an item to the skus that contain the item and the number
    /// per-expansions.
    ///
    /// FIXME: This uses `Item.xws` as the SKU, which is confusing.
    pub sources: BTreeMap<Item, Vec<ItemCount>>,
}

impl Catalog {
    pub fn has_item(&self, item: &Item) -> bool {
        for (_, e) in self.expansions.iter() {
            for i in &e.contents {
                if &i.item == item {
                    return true;
                }
            }
        }
        false
    }

    pub fn load() -> Result<Self, io::Error> {
        //TODO: embed with rust-embed or include_bytes! or something
        let buffer = fs::read_to_string("./src/expansions/expansions.json")?;

        let mut list: Vec<Expansion> = serde_json::from_str(&buffer)?;

        let mut catalog = Catalog {
            ..Default::default()
        };

        for expansion in list.drain(..) {
            let sku = expansion.sku.to_owned(); //FIXME, this is just for error message

            for c in &expansion.contents {
                catalog
                    .sources
                    .entry(c.item.clone())
                    .and_modify(|s| {
                        s.push(ItemCount {
                            item: Item {
                                r#type: c.item.r#type,
                                xws: sku.clone(),
                            },
                            count: c.count,
                        })
                    })
                    .or_insert(vec![ItemCount {
                        item: Item {
                            r#type: c.item.r#type,
                            xws: sku.clone(),
                        },
                        count: c.count,
                    }]);
            }

            if catalog
                .expansions
                .insert(expansion.sku.to_owned(), expansion)
                .is_some()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate sku: {}", sku),
                ));
            }
        }

        Ok(catalog)
    }
}

#[cfg(test)]
mod test {
    use std::{io::Write, path::Path};

    use super::*;
    use crate::xwingdata2::known_missing;
    use crate::xwingdata2::Data;

    #[test]
    fn test_valid_xws() {
        // checks if all the contents are valid xwsdata
        let cat = Catalog::load().unwrap();

        let d = Data::load_from_manifest(Path::new("xwing-data2")).unwrap();

        for (_, e) in cat.expansions.iter() {
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
