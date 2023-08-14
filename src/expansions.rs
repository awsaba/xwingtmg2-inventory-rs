use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Item {
    #[serde(alias = "ship")]
    Ship { xws: String, count: u32 },
    #[serde(alias = "obstacle")]
    Obstacle { xws: String, count: u32 },
    #[serde(alias = "pilot")]
    Pilot { xws: String, count: u32 },
    #[serde(alias = "upgrade")]
    Upgrade { xws: String, count: u32 },
    #[serde(alias = "damage")]
    Damage { xws: String, count: u32 },
}

pub type Expansions = HashMap<String, Vec<Item>>;

/// Loads a yasb based expansion content list from embedded file.
pub fn load_expansions() -> Result<Expansions, io::Error> {
    //TODO: embed with rust-embed or include_bytes! or something
    let buffer = fs::read_to_string("./src/expansions.json")?;

    let m: Value = serde_json::from_str(&buffer)?;
    if let Value::Object(ref o) = m {
        // FIXME: probably hugely memory inefficient
        let mut expansions: Expansions = HashMap::new();
        for (k, v) in o {
            let items: Vec<Item> = serde_json::from_value(v.to_owned())?;
            expansions.insert(k.to_owned(), items);
        }
        return Ok(expansions);
    }
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "not an expansion listing",
    ))
}

#[cfg(test)]
mod test {
    use std::{io::Write, path::Path};

    use super::*;
    use crate::xwingdata2;
    use crate::xwingdata2::known_missing;

    #[test]
    fn test_valid_xws() {
        // checks if all the contents are valid xwsdata
        let r = load_expansions().unwrap();

        let d = xwingdata2::load_from_manifest(Path::new("xwing-data2")).unwrap();

        for (_, contents) in r.iter() {
            for item in contents {
                let result = match item {
                    Item::Ship { xws, .. } => d.get_ship(xws).is_some() || known_missing(xws),
                    Item::Pilot { xws, .. } => d.get_pilot(xws).is_some() || known_missing(xws),
                    Item::Upgrade { xws, .. } => d.get_upgrade(xws).is_some() || known_missing(xws),
                    _ => continue,
                };

                println!("{:?}", item);
                assert!(result, "missing expansion item");

                io::stdout().flush().unwrap();
            }
        }
    }
}
