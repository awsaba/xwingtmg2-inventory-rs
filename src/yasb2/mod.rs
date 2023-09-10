//! Support for yasb.app collections.
//!
//! YASBs `collection`s use basically their full display name for expansions and
//! cards, only translating to XWS for export. For collections that have been
//! around a while, there are legacy entries that may not be cleared by the
//! current "Reset Collection" functionality.
//!
//! This module does it's best to match YASB's reported collection contents.
//!
//! Note: YASB uses strings for the counts, so that is why it is used as value
//! types for the various maps.
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::expansions;
use crate::expansions::{Item, ItemType};

/// Additional single items in a YASB colletion.
#[derive(Deserialize, Serialize, Debug)]
pub struct Singletons {
    pub ship: Option<HashMap<String, String>>,
    pub upgrade: Option<HashMap<String, String>>,
    pub pilot: Option<HashMap<String, String>>,
}

/// A YASB collection.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Collection {
    /// A map of expansion names to counts.
    pub expansions: HashMap<String, String>,
    pub singletons: Option<Singletons>,
}

/// A Collection file is an object with a single field: `collection`.
#[derive(Deserialize, Serialize, Debug)]
pub struct CollectionFile {
    pub collection: Collection,
}

impl Collection {
    /// Load a raw YASB collection obtained from <https://login.yash.app/collection>.
    /// Intermediate step that does not turn the strings for the counts back
    /// into numbers.
    pub fn load(path: &Path) -> Result<Self, io::Error> {
        let buffer = fs::read_to_string(path)?;

        let f: CollectionFile = serde_json::from_str(&buffer)?;
        Ok(f.collection)
    }
}

/// A real basic function that can turn some YASB names into xws ids, or at
/// least enough to disambiguate.
pub fn to_canonical(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || c == &'(')
        .map(|c| match c {
            '(' => '-',
            c => char::to_ascii_lowercase(&c),
        })
        .collect::<String>()
}

/// This function implements special cases where the yasb name does not match
/// the generated xws name or performs standard xws mapping for everything else.
///
/// It would be necessary to either import the full card list from YASB's source,
/// which includes the xws id, but this seems like the least bad way to do this
/// with the idea of supporting more collection sources.
fn to_xws(name: &str, typ: expansions::ItemType) -> String {
    let mut canonical = to_canonical(name);
    // these particular bad capitilazation is problematic because it conflicts
    // the correct one, which does correctly cannonicalize to xws.
    match name {
        "TIE/FO Fighter" | "E-Wing" | "T-70 X-Wing" | "TIE/SF Fighter" => {
            canonical.push_str("-legacyyasb");
            return canonical;
        }
        _ => (),
    }
    match typ {
        ItemType::Pilot => match canonical.as_str() {
            "adigallia-delta7b" => "adigallia-delta7baethersprite",
            "ahsokatano-awing" => "ahsokatano-rz1awing",
            "blacksquadronace-t70" => "blacksquadronace-t70xwing",
            "bossk-z95headhunter" => "bossk-z95af4headhunter",
            "chewbacca-resistance" => "chewbacca-scavengedyt1300",
            "corranhorn-xwing" => "corranhorn-t65xwing",
            "dalanoberos-starviper" => "dalanoberos-starviperclassattackplatform",
            "darthvader-tiedefender" => "darthvader-tieddefender",
            "durge-separatist" => "durge-separatistalliance",
            "ezrabridger-sheathipede" => "ezrabridger-sheathipedeclassshuttle",
            "ezrabridger-tiefighter" => "ezrabridger-tielnfighter",
            "fennrau-sheathipede" => "fennrau-sheathipedeclassshuttle",
            "garvendreis-xwing" => "garvendreis-t65xwing",
            "gideonhask-tieinterceptor" => "gideonhask-tieininterceptor",
            "hansolo-resistance" => "hansolo-scavengedyt1300",
            "herasyndulla-awing" => "herasyndulla-rz1awing",
            "herasyndulla-bwing" => "herasyndulla-asf01bwing",
            "herasyndulla-vcx100" => "herasyndulla-vcx100lightfreighter",
            "landocalrissian-resistance" => "landocalrissian-scavengedyt1300",
            "norrawexley-ywing" => "norrawexley-btla4ywing",
            "poedameron-yt1300" => "poedameron-scavengedyt1300",
            "sabinewren-awing" => "sabinewren-rz1awing",
            "sabinewren-scum" => "sabinewren-lancerclasspursuitcraft",
            "sabinewren-tiefighter" => "sabinewren-tielnfighter",
            "sharabey-awing" => "sharabey-rz1awing",
            "vultskerris-tieinterceptor" => "vultskerris-tieininterceptor",
            "wedgeantilles-awing" => "wedgeantilles-rz1awing",
            "zeborrelios-sheathipede" => "zeborrelios-sheathipedeclassshuttle",
            "zeborrelios-tiefighter" => "zeborrelios-tielnfighter",
            x => x,
        },
        ItemType::Upgrade => match canonical.as_str() {
            "b6bladewingprototype-epic" => "b6bladewingprototype-command",
            "c3po-resistance" => "c3po-crew",
            "chewbacca-resistance" => "chewbacca-crew",
            "chopper-astromech" => "chopper",
            "hansolo-resistance" => "hansolo-crew",
            "rey" => "rey-gunner",
            "vectoredcannons-rz1" => "vectoredcannonsrz1",
            x => x,
        },
        ItemType::Ship => match canonical.as_str() {
            /* in case anyone asks: these are 1.0 ships that have been
            renamed over time in yasb, but the old records are not
            cleared with "Reset my Collection" */
            "arc170"
            | "awing"
            | "bsf17bomber"
            | "bwing"
            | "firespray31"
            | "hwk290"
            | "kwing"
            | "lambdaclassshuttle"
            | "quadjumper"
            | "starviper"
            | "tieadvanced"
            | "tieadvancedprototype"
            | "tieaggressor"
            | "tiebomber"
            | "tiedefender"
            | "tiefighter"
            | "tieinterceptor"
            | "tiesilencer"
            | "tiestriker"
            | "upsilonclasscommandshuttle"
            | "uwing"
            | "vcx100"
            | "xwing"
            | "yt1300-resistance"
            | "yt1300"
            | "yt2400"
            | "ywing"
            | "z95headhunter" => {
                canonical.push_str("-legacyyasb");
                canonical.as_str()
            }
            _ => canonical.as_str(),
        },
        _ => canonical.as_str(),
    }
    .to_string()
}

