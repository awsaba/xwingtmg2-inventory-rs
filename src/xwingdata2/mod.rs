// This module is not a complete implementation of xwing-data2, just what
// is necessary for some basic collection management.

use serde::Deserialize;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Error;
use std::path::Path;

pub trait Xws {
    /// Returns the xws id of the item.
    fn xws(&self) -> &str;
}

impl Hash for dyn Xws {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.xws().hash(state);
    }
}

/// Returns true if the card is known to not have a canonical xws id. This is
/// mostly epic only cards.
#[allow(dead_code)]
pub fn known_missing(xws: &str) -> bool {
    matches!(xws, "sabinewren-swz93")
}

#[derive(Deserialize, Debug)]
pub struct Pilot {
    pub name: String,
    pub xws: String,
    pub initiative: u32,
}

impl Xws for Pilot {
    fn xws(&self) -> &str {
        &self.xws
    }
}

#[derive(Deserialize, Debug)]
pub struct Ship {
    pub name: String,
    pub xws: String,
    pub faction: String,
    pub pilots: Vec<Pilot>,
}

impl Xws for Ship {
    fn xws(&self) -> &str {
        &self.xws
    }
}

#[derive(Deserialize, Debug)]
pub struct Side {
    pub r#type: String,
}

pub enum Restriction {
    Factions,
    Sizes,
    Ships,
    Arcs,
    Keywords,
    ForceSide,
    //Equipped,
    //Action,
}

#[derive(Deserialize, Debug)]
pub struct ActionDifficulty {
    pub r#type: String,
    pub difficulty: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Restrictions {
    #[serde(default)]
    pub factions: Vec<String>,
    #[serde(default)]
    pub sizes: Vec<String>,
    #[serde(default)]
    pub ships: Vec<String>,
    #[serde(default)]
    pub arcs: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub force_side: Vec<String>,
    #[serde(default)]
    pub equipped: Vec<String>,
    #[serde(default)]
    pub action: Option<ActionDifficulty>,
}

#[derive(Deserialize, Debug)]
pub struct Upgrade {
    pub name: String,
    pub xws: String,
    pub sides: Vec<Side>,

    #[serde(default)]
    pub restrictions: Vec<Restrictions>,
}

impl Xws for Upgrade {
    fn xws(&self) -> &str {
        &self.xws
    }
}

// TODO: The goal is have them all in one list, and let a spreadsheet
#[derive(Deserialize, Debug)]
pub struct Data {
    pub ships: Vec<Ship>,
    // editor of some sort do the grouping/ordering
    pub upgrades: Vec<Upgrade>,
}

impl Data {
    pub fn get_pilot(&self, xws: &str) -> Option<(&Ship, &Pilot)> {
        for s in &self.ships {
            for p in &s.pilots {
                if p.xws == xws {
                    return Some((s, p));
                }
            }
        }
        None
    }

    pub fn get_upgrade(&self, xws: &str) -> Option<&Upgrade> {
        self.upgrades.iter().find(|&u| u.xws == xws)
    }

    pub fn get_ship(&self, xws: &str) -> Option<&Ship> {
        self.ships.iter().find(|&s| s.xws == xws)
    }
}

#[derive(Deserialize, Debug)]
struct ShipFaction {
    pub ships: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Manifest {
    pilots: Vec<ShipFaction>,
    upgrades: Vec<String>,
}

/// Loads from a xwing-data2/ data source.
///
/// # Errors
///
/// This function will return an error if any of the paths are invalid
/// or can't be parsed.
pub fn load_from_manifest(path: &Path) -> Result<Data, Error> {
    // read the whole manifest
    let manifest_path = path.join("data/manifest.json");
    let buffer = fs::read_to_string(manifest_path)?;

    let manifest: Manifest = serde_json::from_str(&buffer)?;

    let mut data = Data {
        ships: vec![],
        upgrades: vec![],
    };

    for upgrade_path in &manifest.upgrades {
        //println!("loading: {}", &upgrade_path);
        let path = path.join(upgrade_path);
        let buffer = fs::read_to_string(path)?;
        // TODO: the individual fils are pretty straightforward, so choosing
        // not to create a struct here yet
        let mut upgrades: Vec<Upgrade> = serde_json::from_str(&buffer)?;
        data.upgrades.append(&mut upgrades);
    }

    for faction in &manifest.pilots {
        for pilot_path in &faction.ships {
            let path = path.join(pilot_path);
            let buffer = fs::read_to_string(path)?;
            let ship: Ship = serde_json::from_str(&buffer)?;
            data.ships.push(ship);
        }
    }

    Ok(data)
}