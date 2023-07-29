use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::expansions;
use crate::expansions::Item;
use crate::xwingdata2;
use xwingdata2::CardType;

/// Ships are probably this most common.
#[derive(Deserialize, Serialize, Debug)]
pub struct Singletons {
    pub ships: Option<HashMap<String, String>>,
    pub upgrades: Option<HashMap<String, String>>,
    pub pilots: Option<HashMap<String, String>>,
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
        .filter(|c| c.is_alphanumeric() || c == &'(' || c == &'-')
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
pub(crate) fn to_xws(name: &str, typ: xwingdata2::CardType) -> String {
    let xws = to_canonical(name);
    match typ {
        CardType::Pilot => match xws.as_str() {
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
        CardType::Upgrade => match xws.as_str() {
            "b6bladewingprototype-epic" => "b6bladewingprototype-command",
            "c3po-resistance" => "c3po-crew",
            "chewbacca-resistance" => "chewbacca-crew",
            "chopper-astromech" => "chopper",
            "hansolo-resistance" => "hansolo-crew",
            "rey" => "rey-gunner",
            "vectoredcannons-rz1" => "vectoredcannonsrz1",
            x => x,
        },
    }
    .to_string()
}

/// Returns base counts
///
/// Does not verify the xws ids that are returned agains xwing-data2.
///
/// The XWS spec defines `pilot` and `updgrade` as being independent namespaces,
/// so a combined key could be used, but just be explicit with separate
/// collections for now.
pub fn collection_to_xws_count(
    collection: &Collection,
    expansions: &expansions::Expansions,
) -> (HashMap<String, u32>, HashMap<String, u32>, Vec<String>) {
    let mut pilots = HashMap::new();
    let mut upgrades = HashMap::new();
    let mut not_found: Vec<String> = vec![];

    // TODO: This is some terrible, non-idiomatic rust
    if let Collection {
        singletons: Some(Singletons {
            upgrades: Some(us), ..
        }),
        ..
    } = collection
    {
        for (name, c) in us {
            let n: u32 = c.parse().unwrap(); // FIXME:
            let xws = to_xws(name, CardType::Upgrade);
            // TODO: make upgrades into a map?
            // FIXME: at the very least, check for dupes somewhere
            let total = upgrades.get(&xws).unwrap_or(&0) + n;
            upgrades.insert(xws, total);
        }
    }
    if let Collection {
        singletons: Some(Singletons {
            pilots: Some(ps), ..
        }),
        ..
    } = collection
    {
        for (name, c) in ps {
            let n: u32 = c.parse().unwrap(); // FIXME:
            let xws = to_xws(name, CardType::Pilot);
            // TODO: make upgrades into a map?
            // FIXME: at the very least, check for dupes somewhere
            let total = pilots.get(&xws).unwrap_or(&0) + n;
            pilots.insert(xws, total);
        }
    }

    for (e, c) in &collection.expansions {
        let n: u32 = c.parse().unwrap(); // FIXME:
        if n == 0 {
            continue;
        }
        let xws = to_canonical(e);
        let items = match expansions.get(&xws) {
            None => {
                not_found.push(e.to_owned());
                continue;
            }
            Some(items) => items,
        };
        for item in items {
            match item {
                Item::Pilot { ref xws, count } => {
                    let total = pilots.get(xws).unwrap_or(&0) + n * count;
                    pilots.insert(xws.clone(), total);
                }
                Item::Upgrade { ref xws, count } => {
                    let total = upgrades.get(xws).unwrap_or(&0) + n * count;
                    upgrades.insert(xws.clone(), total);
                }
                _ => (),
            };
        }
    }

    (pilots, upgrades, not_found)
}
