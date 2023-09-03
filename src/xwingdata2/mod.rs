// This module is not a complete implementation of xwing-data2, just what
// is necessary for some basic collection management.

use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Error;
use std::path::Path;

#[derive(Deserialize, Serialize, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum SlotKind {
    Astromech,
    Cannon,
    Cargo,
    Command,
    Configuration,
    Crew,
    Device,
    #[serde(alias = "Force Power")]
    ForcePower,
    Gunner,
    Hardpoint,
    Hyperdrive,
    Illicit,
    Missile,
    Modification,
    Sensor,
    #[serde(alias = "Tactical Relay")]
    TacticalRelay,
    Talent,
    Team,
    Tech,
    Title,
    Torpedo,
    Turret,
}

#[derive(Deserialize, Serialize, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum XwsKind {
    #[serde(alias = "ship")]
    Ship,
    #[serde(alias = "obstacle")]
    Obstacle,
    #[serde(alias = "pilot")]
    Pilot,
    //TODO: Using the "type" of the main side for now, but should be expanded
    // to account for the multiple slot cards
    #[serde(alias = "upgrade")]
    Upgrade(SlotKind),
    #[serde(alias = "damage")]
    Damage,
}

/// `XwsId` are the "unique" combination of the item types (roughly the
/// top-level kinds of things). In practices, all the upgrade and pilot cards
/// should be unique, but
pub trait XwsId {
    /// Returns the xws id of the item.
    fn xws(&self) -> &str;

    /// Returns the kind of the item
    fn kind(&self) -> XwsKind;
}

impl Hash for dyn XwsId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.xws().hash(state);
        self.kind().hash(state);
    }
}

impl PartialOrd for dyn XwsId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (
            self.xws().partial_cmp(other.xws()),
            self.kind().partial_cmp(&other.kind()),
        ) {
            (Some(o1), Some(o2)) => Some(o1.then(o2)),
            _ => None,
        }
    }
}

impl PartialEq for dyn XwsId {
    fn eq(&self, other: &Self) -> bool {
        self.xws() == other.xws() && self.kind() == other.kind()
    }
}

/// Returns true if the card is known to not have a canonical xws id. This is
/// mostly epic only cards.
#[allow(dead_code)]
pub fn known_missing(xws: &str) -> bool {
    matches!(xws, "sabinewren-swz93") // this is the epic commmand/crew card
}

#[derive(Deserialize, Clone, Debug)]
pub struct Pilot {
    pub name: String,
    pub xws: String,
    pub initiative: u32,
}

impl XwsId for Pilot {
    fn xws(&self) -> &str {
        &self.xws
    }
    fn kind(&self) -> XwsKind {
        XwsKind::Pilot
    }
}

#[derive(Deserialize, Debug)]
pub struct Ship {
    pub name: String,
    pub xws: String,
    pub faction: String,
    pub pilots: Vec<Pilot>,
}

impl XwsId for Ship {
    fn xws(&self) -> &str {
        &self.xws
    }
    fn kind(&self) -> XwsKind {
        XwsKind::Ship
    }
}

#[derive(Deserialize, Debug)]
pub struct Side {
    pub r#type: SlotKind,
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

#[derive(Hash, Eq, PartialEq, Deserialize, Debug)]
pub struct ActionDifficulty {
    pub r#type: String,
    pub difficulty: Option<String>,
}

#[derive(Hash, PartialEq, Eq, Deserialize, Default, Debug)]
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

impl XwsId for Upgrade {
    fn xws(&self) -> &str {
        &self.xws
    }
    fn kind(&self) -> XwsKind {
        XwsKind::Upgrade(self.sides[0].r#type)
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
