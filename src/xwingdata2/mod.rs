//! This module is not a complete implementation of xwing-data2, just what
//! is necessary for some basic collection management.
//!
//! ```rust
//! use std::path::Path;
//! use xwingtmg2_inventory_rs::xwingdata2::Data;
//!
//! let data = Data::load_from_manifest(Path::new("./xwing-data2")).unwrap();
//! match data.get_pilot("zeborrelios") {
//!    Some((ship, pilot)) => println!("{}: {} - {}", ship.name, pilot.name, pilot.initiative),
//!    None => println!("not found"),
//! };
//! ```

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
    #[serde(alias = "action")]
    Action,
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

#[derive(Deserialize, Clone, Debug)]
pub struct Faction {
    pub xws: String,
    pub name: String,
    // TODO: icon
}

impl XwsId for Faction {
    fn xws(&self) -> &str {
        &self.xws
    }
    fn kind(&self) -> XwsKind {
        XwsKind::Pilot
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
    pub caption: Option<String>,
    pub xws: String,
    pub initiative: u32,
    #[serde(alias = "standardLoadout")]
    pub standard_loadout: Option<Vec<String>>,
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

/// Top-level model of loaded xwing-data2 data.
#[derive(Deserialize, Debug)]
pub struct Data {
    pub ships: Vec<Ship>,
    pub upgrades: Vec<Upgrade>,
    // List of factions loaded from the manifest for looking up a display name
    // from the xws id used to reference them.
    pub factions: Vec<Faction>,
}

fn load_type<T: for<'a> Deserialize<'a>>(root: &Path, paths: &[String]) -> Result<Vec<T>, Error> {
    let mut result = Vec::new();

    for path in paths {
        //println!("loading: {}", &faction_path);
        let path = root.join(path);
        let buffer = fs::read_to_string(path)?;
        let mut factions: Vec<T> = serde_json::from_str(&buffer)?;
        result.append(&mut factions);
    }

    Ok(result)
}

impl Data {
    /// Loads from a xwing-data2/ data source.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the paths are invalid
    /// or can't be parsed.
    pub fn load_from_manifest(path: &Path) -> Result<Self, Error> {
        // read the whole manifest
        let manifest_path = path.join("data/manifest.json");
        let buffer = fs::read_to_string(manifest_path)?;

        let manifest: Manifest = serde_json::from_str(&buffer)?;

        let mut data = Data {
            ships: vec![],
            upgrades: load_type(path, &manifest.upgrades)?,
            factions: load_type(path, &manifest.factions)?,
        };

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

    pub fn get_faction(&self, xws: &str) -> Option<&Faction> {
        self.factions.iter().find(|&s| s.xws == xws)
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
    factions: Vec<String>,
}