impl Collection {
    /// Attempts to turn YASB collection expansion names into their proper SKUs.
    /// Returns a list of any expansion names that couldn't be found.
    pub fn expansion_skus(
        &self,
        catalog: &expansions::Catalog,
    ) -> (BTreeMap<String, u32>, Vec<String>) {
        let mut skus = BTreeMap::new();
        let mut missing = vec![];

        'exp_search: for (e, c) in &self.expansions {
            let n: u32 = c.parse().unwrap(); // FIXME:
            if n == 0 {
                continue;
            }
            for (sku, expansion) in &catalog.expansions {
                if &expansion.name == e {
                    skus.insert(sku.to_owned(), n);
                    continue 'exp_search;
                }
            }
            missing.push(e.to_owned())
        }

        (skus, missing)
    }

    /// Does not do any checking of correctness/missing items, just tries
    /// to use the hard-coded YASB-to-xws lookup/rules for the singles.
    pub fn singles_as_xws(&self) -> BTreeMap<Item, u32> {
        let mut item_counts = BTreeMap::new();

        if let Collection {
            singletons: Some(ref singles),
            ..
        } = self
        {
            for (name, c) in singles.upgrade.as_ref().unwrap_or(&HashMap::new()) {
                let n: u32 = c.parse().unwrap(); // FIXME:
                if n == 0 {
                    continue;
                }
                let item = Item {
                    r#type: ItemType::Upgrade,
                    xws: to_xws(name, ItemType::Upgrade),
                };
                if item_counts.get(&item).is_some() {
                    println!("YASB: ignoring duplicate item: {}", name);
                    continue;
                }
                item_counts.insert(item, n);
            }
            for (name, c) in singles.pilot.as_ref().unwrap_or(&HashMap::new()) {
                let n: u32 = c.parse().unwrap(); // FIXME:
                if n == 0 {
                    continue;
                }
                let item = Item {
                    r#type: ItemType::Pilot,
                    xws: to_xws(name, ItemType::Pilot),
                };
                if item_counts.get(&item).is_some() {
                    println!("YASB: ignoring duplicate item: {}", name);
                    continue;
                }
                item_counts.insert(item, n);
            }
            for (name, c) in singles.ship.as_ref().unwrap_or(&HashMap::new()) {
                let n: u32 = c.parse().unwrap(); // FIXME:
                if n == 0 {
                    continue;
                }
                let item = Item {
                    r#type: ItemType::Ship,
                    xws: to_xws(name, ItemType::Ship),
                };
                if item_counts.get(&item).is_some() {
                    println!("YASB: ignoring duplicate item: {}", name);
                    continue;
                }
                item_counts.insert(item, n);
            }
        }
        item_counts
    }
}
