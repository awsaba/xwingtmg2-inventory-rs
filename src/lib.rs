//! A Rust library for keeping track of an X-Wing: The Miniatures Game, 2nd
//! edition collection, based on product skus and xws id.
//!
//! It currently suppports importing a collection from `yasb.app` and producing
//! a usable JSON document.
//!
//! The "*Record" types are subsets of the xwing-data2 info that is relevant
//! to me for sorting my collection after importing into a spreadsheet.
//!
//! See the project README.md for example usage of the included CLI utility.
use crate::expansions::Item;
use crate::xwingdata2::Restriction;
pub mod expansions;
pub mod xwingdata2;
pub mod yasb2;

use expansions::{Catalog, ItemCount, ItemType, SKU};
use serde::{Deserialize, Serialize};
use xwingdata2::Data;

use std::collections::BTreeMap;

pub enum ErrorKind {
    NotFound,
}

/// A collection is:
/// - A list of expansions and their counts, indexed by SKU
/// - A list of additional `singles` identified by their type and xws id.
///
/// Minimal error checking is done by the collection itself, it mostly defines
/// a tool agnostic, unambigous definition of a collection.
#[derive(Default, Serialize, Deserialize)]
pub struct Collection {
    pub skus: BTreeMap<SKU, u32>,
    pub singles: BTreeMap<Item, u32>,
}

/// An Inventory is a just a count of Items, where Items have just enough
/// information to look them up in xwing-data2 or an catalog of expansion
/// contents.
pub type Inventory = BTreeMap<Item, u32>;

impl Collection {
    /// Produce a count of all items in expansions and add them to the singles.
    ///
    /// Returns a list of expansions that weren't found in the catalog.
    pub fn inventory(&self, catalog: &expansions::Catalog) -> (Inventory, Vec<String>) {
        let mut inventory = self.singles.clone();
        let mut missing_expansions = vec![];

        for (sku, c) in &self.skus {
            match catalog.expansions.get(sku) {
                Some(expansion) => {
                    for item_count in &expansion.contents {
                        let total =
                            inventory.get(&item_count.item).unwrap_or(&0) + c * item_count.count;
                        inventory.insert(item_count.item.clone(), total);
                    }
                }
                None => missing_expansions.push(sku.to_owned()),
            };
        }
        (inventory, missing_expansions)
    }
}

/// This is the full ship as defined by the expansions.
///
/// TODO: Add a "miniature/chassis" type compatibility that reflects usability
/// per tournament regulations.
#[derive(Serialize, Debug)]
pub struct ShipRecord {
    pub name: String,
    pub xws: String,

    pub count: u32,

    // just a long string of the sources for informational purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl ShipRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(
        xws: &str,
        count: u32,
        data: &Data,
        expansions: &Catalog,
    ) -> Result<Self, ErrorKind> {
        match data.get_ship(xws) {
            None => Err(ErrorKind::NotFound),
            Some(s) => Ok(Self {
                name: s.name.to_owned(),
                xws: s.xws.to_owned(),
                sources: expansions
                    .sources
                    .get(&Item {
                        r#type: ItemType::Ship,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(expansions, s)),
                count,
            }),
        }
    }
}

/// PilotRecord has fields that I want to sort by so that I can organize my
/// collection, either in binders or boxes.
#[derive(Serialize, Debug)]
pub struct PilotRecord {
    pub faction: String,
    pub ship: String,
    pub xws: String,
    pub name: String,
    pub initiative: u32,

    pub count: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl PilotRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(
        xws: &str,
        count: u32,
        data: &Data,
        expansions: &Catalog,
    ) -> Result<Self, ErrorKind> {
        // TODO: there must be a better way to do the restrictions
        match data.get_pilot(xws) {
            None => Err(ErrorKind::NotFound),
            Some((s, p)) => Ok(Self {
                faction: s.faction.to_owned(),
                ship: s.name.to_owned(),
                name: p.name.to_owned(),
                xws: p.xws.to_owned(),
                initiative: p.initiative,
                count,
                sources: expansions
                    .sources
                    .get(&Item {
                        r#type: ItemType::Pilot,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(expansions, s)),
            }),
        }
    }
}

/// UpgradeRecord are the fields I sort my collection by.
#[derive(Serialize, Debug)]
pub struct UpgradeRecord {
    pub xws: String,
    pub r#type: String,
    pub name: String,
    pub faction_restriction: String,
    pub size_restriction: String,
    pub ship_restriction: String,
    pub arc_restriction: String,
    pub keyword_restriction: String,

    pub count: u32,
    pub force_side_restriction: String,

    // just a long string of the sources for informational purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl UpgradeRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(
        xws: &str,
        count: u32,
        data: &Data,
        expansions: &Catalog,
    ) -> Result<Self, ErrorKind> {
        // TODO: there must be a better way to do the restrictions
        match data.get_upgrade(xws) {
            None => Err(ErrorKind::NotFound),
            Some(u) => Ok(Self {
                name: u.name.to_owned(),
                xws: u.xws.to_owned(),
                count,
                r#type: u
                    .sides
                    .first()
                    .map(|s| format!("{:?}", s.r#type)) //FIXME
                    .unwrap_or("not found".to_owned())
                    .to_owned(),
                faction_restriction: format_restriction(&u.restrictions, Restriction::Factions),
                size_restriction: format_restriction(&u.restrictions, Restriction::Sizes),
                ship_restriction: format_restriction(&u.restrictions, Restriction::Ships),
                keyword_restriction: format_restriction(&u.restrictions, Restriction::Keywords),
                force_side_restriction: format_restriction(&u.restrictions, Restriction::ForceSide),
                arc_restriction: format_restriction(&u.restrictions, Restriction::Arcs),
                sources: expansions
                    .sources
                    .get(&Item {
                        r#type: ItemType::Upgrade,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(expansions, s)),
            }),
        }
    }
}

fn format_restriction(
    restrictions: &Vec<xwingdata2::Restrictions>,
    kind: xwingdata2::Restriction,
) -> String {
    // TODO: I'm sure there is more efficient way to append these
    let mut tmp: Vec<String> = vec![];
    for r in restrictions {
        let criteria = match kind {
            Restriction::Factions => &r.factions,
            Restriction::Sizes => &r.sizes,
            Restriction::Ships => &r.ships,
            Restriction::Arcs => &r.arcs,
            Restriction::Keywords => &r.keywords,
            Restriction::ForceSide => &r.force_side,
        };
        if !criteria.is_empty() {
            tmp.push(criteria.join(","))
        }
    }
    tmp.join(",")
}

fn format_sources(expansions: &expansions::Catalog, sources: &Vec<ItemCount>) -> String {
    let mut strs = vec![];

    for s in sources {
        let name = expansions
            .expansions
            .get(&s.item.xws)
            .map_or("unknown", |e| &e.name);
        strs.push(format!("{}:{}:{}", name, s.item.xws, s.count));
    }

    strs.join(",")
}
