// This module is not a complete implementation of xwing-data2, just what
// is necessary for some basic collection management.

use serde::Deserialize;
use serde_json;
use std::fs;
use std::io::Error;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Pilot {
    name: String,
    xws: String,
    initiative: u32,
}

#[derive(Deserialize, Debug)]
pub struct Ship {
    name: String,
    xws: String,
    faction: String,
    pilots: Vec<Pilot>,
}

#[derive(Deserialize, Debug)]
pub struct Upgrade {
    name: String,
    xws: String,
    restritions: Option<serde_json::Value>,
}

// TODO: The goal is have them all in one list, and let a spreadsheet
#[derive(Deserialize, Debug)]
pub struct Data {
    pilots: Vec<Ship>,
    // editor of some sort do the grouping/ordering
    upgrades: Vec<Upgrade>,
}

#[derive(Deserialize, Debug)]
struct ShipFaction {
    faction: String,
    ships: Vec<String>,
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
        pilots: vec![],
        upgrades: vec![],
    };

    for upgrade_path in &manifest.upgrades {
        let path = path.join(upgrade_path);
        let buffer = fs::read_to_string(path)?;
        // TODO: the individual fils are pretty straightforward, so choosing
        // not to create a struct here yet
        let mut upgrades: Vec<Upgrade> = serde_json::from_str(&buffer)?;
        data.upgrades.append(&mut upgrades);
    }

    Ok(data)
}
