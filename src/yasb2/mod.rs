use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::expansions;
use crate::expansions::{Item, ItemCount, ItemType};

/// Ships are probably this most common.
#[derive(Deserialize, Serialize, Debug)]
pub struct Singletons {
    pub ship: Option<HashMap<String, String>>,
    pub upgrade: Option<HashMap<String, String>>,
    pub pilot: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Collection {
    pub expansions: HashMap<String, String>,
    pub singletons: Option<Singletons>,
}

/// Intermediate Collection file because yasb returns the counts as strings?
#[derive(Deserialize, Serialize, Debug)]
pub struct CollectionFile {
    pub collection: Collection,
}

/// Load a raw YASB collection obtained from https://yash.app/collection.
/// Intermediate step that does not turn the strings for the counts back
/// into numbers.
pub fn load_collection_file(path: &Path) -> Result<Collection, io::Error> {
    let buffer = fs::read_to_string(path)?;

    let f: CollectionFile = serde_json::from_str(&buffer)?;
    Ok(f.collection)
}

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
pub(crate) fn to_xws(name: &str, typ: expansions::ItemType) -> String {
    let xws = to_canonical(name);
    match typ {
        ItemType::Pilot => match xws.as_str() {
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
        ItemType::Upgrade => match xws.as_str() {
            "b6bladewingprototype-epic" => "b6bladewingprototype-command",
            "c3po-resistance" => "c3po-crew",
            "chewbacca-resistance" => "chewbacca-crew",
            "chopper-astromech" => "chopper",
            "hansolo-resistance" => "hansolo-crew",
            "rey" => "rey-gunner",
            "vectoredcannons-rz1" => "vectoredcannonsrz1",
            x => x,
        },
        ItemType::Ship => match xws.as_str() {
            /* in case anyone asks: these are 1.0 ships that have been
               renamed over time in yasb, but the old records are not
               cleared with "Reset my Collection" */
            /* "arc170" => "arc170starfighter",
            "awing" => "rz1awing",
            "bsf17bomber" => "mg100starfortress",
            "bwing" => "asf01bwing",
            "firespray31" => "firesprayclasspatrolcraft",
            "gozanticlasscruiser" => "gozanticlasscruiser",
            "hwk290" => "hwk290lightfreighter",
            "kwing" => "btls8kwing",
            "lambdaclassshuttle" => "lambdaclasst4ashuttle",
            "mg100starfortress" => "mg100starfortress",
            "quadjumper" => "quadrijettransferspacetug",
            "starviper" => "starviperclassattackplatform",
            "tieadvanced" => "tieadvancedx1",
            "tieadvancedprototype" => "tieadvancedv1",
            "tieaggressor" => "tieagaggressor",
            "tiebomber" => "tiesabomber",
            "tiedefender" => "tieddefender",
            "tiefighter" => "tielnfighter",
            "tieinterceptor" => "tieininterceptor",
            "tiesilencer" => "tievnsilencer",
            "tiestriker" => "tieskstriker",
            "upsilonclasscommandshuttle" => "upsilonclassshuttle",
            "uwing" => "ut60duwing",
            "vcx100" => "vcx100lightfreighter",
            "xwing" => "t65xwing",
            "yt1300-resistance" => "scavengedyt1300",
            "yt1300" => "modifiedyt1300lightfreighter",
            "yt2400" => "yt2400lightfreighter",
            "ywing" => "btla4ywing",
            "z95headhunter" => "z95af4headhunter",
            */
            _ => xws.as_str(),
        },
        _ => xws.as_str(),
    }
    .to_string()
}

/// Flat list of item counts and not found things in a collection.
/// TODO: Move to a more generic module that can be referenced by multiple
/// collection sources.
pub struct XwsCollection {
    pub item_counts: Vec<ItemCount>,
    pub missing_singles: Vec<Item>,
    pub missing_expansions: Vec<String>,
}

impl Collection {
    /// Returns base counts
    ///
    /// Assumes that the `expansiions` contains all valid items that exist in
    /// xwing-data2 when determining what is missing.
    pub fn to_xws_collection(&self, expansions: &expansions::Expansions) -> XwsCollection {
        let mut item_counts: HashMap<Item, u32> = HashMap::new();
        let mut missing_singles: Vec<Item> = vec![];
        let mut missing_expansions: Vec<String> = vec![];

        // TODO: This is some terrible, non-idiomatic rust
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
                    xws: to_xws(&name, ItemType::Upgrade),
                };
                if !expansions::has_item(expansions, &item) {
                    missing_singles.push(item);
                    continue;
                }
                if let Some(_) = item_counts.get(&item) {
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
                if !expansions::has_item(expansions, &item) {
                    missing_singles.push(item);
                    continue;
                }
                if let Some(_) = item_counts.get(&item) {
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
                if !expansions::has_item(expansions, &item) {
                    missing_singles.push(item);
                    continue;
                }
                if let Some(_) = item_counts.get(&item) {
                    println!("YASB: ignoring duplicate item: {}", name);
                    continue;
                }
                item_counts.insert(item, n);
            }
        }

        'exp_search: for (e, c) in &self.expansions {
            let n: u32 = c.parse().unwrap(); // FIXME:
            if n == 0 {
                continue;
            }

            for expansion in expansions {
                if expansion.edition() != 2 {
                    continue;
                }
                if &expansion.name == e {
                    for item_count in &expansion.contents {
                        let total =
                            item_counts.get(&item_count.item).unwrap_or(&0) + n * item_count.count;
                        item_counts.insert(item_count.item.clone(), total);
                    }
                    continue 'exp_search;
                }
            }

            missing_expansions.push(e.to_owned());
        }

        XwsCollection {
            item_counts: item_counts
                .iter()
                .map(|(k, v)| ItemCount {
                    item: Item {
                        r#type: k.r#type,
                        xws: k.xws.clone(),
                    },
                    count: *v,
                })
                .collect(),
            missing_expansions,
            missing_singles,
        }
    }
}
