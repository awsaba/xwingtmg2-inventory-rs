//! A generic xws-based collection is:
//!
//! - A list of expansions and their counts, indexed by SKU
//! - A list of additional components by their xws IDs.
//!
//! An Inventory is the complete (as implemented) xws data for each kind with
//! its count.
use crate::expansions::Item;
use crate::xwingdata2::Restriction;
pub mod expansions;
pub mod xwingdata2;
pub mod yasb2;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

#[derive(Default, Serialize, Deserialize)]
pub struct Collection {
    pub skus: BTreeMap<String, u32>,
    pub singles: BTreeMap<Item, u32>,
}

impl Collection {
    pub fn inventory(
        &self,
        expansions: expansions::Expansions,
    ) -> (BTreeMap<Item, u32>, Vec<String>) {
        let mut inventory = self.singles.clone();
        let mut missing_expansions = vec![];

        'sku_search: for (sku, c) in &self.skus {
            for expansion in &expansions {
                if &expansion.sku == sku {
                    for item_count in &expansion.contents {
                        let total =
                            inventory.get(&item_count.item).unwrap_or(&0) + c * item_count.count;
                        inventory.insert(item_count.item.clone(), total);
                    }
                    continue 'sku_search;
                }
            }

            missing_expansions.push(sku.to_owned());
        }
        (inventory, missing_expansions)
    }
}

/// This is the full ship as defined by the expansions.
///
/// TODO: Add a "miniature/chassis" type that reflects usability per tournament
/// regulations.
#[derive(Serialize, Debug)]
pub struct ShipRecord {
    pub name: String,
    pub xws: String,

    pub count: u32,
}

impl ShipRecord {
    pub fn new(s: &xwingdata2::Ship, count: u32) -> ShipRecord {
        ShipRecord {
            name: s.name.to_owned(),
            xws: s.xws.to_owned(),
            count,
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
}

/// UpgradeRecord are the fields I sort my collection by.
#[derive(Serialize, Debug)]
pub struct UpgradeRecord {
    pub r#type: String,
    pub name: String,
    pub faction_restriction: String,
    pub size_restriction: String,
    pub ship_restriction: String,
    pub arc_restriction: String,
    pub keyword_restriction: String,

    pub count: u32,
    pub force_side_restriction: String,

    pub xws: String,
}

impl UpgradeRecord {
    pub fn new(u: &xwingdata2::Upgrade, c: u32) -> UpgradeRecord {
        // TODO: there must be a better way

        UpgradeRecord {
            name: u.name.to_owned(),
            xws: u.xws.to_owned(),
            count: c,
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
